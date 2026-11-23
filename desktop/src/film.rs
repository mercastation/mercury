//! What an agent does in its project's browser, shown in its conversation: the page as the
//! agent sees it, with Mercury's pointer where it presses and a balloon saying what. The
//! browser is hidden and signed in as the owner, so this is the one place they see what was
//! done there in their name. Its host keeps a film of each thread (`main/project-browser/
//! film.mts`); this window reads it off the disk and draws over its pictures, never inside the
//! page.
//!
//! Three places show it:
//! - while a run is up and has used the browser in the last moments, a card over the line at
//!   the conversation's foot: the newest picture, the pointer gliding to each press, its words;
//! - in a run's steps, a tile under the line its browser steps fold into, and each of those
//!   steps in the agent's own words (`clicked New issue`), which open the film there;
//! - the film opened big, over the window, step by step with ← and →, Escape back.
//!
//! WHAT IT COSTS. Nothing is read while no conversation of a project with a browser is open,
//! and a finished run's film is read once, when its conversation opens. While a run is up on
//! the open conversation the film's file is looked at every two seconds, and every quarter of a
//! second once the run is in the browser: one `stat` and the lines written since, off the main
//! thread. The live picture is one handle whose pixels are replaced (at most four a second, as
//! the host writes them), decoded off the main thread at the page's own size, so it costs one
//! picture's memory however long the run. The pictures of steps go through the conversation's
//! own picture shelf, with its budget (`pictures.rs`).

use std::collections::HashMap;
use std::io::{Read as _, Seek, SeekFrom};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use rune::{Cursor, El, Icon, Latest, Pixels, SurfaceFit, TextStyle, Tx, Waker, col, el, icon, row, text};
use serde_json::json;

use crate::pictures::{Look, Want};
use crate::rules::film::{self as rules, Step};
use crate::rules::live;
use crate::source::Source;
use crate::theme::*;
use crate::{State, View, icons};

/// How often the film is looked at while a run is in the browser, and while one is up but
/// has not used it lately.
const LIVE_EVERY: Duration = Duration::from_millis(250);
const QUIET_EVERY: Duration = Duration::from_secs(2);

/// The page's file names, as the host writes them.
const STEPS_FILE: &str = "steps.jsonl";
const LIVE_FILE: &str = "live.jpg";

/// A file's length and when it was last written.
type Stamp = (u64, u128);

fn stamp(path: &Path) -> Option<Stamp> {
    let m = std::fs::metadata(path).ok()?;
    Some((m.len(), m.modified().ok()?.duration_since(UNIX_EPOCH).ok()?.as_nanos()))
}

/// A thread's film folder, named as the host names it (`safe` in `film.mts`).
fn folder(root: &str, thread: &str) -> String {
    let mut safe: String = thread.chars().map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '_' }).collect();
    if safe.starts_with('.') {
        safe.replace_range(0..1, "_");
    }
    if safe.is_empty() {
        safe.push('_');
    }
    format!("{root}/{safe}")
}

/// One thread's film, as far as it has been read.
pub struct Log {
    /// `product/id`, as the conversation keys it.
    pub key: String,
    pub dir: String,
    pub steps: Vec<Step>,
    /// How far into the steps file has been read, to the end of its last whole line.
    offset: u64,
    /// The live picture last decoded.
    live: Option<Stamp>,
    /// When it was last asked for, on the window's clock in milliseconds.
    asked: f64,
}

/// What one look at the disk found.
struct Found {
    key: String,
    lines: String,
    offset: u64,
    live: Option<(Stamp, u32, u32, Vec<u8>)>,
}

/// The film opened big: which thread's, and which of its steps.
#[derive(Clone, PartialEq, Debug)]
pub struct Open {
    pub key: String,
    pub step: usize,
}

#[derive(Default)]
pub struct Films {
    /// Each project's films folder, from its browser's status (`zero:project-browser`), or none
    /// when the project has no browser.
    roots: HashMap<String, Option<String>>,
    root_read: Latest<(String, Option<String>)>,
    asking_root: Option<String>,
    pub log: Option<Log>,
    reading: Latest<Found>,
    busy: bool,
    /// The newest picture of the open thread's film, one handle whose pixels are replaced.
    pub live: Pixels,
    live_size: (u32, u32),
    pub open: Option<Open>,
}

