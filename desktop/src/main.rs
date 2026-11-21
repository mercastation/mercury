//! Mercury's desktop window, drawn natively. Phase 1: it reads. Home, the Inbox and one
//! conversation, from the same snapshot the web window gets, and kept current by the engine.
//!
//!   cargo run -- --door http://127.0.0.1:7777 --token <key>          # live, from the engine
//!   cargo run -- --fixture shots/home-fixture.json --snap shots/home.png --size 1280x760 --scale 2
//!   cargo run -- --fixture shots/inbox-fixture.json --view inbox --snap shots/inbox.png --size 1280x760 --scale 2
//!   cargo run -- --fixture shots/inbox-later-kinds-fixture.json --view inbox --tab later --snap shots/inbox-later-kinds.png --size 1280x760 --scale 2
//!   cargo run -- --fixture shots/thread-fixture.json --thread --snap shots/thread.png --size 1280x760 --scale 2
//!
//! Every size, colour, border and shadow was read off the running renderer, so a screen here
//! can be laid over the web app's and compared pixel by pixel (`tools/`).

#![forbid(unsafe_code)]

mod chrome;
mod data;
mod home;
mod icons;
mod inbox;
mod markdown;
mod rules;
mod sky;
mod source;
mod talk;
mod theme;
mod thread;

use std::collections::HashMap;

use rune::{App, AppSpec, Cx, El, FontSmoothing, Latest, Pixels, ScrollSpec, Tx, Waker, el};
use serde_json::{Map, Value};

use data::{Lists, Local, Snapshot, Tab};
use source::{Mailbox, Source};

/// Which screen the card is showing.
#[derive(Clone, PartialEq, Debug)]
pub enum View {
    Home,
    Inbox,
    Thread { id: String, product: String },
}

/// Both skies for one card size, painted off the main thread, so changing screen never waits.
struct Painted {
    home: (u32, u32, Vec<u8>),
    inbox: (u32, u32, Vec<u8>),
    points: (f32, f32),
}

pub struct State {
    /// The window, in points.
    pub size: (f32, f32),
    /// Home's sky, and the Inbox's (drawn at the card's foot; `inbox.rs` turns it over).
    pub sky_home: Pixels,
    pub sky_inbox: Pixels,
    /// The card size the skies on screen were painted for.
    pub sky_size: (f32, f32),
    painting: Latest<Painted>,
    pub snap: Snapshot,
    /// What the screens draw, worked out from `snap` whenever it or the minute changes.
    pub lists: Lists,
    pub local: Local,
    /// The rest of a fixture file, for the screen that knows what it is (a thread's words).
    pub fixture: Map<String, Value>,
    /// The clock the rows read, in milliseconds. A fixture brings its own.
    pub now: f64,
    fixed_clock: bool,
    pub view: View,
    /// The Inbox's open tab.
    pub inbox_tab: Tab,
    /// What the person has unfolded in the conversations they opened.
    pub reading: thread::Reading,
    /// What the engine answered about each conversation opened so far, by `product/id` and
    /// then by channel, the shape a fixture's `threads` has (`docs/thread-data.md`).
    pub threads: HashMap<String, Value>,
    /// A conversation being asked for, and what each was last asked at: its row's last change.
    asking: Latest<(String, Result<Value, String>)>,
    asked: HashMap<String, f64>,
    pub history: Vec<View>,
    pub source: Source,
    mailbox: Mailbox,
    /// Why the engine could not be read, when it could not.
    pub error: Option<String>,
}

impl State {
    pub fn go(&mut self, to: View) {
        if self.view != to {
            self.history.push(std::mem::replace(&mut self.view, to));
        }
    }
    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.view = prev;
        }
    }
    pub fn card_size(&self) -> (f32, f32) {
        ((self.size.0 - chrome::CARD[0] - chrome::CARD[2]).max(1.0), (self.size.1 - chrome::CARD[1] - chrome::CARD[3]).max(1.0))
    }
    /// The page's box inside the window: under the card's top strip.
    pub fn page_size(&self) -> (f32, f32) {
        ((self.size.0 - chrome::CARD[0] - chrome::CARD[2]).max(1.0), (self.size.1 - chrome::page_top(&self.view) - chrome::CARD[3]).max(1.0))
    }
    fn refresh(&mut self) {
        self.lists = self.snap.lists(self.now, &self.local);
    }
    /// Asks the engine for the open conversation's lines when it has not been asked since the
    /// thread last changed. Off the main thread: the answer lands in `poll`.
    fn ask_for_the_open_thread(&mut self) {
        let (View::Thread { id, product }, Source::Door { url, token }) = (&self.view, &self.source) else {
            return;
        };
        let key = format!("{product}/{id}");
        let changed = self.snap.items.iter().find(|i| &i.id == id && &i.product == product).and_then(|i| i.updated_at).unwrap_or(0.0);
        if self.asked.get(&key) == Some(&changed) {
            return;
        }
        self.asked.insert(key.clone(), changed);
        let payload = serde_json::json!({ "product": product, "id": id });
        self.asking.request((url.clone(), token.clone(), key, payload), |(url, token, key, payload)| {
            let lines = source::call(&url, &token, "zero:item-history", &payload);
            (key, lines)
        });
    }
    fn hang(&mut self, p: Painted) {
        self.sky_home.set(p.home.0, p.home.1, p.home.2);
        self.sky_inbox.set(p.inbox.0, p.inbox.1, p.inbox.2);
        self.sky_size = p.points;
    }
}

