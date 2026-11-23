//! The command bar ⌘K opens: the rows it offers, which of them a typed query keeps, the
//! threads the same words find, and the order and headings all of it is drawn under. The web
//! window's own rules: `renderer/src/palette-rows.ts`, `palette-pick.ts`, `palette-keys.ts`
//! and `palette-glyphs.ts`, the order in `renderer/src/components/Palette.tsx`, and the rows
//! `renderer/src/App.tsx` hands the bar (`itemCommands`, `findThreads`).
//!
//! Every row of the web's bar whose doing this window has is offered here, under the web's own
//! words, drawing, key and hint, where the web puts it: on the thread under the keys, approving
//! it, Run now, marking it done or reopening it, replying, putting the agent back on it, marking
//! it read, Later, pinning it, where it runs, its model, its project, sending it back, restarting
//! its agent, opening it and its priority; then the Projects page and ranking a project first,
//! the cloud's rows, Home, the Inbox's tabs, Files and its projects, updates and restarts,
//! pausing and resuming agents, usage, Settings, the shortcuts and the key hints. What it leaves
//! out needs a screen this window does not have, and `docs/parity-command-bar.md` lists each row
//! of the web's with what this window does of it.
//!
//! One person, with no team: with a team's tabs on, Later and Done are columns of In progress
//! and the web's bar goes to two pages, not four (`pageRows({ team })`).

use std::collections::HashSet;

use rune::time::LocalTime;
use serde_json::{Value, json};

use super::compose::{self, Engines, Level, Models, Pick};
use super::js::{self, re};
use super::place::{self, Act, Place};
use super::settings::PageId;
use super::writes::Write;
use super::{cloud, format, list, search, status, title};
use crate::data::{Item, Local, Snapshot, Tab};

/// A thread, as `(id, product)`.
pub type Thread = (String, String);

/// What pressing a row does. The window carries it out (`src/palette.rs`); here it is only
/// named, so the rows can be decided and tested with no window.
#[derive(Clone, Debug, PartialEq)]
pub enum Does {
    /// Opens the box a new thread is written in.
    NewThread,
    Home,
    /// The Inbox, on this tab.
    Tab(Tab),
    Open(Thread),
    /// Opens the thread with the caret in its reply box.
    Reply(Thread),
    Done(Thread),
    Reopen(Thread),
    /// Opens the Later picker on it.
    Later(Thread),
    /// Brings it back from Later.
    Back(Thread),
    /// The Files place: All files, or one project's page.
    Files(Option<String>),
    /// Puts an agent back on a thread nothing will start without the person.
    TryAgain(Thread),
    /// Stops its agent, which moves it to Needs you.
    Stop(Thread),
    /// Approves what the agent asks, held three seconds (`resolve`).
    Approve(Thread),
    /// Pushes it ahead of the queue (`runNow`).
    RunNow(Thread),
    /// Puts an agent back on it, whatever state it is in (`putAgentBackOn`).
    Resume(Thread),
    /// Counts it as read now (`markSeen`).
    MarkRead(Thread),
    /// Pins it to the top of the list, or lets it go (`pinThread`).
    Pin(Thread, bool),
    /// Where it runs, as the place button's panel picks it (`placeThread`).
    Place(Thread, Act),
    /// The model, and so the agent, its next run is on (`handToModel`).
    Model(Thread, Pick),
    /// Files it in another project, by slug (`moveToProject`).
    Move(Thread, String),
    /// Says why it cannot be moved.
    CannotMove(String),
    /// Sends a blocked thread back to its agent (`sendBack`).
    SendBack(Thread),
    /// A fresh agent for a thread whose agent stopped (`redeliver`).
    Redeliver(Thread),
    /// Its level, written at once (`retag`).
    Priority(Thread, Level),
    /// Settings, on that page.
    Settings(PageId),
    /// That project first in the running order (`onRankFirst`).
    RankFirst(String),
    /// The cloud set to run so (`onCloudMode`): off asks first.
    Cloud(cloud::Mode),
    /// The restart onto the version that is ready (`updateInstall`).
    InstallUpdate,
    /// The app moved into Applications, where it can update itself (`updateMove`).
    MoveToApplications,
    /// A look for a new version now (`updateCheck`).
    CheckForUpdates,
    /// Nothing at all: the bar stays open.
    Nothing,
    /// Agents paused, or let go again (`pauseSupervisor`).
    Pause(bool),
    /// Every interrupted agent respawned (`resumeAgents(null)`).
    ResumeStopped,
    /// The key hints turned on or off (`setHints`).
    KeyHints(bool),
    /// Opens the code its run changed beside the conversation.
    Code(Thread),
}

/// "View code changes", on the open conversation while its page draws the code button
/// (`f.open?.change` in `renderer/src/thread-commands.ts`): after Later, where the web's thread
/// commands follow it, and before the places.
pub fn with_code_row(rows: &mut Vec<Row>, thread: Thread) {
    let at = rows.iter().position(|r| r.id == "snooze").map_or(rows.len(), |at| at + 1);
    rows.insert(at, Row::command("code", "View code changes", Does::Code(thread)).found_by("code changes diff files what changed"));
}

/// One row of the bar (`Command` in `Palette.tsx`).
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// The web row's own id. Its mark and its place among the matches hang on it.
    pub id: String,
    pub label: String,
    /// What is said at the row's end. Shown, never matched: it is copy, and a search that
    /// leaned on copy would break the next time someone improved the sentence.
    pub hint: Option<String>,
    /// A thread's line under its title: the sentence the query was found in.
    pub sub: Option<String>,
    /// A finished thread, drawn dimmed.
    pub done: bool,
    /// Words a person might type that the label does not say. Matched, never shown.
    pub keywords: &'static str,
    /// What the row was called before a rename: typing one ranks the row as its label would.
    pub old_names: &'static [&'static str],
    /// The key that does the same, drawn at the row's end in place of a hint.
    pub key: Option<&'static str>,
    /// A word the query must name before the row is offered (`onlyWith`).
    pub only_with: Option<&'static str>,
    /// Not in the list until something is typed (`unlisted`).
    pub unlisted: bool,
    pub does: Does,
}

impl Row {
    fn command(id: &str, label: &str, does: Does) -> Row {
        Row { id: id.to_string(), label: label.to_string(), hint: None, sub: None, done: false, keywords: "", old_names: &[], key: None, only_with: None, unlisted: false, does }
    }
    fn found_by(mut self, keywords: &'static str) -> Row {
        self.keywords = keywords;
        self
    }
    fn on_key(mut self, key: &'static str) -> Row {
        self.key = Some(key);
        self
    }
    fn hinted(mut self, hint: impl Into<String>) -> Row {
        self.hint = Some(hint.into());
        self
    }
}

/* --------------------------------- the rows offered --------------------------------- */

/// The row is called Later, after the key. It said "Remind Me (Snooze)" once, and the old
/// names still find it, ahead of any thread they also find (`LATER_KEYWORDS`, `LATER_OLD_NAMES`).
pub const LATER_KEYWORDS: &str = "remind me snooze put off defer schedule";
pub const LATER_OLD_NAMES: &[&str] = &["remind me", "snooze"];

/// No "inbox" among them: typed alone it means going there, and this row would sit above Go
/// to Inbox under the Return key (`REOPEN_KEYWORDS`).
pub const REOPEN_KEYWORDS: &str = "unmark undo undone not done open again bring back put back";

/// New thread, which N also does (`new-thread` in `Palette.tsx`): the row the web's list has
/// second, after New project. It is offered only once this window can start a thread.
pub fn new_thread_row() -> Row {
    let mut row = Row::command("new-thread", "New thread", Does::NewThread).found_by("task compose write message ask start").on_key("N");
    row.hint = Some("ask an agent".to_string());
    row
}

/// Home, then the tabs ⌘K goes to (`view-home` in `Palette.tsx`, and `pageRows`). Home stands
/// above the Inbox, as in the rail, so "home" lands on it although the Inbox answers to it too.
pub fn page_rows() -> Vec<Row> {
    vec![
        Row::command("view-home", "Go to Home", Does::Home).found_by("start overview"),
        Row::command("view-inbox", "Go to Inbox", Does::Tab(Tab::Needs)).found_by("needs you home list threads"),
        Row::command("view-progress", "Go to In progress", Does::Tab(Tab::Progress)).found_by("working running"),
        Row::command("view-later", "Go to Later", Does::Tab(Tab::Later)).found_by(LATER_KEYWORDS),
        Row::command("view-done", "Go to Done", Does::Tab(Tab::Done)).found_by("finished closed"),
    ]
}

/// The words a Files row is found by besides its name (`filePages` in `Palette.tsx`, whose
/// keywords end with the row's title, which a query finds in its name here).
const FILES_KEYWORDS: &str = "files file documents docs pictures images films videos decks slides pdfs designs shared";

/// The Files place and each project's page in it (`filePages` in `App.tsx`, after the tabs
/// there): Go to Files, then Files in each project the place has listed.
pub fn file_rows(projects: &[String]) -> Vec<Row> {
    let mut all = Row::command("files-all", "Go to Files", Does::Files(None)).found_by(FILES_KEYWORDS);
    all.hint = Some("every file you can see".to_string());
    let mut rows = vec![all];
    for p in projects {
        rows.push(Row::command(&format!("files-project-{p}"), &format!("Files in {p}"), Does::Files(Some(p.clone()))).found_by(FILES_KEYWORDS));
    }
    rows
}

/// When a thread that was put off comes back (`returnAt` in `App.tsx`).
fn return_at(item: &Item) -> f64 {
    list::deferred_until(item).max(list::hidden_until(item, 0.0))
}

/// What the rows are decided from besides the thread under the keys: the engine's snapshot,
/// what the web window keeps of what has been read, the clock, the models this Mac's agents
/// offer, the Files place's projects once it has been read, the key hints switch, and whether
/// this window can start a thread.
pub struct Facts<'a> {
    pub snap: &'a Snapshot,
    pub local: &'a Local,
    pub now: f64,
    pub models: &'a Models,
    pub file_projects: &'a [String],
    pub key_hints: bool,
    pub starts_threads: bool,
}

/// The ids on one of the supervisor's lists.
fn ids_on<'a>(snap: &'a Snapshot, list: &str) -> impl Iterator<Item = &'a str> {
    snap.supervisor.rest.get(list).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str)
}

fn on_list(snap: &Snapshot, list: &str, id: &str) -> bool {
    ids_on(snap, list).any(|on| on == id)
}

/// A run is on it this second (`working` in `App.tsx`).
fn working(snap: &Snapshot, item: &Item) -> bool {
    snap.supervisor.running.iter().any(|r| r.item_id == item.id)
}

/// It belongs In progress by the tab's own rule, with what the bar hands it there: a run on it
/// or one starting, nothing owed a ship, and stopped when the supervisor stopped it or it waits
/// on a sign-in (`belongsInProgress` with `liveIds` and `stalledIds`).
fn in_progress(f: &Facts, item: &Item) -> bool {
    let live = working(f.snap, item) || on_list(f.snap, "starting", &item.id);
    let signed_out = f.snap.supervisor.rest.get("signInNeeded").and_then(Value::as_object).is_some_and(|m| m.contains_key(&item.id));
    let stopped = on_list(f.snap, "stalled", &item.id) || signed_out;
    list::belongs_in_progress(item, &list::Facts { deferred_until: list::deferred_until(item), now: f.now, live, stopped, ..list::Facts::default() })
}

/// It can be stopped now: In progress, not finished, not an outside agent's (`stoppableNow`).
fn stoppable(f: &Facts, item: &Item) -> bool {
    !item.truthy("agent") && item.status != "done" && in_progress(f, item)
}

/// Whether Try again stands where Resume this agent and Restart the agent would, with what
/// `factsFor` says of the thread and, for the first, whether it can be stopped.
fn try_again(f: &Facts, item: &Item, in_progress: bool) -> bool {
    let approvals = f.snap.rest.get("approvals").and_then(Value::as_array).into_iter().flatten();
    let asking = approvals.filter_map(|a| a.get("item").and_then(Value::as_str)).any(|id| id == item.id) || super::kinds::open_sign_in(item);
    let session = f.snap.supervisor.running.iter().rev().find(|r| r.item_id == item.id);
    status::offers_try_again(item, &status::Facts { supervisor: &f.snap.supervisor, session, asking, in_progress, practice: false, later: false, now: f.now })
}

/// Unread while its latest answer is newer than when it was last opened; one never opened is
/// read from when opened times began to be kept (`isUnread` in `renderer/src/unread.ts`).
pub fn is_unread(item: &Item, local: &Local) -> bool {
    let arrived = item.wrote("result").and_then(|w| w.ts).or(item.created_at).unwrap_or(0.0);
    arrived > local.seen_at.get(&item.id).copied().unwrap_or(local.unread_since)
}

/// Run now is offered on a thread with no run on it that is not pushed already and is waiting
/// its turn or belongs In progress (`offersRunNow` in `renderer/src/run-now.ts`).
fn offers_run_now(f: &Facts, item: &Item) -> bool {
    !item.truthy("agent") && !working(f.snap, item) && !on_list(f.snap, "runNow", &item.id) && (on_list(f.snap, "queued", &item.id) || in_progress(f, item))
}

/// What the agent is told when its proposal is approved as it stands (`RUN_IT`).
pub const RUN_IT: &str = "Approved. Run it.";