impl Films {
    pub fn set_waker(&self, wake: Waker) {
        self.root_read.set_waker(wake.clone());
        self.reading.set_waker(wake);
    }
}

/// Reads what was written since `offset` up to the last whole line, and the live picture when
/// it changed and is wanted. Off the main thread.
fn look(key: String, dir: String, offset: u64, had_live: Option<Stamp>, want_live: bool) -> Found {
    let mut found = Found { key, lines: String::new(), offset, live: None };
    if let Ok(mut file) = std::fs::File::open(format!("{dir}/{STEPS_FILE}")) {
        let mut bytes = Vec::new();
        if file.seek(SeekFrom::Start(offset)).is_ok() && file.read_to_end(&mut bytes).is_ok() {
            if let Some(last) = bytes.iter().rposition(|b| *b == b'\n') {
                found.lines = String::from_utf8_lossy(&bytes[..=last]).into_owned();
                found.offset = offset + last as u64 + 1;
            }
        }
    }
    if want_live {
        let path = format!("{dir}/{LIVE_FILE}");
        let now = stamp(Path::new(&path));
        if now.is_some() && now != had_live {
            let picture = std::fs::read(&path).ok().and_then(|b| crate::pictures::decode(&b));
            if let (Some(now), Some(picture)) = (now, picture) {
                let rgba = picture.into_rgba8();
                found.live = Some((now, rgba.width(), rgba.height(), rgba.into_raw()));
            }
        }
    }
    found
}

/// What a look found, put where the page finds it. Says whether anything changed.
fn land(s: &mut State, found: Found) -> bool {
    let Some(log) = s.films.log.as_mut().filter(|l| l.key == found.key) else { return false };
    let mut changed = false;
    if found.offset != log.offset {
        rules::take_lines(&mut log.steps, &found.lines);
        log.offset = found.offset;
        changed = true;
    }
    if let Some((at, w, h, rgba)) = found.live {
        log.live = Some(at);
        s.films.live.set(w, h, rgba);
        s.films.live_size = (w, h);
        changed = true;
    }
    changed
}

/// The run up on the open thread, if any.
fn run_on(s: &State, id: &str, product: &str) -> Option<live::Session> {
    live::session_of(&s.snap.supervisor, id).filter(|run| run.product == product)
}

