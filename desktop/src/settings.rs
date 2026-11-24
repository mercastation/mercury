//! Settings, drawn here: a screen over everything, as the web window's is
//! (`renderer/src/components/Settings.tsx`, `settings.css`). Its menu takes the rail's place at
//! the rail's open width, Back to app and a search box over the pages in four groups, and the
//! page beside it is one centred column on the card, a title over headed cards of rows.
//!
//! The pages the person uses are drawn here (`PageId::native`); the rest are the web window's,
//! and a press on their row brings that window forward on that page through the desktop app's
//! door (`front::OPEN_SETTINGS` with the page's id).
//!
//! Nothing is cached, as in the web window: the settings are read when the screen opens and
//! every write answers with the settings the engine now believes, which the page redraws from.
//! A switch flips on the press and shows what was saved once the answer lands. Every read and
//! write goes off the main thread (`job`) and lands in `tend`.

mod agents;
mod kit;
mod marks;
mod pages;
mod projects;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use rune::edit::Key;
use rune::{Cursor, El, KeyEvent, ScrollSpec, TextEdit, TextStyle, Tx, Waker, col, el, icon, row, text, text_input};
use serde_json::{Value, json};

use crate::rules::settings::{self as rules, Group, PageId, Pane};
use crate::source::Source;
use crate::theme::*;
use crate::{State, front};

/// The rail's open width, which the menu takes (`--workspace-width: 202px` under
/// `.workspace-settings`), and where the card starts beside it.
const NAV_W: f32 = 202.0;
pub const CARD_LEFT: f32 = 22.0 + NAV_W + 22.0;

/// The page's column: at most 640 points of words inside 40 either side, in the pane's own 32
/// (`.set-inner`, `.set-pane`).
const COLUMN: f32 = 640.0;

/// What the screen holds while it is open, and nothing once it closes: the web window's screen
/// is unmounted when it closes, and opens again on General.
pub struct Screen {
    pub open: bool,
    pub pane: Pane,
    /// Each visit to a page starts at its top (`paneEl.scrollTo({ top: 0 })`): the scroller is
    /// a new one for each.
    visit: u32,
    /// The settings, as the engine last answered them.
    pub model: Option<Value>,
    /// A read or write the engine refused, in its own words (`.set-error`).
    pub error: Option<String>,
    pub search: TextEdit,
    /// The search's lit hit.
    pub hit_at: usize,
    /// The picker that is open, by its key on the page.
    pub menu: Option<String>,
    /// What a control was set to and the engine has not answered yet, by the control's key: a
    /// switch flips on the press (`Switch`'s `want`).
    pub wanted: HashMap<String, Value>,
    /// Buttons whose press is still out: Refresh, Check for updates, Tell it.
    pub busy: HashSet<String>,
    /// What the page's own reads answered (`zero:account-usage` and the rest).
    pub reads: Reads,
    /// The Projects page's own state (`ProjectsPage`).
    pub projects: projects::Ui,
    /// A box open on the page: a project's name, or the sites its agents may open.
    pub editing: Option<Editing>,
    /// A line a row says in place of its sentence after a press (a refusal, a key copied).
    pub said: HashMap<String, String>,
    /// The pickers drawn this frame, by key (`kit::away`).
    drawn: RefCell<Vec<String>>,
    /// How far down the page a picture was taken, scrolled to on the first frame.
    scroll: Option<f32>,
    /// The group a search hit names, brought to the middle of the page once it is drawn.
    landing: Option<String>,
    /// Whether the page now open has been laid out, as the view last found it.
    laid_out: std::cell::Cell<bool>,
    inbox: Arc<Mutex<Vec<Arrived>>>,
    waker: Option<Waker>,
}

/// What the page's own reads answered, kept until the screen closes.
#[derive(Default)]
pub struct Reads {
    /// Each Claude Code login's own usage (`accountReadings`).
    pub accounts: Vec<Value>,
    /// Each Codex login's, and the reading Refresh fetched (`codexReadings`, `codexReading`).
    pub codex: Vec<Value>,
    pub codex_now: Option<Value>,
    /// The cloud plan (`zero:cloud-account-plan`) and what is kept in the cloud
    /// (`zero:files-cloud`).
    pub plan: Option<Value>,
    pub files: Option<Value>,
    /// A project's branches and its browser, by its slug.
    pub branches: Option<(String, Value)>,
    pub browser: Option<(String, Value)>,
    /// What the update check answered, with the snapshot's update it was asked against: the
    /// snapshot wins once it moves (`Updates`).
    pub update: Option<(String, Value)>,
}