/// What Approve approves (`resolve` in `App.tsx`, on the rows `itemCommands` offers it on).
#[derive(Clone, Debug, PartialEq)]
pub enum Approval {
    /// A question: its recommended option, else its first.
    Option(format::Choice),
    /// A review: go ahead as proposed.
    Review,
    /// An open proposal the person did not file: run it.
    Run,
}

/// Approve is offered on a thread that is not an outside agent's and has no answer yet, when it
/// is a review, a question with options, or an open proposal the person did not file.
pub fn approval(item: &Item) -> Option<Approval> {
    if item.truthy("agent") || item.text("answer").is_some() {
        return None;
    }
    match item.kind.as_deref() {
        Some("review") => Some(Approval::Review),
        Some("question") => {
            let options = format::item_options(item);
            options.iter().find(|o| o.recommended).or(options.first()).cloned().map(Approval::Option)
        }
        _ if item.status == "open" && !super::answers::filed_by_the_person(item) => Some(Approval::Run),
        _ => None,
    }
}

impl Approval {
    pub fn label(&self) -> &'static str {
        match self {
            Approval::Option(_) => "Approve recommended",
            Approval::Review => "Approve as proposed",
            Approval::Run => "Approve and run",
        }
    }
    /// The words it answers with.
    pub fn answer(&self) -> String {
        match self {
            Approval::Option(choice) => super::writes::option_words(choice),
            Approval::Review => "Approved. Proceed as you proposed.".to_string(),
            Approval::Run => RUN_IT.to_string(),
        }
    }
    /// The line said while it waits its three seconds, `project` being where the thread is.
    pub fn said(&self, item: &Item, project: &str) -> String {
        match self {
            Approval::Option(choice) => format!("Approved: option {} → {project}", choice.n),
            Approval::Review => format!("Approved → {project} will enact it"),
            Approval::Run => format!("Running → {}", js::slice(item.title.as_deref().unwrap_or(""), 0, 40)),
        }
    }
}

/// The words a Move row is found by (`MOVE_KEYWORDS` in `shared/move-thread.mts`).
pub const MOVE_KEYWORDS: &str = "move to project file refile wrong another switch put";

/// Why a thread cannot be moved to another project, or nothing when it can (`whyItCannotMove`).
pub fn why_it_cannot_move(item: &Item) -> Option<&'static str> {
    if item.truthy("agent") {
        Some("An agent\u{2019}s own session does not live in a project.")
    } else if item.status == "done" {
        Some("This thread is finished. Moving it would only move the record.")
    } else if super::shipping::not_shipped_line(item).is_some() {
        Some("This thread changed code in its project. Ship it or close it there first.")
    } else {
        None
    }
}

/// Every word that should find the model rows, whichever one somebody reaches for (`MODEL_KEYWORDS`).
pub const MODEL_KEYWORDS: &str = "model models engine harness switch change use agent claude code codex gpt opus sonnet haiku fable";

/// The model rows for a thread (`modelCommands` in `renderer/src/thread-model-commands.ts`): up to
/// three of the models used lately, never the one it is on, the other agent's own first where
/// this Mac offers it, since reaching the other agent is what the rows are for. Picking a model
/// is picking the agent that runs it.
pub fn model_rows(item: &Item, items: &[Item], models: &Models, engines: &Engines) -> Vec<Row> {
    const ROWS: usize = 3;
    let (codex, claude) = (engines.offers("codex"), engines.offers("claude"));
    if !codex && !claude {
        return Vec::new();
    }
    let here = item.text("model").map(|model| Pick::new(match item.engine.as_deref() {
        Some(engine @ ("codex" | "antigravity")) => engine,
        _ => "claude",
    }, model));
    let picks = compose::recent_models(items, models, engines);
    let elsewhere: Vec<Pick> = match here.as_ref().map(|p| p.engine.as_str()) {
        Some("claude") if codex => vec![Pick::new("codex", models.codex_default.as_deref().or(models.codex.first().map(|m| m.id.as_str())).unwrap_or(""))],
        Some("codex") if claude => picks.iter().filter(|p| p.engine == "claude").take(1).cloned().collect(),
        _ => Vec::new(),
    };
    let mut rows: Vec<Row> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for pick in elsewhere.into_iter().chain(picks) {
        if pick.model.is_empty() || rows.len() >= ROWS {
            continue;
        }
        let key = format!("{}:{}", pick.engine, pick.model);
        if seen.contains(&key) || here.as_ref() == Some(&pick) {
            continue;
        }
        seen.insert(key.clone());
        let label = format!("Model: {}", compose::model_label(&pick, models));
        let hint = super::engine::engine_label(&pick.engine);
        rows.push(Row::command(&format!("model-{key}"), &label, Does::Model((item.id.clone(), item.product.clone()), pick)).found_by(MODEL_KEYWORDS).hinted(hint));
    }
    rows
}

/// What the place row says and does for each place, as the thread's menu words it (`PLACE_ROW`
/// and `placeAct` in `renderer/src/threads/ThreadMenu.tsx`): its detail is its hint, from a small
/// letter.
fn place_row(place: Place) -> Option<(&'static str, &'static str, Act)> {
    Some(match place {
        Place::Cloud => ("Run on this Mac", "in the cloud now", Act::Keep(true)),
        Place::Here => ("Keep on this Mac", "cloud allowed", Act::Keep(true)),
        Place::Kept => ("Run on the cloud", "this Mac only", Act::Keep(false)),
        Place::Move => ("Move to the cloud", "on this Mac now", Act::Move),
        Place::Needs => ("Run on the cloud", "needs this Mac", Act::Keep(false)),
        Place::Off => return None,
    })
}

/// The level a thread shows, which is the level it runs at, an untagged one as Medium; a
/// conversation between people keeps the number it carries (`priorityIdOf(shownPriority(…))`).
fn shown_level(item: &Item, conversation: bool) -> Level {
    if !conversation {
        return Level::of(item.priority);
    }
    match item.priority.unwrap_or(5.0) {
        n if n >= 9.0 => Level::Urgent,
        n if n >= 7.0 => Level::High,
        n if n <= 2.0 => Level::Low,
        _ => Level::Medium,
    }
}

/// The four levels, the one the thread shows said so (`priorityCommands`). There is no key: the
/// levels are found by name, and "high" and "prio" both find them.
pub fn priority_rows(item: &Item, snap: &Snapshot) -> Vec<Row> {
    let conversation = snap.products.iter().any(|p| p.slug == item.product && compose::is_direct(p));
    let shown = shown_level(item, conversation);
    compose::LEVELS
        .into_iter()
        .map(|level| {
            let word = compose::level_word(level);
            let current = if level == shown { " (current)" } else { "" };
            Row::command(&format!("priority-{}", word.to_lowercase()), &format!("Priority: {word}{current}"), Does::Priority((item.id.clone(), item.product.clone()), level))
        })
        .collect()
}

/// What can be done to the thread the keys are on, in the web's order (`itemCommands` in
/// `App.tsx`), so ⌘K opened over a row cannot mean another thread than E pressed over it.
/// `open` says the thread is the conversation on screen, which has no row to open.
///
/// Left out: Remote control and Open terminal (no such panels here), Close task, Mark unread
/// (only the web window keeps what has been read, and nothing sets it back), the team's rows,
/// Rename (no box to rename in), Stop repeating (repeating tasks are not read here) and Add
/// people. Stop is added by the bar where the thread is under way (`stop_row`), and Try again
/// after Reply where the window offers it (`with_try_again`), which these rules cannot tell;
/// View code changes is its own control's to bring.
pub fn thread_commands(f: &Facts, item: &Item, open: bool) -> Vec<Row> {
    let thread = || (item.id.clone(), item.product.clone());
    let agent = item.truthy("agent");
    let mut rows = Vec::new();
    // Approve lives here by name only: no key drives it.
    if let Some(approval) = approval(item) {
        rows.push(Row::command("approve", approval.label(), Does::Approve(thread())));
    }
    if offers_run_now(f, item) {
        rows.push(Row::command("run-now", "Run now", Does::RunNow(thread())).found_by("start go next urgent jump queue"));
    }
    // "Mark done" is what the thread's own menu calls it; an outside agent's row is Done.
    rows.push(if item.status == "done" && !agent {
        Row::command("reopen", "Reopen", Does::Reopen(thread())).found_by(REOPEN_KEYWORDS)
    } else {
        Row::command("done", if agent { "Done" } else { "Mark done" }, Does::Done(thread())).on_key("E")
    });
    rows.push(Row::command("reply", "Reply", Does::Reply(thread())).on_key("R"));
    // Carry on with it, on any status an agent is not already on, where Try again does not
    // stand for the same press.
    let stoppable = stoppable(f, item);
    if !try_again(f, item, stoppable) && !agent && !working(f.snap, item) && !stoppable {
        let label = if item.status == "done" { "Resume this agent (reopen and carry on)" } else { "Resume this agent" };
        rows.push(Row::command("resume-any", label, Does::Resume(thread())));
    }
    if is_unread(item, f.local) {
        rows.push(Row::command("read", "Mark read", Does::MarkRead(thread())));
    }
    if return_at(item) > f.now {
        // A moment an agent set is the agent's own wait, not the person's reminder.
        let label = if list::parked_by_agent(item, f.now) { "Let it run now" } else { "Back to Inbox" };
        rows.push(Row::command("unsnooze", label, Does::Back(thread())).on_key("E"));
    }
    let mut later = Row::command("snooze", "Later", Does::Later(thread())).found_by(LATER_KEYWORDS).on_key("L");
    later.old_names = LATER_OLD_NAMES;
    rows.push(later);
    // What the thread's own controls do (`threadCommands`): its pin and where it runs.
    let live = item.status != "done" && !agent;
    if live {
        let pinned = item.truthy("pinned");
        rows.push(Row::command("pin", if pinned { "Unpin thread" } else { "Pin thread" }, Does::Pin(thread(), !pinned)).found_by("pin pinned unpin top stick sticky"));
    }
    if let Some((label, hint, act)) = place::thread_place(f.snap, item).and_then(place_row) {
        rows.push(Row::command("place", label, Does::Place(thread(), act)).found_by("cloud mac computer here local where runs machine").hinted(hint));
    }
    // Which model, and so which agent, picks it up next. Not on an outside agent's row, which
    // takes only words.
    if !agent {
        rows.extend(model_rows(item, &f.snap.items, f.models, &Engines::of(f.snap)));
    }
    // And which project it is filed in: one row per project, or one that says why it cannot move.
    match why_it_cannot_move(item) {
        Some(why) => rows.push(Row::command("move-refused", "Move to another project", Does::CannotMove(why.to_string())).found_by(MOVE_KEYWORDS).hinted(why)),
        None => {
            let ranked = compose::rank_products(&f.snap.products, &f.snap.supervisor.product_order);
            for p in compose::projects_offered(&ranked) {
                if p.slug == item.product || p.rest.get("practice") == Some(&Value::Bool(true)) {
                    continue;
                }
                rows.push(Row::command(&format!("move-{}", p.slug), &format!("Move to {}", p.name), Does::Move(thread(), p.slug.clone())).found_by(MOVE_KEYWORDS));
            }
        }
    }
    if item.status == "blocked" {
        rows.push(Row::command("send-back", "Send back to agent", Does::SendBack(thread())));
    }
    if on_list(f.snap, "stalled", &item.id) && !try_again(f, item, false) {
        rows.push(Row::command("redeliver", "Restart the agent (it stopped)", Does::Redeliver(thread())));
    }
    if !open {
        rows.push(Row::command("open", "Open", Does::Open(thread())).on_key("↵"));
    }
    rows.extend(priority_rows(item, f.snap));
    rows
}

/// The Projects page, then a row for each project that ranks it first, in the running order:
/// where it stands now, once it is ranked; until then, that the app made it for the person, or
/// that it is not ranked yet (`projects-page` and `rank-` in `Palette.tsx`). A conversation
/// with a teammate is kept as a project too, and is not one here.
pub fn project_rows(snap: &Snapshot) -> Vec<Row> {
    let mut rows = vec![Row::command("projects-page", "Projects…", Does::Settings(PageId::Projects))
        .found_by("priority order rank reorder sort first top priorities rename project settings")
        .hinted("order, names and rules")];
    let ranked: Vec<_> = compose::rank_products(&snap.products, &snap.supervisor.product_order).into_iter().filter(|p| !compose::is_direct(p)).collect();
    for (at, p) in ranked.iter().enumerate() {
        let hint = if snap.supervisor.product_order.contains(&p.slug) {
            format!("now #{} of {}", at + 1, ranked.len())
        } else if p.rest.get("personal") == Some(&Value::Bool(true)) {
            "made for you".to_string()
        } else {
            "not ranked yet".to_string()
        };
        rows.push(Row::command(&format!("rank-{}", p.slug), &format!("Rank {} first", p.name), Does::RankFirst(p.slug.clone())).hinted(hint));
    }
    rows
}

const CLOUD_KEYWORDS: &str = "cloud where threads run mac overflow everything move on off";