/// The window's turn: the open conversation's film is found, read as often as it can change,
/// and what was read lands. Says whether the page should be drawn again.
pub fn tend(s: &mut State, tx: &mut Tx) -> bool {
    let mut changed = false;
    if let Some((product, root)) = s.films.root_read.take() {
        s.films.roots.insert(product, root);
        s.films.asking_root = None;
        changed = true;
    }
    if let Some(found) = s.films.reading.take() {
        s.films.busy = false;
        changed |= land(s, found);
    }
    let View::Thread { id, product } = s.view.clone() else {
        if s.films.log.take().is_some() || s.films.open.take().is_some() {
            changed = true;
        }
        return changed;
    };
    let key = format!("{product}/{id}");
    if s.films.open.as_ref().is_some_and(|o| o.key != key) {
        s.films.open = None;
        changed = true;
    }
    // A picture of the window with no app behind it is told where the films are
    // (`MERCURY_FILM_ROOT`), as it is told everything else by its fixture.
    if !matches!(s.source, Source::Door(_)) && !s.films.roots.contains_key(&product) {
        let root = std::env::var("MERCURY_FILM_ROOT").ok().filter(|r| !r.is_empty());
        s.films.roots.insert(product.clone(), root);
    }
    // The project's films folder, asked of the app once.
    let Some(root) = s.films.roots.get(&product).cloned() else {
        if s.films.asking_root.is_none() {
            if let Source::Door(door) = &s.source {
                if let Some((url, token)) = door.now() {
                    s.films.asking_root = Some(product.clone());
                    let asked = product.clone();
                    s.films.root_read.request((), move |_| {
                        let answer = crate::source::call(&url, &token, "zero:project-browser", &json!({ "product": asked }));
                        let root = answer.ok().and_then(|v| v.get("film").and_then(|f| f.as_str()).map(str::to_string));
                        (asked, root)
                    });
                }
            }
        }
        return changed;
    };
    let Some(root) = root else {
        return changed;
    };
    if s.films.log.as_ref().is_none_or(|l| l.key != key) {
        s.films.log = Some(Log { key: key.clone(), dir: folder(&root, &id), steps: Vec::new(), offset: 0, live: None, asked: f64::NEG_INFINITY });
        s.films.live_size = (0, 0);
        s.films.busy = false;
        changed = true;
    }
    let run = run_on(s, &id, &product);
    let now = s.now;
    let log = s.films.log.as_ref().expect("just set");
    // A finished run's film is read once; one up is read again, often while it is in the browser.
    let every = match &run {
        None if log.asked.is_finite() => None,
        None => Some(Duration::ZERO),
        Some(r) if rules::live_step(&log.steps, r.started_at, now).is_some() => Some(LIVE_EVERY),
        Some(_) => Some(QUIET_EVERY),
    };
    let Some(every) = every else { return changed };
    if !every.is_zero() {
        s.wakes.every(tx, "the browser film", every);
    }
    if s.films.busy || now - log.asked < every.as_millis() as f64 - 5.0 {
        return changed;
    }
    let (dir, offset, had_live) = (log.dir.clone(), log.offset, log.live);
    let want_live = run.is_some();
    if let Some(log) = s.films.log.as_mut() {
        log.asked = now;
    }
    // A still picture cannot wait for a look off the main thread: it looks on the spot.
    if s.pictures.is_still() {
        let found = look(key, dir, offset, had_live, want_live);
        land(s, found);
        return true;
    }
    s.films.busy = true;
    s.films.reading.request((), move |_| look(key, dir, offset, had_live, want_live));
    changed
}

/// The open thread's film, when it is this thread's.
fn log_of<'a>(s: &'a State, key: &str) -> Option<&'a Log> {
    s.films.log.as_ref().filter(|l| l.key == key)
}

/// The steps a person is shown: the ones that did something to the page, not the ones that
/// only read it.
fn shown(steps: &[Step]) -> Vec<usize> {
    (0..steps.len()).filter(|&i| !steps[i].reads()).collect()
}

/* ------------------------------- the pointer and its words ------------------------------- */

/// Mercury's pointer: the accent with a white edge, so it reads on any page.
static POINTER: Icon = Icon {
    name: "m-film-pointer",
    view: 24.0,
    body: r##"<path d="M4.2 2.9 10.7 20.6l2.6-7.4 7.4-2.6z" fill="currentColor" stroke="#ffffff" stroke-width="1.6" stroke-linejoin="round"/>"##,
};

const BALLOON: f32 = 13.0;

fn balloon_font() -> TextStyle {
    sans(BALLOON, 500.0)
}

/// The balloon's words on the accent, as wide as they are.
fn balloon(words: &str, room_w: f32) -> (El, (f32, f32)) {
    let most = (room_w - 12.0).max(40.0);
    let wide = (rune::text::measure(words, &[], balloon_font(), None).0 + 24.0).min(most);
    let tall = 28.0;
    let b = row()
        .size(wide, tall)
        .px(12.0)
        .items_center()
        .radius(tall / 2.0)
        .fill(accent())
        .child(row().min_w(0.0).shrink().child(text(words.to_string()).font(balloon_font()).color(page()).truncate()));
    (b, (wide, tall))
}

/// The pointer and the balloon over a picture `room` large: the pointer at `tip` when the step
/// pressed somewhere, else the balloon alone at the picture's foot. `glide` lets the pointer
/// travel from where it was to where it is, as the live view's does.
fn over_picture(tip: Option<(f32, f32)>, words: &str, room: (f32, f32), glide: bool) -> Vec<El> {
    let (b, size) = balloon(words, room.0);
    let Some(tip) = tip else {
        return vec![b.absolute(12.0, (room.1 - size.1 - 12.0).max(0.0))];
    };
    let at = rules::balloon_at(tip, size, room);
    // The arrow's tip is the icon's top-left corner, a little in.
    let pointer = el().absolute(0.0, 0.0).size(24.0, 24.0).child(icon(&POINTER, 24.0).color(accent())).offset(tip.0 - 4.0, tip.1 - 3.0);
    let (pointer, b) = if glide {
        let travel = rune::Spring::new(0.45, 0.0);
        (
            pointer.key("film-pointer").animated_changes().offset_motion(travel),
            b.absolute(0.0, 0.0).offset(at.0, at.1).key("film-balloon").animated_changes().offset_motion(travel),
        )
    } else {
        (pointer, b.absolute(at.0, at.1))
    };
    vec![b, pointer]
}