fn clock() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64)
}

fn paint_skies(points: (f32, f32)) -> Painted {
    let (w, h) = (points.0.floor() as f64, points.1.floor() as f64);
    let art = sky::ember();
    // Home's: dots from 40 points down, fully in 300 points later (`SkyBackdrop` in App.tsx).
    let home = sky::draw(&art, w, h, 40.0, 300.0, 0.0, 2);
    // The Inbox's: a band 190 points tall at the card's foot (`InboxSky`, shape "head").
    let start = (h - 190.0).round();
    let inbox = sky::draw(&art, w, h, start, 120.0, start - 60.0, 2);
    Painted { home, inbox, points }
}

const DEFAULT_SIZE: (f32, f32) = (1280.0, 760.0);

fn init() -> State {
    let args = rune::tooling::extra_args();
    let source = source::from_args(args);
    // A headless picture has its size before the first frame; a window reports its own.
    let size = rune::tooling::headless_run().map_or(DEFAULT_SIZE, |run| run.size());
    let mut s = State {
        size,
        sky_home: Pixels::new(),
        sky_inbox: Pixels::new(),
        sky_size: (0.0, 0.0),
        painting: Latest::default(),
        snap: Snapshot::default(),
        lists: Lists::default(),
        local: Local::default(),
        fixture: Map::new(),
        now: clock(),
        fixed_clock: false,
        view: match args.iter().position(|a| a == "--view").and_then(|i| args.get(i + 1)).map(String::as_str) {
            Some("inbox") => View::Inbox,
            _ => View::Home,
        },
        // `--tab later` opens the Inbox on that tab, for a picture of it.
        inbox_tab: Tab::from_args(args),
        reading: thread::Reading::default(),
        threads: HashMap::new(),
        asking: Latest::default(),
        asked: HashMap::new(),
        history: Vec::new(),
        source,
        mailbox: Mailbox::default(),
        error: None,
    };
    // The first skies and a fixture are in place before the first frame, so a picture is whole.
    let first = paint_skies(s.card_size());
    s.hang(first);
    if let Source::Fixture(path) = &s.source {
        match source::read_fixture(path) {
            Ok(fixture) => {
                s.snap = fixture.snapshot;
                s.local = fixture.local;
                s.fixture = fixture.rest;
                if let Some(now) = fixture.now {
                    s.now = now;
                    s.fixed_clock = true;
                }
            }
            Err(e) => s.error = Some(e),
        }
    }
    // A headless picture cannot wait for a thread to bring the engine's answer, so it asks once here.
    if let (Source::Door { url, token }, true) = (&s.source, rune::tooling::headless_run().is_some()) {
        match source::fetch_snapshot(url, token) {
            Ok(snapshot) => s.snap = snapshot,
            Err(e) => s.error = Some(e),
        }
        if let Ok(Some(local)) = source::fetch_keeps(url, token) {
            s.local = local;
        }
    }
    s.refresh();
    // A window opened straight onto the Inbox came there from Home, as far as Back is concerned.
    if s.view == View::Inbox {
        s.history.push(View::Home);
    }
    // `--thread` opens the first thread that needs the person, for a picture of it; `--thread <id>` that one.
    if let Some(at) = args.iter().position(|a| a == "--thread") {
        let wanted = args.get(at + 1).filter(|a| !a.starts_with("--"));
        let row = match wanted {
            Some(id) => s.lists.needs.iter().chain(&s.lists.progress).chain(&s.lists.later).chain(&s.lists.done).find(|r| &r.id == id),
            None => s.lists.needs.first(),
        };
        if let Some(r) = row {
            s.view = View::Thread { id: r.id.clone(), product: r.product.clone() };
        }
    }
    s
}

fn setup(s: &mut State, wake: Waker) {
    s.painting.set_waker(wake.clone());
    s.asking.set_waker(wake.clone());
    if let Source::Door { url, token } = s.source.clone() {
        source::follow_door(url, token, s.mailbox.clone(), move || wake());
    }
}