/// The cloud on or off with its key, then each meaning of on, on a Mac set up for the cloud
/// (`cloudCommands`). Off is offered only to words that name the cloud, as ⌘C and ⌘K were both
/// pressed by accident.
pub fn cloud_rows(snap: &Snapshot) -> Vec<Row> {
    let Some(c) = snap.rest.get("cloud").filter(|c| crate::data::truthy(c.get("setUp"))) else {
        return Vec::new();
    };
    let on = cloud::is_on(c);
    let mut toggle = Row::command("cloud-toggle", if on { "Turn the cloud off" } else { "Turn the cloud on" }, Does::Cloud(cloud::toggle_target(c))).found_by(CLOUD_KEYWORDS).on_key("⌘C");
    if on {
        toggle.only_with = Some("cloud");
    }
    let mut rows = vec![toggle];
    for (mode, label, _) in cloud::WHEN_ON {
        let mut row = Row::command(&format!("cloud-{}", mode.id()), &format!("Cloud on: {}{}", label[..1].to_lowercase(), &label[1..]), Does::Cloud(mode)).found_by(CLOUD_KEYWORDS);
        if on && c.get("mode").and_then(Value::as_str) == Some(mode.id()) {
            row = row.hinted("On now");
        }
        rows.push(row);
    }
    rows
}

/// A new version ready, a copy that cannot update itself where it is, a look for one now while
/// none waits, and a restart this build's main process needs (`update`, `update-move`,
/// `update-check` and `stale` in `Palette.tsx`, with `announcesUpdate` and `announcesMove`).
pub fn update_rows(snap: &Snapshot) -> Vec<Row> {
    use super::settings::NAME;
    let update = snap.rest.get("update");
    let on = |field: &str| update.is_some_and(|u| crate::data::truthy(u.get(field)));
    let ready = (on("ready") || on("installing")).then(|| update.and_then(|u| u.get("newVersion")).and_then(Value::as_str).filter(|v| !v.is_empty())).flatten();
    let mut rows = Vec::new();
    if let Some(version) = ready {
        rows.push(Row::command("update", &format!("Restart to update {NAME}"), Does::InstallUpdate).found_by("update upgrade new version restart").hinted(format!("version {version} is ready")));
    }
    if on("canMove") && !on("installing") {
        rows.push(Row::command("update-move", "Move to Applications", Does::MoveToApplications).found_by("update upgrade move applications folder disk image read-only stuck").hinted("cannot update from here"));
    }
    if ready.is_none() {
        rows.push(Row::command("update-check", "Check for updates", Does::CheckForUpdates).found_by("update upgrade new version release newest restart").hinted("look for one now"));
    }
    let stale = snap.rest.get("restartNeeded").and_then(|r| r.get("files")).and_then(Value::as_array).map_or(0, Vec::len);
    if stale > 0 {
        rows.push(Row::command("stale", &format!("Restart {NAME} to pick up main-process changes"), Does::Nothing).hinted(format!("{stale} file{} changed", if stale == 1 { "" } else { "s" })));
    }
    rows
}

/// Pausing the agents or letting them go again, and resuming the interrupted ones (`pause` and
/// `resume-stopped`). One id for both ways of the first, so its label says which way it goes.
pub fn agent_rows(snap: &Snapshot) -> Vec<Row> {
    let paused = crate::data::truthy(snap.supervisor.rest.get("paused"));
    vec![
        Row::command("pause", if paused { "Unpause agents" } else { "Pause agents" }, Does::Pause(!paused)).hinted(if paused { "sessions will spawn again" } else { "no new sessions spawn" }),
        Row::command("resume-stopped", "Resume interrupted agents", Does::ResumeStopped).hinted("after limits or crashes"),
    ]
}

/// Usage and limits, read on Claude Code's page; Settings on General; the Shortcuts page
/// (`usage`, `settings`, `shortcuts`). Their keywords are what is on the pages, as the label is
/// a word and the word typed is usually the thing to change.
pub fn settings_rows() -> Vec<Row> {
    use super::settings::NAME;
    vec![
        Row::command("usage", "Usage and limits", Does::Settings(PageId::Claude)).found_by("usage limit limits left remaining quota tokens cost spend plan subscription accounts").hinted("what is left"),
        Row::command("settings", "Settings…", Does::Settings(PageId::General))
            .found_by("settings preferences options usage limits accounts account sign in theme appearance look phone cloud permissions agents rooms groups alpha codex claude")
            .hinted("workspace and every project"),
        Row::command("shortcuts", "Keyboard shortcuts", Does::Settings(PageId::Shortcuts)).found_by("keys keyboard shortcut hotkeys commands cheatsheet help what does press bindings").hinted(format!("every key {NAME} has")),
    ]
}

/// The key hints, turned the other way (`keyhints`).
pub fn key_hints_row(on: bool) -> Row {
    Row::command("keyhints", if on { "Turn off keyboard shortcut hints" } else { "Turn on keyboard shortcut hints" }, Does::KeyHints(!on)).found_by("keys keyboard shortcut hints").hinted("shown when you point")
}

/// Stopping the thread under the keys while it is under way (`stop` in `itemCommands`): "Stop
/// Agent" while a run is up, else "Stop Task" with what is true of it as its hint. It stands
/// after the rows above and before Open, where the web's stands after its Move rows.
pub fn stop_row(running: bool, thread: Thread) -> Row {
    let (label, hint) = super::answering::stop_row_words(running);
    let mut row = Row::command("stop", label, Does::Stop(thread));
    row.hint = hint.map(str::to_string);
    row
}

/// Try again on the thread under the keys, which the window offers where nothing will start
/// it without the person (`try-again` in `itemCommands`, in place of Resume this agent). It
/// goes after Reply, as the web window's does.
pub fn try_again_row(thread: Thread) -> Row {
    Row::command("try-again", "Try again", Does::TryAgain(thread)).found_by("resume restart retry start stuck run again")
}

/// Puts Try again after Reply in `rows`, as the web's list has it.
pub fn with_try_again(rows: &mut Vec<Row>, thread: Thread) {
    let at = rows.iter().position(|r| r.id == "reply").map_or(rows.len(), |at| at + 1);
    rows.insert(at, try_again_row(thread));
}

/// Every row the bar has before anything is typed, in the web's order: starting a thread,
/// then what acts on the thread under the keys, so every key can be searched for, then the
/// Projects page and the projects, the cloud, the places, Files, updates, the agents,
/// Settings and the key hints (`commands` in `Palette.tsx`). `under` is the thread the keys
/// are on, and whether it is the conversation on screen.
pub fn commands(f: &Facts, under: Option<(&Item, bool)>) -> Vec<Row> {
    let mut rows = Vec::new();
    if f.starts_threads {
        rows.push(new_thread_row());
    }
    if let Some((item, open)) = under {
        rows.extend(thread_commands(f, item, open));
    }
    rows.extend(project_rows(f.snap));
    rows.extend(cloud_rows(f.snap));
    rows.extend(page_rows());
    rows.extend(file_rows(f.file_projects));
    rows.extend(update_rows(f.snap));
    rows.extend(agent_rows(f.snap));
    rows.extend(settings_rows());
    rows.push(key_hints_row(f.key_hints));
    rows
}

/* ------------------------------- what a row writes ------------------------------- */

fn thread_of(item: &Item) -> Value {
    json!({ "product": item.product, "id": item.id })
}

fn with(item: &Item, field: &str, value: Value) -> Value {
    let mut payload = thread_of(item);
    payload[field] = value;
    payload
}

/// An approval, as `api.answer` sends it: the words and nothing else.
pub fn approve(item: &Item, approval: &Approval) -> Value {
    with(item, "answer", json!(approval.answer()))
}

/// What Z writes once an approval is written: its agent stopped, then the answer taken back
/// and the thread open again (`approveWith`'s `pushUndo` in `renderer/src/App.tsx`).
pub fn approval_withdrawn(item: &Item) -> Vec<Write> {
    vec![
        (super::answering::STOP, json!({ "product": item.product, "id": item.id })),
        (super::writes::ANSWER, json!({ "product": item.product, "id": item.id, "answer": "(withdrawn)", "status": "open" })),
    ]
}
/// What that Z says, what it asks once the approval is half a minute old, and the line it
/// leaves in the thread.
pub const APPROVAL_WITHDRAWN: &str = "Approval withdrawn, back in your inbox";
pub const APPROVAL_UNDOES: &str = "take back that approval and stop the agent";
pub const APPROVAL_UNDID: &str = "Undid your approval";

/// Reopen, from the bar: the thread opened again with nothing said, which Z marks done again
/// (`reopenDone`).
pub fn reopen_done(item: &Item) -> Write {
    (super::writes::ANSWER, with(item, "status", json!("open")))
}

/// Z after Reopen: marked done again. Named by `(id, product)`, as the row may have changed by then.
pub fn done_again((id, product): &Thread) -> Write {
    (super::writes::ANSWER, json!({ "product": product, "id": id, "status": "done" }))
}

pub fn run_now(item: &Item) -> Write {
    ("zero:run-now", thread_of(item))
}

/// A worker back on the thread, whatever state it is in (`resumeItems` with its id).
pub fn resume(item: &Item) -> Write {
    ("zero:resume-items", json!({ "ids": [item.id] }))
}

/// Pinned, or unpinned, which also forgets where it was dragged to (`pinThread` from the menu).
pub fn pin(item: &Item, pin: bool) -> Vec<Write> {
    let mut writes = vec![("zero:set-thread-pins", json!({ "pins": { item.id.clone(): if pin { json!(true) } else { Value::Null } } }))];
    if !pin {
        writes.push(("zero:set-thread-places", json!({ "places": { item.id.clone(): Value::Null } })));
    }
    writes
}

/// The agent and model its next run is on, with no words, which starts nothing (`handToModel`).
pub fn hand_to_model(item: &Item, pick: &Pick) -> Write {
    let mut payload = thread_of(item);
    payload["engine"] = json!(pick.engine);
    payload["model"] = json!(pick.model);
    (super::writes::ANSWER, payload)
}

pub fn move_item(item: &Item, to: &str) -> Write {
    ("zero:move-item", with(item, "to", json!(to)))
}

/// Z after a move: the copy closed, and the thread that was moved, `(id, product)`, open again
/// where it was. The moved row has left the engine's lists by then, so it is named, not read.
pub fn move_back(made: &Value, to: &str, (id, product): &Thread) -> Vec<Write> {
    let copy = made.get("id").and_then(Value::as_str).unwrap_or("");
    let copied_to = made.get("product").and_then(Value::as_str).unwrap_or(to);
    vec![
        (super::writes::ANSWER, json!({ "product": copied_to, "id": copy, "status": "done" })),
        (super::writes::ANSWER, json!({ "product": product, "id": id, "status": "open" })),
    ]
}

/// Back to its agent: a blocked thread goes back in the queue (`sendBack`).
pub fn send_back(item: &Item) -> Write {
    (super::writes::REOPEN, thread_of(item))
}

pub fn redeliver(item: &Item) -> Write {
    ("zero:redeliver", thread_of(item))
}

/// The running order with that project first (`moveProduct(rankedProducts, slug, 0)`).
pub fn rank_first(snap: &Snapshot, slug: &str) -> Write {
    let ranked: Vec<String> = compose::rank_products(&snap.products, &snap.supervisor.product_order).iter().map(|p| p.slug.clone()).collect();
    ("zero:set-product-order", json!({ "order": super::settings::move_product(&ranked, slug, 0) }))
}

pub fn pause(paused: bool) -> Write {
    ("zero:supervisor-pause", json!(paused))
}

/// Every interrupted agent respawned (`resumeAgents(null)`): the door hands the handler `null`.
pub fn resume_stopped() -> Write {
    ("zero:resume-agents", Value::Null)
}

/* --------------------------- what is said once it lands --------------------------- */

/// What Run now says of the engine's answer (`runNow`).
pub fn run_now_said(answer: &Value) -> &'static str {
    if answer.get("ok") == Some(&Value::Bool(true)) {
        return "Up next. It starts as soon as an agent is free.";
    }
    match answer.get("reason").and_then(Value::as_str) {
        Some("running") => "Already running.",
        Some("paused") => "Agents are paused. Turn them back on to run this.",
        Some("scheduled") => "This task is scheduled for later.",
        Some("held") => "Another session still holds this task. It starts when that lets go.",
        _ => "This task is no longer waiting.",
    }
}

/// A value as JavaScript's template string writes it: a number without a needless fraction,
/// a list as its items joined by commas.
fn js_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), |f| if f.fract() == 0.0 && f.abs() < 1e21 { format!("{}", f as i64) } else { f.to_string() }),
        Value::Array(items) => items.iter().map(|i| if i.is_null() { String::new() } else { js_string(i) }).collect::<Vec<_>>().join(","),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// What putting an agent back on one thread says, read the way the web window reads the
/// engine's answer, and whether the thread then names itself for O (`putAgentBackOn`). The
/// answer carries the supervisor's own state after the count, so its `queued` is the queue's
/// list of ids rather than how many were queued: every answer then says it queued, as the web
/// window's does.
pub fn resumed_said(answer: &Value) -> (String, bool, bool) {
    let field = |name: &str| answer.get(name).filter(|v| !v.is_null()).cloned().unwrap_or(json!(0));
    let (started, queued) = (field("resumed"), field("queued"));
    let (s, q) = (crate::data::truthy(Some(&started)), crate::data::truthy(Some(&queued)));
    let said = if s || q {
        [if s { format!("{} resumed", js_string(&started)) } else { String::new() }, if q { format!("{} waiting for a slot", js_string(&queued)) } else { String::new() }]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    } else if crate::data::truthy(answer.get("working")) {
        "Already working on it".to_string()
    } else {
        "Nothing to resume".to_string()
    };
    (said, s || q, started.as_f64().is_some_and(|n| n > 0.0))
}