/// A box open on the page, its words and what it edits.
pub struct Editing {
    pub what: Edits,
    pub words: TextEdit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Edits {
    /// The project's name, in place of its title.
    Name(String),
    /// The sites a project's agents may open.
    Sites(String),
}

/// What a job's answer is, for `tend` to put where it goes.
#[derive(Clone, Debug)]
enum What {
    /// The settings themselves, read.
    Model,
    /// A write the engine answers with the settings; `key` is the control that waits on it.
    Wrote(String),
    Accounts,
    /// Codex's reading and each login's: `{ reading, readings }`.
    Codex,
    Plan,
    Files,
    Branches(String),
    Browser(String),
    /// The update check, stamped with the snapshot's update it was asked against.
    Update(String),
    /// A press whose busy mark comes off when it lands, the settings read after it.
    Done(String),
    /// A press that answers nothing the page draws, said only when refused.
    Quiet,
}

struct Arrived {
    what: What,
    answer: Result<Value, String>,
}

impl Default for Screen {
    fn default() -> Screen {
        Screen {
            open: false,
            pane: Pane::Page(PageId::General),
            visit: 0,
            model: None,
            error: None,
            search: TextEdit::new(""),
            hit_at: 0,
            menu: None,
            wanted: HashMap::new(),
            busy: HashSet::new(),
            reads: Reads::default(),
            projects: projects::Ui::default(),
            editing: None,
            said: HashMap::new(),
            drawn: RefCell::new(Vec::new()),
            scroll: None,
            landing: None,
            laid_out: std::cell::Cell::new(false),
            inbox: Arc::new(Mutex::new(Vec::new())),
            waker: None,
        }
    }
}

impl Screen {
    pub fn wake_with(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }
    /// The workspace's settings, when they have been read.
    pub fn workspace(&self) -> Option<&Value> {
        self.model.as_ref().and_then(|m| m.get("workspace")).filter(|w| w.is_object())
    }
    /// The projects the settings know, in their order.
    pub fn projects(&self) -> &[Value] {
        self.model.as_ref().map_or(&[], |m| rules::list(m, "projects"))
    }
    /// The project whose page is open.
    pub fn current(&self) -> Option<&Value> {
        let Pane::Project(slug) = &self.pane else { return None };
        self.projects().iter().find(|p| rules::text(p, "slug") == Some(slug.as_str()))
    }
    /// What a control shows: what was just set on it, else what the settings say.
    pub fn shown<'a>(&'a self, key: &str, saved: &'a Value) -> &'a Value {
        self.wanted.get(key).unwrap_or(saved)
    }
}

/* -------------------------------- open and close -------------------------------- */

/// Opens the screen on a page, as the rail's Settings and the person's own row do (the web
/// window's cog lands on General).
pub fn open(s: &mut State, page: PageId, tx: &mut Tx) {
    let waker = s.settings.waker.take();
    s.settings = Screen { open: true, waker, ..Screen::default() };
    // One thing stands over the page at a time.
    s.close_palette();
    s.close_later();
    s.fold_reply();
    s.compose.menu = None;
    s.compose.card = false;
    read(s, What::Model);
    go(s, Pane::Page(page), tx);
}

/// Back to the app, onto the page it was opened over.
pub fn close(s: &mut State) {
    let waker = s.settings.waker.take();
    s.settings = Screen { waker, ..Screen::default() };
}

/// Another page, as a press on its row or a search hit opens it. A page this window does not
/// draw is the web window's: it comes forward on it, and this screen stays where it was.
pub fn go(s: &mut State, pane: Pane, tx: &mut Tx) {
    if let Pane::Page(page) = &pane {
        if !page.native() {
            bridge(s, *page, tx);
            return;
        }
    }
    let screen = &mut s.settings;
    screen.menu = None;
    screen.editing = None;
    screen.said.clear();
    if screen.pane != pane {
        screen.pane = pane.clone();
        screen.visit += 1;
        screen.projects = projects::Ui::default();
    }
    // What the page reads for itself when it opens, as the web window's effects do.
    match &pane {
        Pane::Page(PageId::Claude) => read(s, What::Accounts),
        Pane::Page(PageId::Codex) => read(s, What::Codex),
        Pane::Page(PageId::Cloud) => read(s, What::Plan),
        Pane::Page(PageId::Files) => read(s, What::Files),
        Pane::Project(slug) => {
            let slug = slug.clone();
            read(s, What::Branches(slug.clone()));
            read(s, What::Browser(slug));
        }
        _ => {}
    }
}

/// The web window comes forward on one of its own pages.
pub fn bridge(s: &mut State, page: PageId, tx: &Tx) {
    let refused = "Mercury's other window is not open now, so this page cannot be shown.";
    match &s.source {
        Source::Door(door) => front::ask_the_app_with(door, front::OPEN_SETTINGS, json!({ "page": page.id() }), refused),
        _ => s.tell(refused, tx),
    }
}

/// A picture of the screen, as its fixture says it was taken: the page, the settings and what
/// the page's reads were answered (`tools/scenes/settings.mjs`).
pub fn picture(s: &mut State) {
    let Some(page) = s.fixture.get("settingsPage").and_then(Value::as_str).map(str::to_string) else {
        return;
    };
    let pane = match page.split_once(':') {
        Some((_, slug)) => Pane::Project(slug.to_string()),
        None => Pane::Page(PageId::from_id(&page).unwrap_or(PageId::General)),
    };
    let screen = &mut s.settings;
    screen.open = true;
    screen.pane = pane;
    screen.model = s.fixture.get("settings").cloned();
    let answers = s.fixture.get("answers").cloned().unwrap_or(Value::Null);
    let part = |key: &str| answers.get(key).cloned();
    screen.reads.accounts = part("accountUsage").map(|a| rules::list(&a, "readings").to_vec()).unwrap_or_default();
    screen.reads.codex = part("codexUsage").map(|a| rules::list(&a, "readings").to_vec()).unwrap_or_default();
    screen.reads.plan = part("cloudAccountPlan");
    screen.reads.files = part("filesCloud");
    if let Pane::Project(slug) = &screen.pane {
        screen.reads.branches = part("projectBranches").map(|b| (slug.clone(), b));
        screen.reads.browser = part("projectBrowser").map(|b| (slug.clone(), b));
    }
    screen.menu = s.fixture.get("picker").and_then(Value::as_str).map(str::to_string);
    screen.scroll = s.fixture.get("scrolled").and_then(Value::as_f64).map(|y| y as f32);
    if let Some(words) = s.fixture.get("search").and_then(Value::as_str) {
        screen.search = TextEdit::new(words);
        screen.search.focused = true;
    }
}

/* ------------------------------- reads and writes ------------------------------- */

/// Runs `work` against the engine's door off the main thread; its answer lands in `tend`. A
/// window with no door (a fixture) has nobody to ask and asks nothing.
fn job(s: &mut State, what: What, work: impl FnOnce(&str, &str) -> Result<Value, String> + Send + 'static) -> bool {
    let Source::Door(door) = &s.source else {
        return false;
    };
    let Some((url, token)) = door.now() else {
        s.settings.error = Some("Mercury is not running, so its settings cannot be read.".to_string());
        return false;
    };
    let (inbox, waker) = (s.settings.inbox.clone(), s.settings.waker.clone());
    std::thread::spawn(move || {
        let answer = work(&url, &token);
        if let Ok(mut arrived) = inbox.lock() {
            arrived.push(Arrived { what, answer });
        }
        if let Some(wake) = waker {
            wake();
        }
    });
    true
}

/// One call to a channel.
fn call(url: &str, token: &str, channel: &str, payload: Value) -> Result<Value, String> {
    crate::source::call(url, token, channel, &payload)
}

/// One of the page's reads.
fn read(s: &mut State, what: What) {
    let ask: (&'static str, Value) = match &what {
        // As the web window asks, which looks for the coding agents again.
        What::Model => ("zero:settings-read", Value::Null),
        What::Accounts => ("zero:account-usage", json!({ "refresh": false })),
        What::Plan => ("zero:cloud-account-plan", Value::Null),
        What::Files => ("zero:files-cloud", Value::Null),
        What::Branches(slug) => ("zero:project-branches", json!({ "product": slug })),
        What::Browser(slug) => {
            // A project with no site listed has no browser to ask about.
            let listed = s.settings.projects().iter().find(|p| rules::text(p, "slug") == Some(slug.as_str())).is_some_and(|p| !rules::list(p, "browserSites").is_empty());
            if !listed {
                return;
            }
            ("zero:project-browser", json!({ "product": slug }))
        }
        What::Codex => {
            // Opening the page reads what this computer has; only Refresh asks Codex itself.
            job(s, What::Codex, |url, token| codex_reads(url, token, false));
            return;
        }
        _ => return,
    };
    job(s, what, move |url, token| call(url, token, ask.0, ask.1));
}

/// Codex's reading and each of its logins', in one answer.
fn codex_reads(url: &str, token: &str, ask: bool) -> Result<Value, String> {
    let reading = call(url, token, "zero:refresh-usage", json!({ "engine": "codex", "ask": ask })).unwrap_or(Value::Null);
    let readings = call(url, token, "zero:account-usage", json!({ "engine": "codex" }))?;
    Ok(json!({ "reading": reading, "readings": readings.get("readings").cloned().unwrap_or(json!([])) }))
}

/// A workspace setting, written (`setWorkspaceSetting`): `key` is the control that shows it
/// meanwhile. The answer is the whole settings, which the page then draws.
pub fn write_workspace(s: &mut State, control: &str, key: &str, value: Value) {
    s.settings.wanted.insert(control.to_string(), value.clone());
    let payload = json!({ "key": key, "value": value });
    job(s, What::Wrote(control.to_string()), move |url, token| call(url, token, "zero:settings-set-workspace", payload));
}

/// A project's own setting, written (`setProjectSetting`).
pub fn write_project(s: &mut State, control: &str, product: &str, key: &str, value: Value) {
    s.settings.wanted.insert(control.to_string(), value.clone());
    let payload = json!({ "product": product, "key": key, "value": value });
    job(s, What::Wrote(control.to_string()), move |url, token| call(url, token, "zero:settings-set-project", payload));
}

/// A write on another channel that also answers with the settings: a project renamed, archived
/// or brought back (`zero:project-rename`, `zero:project-archive`).
pub fn write_other(s: &mut State, control: &str, channel: &'static str, payload: Value) {
    job(s, What::Wrote(control.to_string()), move |url, token| call(url, token, channel, payload));
}

/// A press with its button busy until it lands, after which the settings are read again:
/// `steps` are the calls, made in order.
pub fn press(s: &mut State, busy: &str, steps: Vec<(&'static str, Value)>) {
    if !s.settings.busy.insert(busy.to_string()) {
        return;
    }
    let started = job(s, What::Done(busy.to_string()), move |url, token| {
        let mut last = Value::Null;
        for (channel, payload) in steps {
            last = call(url, token, channel, payload)?;
        }
        Ok(last)
    });
    if !started {
        s.settings.busy.remove(busy);
    }
}

/// A press that changes nothing the page draws: said only when it is refused.
pub fn quiet(s: &mut State, channel: &'static str, payload: Value) {
    job(s, What::Quiet, move |url, token| call(url, token, channel, payload));
}

/// Refresh on Claude Code's page: every login's usage read again, then the settings, since a
/// reading can take back a stale mark (`readAccounts(true)`).
pub fn refresh_accounts(s: &mut State) {
    if !s.settings.busy.insert("refresh".to_string()) {
        return;
    }
    if !job(s, What::Accounts, |url, token| call(url, token, "zero:account-usage", json!({ "refresh": true }))) {
        s.settings.busy.remove("refresh");
    }
}

/// Refresh on Codex's page, which asks Codex itself (`refreshCodex(true)`).
pub fn refresh_codex(s: &mut State) {
    if !s.settings.busy.insert("refresh".to_string()) {
        return;
    }
    if !job(s, What::Codex, |url, token| codex_reads(url, token, true)) {
        s.settings.busy.remove("refresh");
    }
}

/// A project's browser opened for its owner to sign in, or closed once they are done
/// (`zero:project-browser`): its button waits on the answer, which is the browser as it stands.
pub fn browser_press(s: &mut State, slug: &str, action: &'static str) {
    if !s.settings.busy.insert("browser".to_string()) {
        return;
    }
    let payload = json!({ "product": slug, "action": action });
    if !job(s, What::Browser(slug.to_string()), move |url, token| call(url, token, "zero:project-browser", payload)) {
        s.settings.busy.remove("browser");
    }
}

/// Check for updates: the check's own answer is drawn until the snapshot's update moves.
pub fn check_for_updates(s: &mut State) {
    let was = s.snap.rest.get("update").map(Value::to_string).unwrap_or_default();
    if !s.settings.busy.insert("update".to_string()) {
        return;
    }
    if !job(s, What::Update(was), |url, token| call(url, token, "zero:update-check", Value::Null)) {
        s.settings.busy.remove("update");
    }
}

/// The poll's part: what the engine answered lands where it goes. True when anything did.
pub fn tend(s: &mut State, tx: &mut Tx) -> bool {
    // A picture taken down the page, and a search hit's group, once the page is laid out.
    let pane = format!("settings-pane-{}", s.settings.visit);
    // Only the view can read the layout, so it says whether the page it drew is laid out yet.
    let laid_out = s.settings.laid_out.get();
    let mut scrolled = false;
    if laid_out && let Some(y) = s.settings.scroll.take() {
        tx.scroll_to_instant(pane.clone(), 0.0, y);
        scrolled = true;
    }
    if laid_out && let Some(anchor) = s.settings.landing.take() {
        tx.scroll_center_in_instant(pane, format!("find-{anchor}"));
        scrolled = true;
    }
    if s.settings.scroll.is_some() || s.settings.landing.is_some() {
        tx.wake_after(std::time::Duration::from_millis(16));
    }
    // A project's name box that lost the keyboard to a press elsewhere keeps what was typed, as
    // the web window's saves on blur. The field says so in its model.
    if s.settings.editing.as_ref().is_some_and(|e| matches!(e.what, Edits::Name(_)) && !e.words.focused) {
        projects::rename(s, tx);
        scrolled = true;
    }
    // The cloud's switch reads the snapshot, which learns of the change after the write: it
    // shows what was set until the snapshot says the same.
    if let Some(want) = s.settings.wanted.get("sw:cloud").and_then(Value::as_bool) {
        let on = s.snap.rest.get("cloud").and_then(|c| rules::text(c, "mode")).is_some_and(|m| m != "mac");
        if on == want && !s.settings.busy.contains("cloud-runs") {
            s.settings.wanted.remove("sw:cloud");
        }
    }
    let arrived: Vec<Arrived> = s.settings.inbox.lock().map(|mut a| std::mem::take(&mut *a)).unwrap_or_default();
    if arrived.is_empty() || !s.settings.open {
        return scrolled || !arrived.is_empty();
    }
    for Arrived { what, answer } in arrived {
        land(s, what, answer);
    }
    true
}

/// Settings as an answer gives them: the page draws them, or says why they did not come.
fn take_model(s: &mut State, answer: &Value) {
    if answer.get("ok") == Some(&Value::Bool(false)) {
        s.settings.error = Some(rules::text(answer, "error").unwrap_or("could not save that").to_string());
        return;
    }
    s.settings.model = Some(answer.clone());
    s.settings.error = None;
}

fn land(s: &mut State, what: What, answer: Result<Value, String>) {
    match what {
        What::Model => match answer {
            Ok(model) => {
                if model.get("ok") == Some(&Value::Bool(false)) {
                    s.settings.error = Some(rules::text(&model, "error").unwrap_or("could not read settings").to_string());
                }
                if s.settings.model.is_none() || model.get("ok") != Some(&Value::Bool(false)) {
                    s.settings.model = Some(model);
                }
            }
            Err(e) => s.settings.error = Some(e),
        },
        What::Wrote(control) => {
            s.settings.wanted.remove(&control);
            match answer {
                Ok(model) => take_model(s, &model),
                Err(e) => s.settings.error = Some(e),
            }
        }
        What::Accounts => {
            let refreshed = s.settings.busy.remove("refresh");
            if let Ok(answer) = answer {
                let readings = rules::list(&answer, "readings").to_vec();
                // A reading can take back a stale mark, so the settings are read again after one.
                if !readings.is_empty() || refreshed {
                    read(s, What::Model);
                }
                s.settings.reads.accounts = readings;
            }
        }
        What::Codex => {
            s.settings.busy.remove("refresh");
            if let Ok(answer) = answer {
                let reading = answer.get("reading").filter(|r| r.get("ok") == Some(&Value::Bool(true))).and_then(|r| r.get("reading")).cloned();
                if reading.is_some() {
                    s.settings.reads.codex_now = reading;
                }
                s.settings.reads.codex = rules::list(&answer, "readings").to_vec();
            }
        }
        What::Plan => s.settings.reads.plan = answer.ok(),
        What::Files => s.settings.reads.files = answer.ok(),
        What::Branches(slug) => s.settings.reads.branches = answer.ok().map(|b| (slug, b)),
        What::Browser(slug) => {
            s.settings.busy.remove("browser");
            s.settings.reads.browser = answer.ok().map(|b| (slug, b));
        }
        What::Update(was) => {
            s.settings.busy.remove("update");
            if let Ok(got) = answer {
                s.settings.reads.update = Some((was, got));
            }
        }
        What::Done(busy) => {
            s.settings.busy.remove(&busy);
            match answer {
                // A write that answers with the settings is drawn from them; any other is
                // followed by a read of them.
                Ok(answer) if answer.get("workspace").is_some() => take_model(s, &answer),
                Ok(answer) if answer.get("ok") == Some(&Value::Bool(false)) => {
                    s.settings.error = Some(rules::text(&answer, "error").or_else(|| rules::text(&answer, "message")).unwrap_or("could not do that").to_string());
                }
                Ok(_) => read(s, What::Model),
                Err(e) => s.settings.error = Some(e),
            }
        }
        What::Quiet => match answer {
            Ok(answer) if answer.get("ok") == Some(&Value::Bool(false)) => {
                s.settings.error = Some(rules::text(&answer, "error").or_else(|| rules::text(&answer, "message")).unwrap_or("could not do that").to_string());
            }
            Err(e) => s.settings.error = Some(e),
            _ => {}
        },
    }
}

/* ------------------------------------- keys ------------------------------------- */

/// The keys while the screen is up: it owns the keyboard, as the web window's does, whose own
/// keys stand down while it is open (`if (settingsOpen) return`). None while it is closed.
pub fn key(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> Option<bool> {
    if !s.settings.open {
        return None;
    }
    // ⌘F, or / from anywhere that is not a box being typed in, goes to the search.
    let slash = e.key == Key::Text("/".into()) && !e.mods.cmd && !e.mods.ctrl;
    if e.is_command('f') || (slash && s.settings.editing.is_none() && !s.settings.projects.filter.focused) {
        s.settings.menu = None;
        let search = &mut s.settings.search;
        search.focused = true;
        (search.anchor, search.focus) = (0, search.text.len());
        tx.focus("settings-search", false);
        return Some(true);
    }
    if e.mods.alt && !e.mods.cmd && !e.mods.ctrl && matches!(e.key, Key::Up | Key::Down) && projects::nudge_held(s, e.key == Key::Up, tx) {
        return Some(true);
    }
    if e.mods.cmd || e.mods.ctrl || e.mods.alt {
        return Some(false);
    }
    if e.key == Key::Escape {
        // An open menu takes Escape first; then a box being edited, whose own keys put it back;
        // then the screen closes (`whatTakesEscape`).
        if s.settings.menu.take().is_some() {
            return Some(true);
        }
        if s.settings.editing.take().is_some() {
            return Some(true);
        }
        close(s);
        return Some(true);
    }
    // Nothing behind the screen hears a key.
    Some(true)
}

/// The search box's own keys (`searchKeys`): Escape empties it first, the arrows walk the hits
/// and Enter opens the lit one.
fn search_keys(s: &mut State, e: &KeyEvent, tx: &mut Tx) -> bool {
    if e.key == Key::Escape {
        if s.settings.search.text.is_empty() {
            close(s);
        } else {
            s.settings.search = TextEdit::new("");
            s.settings.search.focused = true;
            s.settings.hit_at = 0;
        }
        return true;
    }
    let hits = hits(s);
    if hits.is_empty() {
        return false;
    }
    match e.key {
        Key::Down => s.settings.hit_at = (s.settings.hit_at + 1).min(hits.len() - 1),
        Key::Up => s.settings.hit_at = s.settings.hit_at.saturating_sub(1),
        Key::Enter => {
            let hit = hits.get(s.settings.hit_at).or(hits.first()).cloned();
            if let Some(hit) = hit {
                land_on(s, hit, tx);
                s.settings.search.focused = false;
            }
        }
        _ => return false,
    }
    true
}

/// Where a hit lands: the page, scrolled to the group it names (`go`).
fn land_on(s: &mut State, hit: rules::Hit, tx: &mut Tx) {
    s.settings.search = TextEdit::new("");
    s.settings.hit_at = 0;
    go(s, hit.pane, tx);
    // The page lands on the group the hit names, in its middle, once it is drawn.
    s.settings.landing = hit.anchor.map(str::to_string);
    tx.wake_after(std::time::Duration::from_millis(16));
}

/// The pages this Mac has, as the web window's menu lists them.
pub fn pages(s: &State) -> Vec<PageId> {
    let part = |key: &str| s.snap.rest.get(key).filter(|v| v.is_object());
    let team = part("team").is_some_and(|t| rules::on(t, "configured"));
    let cloud = part("cloud").is_some_and(|c| rules::on(c, "managed") || rules::on(c, "offered"));
    let rooms = part("alpha").is_some_and(|a| rules::on(a, "rooms")) && part("team").and_then(|t| t.get("rooms")).and_then(|r| r.get("me")).is_some();
    rules::pages_shown(team, cloud, rooms)
}

/// What the typed words find.
fn hits(s: &State) -> Vec<rules::Hit> {
    let projects: Vec<(String, String)> = s.settings.projects().iter().map(|p| (rules::text(p, "slug").unwrap_or("").to_string(), rules::text(p, "name").unwrap_or("").to_string())).collect();
    rules::search(&s.settings.search.text, &pages(s), &projects)
}

/* ------------------------------------ drawn ------------------------------------- */

/// The screen, over the dotted page: the card, the menu where the rail was, and the page on the
/// card. The rail, the head strip and the page under it are not drawn while it is up
/// (`.workspace-settings`).
pub fn view(s: &State) -> Vec<El> {
    let (w, h) = s.size;
    s.settings.drawn.borrow_mut().clear();
    let anchor = s.settings.landing.as_ref().map(|a| format!("find-{a}"));
    s.settings.laid_out.set(rune::memo::window_layout(&format!("settings-pane-{}", s.settings.visit)).is_some() && anchor.is_none_or(|a| rune::memo::window_layout(&a).is_some()));
    let card = el()
        .pin(Some(CARD_LEFT), Some(crate::chrome::CARD[1]), Some(crate::chrome::CARD[2]), Some(crate::chrome::CARD[3]))
        .radius(6.0)
        .fill(rgba(36, 34, 33, 0.55))
        .box_shadow(shadow(0.0, 40.0, 120.0, 0.0, rgba(0, 0, 0, 0.7), false))
        .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, edge(), true));
    let mut layers = vec![card, nav(s, h), pane(s, w)];
    // Under an open picker lies a layer the size of the window: a press anywhere else closes it
    // on the way down, as the web window's `mousedown` listener does.
    if s.settings.menu.is_some() {
        layers.push(kit::away(s));
    }
    layers
}

/// The menu, in the rail's place (`.set-nav`).
fn nav(s: &State, h: f32) -> El {
    // The menu's rows, its back row and its search are the app sidebar's, so opening Settings
    // does not change how the side of the window looks (`.set-nav-back`, `.set-search`).
    let back = row()
        .h(36.0)
        .px(10.0)
        .gap(11.0)
        .radius(6.0)
        .items_center()
        .cursor(Cursor::Pointer)
        .hover_group()
        .hover(hovered())
        .help("Back to app (esc)")
        // Under the pointer its mark and words come up to the text's ink; esc keeps its own.
        .child(icon(&marks::BACK, 18.0).color(muted()).group_color(ink()))
        .child(row().grow(1.0).child(pen(text("Back to app").font(sans(13.0, 400.0)).color(muted()).group_color(ink()), 13.0)))
        .child(pen(text("esc").font(sans(12.0, 400.0).tracking(0.24)).color(muted()), 12.0).opacity(0.7))
        .on_press(|s: &mut State| close(s));
    let field = text_input(&s.settings.search, sans(13.0, 400.0))
        .key("settings-search")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder("Search", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| {
            s.settings.hit_at = 0;
            &mut s.settings.search
        })
        .on_input_key(search_keys);
    // A row like the ones under it, with no box of its own.
    let search = row()
        .mt(6.0)
        .h(32.0)
        .px(10.0)
        .gap(11.0)
        .items_center()
        .cursor(Cursor::Text)
        .child(icon(&marks::SEARCH, 18.0).color(muted()))
        .child(pen(field, 13.0))
        .on_press_tx(|s: &mut State, tx| {
            s.settings.search.focused = true;
            tx.focus("settings-search", false);
        });
    // The strip moves the window, as the title bar it stands under does; its button and its
    // box keep their presses (`.set-nav-top`).
    let top = col().pb(6.0).window_drag().child(back).child(search);
    let list = if s.settings.search.text.trim().is_empty() { menu_rows(s) } else { hit_rows(s) };
    col()
        .absolute(22.0, 50.0)
        .w(NAV_W)
        .h((h - 72.0).max(1.0))
        .child(top)
        .child(col().grow(1.0).min_h(0.0).scroll(ScrollSpec { stick_bottom: false }).child(col().w(NAV_W).pb(14.0).children(list)))
}

/// Whether a page's row is the one lit: the page open, or Projects while a project's own is.
fn lit(s: &State, page: PageId) -> bool {
    match &s.settings.pane {
        Pane::Page(p) => *p == page,
        Pane::Project(_) => page == PageId::Projects,
    }
}

/// The groups and their pages (`.set-nav-group`, `.set-nav-item`), and the project open under
/// Projects (`.set-nav-sub`).
fn menu_rows(s: &State) -> Vec<El> {
    let shown = pages(s);
    let mut rows = Vec::new();
    for group in Group::ALL {
        let mine: Vec<PageId> = shown.iter().copied().filter(|p| p.group() == group).collect();
        if mine.is_empty() {
            continue;
        }
        rows.push(el().pt(20.0).px(10.0).pb(6.0).child(pen(text(group.label()).font(sans(11.5, 500.0)).color(muted()), 11.5)));
        for (at, page) in mine.into_iter().enumerate() {
            let on = lit(s, page);
            // The mark takes the row's ink, as the words do.
            let mark = icon(marks::page(page), 18.0).color(if on { ink() } else { muted() }).group_color(ink());
            let item = nav_item(mark, page.label(), on, false).on_press_tx(move |s: &mut State, tx| go(s, Pane::Page(page), tx));
            rows.push(if at > 0 { item.mt(3.0) } else { item });
            if page == PageId::Projects {
                if let Some(p) = s.settings.current() {
                    let slug = rules::text(p, "slug").unwrap_or("").to_string();
                    let name = rules::text(p, "name").unwrap_or("").to_string();
                    let mark = projects::mark(&slug, 14.0);
                    rows.push(nav_item(mark, &name, true, true).mt(3.0).on_press_tx(move |s: &mut State, tx| go(s, Pane::Project(slug.clone()), tx)));
                }
            }
        }
    }
    rows
}

/// One row of the menu: its mark and its word, washed and in the text's colour while lit and
/// under the pointer. A project's row, under Projects, stands a step in and is not washed
/// while lit, only under the pointer.
fn nav_item(mark: El, word: &str, on: bool, sub: bool) -> El {
    let tone = if on { ink() } else { muted() };
    let b = row().h(36.0).pl(if sub { 24.0 } else { 10.0 }).pr(10.0).gap(11.0).radius(6.0).items_center().cursor(Cursor::Pointer).hover_group().hover(white(0.04));
    let b = if on && !sub { b.fill(white(0.04)) } else { b };
    b.child(row().size(18.0, 18.0).center().child(mark))
        .child(row().grow(1.0).min_w(0.0).child(pen(text(word).font(sans(13.0, if on { 500.0 } else { 400.0 })).color(tone).group_color(ink()).truncate(), 13.0)))
}

/// What the search found, in place of the menu (`.set-hit`), or the line saying nothing did.
fn hit_rows(s: &State) -> Vec<El> {
    let found = hits(s);
    if found.is_empty() {
        let said = format!("Nothing matches “{}”.", s.settings.search.text.trim());
        return vec![col().py(12.0).px(10.0).child(pen(text(said).font(TextStyle::new(12.5, 400.0).lh(18.75)).color(muted()).wrap(), 12.5))];
    }
    found
        .into_iter()
        .enumerate()
        .map(|(at, hit)| {
            let lit = at == s.settings.hit_at;
            let words = col()
                .gap(1.0)
                .child(pen(text(hit.label.as_str()).font(TextStyle::new(13.0, if lit { 500.0 } else { 400.0 }).lh(kept(17.55))).color(ink()).wrap(), 13.0))
                .child(pen(text(hit.page_label).font(TextStyle::new(11.5, if lit { 500.0 } else { 400.0 }).lh(kept(15.525))).color(muted()), 11.5));
            let b = col().min_h(44.0).py(6.0).px(10.0).radius(6.0).justify_center().cursor(Cursor::Pointer).child(words);
            let b = if lit { b.fill(white(0.04)) } else { b.hover(white(0.04)) };
            let b = if at > 0 { b.mt(3.0) } else { b };
            b.on_hover_change(move |s: &mut State, inside, tx| {
                if inside {
                    s.settings.hit_at = at;
                } else {
                    tx.live_only();
                }
            })
            .on_press_tx(move |s: &mut State, tx| land_on(s, hit.clone(), tx))
        })
        .collect()
}

/// The page on the card, scrolling inside it (`.set-pane`): one column at most 640 points wide,
/// centred, 72 points under the card's top.
fn pane(s: &State, w: f32) -> El {
    let inside = (w - CARD_LEFT - crate::chrome::CARD[2] - 64.0).max(1.0);
    let column = (inside - 80.0).clamp(1.0, COLUMN);
    let left = 32.0 + ((inside - (column + 80.0)) / 2.0).max(0.0) + 40.0;
    let mut page = col().w(column).ml(left).pt(72.0).pb(120.0);
    // A refused write says so where the person is looking, over the page (`.set-error`).
    if let Some(error) = s.settings.error.as_ref().filter(|e| !e.is_empty()) {
        page = page.child(kit::error(error));
    }
    let body = match (&s.settings.model, &s.settings.pane) {
        (None, _) => vec![kit::lede_alone("Reading settings…")],
        (Some(_), Pane::Project(_)) => projects::project_page(s, column),
        (Some(_), Pane::Page(page)) => match page {
            PageId::General => pages::general(s),
            PageId::Running => pages::running(s, column),
            PageId::Claude => agents::claude(s, column),
            PageId::Codex => agents::codex(s, column),
            PageId::Antigravity => agents::antigravity(s, column),
            PageId::Projects => projects::list_page(s, column),
            PageId::Phone => pages::phone(s),
            PageId::Cloud => pages::cloud(s, column),
            PageId::Files => pages::files(s),
            PageId::Alpha => pages::alpha(s),
            // Drawn by the web window, which a press on their rows brings forward.
            _ => Vec::new(),
        },
    };
    el().pin(Some(CARD_LEFT), Some(crate::chrome::CARD[1]), Some(crate::chrome::CARD[2]), Some(crate::chrome::CARD[3]))
        .clip_corners([6.0, 6.0, 6.0, 6.0])
        // A column: a plain box is a row, which would stretch the page to its own height and
        // leave nothing to scroll.
        .child(col().key(format!("settings-pane-{}", s.settings.visit)).w_full().h_full().scroll(ScrollSpec { stick_bottom: false }).child(page.children(body)))
}

#[cfg(test)]
mod tests {
    use rune::{Cursor, Key, KeyEvent, Mods, Runtime};
    use serde_json::{Value, json};