/* ------------------------------------- the live card ------------------------------------- */

/// The card over the line at a conversation's foot while its run is in the browser: the page
/// now, the pointer where the agent presses, its words, and the site. A press opens the film
/// big at this step.
pub fn live_card(s: &State, id: &str, product: &str, column: f32) -> Option<El> {
    let key = format!("{product}/{id}");
    let log = log_of(s, &key)?;
    let run = run_on(s, id, product)?;
    let at = rules::live_step(&log.steps, run.started_at, s.now)?;
    let step = &log.steps[at];
    let wide = column.min(720.0);
    let view = step.view;
    let tall = (wide * view[1] / view[0].max(1.0)).round();
    let room = (wide - 2.0, tall);
    // The pointer stays where it last pressed while a step goes elsewhere (a page opening).
    let tip = log.steps[..=at].iter().rev().find_map(|s| s.point().map(|p| rules::on_picture(p, s.view, room.0)));
    let words = rules::balloon(step, step.running());
    let site = log.steps[..=at].iter().rev().map(Step::site).find(|s| !s.is_empty()).unwrap_or_default();
    let mut picture = el().size(room.0, room.1).clip_rounded(0.0);
    picture = if s.films.live_size.0 > 0 { picture.pixels_fit(&s.films.live, SurfaceFit::Fill) } else { picture.fill(rgb(47, 45, 44)) };
    // Over the page, the pointer only once there is a page under it.
    let marks = if s.films.live_size.0 > 0 { over_picture(if step.mark.is_some() || !step.running() { tip } else { None }, &words, room, true) } else { over_picture(None, &words, room, false) };
    let picture = picture.children(marks);
    let live = row().gap(6.0).items_center().child(el().size(7.0, 7.0).radius(3.5).fill(accent())).child(label(if step.running() { "Live" } else { "Just now" }, 12.0, 400.0, muted()));
    let head = row()
        .h(34.0)
        .px(12.0)
        .gap(8.0)
        .items_center()
        .child(icon(&icons::PANE_GLOBE, 14.0).color(muted()))
        .child(row().min_w(0.0).shrink().child(text(if site.is_empty() { "The project's browser".to_string() } else { site }).font(sans(13.0, 400.0)).color(dim()).truncate()))
        .child(el().grow(1.0))
        .child(live);
    let opened = key.clone();
    let card = col()
        .w(wide)
        .radius(10.0)
        .clip_rounded(10.0)
        .fill(rgba(40, 38, 37, 0.9))
        .border(1.0, edge())
        .border_layout()
        .child(head)
        .child(el().h(1.0).w(room.0).fill(edge()))
        .child(picture)
        .hover_group()
        .cursor(Cursor::Pointer)
        .help("Open what it did in the browser")
        .role("button")
        .label(format!("The agent in the browser: {words}"))
        .on_press(move |s: &mut State| open_at(s, &opened, at));
    Some(card)
}

/* --------------------------------- in a run's steps --------------------------------- */

/// What a run's lines are to the film: for each line, the step it is when it is the browser's.
pub fn steps_of_lines(s: &State, thread: &str, lines: &[(f64, &str)]) -> Vec<Option<usize>> {
    match log_of(s, thread) {
        Some(log) if lines.iter().any(|(_, v)| rules::is_browser_line(v)) => rules::match_lines(lines, &log.steps),
        _ => vec![None; lines.len()],
    }
}

/// A browser step's words in the conversation's own kind (`clicked`, `New issue`), when the
/// film has it.
pub fn line_words(s: &State, thread: &str, step: usize) -> Option<(String, String)> {
    log_of(s, thread)?.steps.get(step).map(|st| rules::words(st, false))
}