/// What resuming the interrupted agents says (`resumeAgents`).
pub fn resume_stopped_said(answer: &Value) -> String {
    match answer.get("cleared").filter(|c| crate::data::truthy(Some(c))) {
        Some(n) => format!("Resuming: {} interrupted {} respawning", js_string(n), if n.as_f64() == Some(1.0) { "agent" } else { "agents" }),
        None => "Nothing stranded. Anything waiting respawns on its own".to_string(),
    }
}

/// What a fresh agent on a stopped thread says, `project` being where it is (`redeliver`).
pub fn redeliver_said(answer: &Value, project: &str) -> Option<String> {
    (answer.get("ok") == Some(&Value::Bool(true))).then(|| format!("A fresh agent is picking this up → {project}"))
}

/// What a thread put back in the queue says (`sendBack`).
pub fn sent_back_said(project: &str) -> String {
    format!("Back in the queue → {project}")
}

/// What reopening a finished thread says, and what Z then says (`reopenDone`).
pub fn reopened_said(item: &Item) -> (String, String) {
    let title = super::writes::clip_to_sentence(item.title.as_deref().unwrap_or(""), 48);
    (format!("Reopened: {title} · press Z to undo"), format!("Done again: {title}"))
}

/* ------------------------------- what a query keeps ------------------------------- */

/// Every word of the query must land, in any order, anywhere in the label or the keywords:
/// "dark mode" appears as one string in no label, and as two words it finds its row. A word
/// may land inside another, and nothing is fuzzy: a fuzzy match on a list this long puts rows
/// nobody asked for under the Return key (`matchesQuery`). A row held back until one word is
/// named needs a word of two letters or more that starts that word, or that the word starts;
/// with nothing typed it is there, unless it is held back from the untyped list too.
pub fn matches_query(query: &str, row: &Row) -> bool {
    let query = query.to_lowercase();
    let words = js::words(&query);
    if words.is_empty() {
        return !row.unlisted;
    }
    if let Some(must) = row.only_with.map(str::to_lowercase) {
        if !words.iter().any(|w| js::len(w) >= 2 && (must.starts_with(w) || w.starts_with(must.as_str()))) {
            return false;
        }
    }
    let hay = format!("{} {}", row.label, row.keywords).to_lowercase();
    words.iter().all(|w| hay.contains(w))
}

/// When two rows answer to the same letters, the one meant goes first. "rem" is Later, which
/// was Remind Me, and not a row used once a month that starts with the same three letters: no
/// rule of strings separates the two, so the winner is named (`FIRST_AMONG_MATCHES`).
const FIRST_AMONG_MATCHES: [&str; 1] = ["snooze"];

/// The matches, with the named winner first, then a row whose name starts with what was
/// typed: "go to" is Go to Inbox, not a row that matched because "logo" holds "go". The rest
/// keep the order the list was put together in (`rankMatches`).
pub fn rank_matches(query: &str, mut rows: Vec<Row>) -> Vec<Row> {
    let q = js::trim(query).to_lowercase();
    if q.is_empty() {
        return rows;
    }
    let rank = |row: &Row| {
        if FIRST_AMONG_MATCHES.contains(&row.id.as_str()) {
            0
        } else if row.label.to_lowercase().starts_with(&q) {
            1
        } else {
            2
        }
    };
    rows.sort_by_key(rank);
    rows
}

/// Words as they are compared: small letters, one space between them (`norm` in `palette-pick.ts`).
fn said(s: &str) -> String {
    js::words(&s.to_lowercase()).join(" ")
}

re!(NOT_A_LETTER, r"[^\p{L}\p{N}]+", "gu");

/// Every word typed starts a word of the name (`namesIt`): "sum up" is not "reSUMe
/// interrUPted agents".
pub fn names_it(query: &str, label: &str) -> bool {
    let (typed, name) = (said(query), said(label));
    let words: Vec<&str> = typed.split(' ').filter(|w| !w.is_empty()).collect();
    let own: Vec<&str> = NOT_A_LETTER.split(&name).into_iter().filter(|w| !w.is_empty()).collect();
    !words.is_empty() && words.iter().all(|w| own.iter().any(|o| o.starts_with(w)))
}

/// The thread whose title is the words first, then one that starts with them, then one that
/// has them; otherwise the order they came in. A thread whose body names the words must not
/// sit above the thread whose title is them (`titleFirst`).
pub fn title_first(mut rows: Vec<Row>, query: &str) -> Vec<Row> {
    let q = said(query);
    if q.is_empty() {
        return rows;
    }
    rows.sort_by_key(|r| {
        let t = said(&r.label);
        if t == q {
            0
        } else if t.starts_with(&q) {
            1
        } else if t.contains(&q) {
            2
        } else {
            3
        }
    });
    rows
}

/* -------------------------------- the threads found -------------------------------- */

/// How many threads the bar lists (`THREADS_SHOWN`). The web's bar ends them on a row that
/// opens its search page on the rest; this window has no search page, so the best few are all.
pub const THREADS_SHOWN: usize = 5;

/// The bar's short age for a thread under a day old (`agoShort` in `renderer/src/home/home-model.ts`).
pub fn ago_short(at: f64, now: f64) -> String {
    let m = ((now - at) / 60_000.0).floor().max(0.0);
    if m < 1.0 {
        return "now".to_string();
    }
    if m < 60.0 {
        return format!("{m} min");
    }
    let h = (m / 60.0).floor();
    if h < 24.0 { format!("{h} hr") } else { format!("{} d", (h / 24.0).floor()) }
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A found thread's row: what it is called, the line it was found by, where it is and how
/// old, and whether it is finished (`findThreads` in `App.tsx`, `threadResult` in
/// `palette-rows.ts`). A day old or more it says its date, not a count of days; the web window
/// writes that date in the system's language, and this writes it in English, as the rest of
/// the window speaks.
pub fn thread_row(item: &Item, summary: String, project: &str, now: f64) -> Row {
    let at = item.updated_at.or(item.created_at).unwrap_or(0.0);
    let age = if now - at < 86_400_000.0 {
        ago_short(at, now)
    } else {
        let day = LocalTime::at(at as i64);
        format!("{} {}", MONTHS[day.month as usize - 1], day.day)
    };
    let mut row = Row::command(&format!("thread-{}-{}", item.product, item.id), &title::shown_title(item), Does::Open((item.id.clone(), item.product.clone())));
    row.hint = Some(format!("{project} · {age}"));
    row.sub = Some(summary).filter(|s| !s.is_empty());
    row.done = item.status == "done";
    row
}

/// The threads a query adds to the bar: the best few of what the list's own search finds,
/// the one titled by the words first (`threadRows` and `titleFirst` in `Palette.tsx`). Nothing
/// typed finds nothing. The web's bar also finds repeating tasks and teammates' threads here,
/// which this window does not list.
pub fn threads_found(index: &search::Index, snap: &Snapshot, query: &str, now: f64) -> Vec<Row> {
    let q = js::trim(query);
    if q.is_empty() {
        return Vec::new();
    }
    // A project the snapshot does not name is called by its slug.
    let project = |slug: &str| snap.products.iter().find(|p| p.slug == slug).map_or_else(|| slug.to_string(), |p| p.name.clone());
    let found = search::search_items(index, &snap.items, q);
    let rows = found.into_iter().take(THREADS_SHOWN).map(|hit| thread_row(hit.item, hit.summary, &project(&hit.item.product), now)).collect();
    title_first(rows, query)
}

/* ------------------------------ the order it is drawn in ------------------------------ */

/// The rows in the order drawn, and the heading over the row each section starts at.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listed {
    pub rows: Vec<Row>,
    pub labels: Vec<(usize, &'static str)>,
}

impl Listed {
    pub fn label_over(&self, at: usize) -> Option<&'static str> {
        self.labels.iter().find(|(row, _)| *row == at).map(|(_, label)| *label)
    }
}

/// Commands first, then the threads found, under their heading. Once something is typed, a
/// command goes ahead of the threads only when the words start words of its name, or of a
/// name it had: one the words merely touch by keyword comes after the threads, under a
/// heading of its own. And a command that carries a found thread's whole title ("Waits on
/// that thread") stays below that thread unless its own label starts with the words
/// (`paletteOrder`). The web's bar also heads people and groups, and holds one row back until
/// it is asked for by name; this window has neither.
pub fn order(matched: Vec<Row>, threads: Vec<Row>, query: &str) -> Listed {
    let named = |c: &Row| names_it(query, &c.label) || c.old_names.iter().any(|n| names_it(query, n));
    let named_titles: Vec<String> = threads.iter().filter(|t| names_it(query, &t.label)).map(|t| said(&t.label)).collect();
    let carries_a_named_title = |c: &Row| named_titles.iter().any(|t| said(&c.label).contains(t.as_str()));
    let ahead = |c: &Row| named(c) && (!carries_a_named_title(c) || said(&c.label).starts_with(&said(query)));
    let mut labels = Vec::new();
    if !threads.is_empty() && !js::trim(query).is_empty() {
        let (first, after): (Vec<Row>, Vec<Row>) = matched.into_iter().partition(ahead);
        labels.push((first.len(), "Threads"));
        if !after.is_empty() {
            labels.push((first.len() + threads.len(), "Commands"));
        }
        return Listed { rows: first.into_iter().chain(threads).chain(after).collect(), labels };
    }
    if !threads.is_empty() {
        labels.push((matched.len(), "Threads"));
    }
    Listed { rows: matched.into_iter().chain(threads).collect(), labels }
}

/// What the bar lists for what is typed: every command when nothing is, else the commands
/// the words keep, ranked, with the threads they find (`PaletteBox` in `Palette.tsx`).
pub fn listed(commands: Vec<Row>, threads: Vec<Row>, query: &str) -> Listed {
    let q = js::trim(query).to_lowercase();
    let matched = if q.is_empty() { commands.into_iter().filter(|c| !c.unlisted).collect() } else { rank_matches(&q, commands.into_iter().filter(|c| matches_query(&q, c)).collect()) };
    order(matched, threads, query)
}

/* ---------------------------------- the row picked ---------------------------------- */

/// One key for each row, in order, unique even when two rows share an id (`commandKeys`).
pub fn row_keys(rows: &[Row]) -> Vec<String> {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    rows.iter()
        .map(|row| {
            let n = match seen.iter_mut().find(|(id, _)| *id == row.id) {
                Some((_, n)) => {
                    *n += 1;
                    *n
                }
                None => {
                    seen.push((row.id.as_str(), 0));
                    0
                }
            };
            if n > 0 { format!("{}#{n}", row.id) } else { row.id.clone() }
        })
        .collect()
}

/// Where the picked row is now: the first row when it is gone or none was picked, and nowhere
/// when there are no rows. The pick is a row, not a place, so a list that sorts again under it
/// keeps the same row lit, and Enter opens the row drawn as picked (`keptPick`).
pub fn kept_pick(key: Option<&str>, keys: &[String]) -> Option<usize> {
    if keys.is_empty() {
        return None;
    }
    Some(key.and_then(|key| keys.iter().position(|k| k == key)).unwrap_or(0))
}

/* ------------------------------- what the bar says ------------------------------- */

/// What the box says while it is empty.
pub const PLACEHOLDER: &str = "Search threads or type a command…";

const ECHO_MAX: usize = 32;

/// What stands where a row would have been when nothing matches. The query is said back,
/// because it is the one thing this line can say that the person does not already know: what
/// exactly was searched for, stray character and all. A pasted paragraph is cut at a word
/// (`emptyLine`).
pub fn empty_line(query: &str) -> String {
    let q = js::trim(query);
    let shown = if js::len(q) > ECHO_MAX {
        let cut = js::slice(q, 0, ECHO_MAX);
        let kept = match js::last_index_of(cut, " ") {
            Some(space) if space > 0 => js::slice(cut, 0, space),
            _ => cut,
        };
        format!("{}…", js::trim_end(kept))
    } else {
        q.to_string()
    };
    format!("Nothing matches \u{201c}{shown}\u{201d}.")
}

/// What a row nobody has drawn yet wears: always the same one (`PLAIN_ICON`).
pub const PLAIN_ICON: &str = "chevron";