    use crate::State;
    use crate::rules::settings::{PageId, Pane};

    /// Settings as the engine answers them for an invented Mac: two Claude Code accounts,
    /// Codex and Antigravity, nine agents on Automatic with the memory check and the sweep of
    /// leftovers on, three projects and one archived.
    fn model() -> Value {
        let project = |slug: &str, name: &str| json!({ "slug": slug, "name": name, "dir": format!("/Users/sam/store/{slug}"), "repoPath": format!("/Users/sam/dev/{slug}"), "autonomous": false, "onPhone": true, "permission": "workspace", "codexMode": "workspace", "account": "default", "instructions": "", "browserSites": [], "running": 0 });
        json!({
            "ok": true,
            "archivedProjects": [{ "slug": "zine", "name": "Zine", "dir": "/Users/sam/store/zine" }],
            "workspace": {
                "agentsAuto": true, "agentsTotal": 9, "agentsAutoTotal": 9, "sessionsAtOnce": 9, "slotsMax": 12, "capacity": 9, "running": 2, "agentsNudge": 0,
                "memoryGate": { "on": true, "slots": null, "slotsAuto": 3, "slotsMax": 12, "reading": { "heavy": null, "waiting": null, "pressure": null, "usedPct": null, "asked": null, "waited": null, "refused": null, "holding": false } },
                "leftovers": { "on": true, "now": "Nothing is left running now." },
                "engineChoices": [{ "id": "claude" }, { "id": "codex" }],
                "permission": "bypassPermissions", "codexMode": "auto", "diagnostics": true, "cornerTag": true, "outsideAgents": "off", "importNew": false,
                "storePath": "/Users/sam/store", "homePath": "/Users/sam", "claudeFound": true, "claudeCertain": true, "phoneAll": true,
                "accounts": [
                    { "profile": "default", "email": "sam@example.com", "chosen": false, "label": "default", "plan": { "label": "Max" } },
                    { "profile": "studio", "email": "sam@studio.example", "chosen": true, "label": "studio", "plan": { "label": "Max" } },
                ],
                "codex": { "found": true, "certain": true, "accounts": [] },
                "antigravity": { "found": true, "certain": true, "signedIn": true, "email": "sam@example.com", "mode": "full-access" },
                "files": { "on": true, "offAfterDays": 30 },
                "alpha": { "rooms": false, "teammates": false, "multiplayer": false, "glass": false },
            },
            "projects": [project("storefront", "Storefront"), project("mobile", "Mobile"), project("docs", "Docs site")],
        })
    }