/// The tile under a run's line when its steps were in the browser: the last page it left, how
/// many steps and on which site, and a press opens them big from the first.
pub fn run_tile(s: &State, thread: &str, matched: &[Option<usize>]) -> Option<El> {
    let log = log_of(s, thread)?;
    let mine: Vec<usize> = matched.iter().flatten().copied().filter(|&i| !log.steps[i].reads()).collect();
    let (&first, &last) = (mine.first()?, mine.last()?);
    let (file, _) = rules::picture_of(&log.steps, last)?;
    let (wide, tall) = (112.0, 63.0);
    let places = vec![format!("{}/{file}", log.dir)];
    let want = Want::Cover(s.pictures.pixels(wide - 2.0), s.pictures.pixels(tall - 2.0));
    let frame = el().size(wide, tall).radius(6.0).border(1.0, edge());
    let frame = match s.pictures.look(places.clone(), want) {
        Look::Here(held) => frame.child(el().absolute(1.0, 1.0).size(wide - 2.0, tall - 2.0).radius(5.0).pixels_fit(&held.pixels, SurfaceFit::Fill)),
        _ => frame.fill(rgb(47, 45, 44)),
    };
    let frame = frame.on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on));
    let site = log.steps[first..=last].iter().rev().map(Step::site).find(|s| !s.is_empty()).unwrap_or_default();
    let count = mine.len();
    let said = if count == 1 { "1 step in the browser".to_string() } else { format!("{count} steps in the browser") };
    let words = col()
        .gap(2.0)
        .min_w(0.0)
        .shrink()
        .child(row().gap(6.0).items_center().child(icon(&icons::PLAY, 10.0).color(dim()).group_color(ink())).child(text(said).font(sans(13.0, 400.0)).color(dim()).group_color(ink())))
        .children((!site.is_empty()).then(|| text(site).font(sans(12.0, 400.0)).color(muted()).truncate()));
    let key = thread.to_string();
    Some(
        row()
            .mt(6.0)
            .gap(12.0)
            .items_center()
            .child(frame)
            .child(words)
            .hover_group()
            .cursor(Cursor::Pointer)
            .help("Watch what it did in the browser")
            .role("button")
            .on_press(move |s: &mut State| open_at(s, &key, first)),
    )
}

/// A browser step's line in a run's fold: its words, and a press that opens the film there.
pub fn step_line(verb: &str, subject: &str, failed: bool, thread: &str, step: usize) -> El {
    let stopped = rgb(232, 120, 104);
    let key = thread.to_string();
    row()
        .min_w(0.0)
        .items_start()
        .gap(8.0)
        .child(icon(&icons::PLAY, 8.0).color(muted()).group_color(dim()).mt(5.75))
        .child(text(verb.to_string()).font(TextStyle::new(13.0, 400.0).lh(19.5)).color(if failed { stopped } else { dim() }).group_color(ink()))
        .children((!subject.is_empty()).then(|| row().min_w(0.0).shrink().mt(1.0).child(text(subject.to_string()).font(mono(12.0, 18.0)).color(muted()).truncate())))
        .hover_group()
        .cursor(Cursor::Pointer)
        .role("button")
        .help("Watch this step")
        .on_press(move |s: &mut State| open_at(s, &key, step))
}

/* -------------------------------------- opened big -------------------------------------- */

fn open_at(s: &mut State, key: &str, step: usize) {
    s.films.open = Some(Open { key: key.to_string(), step });
}

fn step_by(s: &mut State, by: isize) {
    let Some(open) = s.films.open.clone() else { return };
    let Some(log) = log_of(s, &open.key) else { return };
    let list = shown(&log.steps);
    let at = list.iter().position(|&i| i >= open.step).unwrap_or(list.len().saturating_sub(1));
    let next = at as isize + by;
    if next < 0 || next as usize >= list.len() {
        return;
    }
    s.films.open = Some(Open { key: open.key, step: list[next as usize] });
}