fn poll(s: &mut State, _clock: f64, tx: &mut Tx) -> bool {
    let mut changed = false;
    if let Some(p) = s.painting.take() {
        s.hang(p);
        changed = true;
    }
    let mut stale = false;
    {
        let mut m = s.mailbox.lock().unwrap();
        if let Some(snapshot) = m.snapshot.take() {
            s.snap = snapshot;
            stale = true;
        }
        if let Some(local) = m.local.take() {
            s.local = local;
            stale = true;
        }
        if m.error != s.error && matches!(s.source, Source::Door { .. }) {
            s.error = m.error.clone();
            changed = true;
        }
    }
    if !s.fixed_clock {
        // Ages are told to the minute, so the clock only has to turn that often.
        let now = clock();
        stale |= (now / 60_000.0).floor() != (s.now / 60_000.0).floor();
        s.now = now;
        tx.wake_after(std::time::Duration::from_secs(20));
    }
    if stale {
        s.refresh();
    }
    if let Some((key, answer)) = s.asking.take() {
        match answer {
            Ok(history) => {
                s.threads.insert(key, serde_json::json!({ "zero:item-history": history }));
            }
            // Asked again on the thread's next change; until then the row's own words are shown.
            Err(_) => {
                s.asked.remove(&key);
            }
        }
        changed = true;
    }
    s.ask_for_the_open_thread();
    changed || stale
}

fn resized(s: &mut State, [_, _, w, h]: [f32; 4]) {
    if (s.size.0 - w).abs() < 0.5 && (s.size.1 - h).abs() < 0.5 {
        return;
    }
    s.size = (w, h);
    let points = s.card_size();
    s.painting.request(points, paint_skies);
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let (w, h) = s.size;
    let page = match &s.view {
        View::Home => home::view(s),
        View::Inbox => inbox::view(s),
        View::Thread { id, product } => thread::view(s, id, product),
    };
    let root = el().w_full()
        .h_full()
        .fill(theme::page())
        .on_layout(resized)
        .child(chrome::backdrop(w, h))
        .child(chrome::card(s))
        .child(chrome::rail(s))
        .child(chrome::back_forward(s))
        // The page scrolls inside the card, under its top strip.
        .child(
            el().pin(Some(chrome::CARD[0]), Some(chrome::page_top(&s.view)), Some(chrome::CARD[2]), Some(chrome::CARD[3]))
                .clip_corners([0.0, 0.0, 6.0, 6.0])
                .scroll(ScrollSpec { stick_bottom: false })
                .child(page),
        );
    match chrome::notice(s) {
        Some(notice) => root.child(notice),
        None => root,
    }
}

static FONTS: &[&[u8]] = &[include_bytes!("../assets/fonts/Geist-Variable.ttf"), include_bytes!("../assets/fonts/GeistMono-Variable.ttf")];

/// `--watch-door <seconds>`: follow the engine with no window at all, and print one line each
/// time its conversations change. A check that the live path works, for a script or an agent.
fn watch_door(args: &[String]) -> bool {
    let Some(seconds) = args.iter().position(|a| a == "--watch-door").and_then(|i| args.get(i + 1)).and_then(|s| s.parse::<u64>().ok()) else {
        return false;
    };
    let Source::Door { url, token } = source::from_args(args) else {
        eprintln!("--watch-door needs a door: --door and --token, or --door-file");
        std::process::exit(2);
    };
    let mailbox = Mailbox::default();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    source::follow_door(url.clone(), token.clone(), mailbox.clone(), move || {
        let _ = tx.send(());
    });
    let until = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    while let Some(left) = until.checked_duration_since(std::time::Instant::now()) {
        if rx.recv_timeout(left).is_err() {
            break;
        }
        let mut m = mailbox.lock().unwrap();
        if let Some(snapshot) = m.snapshot.take() {
            let lists = snapshot.lists(clock(), &Local::default());
            println!("changed: {} threads, {} need you, {} in progress", snapshot.items.len(), lists.needs.len(), lists.progress.len());
            // And one conversation's lines, the way opening it asks for them.
            if let Some(item) = snapshot.items.last() {
                let lines = source::call(&url, &token, "zero:item-history", &serde_json::json!({ "product": item.product, "id": item.id }));
                let count = lines.as_ref().ok().and_then(|v| v.get("lines")).and_then(Value::as_array).map(Vec::len);
                println!("the newest thread's conversation: {}", count.map_or("could not be read".to_string(), |n| format!("{n} lines")));
            }
        }
        if let Some(e) = m.error.take() {
            println!("could not read the engine: {e}");
        }
    }
    true
}

fn main() {
    // `--rows <out.json>` writes the lists a fixture gives and opens no window.
    if let Some(written) = rules::rows_from_args(&std::env::args().collect::<Vec<_>>()) {
        return written.unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1)
        });
    }
    if watch_door(rune::tooling::extra_args()) {
        return;
    }
    // The web app's own lists: Geist Mono has no ⌘, which the system's monospace then draws.
    let spec = AppSpec::<State> {
        fonts: FONTS,
        families: ("Geist", "Geist Mono, ui-monospace"),
        setup: Some(setup),
        poll: Some(poll),
        min_size: Some((720.0, 520.0)),
        ..AppSpec::new("Mercury", DEFAULT_SIZE, init, view)
    };
    // The web app sets `-webkit-font-smoothing: antialiased`; `Off` is that rendering.
    App::new(spec).font_smoothing(FontSmoothing::Off).run();
}