    /// The window on Settings' `page`, as a picture of it opens, at 1280 by 760.
    fn screen(name: &str, page: &str) -> Runtime<State> {
        screen_at(name, page, (1280.0, 760.0))
    }

    /// The same at another size of window.
    fn screen_at(name: &str, page: &str, size: (f32, f32)) -> Runtime<State> {
        let products: Vec<Value> = [("storefront", "Storefront"), ("mobile", "Mobile"), ("docs", "Docs site")].iter().map(|(slug, name)| json!({ "slug": slug, "name": name })).collect();
        let fixture = json!({
            "now": 1_790_000_000_000.0,
            "snapshot": {
                "products": products, "items": [], "approvals": [],
                "supervisor": { "paused": false, "running": [], "productOrder": ["storefront", "mobile", "docs"], "hiddenProducts": [] },
                "restartNeeded": null, "config": {},
            },
            "local": {},
            "settingsPage": page,
            "settings": model(),
            "answers": {},
        });
        let path = std::env::temp_dir().join(format!("mercury-settings-{name}-{}.json", std::process::id()));
        std::fs::write(&path, fixture.to_string()).expect("the invented settings");
        let s = crate::init_from(&["--fixture".to_string(), path.to_string_lossy().into_owned()]);
        let _ = std::fs::remove_file(&path);
        let text = rune::text::TextSystem::new(crate::FONTS, ("Geist", "Geist Mono, ui-monospace"));
        let mut rt = Runtime::with_text(s, crate::view, size, rune::Env::new(true, 2.0), text);
        rt.poll = Some(crate::poll);
        rt.on_key = Some(crate::keys::pressed);
        rt.frame(0.0);
        rt.frame(1.0 / 60.0);
        rt
    }