/// The drawing each row wears by its id (`BY_ID` in `renderer/src/palette-glyphs.ts`).
const BY_ID: &[(&str, &str)] = &[
    ("new-project", "plus"),
    ("new-thread", "compose"),
    ("record-meeting", "record"),
    ("stop-recording", "stop"),
    ("filter", "filter"),
    ("clear-filter", "clear"),
    ("end-tutorial", "clear"),
    ("projects-page", "folder"),
    ("search", "search"),
    ("all-threads", "search"),
    ("view-home", "home"),
    ("view-inbox", "inbox"),
    ("view-progress", "progress"),
    ("view-later", "later"),
    ("view-done", "done"),
    ("view-groups", "hash"),
    ("view-teammates", "team"),
    ("view-connections", "link"),
    ("new-teammate", "add-person"),
    ("update", "download"),
    ("update-move", "move"),
    ("update-check", "refresh"),
    ("stale", "refresh"),
    ("resume-stopped", "play"),
    ("standing", "instructions"),
    ("usage", "gauge"),
    ("settings", "settings"),
    ("shortcuts", "keyboard"),
    ("keyhints", "keyboard"),
    ("flip-view", "board"),
    ("import-agents", "download"),
    ("tutorial", "help"),
    ("first-run", "refresh"),
    ("demo", "spark"),
    ("remote-control", "swap"),
    ("open-terminal", "terminal"),
    ("approve", "check"),
    ("run-now", "run"),
    ("done", "done"),
    ("close-task", "closed"),
    ("reopen", "reopen"),
    ("send-back", "reopen"),
    ("reply", "reply"),
    ("resume-any", "refresh"),
    ("try-again", "refresh"),
    ("resume", "refresh"),
    ("redeliver", "refresh"),
    ("read", "read"),
    ("unread", "unread"),
    ("snooze", "later"),
    ("unsnooze", "inbox"),
    ("move-refused", "move"),
    ("stop", "stop"),
    ("open", "open"),
    ("see-team", "team"),
    ("see-people", "add-person"),
    ("see-private", "lock"),
    ("pin", "pin"),
    ("rename", "rename"),
    ("summary", "text"),
    ("code", "code"),
    ("place", "cloud"),
    ("end-repeat", "stop-repeat"),
    ("add-people", "add-person"),
    ("shipped", "shipped"),
    ("team-list-shipped", "shipped"),
    ("decide", "flag"),
    ("wait-stop", "wait"),
];

/// And by the start of its id, the longest first, so `teammate-pause-` is read before
/// `teammate-` (`BY_PREFIX`).
const BY_PREFIX: &[(&str, &str)] = &[
    ("teammate-pause-", "pause"),
    ("teammate-", "person"),
    ("chat-person-", "person"),
    ("chat-", "hash"),
    ("thread-", "thread"),
    ("card-", "thread"),
    ("line-", "text"),
    ("files-", "file"),
    ("file-", "file"),
    ("comment-", "comment"),
    ("decision-", "flag"),
    ("team-list-", "flag"),
    ("priority-urgent", "priority-urgent"),
    ("priority-high", "priority-high"),
    ("priority-medium", "priority-medium"),
    ("priority-", "priority-low"),
    ("rank-", "top"),
    ("move-", "move"),
    ("give-", "give"),
    ("take-back", "reopen"),
    ("route-", "give"),
    ("model-", "spark"),
    ("group-", "hash"),
    ("cloud-", "cloud"),
    ("theme-", "theme"),
    ("wait-", "wait"),
];

/// Every name a row can wear, for the drawings to be checked against.
#[cfg(test)]
pub fn icon_names() -> Vec<&'static str> {
    BY_ID.iter().chain(BY_PREFIX).map(|(_, name)| *name).chain([PLAIN_ICON, "pause", "play"]).collect()
}