/// The keys while the film is open big: Escape back, ← and → through its steps; every other
/// key but a chord is the film's, and the page under it gets none.
pub fn key(s: &mut State, e: &rune::KeyEvent, tx: &mut Tx) -> Option<bool> {
    use rune::edit::Key;
    s.films.open.as_ref()?;
    if e.mods.cmd || e.mods.ctrl {
        return Some(false);
    }
    match &e.key {
        Key::Escape => s.films.open = None,
        Key::Right => step_by(s, 1),
        Key::Left => step_by(s, -1),
        _ => {}
    }
    tx.instant();
    Some(true)
}

/// The bar over the film (`viewer.rs`'s own): back, the site, which step of how many, close.
const TOP: f32 = 44.0;
const FOOT: f32 = 52.0;

fn mark(glyph: &'static Icon, help: &'static str) -> El {
    row().size(28.0, 28.0).radius(6.0).center().hover_group().cursor(Cursor::Pointer).help(help).child(icon(glyph, 16.0).color(dim()).group_color(ink()))
}

/// The film opened big over the window.
pub fn layer(s: &State) -> Option<El> {
    let open = s.films.open.as_ref()?;
    let log = log_of(s, &open.key)?;
    let step = log.steps.get(open.step)?;
    let list = shown(&log.steps);
    let place = list.iter().position(|&i| i == open.step).unwrap_or(0);
    let (w, h) = s.size;
    let stage = (w - 120.0, (h - TOP - FOOT - 24.0).max(1.0));
    // The page fitted to the stage, at its own shape.
    // Never larger than the page was: its pictures are taken at its own size.
    let fit = (stage.0 / step.view[0].max(1.0)).min(stage.1 / step.view[1].max(1.0)).min(1.0);
    let room = ((step.view[0] * fit).round(), (step.view[1] * fit).round());
    let words = rules::balloon(step, false);
    let shown_at = rules::picture_of(&log.steps, open.step);
    // The address the picture was taken at: before a press, the page it pressed on.
    let at_url = if shown_at.as_ref().is_some_and(|(_, pointed)| *pointed) { step.url.clone() } else { step.ended_on().to_string() };
    let picture = match shown_at {
        Some((file, pointed)) => {
            let places = vec![format!("{}/{file}", log.dir)];
            let want = Want::Within(s.pictures.pixels(room.0), s.pictures.pixels(room.1));
            let frame = el().size(room.0, room.1).radius(8.0).clip_rounded(8.0).border(1.0, edge());
            let frame = match s.pictures.look(places.clone(), want) {
                Look::Here(held) => frame.pixels_fit(&held.pixels, SurfaceFit::Fill),
                _ => frame.fill(rgb(47, 45, 44)),
            };
            let tip = if pointed { step.point().map(|p| rules::on_picture(p, step.view, room.0)) } else { None };
            frame.children(over_picture(tip, &words, room, false)).on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on))
        }
        None => row().size(room.0, room.1).radius(8.0).border(1.0, edge()).center().child(label("No picture of this step was kept.", 14.0, 400.0, muted())),
    };
    let site = step.site();
    let bar = row()
        .absolute(0.0, 0.0)
        .size(w, TOP)
        .pl(84.0)
        .pr(10.0)
        .gap(14.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .window_drag()
        .child(mark(&icons::FILE_BACK, "Back to the thread (esc)").ml(-6.0).mr(-4.0).on_press(|s: &mut State| s.films.open = None))
        .child(label("The project's browser", 13.0, 400.0, ink()))
        .children((!site.is_empty()).then(|| label(&site, 13.0, 400.0, dim())))
        .child(el().grow(1.0))
        .child(text(format!("{} of {}", place + 1, list.len())).font(mono(11.5, 15.0).features("'tnum' 1")).color(dim()))
        .child(mark(&icons::FILE_CLOSE, "Close (esc)").on_press(|s: &mut State| s.films.open = None));
    let arrow = |glyph: &str, by: isize, x: f32, live: bool| {
        let b = row()
            .absolute(x, TOP + 12.0 + stage.1 / 2.0 - 22.0)
            .size(28.0, 44.0)
            .radius(6.0)
            .center()
            .fill(rgba(40, 38, 37, 0.7))
            .border(1.0, edge())
            .border_layout()
            .child(text(glyph.to_string()).font(TextStyle::new(18.0, 400.0).lh(18.0)).color(dim()).group_color(ink()));
        if live { b.hover_group().cursor(Cursor::Pointer).help(if by < 0 { "Step before (←)" } else { "Next step (→)" }).on_press(move |s: &mut State| step_by(s, by)) } else { b.opacity(0.0) }
    };
    let t = rune::time::LocalTime::at(step.at as i64);
    let when = format!("{}:{:02}:{:02}", if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 }, t.minute, t.second);
    let foot = row()
        .absolute(0.0, h - FOOT)
        .size(w, FOOT)
        .px(24.0)
        .gap(10.0)
        .items_center()
        .justify_center()
        .children(step.failed.then(|| label("This step failed", 14.0, 400.0, rgb(232, 120, 104))))
        .child(row().min_w(0.0).shrink().child(text(rules::address(&at_url)).font(sans(14.0, 400.0)).color(dim()).truncate()))
        .child(text(when).font(mono(12.0, 16.0)).color(muted()));
    let view = el()
        .absolute(0.0, 0.0)
        .size(w, h)
        .fill(page())
        .on_hover(|_: &mut State| {})
        .on_press(|_: &mut State| {})
        .child(row().absolute(60.0, TOP + 12.0).size(stage.0, stage.1).center().child(picture))
        .child(arrow("‹", -1, 16.0, place > 0))
        .child(arrow("›", 1, w - 16.0 - 28.0, place + 1 < list.len()))
        .child(foot)
        .child(bar);
    Some(view.role("dialog").label("What the agent did in the browser"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_threads_film_is_found_where_the_host_writes_it() {
        assert_eq!(folder("/s/.browser/oms/film", "w-1a2b"), "/s/.browser/oms/film/w-1a2b");
        assert_eq!(folder("/r", "a/b c"), "/r/a_b_c");
        assert_eq!(folder("/r", ".x"), "/r/_x");
    }

    #[test]
    fn a_look_reads_only_whole_lines_and_the_live_picture_when_it_changed() {
        let dir = std::env::temp_dir().join(format!("film-look-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let d = dir.to_string_lossy().to_string();
        std::fs::write(dir.join(STEPS_FILE), "{\"n\":1,\"tool\":\"browser_click\",\"at\":1}\n{\"n\":1,\"en").unwrap();
        let found = look("k".into(), d.clone(), 0, None, true);
        assert_eq!(found.lines.lines().count(), 1);
        assert!(found.live.is_none());
        // The cut line is read from its start next time, once whole.
        std::fs::write(dir.join(STEPS_FILE), "{\"n\":1,\"tool\":\"browser_click\",\"at\":1}\n{\"n\":1,\"end\":2}\n").unwrap();
        let more = look("k".into(), d.clone(), found.offset, None, false);
        assert_eq!(more.lines, "{\"n\":1,\"end\":2}\n");
        // A picture is decoded once, and not again while it has not changed.
        let mut png = Vec::new();
        image::RgbaImage::new(4, 3).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        std::fs::write(dir.join(LIVE_FILE), png).unwrap();
        let live = look("k".into(), d.clone(), 0, None, true).live.unwrap();
        assert_eq!((live.1, live.2), (4, 3));
        assert!(look("k".into(), d.clone(), 0, Some(live.0), true).live.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod cost {
    use super::*;

    /// What one live picture costs this window, on a real one (`MERCURY_FILM_DIR`): a
    /// measurement, run by hand with `--ignored --nocapture` in a release build.
    #[test]
    #[ignore]
    fn one_live_picture_read_and_decoded() {
        let Ok(dir) = std::env::var("MERCURY_FILM_DIR") else { return };
        let n = 40;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            assert!(look("k".into(), dir.clone(), 0, None, true).live.is_some());
        }
        let each = t0.elapsed().as_secs_f64() * 1000.0 / n as f64;
        let t1 = std::time::Instant::now();
        for _ in 0..n {
            let _ = look("k".into(), dir.clone(), 1 << 20, Some((0, 0)), false);
        }
        let quiet = t1.elapsed().as_secs_f64() * 1000.0 / n as f64;
        eprintln!("a live picture read and decoded: {each:.2} ms; a look that finds nothing new: {quiet:.3} ms");
    }
}