    fn press(rt: &mut Runtime<State>, x: f32, y: f32) {
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
    }

    fn key(rt: &mut Runtime<State>, key: Key) {
        rt.key(KeyEvent { key, mods: Mods::default() });
        rt.frame(1.0 / 60.0);
    }

    /// The middle of the menu's row for a page: the groups' rows 39 points apart under their
    /// headings, as `.set-nav-item` stands.
    fn row_of(page: PageId) -> (f32, f32) {
        let y = match page {
            PageId::General => 189.0,
            PageId::Appearance => 228.0,
            PageId::Phone => 306.0,
            PageId::Claude => 422.0,
            PageId::Running => 539.0,
            PageId::Instructions => 578.0,
            PageId::Projects => 655.0,
            _ => unreachable!("not a row these tests press"),
        };
        (100.0, y)
    }

    #[test]
    fn the_menu_goes_to_a_page_this_window_draws_and_leaves_the_rest_to_the_web_window() {
        let mut rt = screen("menu", "general");
        let (x, y) = row_of(PageId::Running);
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Pointer);
        press(&mut rt, x, y);
        assert_eq!(rt.state.settings.pane, Pane::Page(PageId::Running));
        // Appearance is the web window's: the screen stays on Running, and with no app behind
        // this window a line says so.
        let (x, y) = row_of(PageId::Appearance);
        press(&mut rt, x, y);
        assert_eq!(rt.state.settings.pane, Pane::Page(PageId::Running));
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.line.contains("other window")), "{:?}", rt.state.toast.as_ref().map(|t| &t.line));
        // Back to app closes it.
        press(&mut rt, 60.0, 66.0);
        assert!(!rt.state.settings.open);
    }

    #[test]
    fn the_search_finds_a_setting_and_escape_empties_it_before_it_closes_the_screen() {
        let mut rt = screen("search", "general");
        rt.key(KeyEvent { key: Key::Command('f'), mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.settings.search.focused, "⌘F puts the caret in the search");
        rt.update(|s, _| {
            s.settings.search = rune::TextEdit::new("memory");
            s.settings.search.focused = true;
        });
        rt.frame(1.0 / 60.0);
        let hits = super::hits(&rt.state);
        assert_eq!(hits.iter().map(|h| h.label.as_str()).collect::<Vec<_>>(), ["Hold heavy work when memory is short", "Stop programs agents leave behind"]);
        // Escape in the box empties it, and the menu is back; a second closes the screen.
        let mut escape = KeyEvent { key: Key::Escape, mods: Mods::default() };
        rt.update(|s, tx| {
            super::search_keys(s, &escape, tx);
        });
        assert!(rt.state.settings.search.text.is_empty());
        assert!(rt.state.settings.open);
        escape.key = Key::Escape;
        rt.update(|s, tx| {
            super::search_keys(s, &escape, tx);
        });
        assert!(!rt.state.settings.open);
        // Enter on a hit lands on its page.
        let mut rt = screen("search-enter", "general");
        rt.update(|s, _| s.settings.search = rune::TextEdit::new("leftovers"));
        rt.frame(1.0 / 60.0);
        let enter = KeyEvent { key: Key::Enter, mods: Mods::default() };
        rt.update(|s, tx| {
            super::search_keys(s, &enter, tx);
        });
        assert_eq!(rt.state.settings.pane, Pane::Page(PageId::Running));
        assert!(rt.state.settings.search.text.is_empty());
    }

    #[test]
    fn a_picker_opens_under_its_chip_picks_and_escape_or_a_press_elsewhere_closes_it() {
        let mut rt = screen("picker", "running");
        let (cx, cy) = rt.find("picker-at-once").expect("the chip of How many run at once");
        press(&mut rt, cx, cy);
        assert_eq!(rt.state.settings.menu.as_deref(), Some("at-once"));
        // Escape closes the menu and not the screen.
        key(&mut rt, Key::Escape);
        assert!(rt.state.settings.menu.is_none() && rt.state.settings.open);
        // Open again, and a press anywhere else closes it.
        press(&mut rt, cx, cy);
        press(&mut rt, 600.0, 160.0);
        assert!(rt.state.settings.menu.is_none());
        // Its fifth row, four agents: the menu closes and the chip says it at once.
        press(&mut rt, cx, cy);
        let row = cy + 14.0 + 6.0 + 1.0 + 4.0 + 4.0 * 30.0 + 14.5;
        press(&mut rt, cx, row);
        assert!(rt.state.settings.menu.is_none());
        assert_eq!(rt.state.settings.wanted.get("pk:at-once"), Some(&Value::String("4".into())));
    }

    #[test]
    fn a_switch_flips_on_the_press() {
        let mut rt = screen("switch", "general");
        // Counts and crash reports, on, at the right end of its row.
        rt.pointer_move(1038.0, 709.75);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.cursor(), Cursor::Pointer);
        press(&mut rt, 1038.0, 709.75);
        assert_eq!(rt.state.settings.wanted.get("ws:diagnostics"), Some(&Value::Bool(false)));
    }

    #[test]
    fn a_projects_arrow_moves_it_in_the_running_order_at_once() {
        let mut rt = screen("order", "projects");
        assert_eq!(super::projects::Ui::default().pending, None);
        let (x, y) = rt.find("project-storefront").expect("Storefront's row");
        // Its down arrow, 26 points square, beside the door at the row's end.
        press(&mut rt, 1025.0, y);
        let shown = rt.state.settings.projects.pending.as_ref().map(|(shown, _)| shown.clone());
        assert_eq!(shown, Some(vec!["mobile".to_string(), "storefront".to_string(), "docs".to_string()]));
        // It stands second now, and ⌥↑ moves the same row back, as the web window's row with the
        // pressed button in it does.
        rt.key(KeyEvent { key: Key::Up, mods: Mods { alt: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        let shown = rt.state.settings.projects.pending.as_ref().map(|(shown, _)| shown.clone());
        assert_eq!(shown, Some(vec!["storefront".to_string(), "mobile".to_string(), "docs".to_string()]));
        // A press on the row itself opens the project's page.
        let (_, y) = rt.find("project-storefront").expect("Storefront's row, moved back");
        press(&mut rt, x - 200.0, y);
        assert_eq!(rt.state.settings.pane, Pane::Project("storefront".into()));
        assert!(rt.find("project-name").is_none(), "its name is a title until pressed");
    }

    #[test]
    fn the_row_in_hand_is_the_one_under_the_pointer_and_only_while_it_is() {
        let mut rt = screen("in-hand", "projects");
        let in_hand = |rt: &Runtime<State>| rt.state.settings.projects.hovered.clone();
        assert_eq!(in_hand(&rt), None);
        let (x, y) = rt.find("project-mobile").expect("Mobile's row");
        rt.pointer_move(x - 200.0, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(in_hand(&rt).as_deref(), Some("mobile"));
        let (_, y) = rt.find("project-docs").expect("Docs site's row");
        rt.pointer_move(x - 200.0, y);
        rt.frame(1.0 / 60.0);
        assert_eq!(in_hand(&rt).as_deref(), Some("docs"));
        // Off every row, on the menu.
        rt.pointer_move(100.0, 400.0);
        rt.frame(1.0 / 60.0);
        assert_eq!(in_hand(&rt), None);
    }

    #[test]
    fn a_plans_price_keeps_its_line_where_the_card_is_wide_and_breaks_at_a_space_where_it_is_not() {
        // Where "month" stands against the price, on the Max card.
        let month_under_price = |size: (f32, f32)| {
            let rt = screen_at("price", "cloud", size);
            let shown = rt.elements();
            let at = |words: &str| shown.iter().find(|e| e.text.as_deref().map(str::trim) == Some(words)).map(|e| e.rect).unwrap_or_else(|| panic!("{words} at {size:?}"));
            // The last card's small words, which are the last to end in "month" but the hours.
            let (price, month) = (at("$200"), shown.iter().rev().find(|e| e.text.as_deref().is_some_and(|t| t.ends_with("month") && !t.contains("hours"))).map(|e| e.rect).expect("the month"));
            month[1] > price[1] + price[3]
        };
        assert!(!month_under_price((1280.0, 760.0)), "one line in a wide card");
        assert!(month_under_price((900.0, 760.0)), "a second line in a narrow one");
    }

    #[test]
    fn the_page_scrolls_under_the_wheel() {
        let mut rt = screen("scroll", "general");
        let before = rt.find("find-storage").expect("a group down the page").1;
        // A negative delta moves the content up, as a browser's positive deltaY does.
        rt.pointer_move(800.0, 400.0);
        rt.wheel(800.0, 400.0, -300.0);
        for _ in 0..30 {
            rt.frame(1.0 / 60.0);
        }
        let after = rt.find("find-storage").expect("the same group").1;
        assert!(after < before - 100.0, "the page moved up: {before} to {after}");
    }

    /// New project on the Projects page opens the Mac's folder window over Settings, which stays
    /// open (`onNewProject` in `App.tsx`'s Settings), rather than handing over to the web window.
    #[test]
    fn new_project_on_the_projects_page_opens_the_folder_window_over_settings() {
        let mut rt = screen("projects-new", "projects");
        let button = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("New project")).expect("its New project");
        press(&mut rt, button.rect[0] + 4.0, button.rect[1] + button.rect[3] / 2.0);
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.file_dialog().map(|(kind, _)| kind), Some(rune::file_dialog::FileDialogKind::Folder));
        assert!(rt.state.settings.open);
        assert!(rt.state.toast.is_none(), "nothing said of the other window");
    }
}