/// The line drawing at the head of a row, by the web's name for it (`iconFor`): one id for both
/// ways of a toggle, so the label says which way it goes.
pub fn icon_for(id: &str, label: &str) -> &'static str {
    let starts = |words: &str| label.get(..words.len()).is_some_and(|head| head.eq_ignore_ascii_case(words));
    if id == "pause" {
        return if starts("unpause") { "play" } else { "pause" };
    }
    if id.starts_with("teammate-pause-") && starts("resume") {
        return "play";
    }
    BY_ID.iter().find(|(of, _)| *of == id).or_else(|| BY_PREFIX.iter().find(|(start, _)| id.starts_with(start))).map_or(PLAIN_ICON, |(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::calendar::Day;
    use serde_json::{Value, json};

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    fn cmd(id: &str, label: &str, keywords: &'static str) -> Row {
        Row::command(id, label, Does::Home).found_by(keywords)
    }

    fn thread(id: &str, label: &str) -> Row {
        Row::command(&format!("thread-{id}"), label, Does::Open((id.to_string(), "shop".to_string())))
    }

    fn ids(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.id.as_str()).collect()
    }

    /// Rows a query keeps, ranked, as the bar does it.
    fn typed(query: &str, rows: &[Row]) -> Vec<String> {
        rank_matches(query, rows.iter().filter(|c| matches_query(query, c)).cloned().collect()).into_iter().map(|r| r.id).collect()
    }

    const NOW: f64 = 1_000_000_000.0;

    fn open_thread() -> Item {
        item(json!({ "id": "w-1", "product": "shop", "status": "open", "title": "Fix the export" }))
    }

    #[test]
    fn a_thread_with_a_change_has_its_code_row_after_later_and_found_by_diff() {
        let mut rows = World::empty().on(&open_thread(), false);
        with_code_row(&mut rows, ("w-1".to_string(), "shop".to_string()));
        let at = ids(&rows).iter().position(|id| *id == "code").expect("the row");
        assert_eq!(ids(&rows)[at - 1], "snooze");
        let code = rows.iter().find(|r| r.id == "code").unwrap();
        assert_eq!(code.label, "View code changes");
        assert_eq!(code.does, Does::Code(("w-1".to_string(), "shop".to_string())));
        for words in ["diff", "what changed", "files", "code"] {
            assert!(typed(words, &rows).contains(&"code".to_string()), "{words} finds it");
        }
        // With no Later row to follow, it goes at the end.
        let mut bare = vec![cmd("home", "Go to Home", "")];
        with_code_row(&mut bare, ("w-1".to_string(), "shop".to_string()));
        assert_eq!(ids(&bare), ["home", "code"]);
    }

    #[test]
    fn the_files_place_and_each_projects_page_are_found_by_the_words_for_files() {
        let rows = file_rows(&["Storefront".to_string(), "Mobile".to_string()]);
        assert_eq!(rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Go to Files", "Files in Storefront", "Files in Mobile"]);
        assert_eq!(rows[0].hint.as_deref(), Some("every file you can see"));
        assert_eq!(typed("pictures", &rows).len(), 3);
        assert_eq!(typed("files mobile", &rows), ["files-project-Mobile"]);
        assert_eq!(icon_for("files-all", "Go to Files"), "file");
        assert_eq!(icon_for("files-project-Mobile", "Files in Mobile"), "file");
        assert_eq!(rows[2].does, Does::Files(Some("Mobile".to_string())));
    }

    #[test]
    fn the_places_say_go_to_and_are_found_by_the_words_for_them() {
        let places = page_rows();
        assert_eq!(places.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Go to Home", "Go to Inbox", "Go to In progress", "Go to Later", "Go to Done"]);
        let inbox = &places[1];
        for q in ["go to", "go to inbox", "inbox", "needs you"] {
            assert!(matches_query(q, inbox), "{q}");
        }
        assert!(!matches_query("go to settings", inbox));
        // "home" finds Home first, though the Inbox answers to the word too.
        assert_eq!(typed("home", &places), ["view-home", "view-inbox"]);
        assert_eq!(typed("finished", &places), ["view-done"]);
        assert_eq!(places[3].does, Does::Tab(Tab::Later));
        // A row whose name starts with what was typed goes above one that matched inside a word.
        let mut rows = vec![cmd("wait-1", "Waits on Two logo rounds are ready to compare", "")];
        rows.extend(page_rows());
        assert_eq!(typed("go to", &rows)[..2], ["view-home", "view-inbox"]);
        assert_eq!(typed("go to", &rows).last().unwrap(), "wait-1");
        assert_eq!(typed("waits", &rows), ["wait-1"]);
    }

    /// What the rows are decided from: a snapshot, nothing read yet, the models the app was built with.
    struct World {
        snap: Snapshot,
        local: Local,
        models: Models,
    }

    impl World {
        fn of(snap: Value) -> World {
            World { snap: serde_json::from_value(snap).unwrap(), local: Local::default(), models: Models::built() }
        }
        fn empty() -> World {
            World::of(json!({ "products": [{ "slug": "shop", "name": "Storefront" }], "supervisor": { "running": [], "queued": [] } }))
        }
        fn facts(&self, now: f64) -> Facts<'_> {
            Facts { snap: &self.snap, local: &self.local, now, models: &self.models, file_projects: &[], key_hints: true, starts_threads: true }
        }
        fn on(&self, item: &Item, open: bool) -> Vec<Row> {
            thread_commands(&self.facts(NOW), item, open)
        }
        fn on_at(&self, item: &Item, now: f64) -> Vec<Row> {
            thread_commands(&self.facts(now), item, false)
        }
    }

    /// The rows the web's bar draws for every thread: the three models used lately (none yet, so
    /// Opus, Sonnet and the next of Claude Code's), and the four levels.
    const MODELS: [&str; 3] = ["model-claude:opus", "model-claude:sonnet", "model-claude:claude-opus-5"];
    const LEVELS: [&str; 4] = ["priority-urgent", "priority-high", "priority-medium", "priority-low"];

    fn with_the_rest<'a>(head: &[&'a str], tail: &[&'a str]) -> Vec<&'a str> {
        head.iter().copied().chain(MODELS).chain(tail.iter().copied()).chain(LEVELS).collect()
    }

    /// A thread its agent answered and finished, as the invented inbox's are.
    fn answered() -> Item {
        item(json!({ "id": "w-1", "product": "shop", "status": "done", "kind": "directive", "title": "Fix the export", "labels": ["founder"], "priority": 9,
            "result": "Fixed and tested.", "createdAt": NOW - 3_600_000.0, "wrote": { "result": { "ts": NOW - 60_000.0, "source": "agent" }, "status": { "ts": NOW - 60_000.0, "source": "agent" } } }))
    }

    #[test]
    fn the_thread_under_the_keys_gets_its_rows_in_the_webs_order() {
        let w = World::empty();
        // The web's own rows for the first thread of its invented inbox, but Rename and Mark
        // unread, which this window does not have.
        let rows = w.on(&answered(), false);
        assert_eq!(ids(&rows), with_the_rest(&["reopen", "reply", "resume-any", "read", "snooze"], &["move-refused", "open"]));
        let label = |id: &str| rows.iter().find(|r| r.id == id).map(|r| (r.label.as_str(), r.hint.as_deref(), r.key)).unwrap();
        assert_eq!(label("resume-any"), ("Resume this agent (reopen and carry on)", None, None));
        assert_eq!(label("model-claude:opus"), ("Model: Opus 5.5", Some("Claude Code"), None));
        assert_eq!(label("move-refused"), ("Move to another project", Some("This thread is finished. Moving it would only move the record."), None));
        assert_eq!(label("open"), ("Open", None, Some("↵")));
        assert_eq!(label("priority-urgent"), ("Priority: Urgent (current)", None, None));
        assert_eq!(label("priority-low"), ("Priority: Low", None, None));
        // The conversation on screen has no row to open.
        assert!(!ids(&w.on(&answered(), true)).contains(&"open"));
        // A finished thread is reopened, not marked done again, and is found by the words for that.
        let reopen = &rows[0];
        assert_eq!((reopen.label.as_str(), reopen.key), ("Reopen", None));
        for q in ["reopen", "unmark done", "undo done", "not done", "bring back", "open again"] {
            assert!(matches_query(q, reopen), "{q}");
        }
        assert!(!matches_query("inbox", reopen));
        // An open thread the person filed waits its turn: Run now, Mark done, Pin thread, and its
        // project's other projects to move to; nothing to resume, as it can be stopped.
        let w = World::of(json!({ "products": [{ "slug": "shop", "name": "Storefront" }, { "slug": "app", "name": "Mobile" }, { "slug": "tutorial", "name": "Practice", "practice": true }],
            "supervisor": { "running": [], "queued": [], "productOrder": ["app"] } }));
        let filed = item(json!({ "id": "w-2", "product": "shop", "status": "open", "kind": "directive", "title": "Fix the export", "labels": ["founder"], "createdAt": NOW - 60_000.0 }));
        let rows = w.on(&filed, false);
        assert_eq!(ids(&rows), with_the_rest(&["run-now", "done", "reply", "read", "snooze", "pin"], &["move-app", "open"]));
        assert_eq!(rows.iter().find(|r| r.id == "done").map(|r| (r.label.as_str(), r.key)), Some(("Mark done", Some("E"))));
        assert_eq!(rows.iter().find(|r| r.id == "move-app").map(|r| r.label.as_str()), Some("Move to Mobile"));
        // An outside agent's row closes as Done and has no level of a model to pick.
        let outside = item(json!({ "id": "agent:7", "product": "shop", "status": "open", "title": "A terminal", "agent": { "pid": 7 } }));
        let rows = w.on(&outside, false);
        assert_eq!(ids(&rows)[..3], ["done", "reply", "snooze"]);
        assert_eq!(rows[0].label, "Done");
        assert!(!ids(&rows).iter().any(|id| id.starts_with("model-") || *id == "pin" || *id == "resume-any"));
    }

    #[test]
    fn approve_is_offered_by_what_the_agent_asks() {
        let w = World::empty();
        let question = item(json!({ "id": "w-3", "product": "shop", "status": "open", "kind": "question", "title": "Which tax rule?", "labels": [],
            "result": "## Options\n\n1. Round first\n2. Round last (recommended)" }));
        let rows = w.on(&question, false);
        assert_eq!((rows[0].id.as_str(), rows[0].label.as_str()), ("approve", "Approve recommended"));
        let approval = approval(&question).unwrap();
        assert_eq!(approve(&question, &approval), json!({ "product": "shop", "id": "w-3", "answer": "Option 2: Round last (recommended)" }));
        assert_eq!(approval.said(&question, "Storefront"), "Approved: option 2 → Storefront");
        let review = item(json!({ "id": "w-4", "product": "shop", "status": "open", "kind": "review", "title": "The new copy" }));
        assert_eq!(approval_of(&review), Some(("Approve as proposed", "Approved. Proceed as you proposed.".to_string())));
        assert_eq!(super::approval(&review).unwrap().said(&review, "Storefront"), "Approved → Storefront will enact it");
        let proposal = item(json!({ "id": "w-5", "product": "shop", "status": "open", "kind": "directive", "title": "Split the checkout service into three smaller ones", "labels": [] }));
        assert_eq!(approval_of(&proposal), Some(("Approve and run", RUN_IT.to_string())));
        assert_eq!(super::approval(&proposal).unwrap().said(&proposal, "Storefront"), "Running → Split the checkout service into three sm");
        // Not on a thread the person filed, one already answered, a question with no options,
        // or an outside agent's.
        let mut filed = proposal.clone();
        filed.labels = vec!["founder".to_string()];
        for none in [filed, item(json!({ "id": "w-6", "product": "shop", "status": "open", "kind": "review", "answer": "Go on." })), item(json!({ "id": "w-7", "product": "shop", "status": "open", "kind": "question", "result": "Which?" })),
            item(json!({ "id": "w-8", "product": "shop", "status": "open", "kind": "review", "agent": { "pid": 1 } }))] {
            assert_eq!(approval_of(&none), None, "{}", none.id);
        }
    }

    fn approval_of(item: &Item) -> Option<(&'static str, String)> {
        approval(item).map(|a| (a.label(), a.answer()))
    }

    #[test]
    fn run_now_resume_restart_and_send_back_follow_what_the_supervisor_says() {
        let open = || item(json!({ "id": "w-1", "product": "shop", "status": "open", "kind": "directive", "title": "Fix it", "labels": ["founder"] }));
        let with = |supervisor: Value| World::of(json!({ "products": [{ "slug": "shop", "name": "Storefront" }], "supervisor": supervisor }));
        let has = |w: &World, item: &Item, id: &str| ids(&w.on(item, false)).contains(&id);
        // Queued: Run now, until it has been pushed or a run is on it.
        assert!(has(&with(json!({ "running": [], "queued": ["w-1"] })), &open(), "run-now"));
        assert!(!has(&with(json!({ "running": [], "queued": ["w-1"], "runNow": ["w-1"] })), &open(), "run-now"));
        assert!(!has(&with(json!({ "running": [{ "itemId": "w-1", "product": "shop" }], "queued": [] })), &open(), "run-now"));
        // A stopped thread is not In progress, so it can be resumed; Try again stands for both
        // Resume this agent and Restart the agent there.
        let stalled = with(json!({ "running": [], "queued": [], "stalled": ["w-1"] }));
        let rows = ids(&stalled.on(&open(), false)).into_iter().map(str::to_string).collect::<Vec<_>>();
        assert!(!rows.contains(&"resume-any".to_string()) && !rows.contains(&"redeliver".to_string()), "{rows:?}");
        // Without the fleet's queue nothing says Try again, and both are offered.
        let unread = with(json!({ "running": [], "stalled": ["w-1"] }));
        assert!(has(&unread, &open(), "resume-any") && has(&unread, &open(), "redeliver"));
        // A blocked thread can be sent back to its agent.
        let mut blocked = open();
        blocked.status = "blocked".to_string();
        assert!(has(&with(json!({ "running": [], "queued": [] })), &blocked, "send-back"));
        assert!(!has(&with(json!({ "running": [], "queued": [] })), &open(), "send-back"));
    }

    #[test]
    fn a_thread_is_unread_until_it_is_opened_after_its_answer() {
        let mut w = World::empty();
        let thread = answered();
        assert!(is_unread(&thread, &w.local));
        w.local.seen_at.insert("w-1".to_string(), NOW);
        assert!(!is_unread(&thread, &w.local));
        assert!(!ids(&w.on(&thread, false)).contains(&"read"));
        // One never opened counts as read from when opened times began to be kept.
        let local = Local { unread_since: NOW, ..Local::default() };
        assert!(!is_unread(&thread, &local));
    }

    #[test]
    fn pin_place_and_the_model_rows_say_what_they_do() {
        let cloud = json!({ "setUp": true, "mode": "full", "projects": ["shop"], "movable": [] });
        let w = World::of(json!({ "products": [{ "slug": "shop", "name": "Storefront" }], "supervisor": { "running": [], "queued": [] }, "cloud": cloud,
            "engines": { "choices": [{ "id": "claude", "label": "Claude Code" }, { "id": "codex", "label": "Codex" }] } }));
        let mut thread = item(json!({ "id": "w-1", "product": "shop", "status": "open", "kind": "directive", "title": "Fix it", "labels": ["founder"], "model": "opus", "pinned": true }));
        let rows = w.on(&thread, false);
        let row = |id: &str| rows.iter().find(|r| r.id == id).unwrap_or_else(|| panic!("{id}: {:?}", ids(&rows)));
        assert_eq!((row("pin").label.as_str(), &row("pin").does), ("Unpin thread", &Does::Pin(("w-1".into(), "shop".into()), false)));
        assert_eq!((row("place").label.as_str(), row("place").hint.as_deref()), ("Keep on this Mac", Some("cloud allowed")));
        assert_eq!(row("place").does, Does::Place(("w-1".into(), "shop".into()), Act::Keep(true)));
        // On Claude Code's Opus the model it is on is not offered, nor Codex's own while this Mac
        // has not said which model that is.
        let models = |rows: &[Row]| rows.iter().filter(|r| r.id.starts_with("model-")).map(|r| (r.label.clone(), r.hint.clone().unwrap_or_default())).collect::<Vec<_>>();
        let said = |label: &str, hint: &str| (label.to_string(), hint.to_string());
        assert_eq!(models(&rows), [said("Model: Sonnet 5.5", "Claude Code")]);
        // Once it has, Codex's comes first.
        let mut w = w;
        w.models.codex = vec![compose::Model { id: "gpt-6-luna".to_string(), label: "GPT-6 Luna".to_string(), ..compose::Model::default() }];
        assert_eq!(models(&w.on(&thread, false)), [said("Model: GPT-6 Luna", "Codex"), said("Model: Sonnet 5.5", "Claude Code")]);
        // On Codex, Claude Code's first model comes first.
        let mut on_codex = thread.clone();
        on_codex.engine = Some("codex".to_string());
        on_codex.rest.insert("model".to_string(), json!("gpt-6-luna"));
        assert_eq!(models(&w.on(&on_codex, false)), [said("Model: Opus 5.5", "Claude Code"), said("Model: Sonnet 5.5", "Claude Code")]);
        // Kept on this Mac, it can run on the cloud; with the whole cloud off there is no row.
        thread.rest.insert("onMac".to_string(), json!(true));
        assert_eq!(w.on(&thread, false).iter().find(|r| r.id == "place").map(|r| (r.label.as_str(), r.hint.clone())), Some(("Run on the cloud", Some("this Mac only".to_string()))));
        let off = World::of(json!({ "products": [], "supervisor": { "running": [] }, "cloud": { "setUp": true, "mode": "mac", "projects": ["shop"] } }));
        thread.rest.remove("onMac");
        assert!(!ids(&off.on(&thread, false)).contains(&"place"));
        // What a pin and a model write.
        assert_eq!(pin(&thread, false), vec![("zero:set-thread-pins", json!({ "pins": { "w-1": null } })), ("zero:set-thread-places", json!({ "places": { "w-1": null } }))]);
        assert_eq!(pin(&thread, true), vec![("zero:set-thread-pins", json!({ "pins": { "w-1": true } }))]);
        assert_eq!(hand_to_model(&thread, &Pick::new("codex", "gpt-5-codex")), ("zero:answer", json!({ "product": "shop", "id": "w-1", "engine": "codex", "model": "gpt-5-codex" })));
    }

    #[test]
    fn a_thread_put_off_can_be_brought_back_and_an_agents_wait_let_go() {
        let w = World::empty();
        let put_off = item(json!({ "id": "w-3", "product": "shop", "status": "open", "title": "Fix the export", "labels": ["founder"], "snoozedUntil": NOW + 60_000.0,
            "wrote": { "snoozedUntil": { "ts": NOW - 5.0, "source": "founder" } } }));
        let rows = w.on(&put_off, false);
        let at = ids(&rows).iter().position(|id| *id == "unsnooze").unwrap();
        assert_eq!(ids(&rows)[at + 1], "snooze");
        assert_eq!((rows[at].label.as_str(), rows[at].key), ("Back to Inbox", Some("E")));
        assert_eq!(rows[at].does, Does::Back(("w-3".to_string(), "shop".to_string())));
        // Once the moment has passed there is nothing to bring back.
        assert!(!ids(&w.on_at(&put_off, NOW + 120_000.0)).contains(&"unsnooze"));
        let parked = item(json!({ "id": "w-4", "product": "shop", "status": "open", "title": "Fix the export", "labels": ["founder"], "runAt": NOW + 60_000.0,
            "wrote": { "runAt": { "ts": NOW - 5.0, "source": "agent" } } }));
        assert_eq!(w.on(&parked, false).iter().find(|r| r.id == "unsnooze").map(|r| r.label.as_str()), Some("Let it run now"));
    }

    #[test]
    fn the_whole_list_is_the_threads_rows_then_the_webs_other_rows_in_its_order() {
        let w = World::empty();
        let head = |under: Option<(&Item, bool)>| commands(&w.facts(NOW), under).into_iter().map(|r| r.id).collect::<Vec<_>>();
        // With no thread under the keys, as on Home: New thread, then the rest of the web's list
        // that this window does.
        assert_eq!(head(None), [
            "new-thread", "projects-page", "rank-shop", "view-home", "view-inbox", "view-progress", "view-later", "view-done", "files-all",
            "update-check", "pause", "resume-stopped", "usage", "settings", "shortcuts", "keyhints",
        ]);
        // The thread's rows go between New thread and the Projects page.
        let rows = head(Some((&answered(), false)));
        assert_eq!((rows[0].as_str(), rows[1].as_str()), ("new-thread", "reopen"));
        assert_eq!(rows[rows.iter().position(|id| id == "priority-low").unwrap() + 1], "projects-page");
        let rows = commands(&w.facts(NOW), None);
        assert_eq!((rows[0].label.as_str(), rows[0].key), ("New thread", Some("N")));
        assert!(matches_query("compose", &rows[0]) && matches_query("new", &rows[0]));
        let said = |id: &str| rows.iter().find(|r| r.id == id).map(|r| (r.label.as_str(), r.hint.as_deref())).unwrap();
        assert_eq!(said("projects-page"), ("Projects…", Some("order, names and rules")));
        assert_eq!(said("rank-shop"), ("Rank Storefront first", Some("not ranked yet")));
        assert_eq!(said("update-check"), ("Check for updates", Some("look for one now")));
        assert_eq!(said("pause"), ("Pause agents", Some("no new sessions spawn")));
        assert_eq!(said("resume-stopped"), ("Resume interrupted agents", Some("after limits or crashes")));
        assert_eq!(said("usage"), ("Usage and limits", Some("what is left")));
        assert_eq!(said("settings"), ("Settings…", Some("workspace and every project")));
        assert_eq!(said("shortcuts"), ("Keyboard shortcuts", Some("every key Mercury has")));
        assert_eq!(said("keyhints"), ("Turn off keyboard shortcut hints", Some("shown when you point")));
        assert_eq!(rows.iter().find(|r| r.id == "usage").unwrap().does, Does::Settings(PageId::Claude));
    }

    #[test]
    fn the_projects_are_ranked_first_by_name_and_say_where_they_stand() {
        let w = World::of(json!({ "products": [{ "slug": "shop", "name": "Storefront", "createdAt": 1 }, { "slug": "app", "name": "Mobile", "createdAt": 2 },
            { "slug": "mine", "name": "My workspace", "personal": true, "createdAt": 3 }, { "slug": "ada", "name": "Ada", "team": { "direct": true } }],
            "supervisor": { "productOrder": ["app", "shop"] } }));
        let rows = project_rows(&w.snap);
        let said: Vec<(&str, &str)> = rows.iter().skip(1).map(|r| (r.label.as_str(), r.hint.as_deref().unwrap())).collect();
        assert_eq!(said, [("Rank Mobile first", "now #1 of 3"), ("Rank Storefront first", "now #2 of 3"), ("Rank My workspace first", "made for you")]);
        assert_eq!(rank_first(&w.snap, "mine"), ("zero:set-product-order", json!({ "order": ["mine", "app", "shop", "ada"] })));
        // "rank" and "priority" find them through the Projects page's words; the project's name finds its own row.
        assert_eq!(typed("mobile", &rows), ["rank-app"]);
        assert_eq!(typed("priority", &rows), ["projects-page"]);
    }

    #[test]
    fn the_clouds_rows_turn_it_on_or_off_and_off_needs_the_word_cloud() {
        let on = World::of(json!({ "cloud": { "setUp": true, "mode": "all", "whenOn": "all" } }));
        let rows = cloud_rows(&on.snap);
        assert_eq!(rows.iter().map(|r| (r.label.as_str(), r.hint.as_deref(), r.key)).collect::<Vec<_>>(), [
            ("Turn the cloud off", None, Some("⌘C")),
            ("Cloud on: when this Mac is full", None, None),
            ("Cloud on: everything", Some("On now"), None),
        ]);
        assert_eq!(rows.iter().map(|r| r.does.clone()).collect::<Vec<_>>(), [Does::Cloud(cloud::Mode::Mac), Does::Cloud(cloud::Mode::Full), Does::Cloud(cloud::Mode::All)]);
        // Listed with nothing typed; typed for, only by words that name the cloud.
        assert!(matches_query("", &rows[0]));
        assert!(!matches_query("off", &rows[0]));
        assert!(matches_query("cloud off", &rows[0]) && matches_query("cl off", &rows[0]));
        assert!(!matches_query("c off", &rows[0]));
        assert!(matches_query("off", &rows[1]));
        // Off, it turns on to what on meant, and needs no word.
        let off = World::of(json!({ "cloud": { "setUp": true, "mode": "mac", "whenOn": "all" } }));
        let rows = cloud_rows(&off.snap);
        assert_eq!((rows[0].label.as_str(), &rows[0].does, rows[0].only_with), ("Turn the cloud on", &Does::Cloud(cloud::Mode::All), None));
        assert!(rows.iter().all(|r| r.hint.is_none()));
        // A Mac not set up for the cloud has none of them.
        assert!(cloud_rows(&World::of(json!({ "cloud": { "setUp": false } })).snap).is_empty());
        assert_eq!(icon_for("cloud-toggle", "Turn the cloud off"), "cloud");
    }

    #[test]
    fn the_update_rows_follow_the_new_version() {
        let rows = |update: Value, restart: Value| update_rows(&World::of(json!({ "update": update, "restartNeeded": restart })).snap).into_iter().map(|r| (r.id, r.label, r.hint)).collect::<Vec<_>>();
        let s = |v: &str| v.to_string();
        assert_eq!(rows(json!({ "phase": "current" }), Value::Null), [(s("update-check"), s("Check for updates"), Some(s("look for one now")))]);
        assert_eq!(rows(json!({ "ready": true, "newVersion": "0.2.0" }), Value::Null), [(s("update"), s("Restart to update Mercury"), Some(s("version 0.2.0 is ready")))]);
        // Ready with no version to name: nothing to restart onto by name, so the check stays.
        assert_eq!(rows(json!({ "ready": true }), Value::Null)[0].0, "update-check");
        assert_eq!(rows(json!({ "canMove": true }), Value::Null)[0], (s("update-move"), s("Move to Applications"), Some(s("cannot update from here"))));
        assert!(rows(json!({ "canMove": true, "installing": true, "newVersion": "0.2.0" }), Value::Null).iter().all(|r| r.0 != "update-move"));
        assert_eq!(rows(Value::Null, json!({ "files": ["main/ipc.mts", "main/store.mts"] })).last().unwrap(), &(s("stale"), s("Restart Mercury to pick up main-process changes"), Some(s("2 files changed"))));
        assert_eq!(rows(Value::Null, json!({ "files": ["main/ipc.mts"] })).last().unwrap().2.as_deref(), Some("1 file changed"));
    }

    #[test]
    fn what_the_engine_answers_is_said_in_the_webs_words() {
        assert_eq!(run_now_said(&json!({ "ok": true })), "Up next. It starts as soon as an agent is free.");
        assert_eq!(run_now_said(&json!({ "ok": false, "reason": "paused" })), "Agents are paused. Turn them back on to run this.");
        assert_eq!(run_now_said(&json!({ "ok": false, "reason": "missing" })), "This task is no longer waiting.");
        // The engine's answer carries the supervisor's own queue after the count, so it reads as
        // queued whatever happened, as the web window reads it.
        assert_eq!(resumed_said(&json!({ "ok": true, "resumed": 1, "queued": [] })), ("1 resumed,  waiting for a slot".to_string(), true, true));
        assert_eq!(resumed_said(&json!({ "ok": true, "resumed": 0, "queued": ["w-2", "w-3"] })), ("w-2,w-3 waiting for a slot".to_string(), true, false));
        assert_eq!(resumed_said(&json!({ "resumed": 0, "queued": 0, "working": 1 })).0, "Already working on it");
        assert_eq!(resumed_said(&json!({})).0, "Nothing to resume");
        assert_eq!(resume_stopped_said(&json!({ "ok": true, "cleared": 1 })), "Resuming: 1 interrupted agent respawning");
        assert_eq!(resume_stopped_said(&json!({ "ok": true, "cleared": 3 })), "Resuming: 3 interrupted agents respawning");
        assert_eq!(resume_stopped_said(&json!({ "ok": true, "cleared": 0 })), "Nothing stranded. Anything waiting respawns on its own");
        assert_eq!(redeliver_said(&json!({ "ok": true }), "Storefront").as_deref(), Some("A fresh agent is picking this up → Storefront"));
        assert_eq!(redeliver_said(&json!({ "ok": false, "error": "no live answer to redeliver" }), "Storefront"), None);
        let thread = answered();
        assert_eq!(reopened_said(&thread), ("Reopened: Fix the export · press Z to undo".to_string(), "Done again: Fix the export".to_string()));
        assert_eq!(reopen_done(&thread), ("zero:answer", json!({ "product": "shop", "id": "w-1", "status": "open" })));
        let named = ("w-1".to_string(), "shop".to_string());
        assert_eq!(done_again(&named), ("zero:answer", json!({ "product": "shop", "id": "w-1", "status": "done" })));
        assert_eq!(move_item(&thread, "app"), ("zero:move-item", json!({ "product": "shop", "id": "w-1", "to": "app" })));
        assert_eq!(move_back(&json!({ "id": "w-9", "product": "app" }), "app", &named), vec![
            ("zero:answer", json!({ "product": "app", "id": "w-9", "status": "done" })),
            ("zero:answer", json!({ "product": "shop", "id": "w-1", "status": "open" })),
        ]);
        assert_eq!((pause(true), resume_stopped(), resume(&thread)), (("zero:supervisor-pause", json!(true)), ("zero:resume-agents", Value::Null), ("zero:resume-items", json!({ "ids": ["w-1"] }))));
        assert_eq!((send_back(&thread).0, redeliver(&thread).0, run_now(&thread).0), ("zero:reopen", "zero:redeliver", "zero:run-now"));
    }

    #[test]
    fn a_thread_that_cannot_move_says_why() {
        let open = item(json!({ "id": "w-1", "product": "shop", "status": "open" }));
        assert_eq!(why_it_cannot_move(&open), None);
        assert_eq!(why_it_cannot_move(&item(json!({ "id": "a", "product": "shop", "status": "open", "agent": { "pid": 1 } }))), Some("An agent\u{2019}s own session does not live in a project."));
        let unshipped = item(json!({ "id": "w-2", "product": "shop", "status": "open", "note": "It did not ship because the branch is behind main.", "wrote": { "note": { "ts": 2, "source": "system" } } }));
        assert_eq!(why_it_cannot_move(&unshipped), Some("This thread changed code in its project. Ship it or close it there first."));
        for q in ["move", "refile", "wrong project"] {
            assert!(matches_query(q, &Row::command("move-app", "Move to Mobile", Does::Home).found_by(MOVE_KEYWORDS)), "{q}");
        }
    }

    #[test]
    fn a_row_held_back_from_the_untyped_list_shows_only_once_words_find_it() {
        let mut demo = cmd("demo", "Open a demo copy", "demo presentation");
        demo.unlisted = true;
        demo.only_with = Some("demo");
        assert!(!matches_query("", &demo));
        assert!(matches_query("demo", &demo) && matches_query("de", &demo));
        assert!(!matches_query("presentation", &demo));
        assert!(listed(vec![demo.clone()], Vec::new(), "").rows.is_empty());
        assert_eq!(ids(&listed(vec![demo], Vec::new(), "demo").rows), ["demo"]);
    }

    /// The rows over an open thread in the web window, where two of them start with the same
    /// three letters.
    fn over_a_thread() -> Vec<Row> {
        vec![
            cmd("remote-control", "Remote control", "phone mobile continue rc"),
            cmd("open-terminal", "Open terminal", "shell command console"),
            cmd("done", "Mark done", ""),
            cmd("reply", "Reply", ""),
            cmd("read", "Mark read", ""),
            cmd("snooze", "Later", LATER_KEYWORDS),
        ]
    }

    #[test]
    fn typing_rem_puts_later_under_the_return_key() {
        let rows = over_a_thread();
        assert_eq!(typed("rem", &rows), ["snooze", "remote-control"]);
        assert_eq!(typed("REm", &rows), ["snooze", "remote-control"]);
        // One letter more still reaches the other row on its own.
        assert_eq!(typed("remo", &rows), ["remote-control"]);
        assert_eq!(typed("phone", &rows), ["remote-control"]);
        // Everything else keeps the order the list was put together in.
        assert_eq!(typed("e", &rows), ["snooze", "remote-control", "open-terminal", "done", "reply", "read"]);
        assert_eq!(typed("con", &rows), ["remote-control", "open-terminal"]);
        assert_eq!(typed("re", &rows), ["snooze", "remote-control", "reply", "read"]);
        assert!(typed("nothing here", &rows).is_empty());
        // Nothing typed is not sorted at all.
        assert_eq!(ids(&rank_matches("   ", rows.clone())), ids(&rows));
        assert_eq!(typed("later", &World::empty().on(&open_thread(), false)), ["snooze"]);
    }

    #[test]
    fn a_word_that_only_touches_a_keyword_comes_after_the_threads_it_found() {
        let rename = cmd("rename", "Rename thread", "name title");
        // A command matched only by a keyword follows the threads, each under its heading.
        let listed = order(vec![rename.clone()], vec![thread("1", "A title for it")], "title");
        assert_eq!(ids(&listed.rows), ["thread-1", "rename"]);
        assert_eq!((listed.label_over(0), listed.label_over(1)), (Some("Threads"), Some("Commands")));
        // A command named by what was typed stays first, and Threads sits under it.
        let listed = order(vec![rename.clone()], vec![thread("1", "Rename the button")], "rename");
        assert_eq!(ids(&listed.rows), ["rename", "thread-1"]);
        assert_eq!((listed.label_over(0), listed.label_over(1)), (None, Some("Threads")));
        assert!(!listed.labels.iter().any(|(_, label)| *label == "Commands"));
        // With no threads found the command is still there, under no heading.
        let listed = order(vec![rename.clone()], Vec::new(), "title");
        assert_eq!(ids(&listed.rows), ["rename"]);
        assert!(listed.labels.is_empty());
        // Nothing typed is the list it always was.
        let listed = order(vec![rename, cmd("tutorial", "Take the tutorial", "")], Vec::new(), "");
        assert_eq!(ids(&listed.rows), ["rename", "tutorial"]);
        assert!(listed.labels.is_empty());
    }

    #[test]
    fn a_command_goes_ahead_only_when_the_words_start_words_of_its_name() {
        // "sum" is inside "resume" and "up" inside "interrupted": that is not its name.
        let resume = cmd("resume", "Resume interrupted agents", "");
        let task = thread("x", "Sum up this week's support mail");
        assert_eq!(ids(&order(vec![resume.clone()], vec![task.clone()], "Sum up").rows), ["thread-x", "resume"]);
        assert_eq!(order(vec![resume, cmd("snooze", "Later", "")], vec![task], "resume").rows[0].id, "resume");
        assert!(names_it("rem", "Remote control"));
        assert!(!names_it("sum up", "Resume interrupted agents"));
        assert!(!names_it("  ", "Later"));
        // A command whose own name has the words goes above threads named by them too.
        let (light, dark) = (cmd("look-light", "Theme: Light", "theme appearance"), cmd("look-dark", "Theme: Dark", "theme appearance"));
        let light_thread = thread("1", "Light theme keyboard shortcut visibility");
        assert_eq!(order(vec![light.clone()], vec![light_thread.clone(), thread("2", "Tray colour detection")], "light").rows[0].id, "look-light");
        assert_eq!(ids(&order(vec![light.clone(), dark], vec![light_thread.clone()], "theme").rows), ["look-light", "look-dark", "thread-1"]);
        // What the words only touch by keyword still goes after.
        assert_eq!(ids(&order(vec![cmd("look", "Appearance", "theme light dark")], vec![light_thread], "light").rows), ["thread-1", "look"]);
    }

    #[test]
    fn typing_a_threads_name_puts_that_thread_under_enter() {
        // A command that carries the whole title of a thread the words name stays under it.
        let waits = cmd("wait-w-9", "Waits on October investor update", "waits waiting blocked by depends needs");
        let investor = thread("w-9", "October investor update");
        let listed = order(vec![waits.clone()], vec![investor.clone()], "investor update");
        assert_eq!(ids(&listed.rows), ["thread-w-9", "wait-w-9"]);
        assert_eq!((listed.label_over(0), listed.label_over(1)), (Some("Threads"), Some("Commands")));
        assert_eq!(order(vec![waits.clone()], vec![investor.clone()], "October investor update").rows[0].id, "thread-w-9");
        assert_eq!(order(vec![waits.clone()], vec![investor.clone()], "invest").rows[0].id, "thread-w-9");
        // Typed by its own name, the command still comes first.
        assert_eq!(order(vec![waits.clone()], vec![investor.clone()], "waits on").rows[0].id, "wait-w-9");
        assert_eq!(order(vec![waits.clone()], vec![investor], "waits on october").rows[0].id, "wait-w-9");
        // And a found thread the words do not name leaves the command first.
        assert_eq!(order(vec![waits], vec![thread("3", "Board pack")], "investor update").rows[0].id, "wait-w-9");
    }

    #[test]
    fn a_command_found_by_a_name_it_had_goes_above_the_threads() {
        let later = World::empty().on(&open_thread(), true).into_iter().find(|r| r.id == "snooze").unwrap();
        let remote = cmd("remote-control", "Remote control", "");
        let reminders = || vec![thread("1", "Remind me to cancel the card"), thread("2", "Reminder to write about testing")];
        let ordered = |q: &str, rows: Vec<Row>| listed(rows, reminders(), q);
        assert_eq!(ids(&ordered("remind me", vec![later.clone()]).rows), ["snooze", "thread-1", "thread-2"]);
        assert_eq!(ordered("snooze", vec![later.clone()]).rows[0].id, "snooze");
        assert_eq!(ids(&ordered("rem", vec![later.clone(), remote]).rows), ["snooze", "remote-control", "thread-1", "thread-2"]);
        let headed = ordered("remind me", vec![later.clone()]);
        assert_eq!(headed.label_over(1), Some("Threads"));
        assert!(!headed.labels.iter().any(|(_, label)| *label == "Commands"));
        // A keyword that was never its name still goes after, and so do letters from inside one.
        assert_eq!(ids(&ordered("defer", vec![later.clone()]).rows), ["thread-1", "thread-2", "snooze"]);
        assert_eq!(ids(&ordered("ind", vec![later]).rows), ["thread-1", "thread-2", "snooze"]);
    }

    #[test]
    fn the_thread_titled_by_the_words_comes_first() {
        let triage = thread("1", "Integration branch security and deployment triage");
        let passwords = thread("2", "Per-clinic staff passwords");
        let quality = thread("3", "Quality lead writes to the vet");
        assert_eq!(ids(&title_first(vec![triage.clone(), quality, passwords.clone()], "Per-clinic")), ["thread-2", "thread-1", "thread-3"]);
        // The exact title, then a title that starts with the words, then one that has them.
        let rows = vec![thread("a", "Staff passwords per-clinic follow-up"), thread("b", "Per-clinic staff passwords plan"), thread("c", "Per-clinic staff passwords")];
        assert_eq!(ids(&title_first(rows, "per-clinic staff passwords")), ["thread-c", "thread-b", "thread-a"]);
        // Rows that match alike keep the search's own order, and nothing typed moves nothing.
        assert_eq!(ids(&title_first(vec![thread("x", "Notes"), thread("y", "Notes")], "notes")), ["thread-x", "thread-y"]);
        assert_eq!(ids(&title_first(vec![triage, passwords], "  ")), ["thread-1", "thread-2"]);
    }

    #[test]
    fn the_pick_stays_on_its_row() {
        let keys = |ids: &[&str]| ids.iter().map(|k| k.to_string()).collect::<Vec<_>>();
        // When the rows sort again under it, the same row stays picked.
        assert_eq!(kept_pick(Some("c"), &keys(&["a", "b", "c"])), Some(2));
        // When its row is gone the first is picked, never a stale place.
        assert_eq!(kept_pick(Some("gone"), &keys(&["a", "b"])), Some(0));
        assert_eq!(kept_pick(None, &keys(&["a"])), Some(0));
        assert_eq!(kept_pick(Some("gone"), &[]), None);
        // Two rows that share an id still have a key each.
        assert_eq!(row_keys(&[cmd("done", "Mark done", ""), cmd("reply", "Reply", ""), cmd("done", "Done (2 selected)", "")]), ["done", "reply", "done#1"]);
    }

    fn storefront() -> Snapshot {
        let now = Day::real(2026, 10, 20).unwrap().at(12, 0);
        let min = 60_000.0;
        serde_json::from_value(json!({
            "products": [{ "slug": "shop", "name": "Storefront" }, { "slug": "app", "name": "Mobile" }],
            "items": [
                { "id": "w-1", "product": "shop", "status": "done", "title": "Checkout fails for some customers", "body": "Checkout fails for some customers",
                  "result": "Found it. Tax rounding broke on large carts.", "createdAt": now - 90.0 * min, "updatedAt": now - 4.0 * min },
                { "id": "w-2", "product": "app", "status": "open", "title": "Onboarding checklist copy", "body": "Onboarding checklist copy", "createdAt": now - 200.0 * min, "updatedAt": now - 180.0 * min },
                { "id": "w-3", "product": "gone", "status": "open", "title": "Release notes", "body": "Write them. Then check the links.", "createdAt": now - 200.0 * min,
                  "updatedAt": Day::real(2026, 10, 16).unwrap().at(9, 0) },
                { "id": "w-4", "product": "shop", "status": "open", "title": "Empty inbox drawing", "body": "Three directions.", "createdAt": now - 20_000.0 }
            ]
        }))
        .unwrap()
    }

    fn find(snap: &Snapshot, query: &str) -> Vec<Row> {
        let mut index = search::Index::default();
        index.read(&snap.items);
        threads_found(&index, snap, query, Day::real(2026, 10, 20).unwrap().at(12, 0))
    }

    #[test]
    fn a_found_thread_says_what_it_is_where_and_how_old() {
        let snap = storefront();
        let rows = find(&snap, "check");
        // Titles that have the word first, newest first; then the one that only mentions it.
        assert_eq!(ids(&rows), ["thread-shop-w-1", "thread-app-w-2", "thread-gone-w-3"]);
        let checkout = &rows[0];
        assert_eq!(checkout.label, "Checkout fails for some customers");
        assert_eq!(checkout.sub.as_deref(), Some("Checkout fails for some customers"));
        assert_eq!(checkout.hint.as_deref(), Some("Storefront · 4 min"));
        // Dimmed once finished, and only then.
        assert!(checkout.done && !rows[1].done);
        assert_eq!(checkout.does, Does::Open(("w-1".to_string(), "shop".to_string())));
        // A found thread wears the thread's drawing, whatever its project and id.
        assert_eq!(icon_for(&checkout.id, &checkout.label), "thread");
        assert_eq!(rows[1].hint.as_deref(), Some("Mobile · 3 hr"));
        // A day old or more it gives its date, and a project nobody named is its slug.
        assert_eq!(rows[2].hint.as_deref(), Some("gone · Nov 16"));
        assert_eq!(rows[2].sub.as_deref(), Some("Then check the links."));
        // Under a minute old it says now, from when it was made if it never changed.
        assert_eq!(find(&snap, "drawing")[0].hint.as_deref(), Some("Storefront · now"));
        assert_eq!(ago_short(0.0, 49.0 * 3_600_000.0), "2 d");
        // Nothing typed finds nothing: the bar then lists commands alone.
        assert!(find(&snap, "  ").is_empty());
    }

    #[test]
    fn the_best_few_threads_are_listed_and_the_one_titled_by_the_words_leads() {
        let items: Vec<Value> = (0..8).map(|n| json!({ "id": format!("w-{n}"), "product": "shop", "status": "open", "title": format!("Export run {n}"), "body": "Export it.", "updatedAt": n })).collect();
        let mut snap: Snapshot = serde_json::from_value(json!({ "products": [], "items": items })).unwrap();
        assert_eq!(ids(&find(&snap, "export")), ["thread-shop-w-7", "thread-shop-w-6", "thread-shop-w-5", "thread-shop-w-4", "thread-shop-w-3"]);
        assert_eq!(find(&snap, "export").len(), THREADS_SHOWN);
        // The thread whose title is the words goes first among the ones listed.
        snap.items[5].title = Some("Export".to_string());
        assert_eq!(ids(&find(&snap, "export"))[0], "thread-shop-w-5");
    }

    #[test]
    fn what_is_typed_decides_the_whole_list() {
        let snap = storefront();
        let over = snap.items[1].clone();
        let (local, models) = (Local::default(), Models::built());
        let facts = Facts { snap: &snap, local: &local, now: NOW, models: &models, file_projects: &[], key_hints: true, starts_threads: true };
        let all = commands(&facts, Some((&over, false)));
        let list = |query: &str| listed(all.clone(), find(&snap, query), query);
        // Nothing typed: every command, no threads, no heading.
        let untyped = list("");
        assert_eq!(untyped.rows.len(), all.len());
        assert!(untyped.labels.is_empty() && untyped.rows.iter().all(|r| !r.id.starts_with("thread-")));
        // A command named by the word leads, then the threads under their heading.
        let found = list("check");
        assert_eq!(ids(&found.rows), ["update-check", "thread-shop-w-1", "thread-app-w-2", "thread-gone-w-3"]);
        assert_eq!(found.labels, [(1, "Threads")]);
        // A word no command has: the threads alone.
        let found = list("checkout");
        assert_eq!(ids(&found.rows), ["thread-shop-w-1"]);
        assert_eq!(found.labels, [(0, "Threads")]);
        // A command's own name leads, and a thread that says the word follows it.
        let inbox = list("inbox");
        assert_eq!(ids(&inbox.rows), ["view-inbox", "thread-shop-w-4"]);
        assert_eq!(inbox.labels, [(1, "Threads")]);
        assert!(list("zebra").rows.is_empty());
    }

    #[test]
    fn nothing_found_says_what_was_searched_for() {
        assert_eq!(empty_line("  zebra "), "Nothing matches \u{201c}zebra\u{201d}.");
        // A pasted paragraph is cut at a word, not in the middle of one.
        assert_eq!(empty_line("the quick brown fox jumps over the lazy dog"), "Nothing matches \u{201c}the quick brown fox jumps over…\u{201d}.");
        assert_eq!(empty_line(&"x".repeat(40)), format!("Nothing matches \u{201c}{}…\u{201d}.", "x".repeat(32)));
    }

    #[test]
    fn every_row_this_window_draws_has_the_webs_drawing() {
        // Every row of a list with the cloud set up, an update waiting, a stale build, a project
        // to move to, a queued thread and a blocked one.
        let w = World::of(json!({ "products": [{ "slug": "shop", "name": "Storefront" }, { "slug": "app", "name": "Mobile" }],
            "supervisor": { "running": [], "queued": ["w-1"], "stalled": ["w-2", "w-3"] }, "cloud": { "setUp": true, "mode": "full", "projects": ["shop"] },
            "update": { "ready": true, "newVersion": "0.2.0", "canMove": true }, "restartNeeded": { "files": ["main/ipc.mts"] } }));
        let blocked = item(json!({ "id": "w-2", "product": "shop", "status": "blocked", "title": "Stuck" }));
        // A finished thread whose agent stopped is not one Try again stands for.
        let stopped = item(json!({ "id": "w-3", "product": "shop", "status": "done", "title": "Stopped" }));
        let facts = Facts { file_projects: &["Mobile".to_string()], ..w.facts(NOW) };
        let rows: Vec<Row> = [(&open_thread(), false), (&blocked, false), (&stopped, true), (&answered(), true)].into_iter().flat_map(|under| commands(&facts, Some(under))).collect();
        for id in ["approve", "run-now", "place", "send-back", "redeliver", "update", "update-move", "stale", "cloud-toggle", "files-project-Mobile", "move-app"] {
            assert!(rows.iter().any(|r| r.id == id), "{id}");
        }
        for row in &rows {
            assert_ne!(icon_for(&row.id, &row.label), PLAIN_ICON, "{}", row.id);
        }
        assert_eq!(icon_for("new-thread", "New thread"), "compose");
        assert_eq!(icon_for("reopen", "Reopen"), "reopen");
        assert_eq!(icon_for("unsnooze", "Back to Inbox"), "inbox");
        assert_eq!(icon_for("thread-shop-w-1", "Fix the export"), "thread");
        assert_eq!(icon_for("something-else", "Something else"), PLAIN_ICON);
        // The longest start wins, and a toggle's label says which way it goes.
        assert_eq!(icon_for("teammate-pause-ada", "Pause Ada"), "pause");
        assert_eq!(icon_for("teammate-pause-ada", "Resume Ada"), "play");
        assert_eq!(icon_for("teammate-ada", "Ada"), "person");
        assert_eq!(icon_for("pause", "Unpause agents"), "play");
        assert_eq!(icon_for("priority-urgent", "Urgent"), "priority-urgent");
        assert_eq!(icon_for("priority-low", "Low"), "priority-low");
    }

    /// `tests/a-stuck-thread-offers-try-again.test.mts`: the bar's row is `try-again`, labelled
    /// Try again, after Reply, found by the words a person reaching for it types.
    #[test]
    fn try_again_goes_after_reply_and_is_found_by_what_a_stuck_thread_is_called() {
        let mut rows = World::empty().on(&open_thread(), true);
        with_try_again(&mut rows, ("w-1".to_string(), "shop".to_string()));
        let at = ids(&rows).iter().position(|id| *id == "try-again").expect("the row");
        assert_eq!(ids(&rows)[at - 1], "reply");
        assert_eq!(rows[at].does, Does::TryAgain(("w-1".to_string(), "shop".to_string())));
        assert_eq!(icon_for("try-again", "Try again"), "refresh");
        for q in ["try again", "retry", "stuck", "restart", "resume"] {
            assert!(matches_query(q, &rows[at]), "{q}");
        }
    }
}
