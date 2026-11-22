//! Whether anything is happening on a thread, and what: the line at the conversation's foot
//! (`renderer/src/live-line.ts`, drawn by `components/Live.tsx`), Try again where nothing will
//! start it without a person (`renderer/src/try-again.ts`), and the line under the title while
//! an agent works (`workingContext` in `renderer/src/one-thing-at-a-time.ts`).
//!
//! Every fact comes off the snapshot: the running session (`supervisor.running`: when it
//! started, the tool calls still running, the sentence being typed, where it runs) and the
//! supervisor's own lists (`queued`, `runNow`, `signInNeeded`, `atLimit`, `cannotStart`,
//! `shipping`, `stalled`, `silent`, `paused`, `remoteHeld`, `whyNotCloud`), read as `factsFor`
//! in `renderer/src/status-word.ts` reads them. The Inbox's Status column reads the same line
//! (`rules::status`), so the two cannot say two things.

use serde::Deserialize;
use serde_json::Value;

use super::js::{self, re};
use super::place::{self, Why};
use crate::data::{Item, Supervisor, truthy};

/// One tool call still running (`activity` on `RunningSession`).
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Activity {
    pub id: String,
    /// "Running the tests".
    pub label: String,
    /// The whole command.
    pub detail: String,
    pub started_at: Option<f64>,
}

/// What the snapshot says of a run that is up (`RunningSession`).
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Session {
    pub item_id: String,
    pub product: String,
    pub started_at: f64,
    /// When the reply a cloud run is working on went in.
    pub turn_started_at: Option<f64>,
    pub activity: Vec<Activity>,
    /// The prose being typed this second, never on disk.
    pub saying: Option<String>,
    pub saying_at: Option<f64>,
    pub helpers: Option<u32>,
    /// `cloud` while the run is in a cloud sandbox.
    #[serde(rename = "where")]
    pub place: Option<String>,
}

/// How long a run has been up, never a seconds counter (`shortSpan`): under a minute it is
/// "just now", and it is floored, so it never says more time has passed than has.
pub fn short_span(ms: f64) -> String {
    let seconds = (ms / 1000.0).floor().max(0.0) as u64;
    if seconds < 60 {
        return "just now".to_string();
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{minutes}m");
    }
    format!("{}h {}m", minutes / 60, minutes % 60)
}

/// When the work on screen began: the reply a cloud run took between turns, else the run's
/// start (`runningSince`).
pub fn running_since(session: &Session) -> f64 {
    session.turn_started_at.filter(|t| *t != 0.0).unwrap_or(session.started_at)
}

/* ------------------------------------ the states ------------------------------------ */

/// Which of the things that can be true of a thread's agent is true (`LiveState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Stopped on a permission request or a sign-in only the person can give.
    Asking,
    Working,
    /// A live claim no session of this window holds: an agent in a terminal or on another Mac.
    Elsewhere,
    /// The app's ship queue still owes it a ship.
    Shipping,
    /// Queued and pushed with Run now.
    Next,
    Queued,
    /// Queued, but the last pass could not start it, for a reason the app can say.
    Cannot,
    SignIn,
    Limit,
    Paused,
    /// The last run ended and wrote nothing down.
    Silent,
    Idle,
}

/// The one word each state gets (`shortWord`). A run's subagents are counted inside it, and
/// only while it works.
pub fn short_word(state: State, helpers: u32) -> String {
    let word = match state {
        State::Working if helpers == 1 => return "1 Subagent Working".to_string(),
        State::Working if helpers > 1 => return format!("{helpers} Subagents Working"),
        State::Asking => "Asking",
        // The same word for a run of ours and one outside this window; the mark says where.
        State::Working | State::Elsewhere => "Working",
        State::Shipping => "Shipping",
        State::Next => "Up next",
        State::Queued => "Queued",
        State::Cannot => "Not started",
        State::SignIn => "Signed out",
        State::Limit => "At its limit",
        State::Paused => "Paused",
        State::Silent => "Nothing came back",
        State::Idle => "Nothing running",
    };
    word.to_string()
}

/// The sentence for a queued row nothing will start yet, or none for a reason it does not carry
/// (`cannotStartLine`). `tool` is "Claude Code" or "Codex".
pub fn cannot_start_line(why: &str, tool: &str) -> Option<String> {
    Some(match why {
        "setup" => "Agents wait while setup is open. This starts as soon as you finish or skip it.".to_string(),
        "asleep" => "This computer looks asleep to the app, so nothing starts until it wakes.".to_string(),
        "cloud-carried" => "This computer is asleep, so the cloud is starting this. It runs here instead if the cloud cannot.".to_string(),
        "engine" => format!("{tool} was not found on this computer, so nothing can start. Install it and this starts on its own."),
        "login" if tool == "Codex" => "Codex has no account that can run right now. Open a terminal and run codex login, and this starts on its own.".to_string(),
        "login" => format!("{tool} has no account that can run right now. Open a terminal, run claude, and type /login, and this starts on its own."),
        "memory" => "This computer is short of memory, so this waits for heavier work to finish.".to_string(),
        "slow-folder" => "Making this thread's own copy of the project is taking much longer than it should, so the agent has not started.".to_string(),
        "error" => "Starting an agent on this failed. The app tries again on its own.".to_string(),
        "brake" => format!("{tool}'s last runs stopped before they began, so the app is waiting before it tries again."),
        _ => return None,
    })
}

/// What the supervisor and the window say about one thread (`LiveFacts`, built as `factsFor`
/// builds them, with what `App.tsx` and `Focus.tsx` add for an open conversation).
#[derive(Clone, Debug, Default)]
pub struct Facts {
    /// The run on it now.
    pub session: Option<Session>,
    /// `supervisor.queued`, or none before the fleet has been read at all.
    pub queued: Option<Vec<String>>,
    /// On `supervisor.runNow`.
    pub run_now: bool,
    /// On `supervisor.remoteHeld`: this window's own session, open between Remote Control turns.
    pub remote_held: bool,
    /// `supervisor.signInNeeded[id]`: the tool it is signed out of.
    pub sign_in_needed: Option<String>,
    /// The Inbox's sign-in bar is up. This window draws no such bar, so it stays false here.
    pub sign_in_bar: bool,
    /// `supervisor.atLimit[id]`: the tool out of usage, and when it resets (0 when no login said).
    pub at_limit: Option<(String, f64)>,
    /// `supervisor.cannotStart[id]`: why the last pass could not start it, and its tool.
    pub cannot_start: Option<(String, String)>,
    pub shipping: bool,
    /// An approval names it, or an open sign-in waits on it.
    pub asking: bool,
    /// A worker died on it.
    pub stalled: bool,
    /// `supervisor.silent[id]`: when the run that wrote nothing ended (0 when not said).
    pub silent: Option<f64>,
    /// Spawning is off.
    pub paused: bool,
    /// `supervisor.whyNotCloud[id]`.
    pub why_not_cloud: Option<Why>,
    /// The thread is under In progress (`belongsInProgress`, or the row's tab).
    pub in_progress: bool,
    /// The moment it is put off to, when that is ahead.
    pub scheduled_until: f64,
    /// The walkthrough's practice thread, with the app's answer to it on its way.
    pub answering: bool,
    /// The row is in the tutorial's practice project, where nothing is ever started.
    pub practice: bool,
    pub now: f64,
}

fn listed(supervisor: &Supervisor, list: &str, id: &str) -> bool {
    supervisor.rest.get(list).and_then(Value::as_array).is_some_and(|ids| ids.iter().any(|v| v.as_str() == Some(id)))
}

fn keyed<'a>(supervisor: &'a Supervisor, map: &str, id: &str) -> Option<&'a Value> {
    supervisor.rest.get(map).and_then(Value::as_object).and_then(|m| m.get(id)).filter(|v| truthy(Some(v)))
}

fn text_of(v: &Value, field: &str) -> Option<String> {
    v.get(field).and_then(|v| v.as_str().map(str::to_string).or_else(|| v.as_f64().map(|n| n.to_string())))
}

/// Why a thread runs or waits where it does, as the supervisor says it (`whyNotCloud[id]`).
pub fn why_not_cloud(supervisor: &Supervisor, id: &str) -> Option<Why> {
    let w = keyed(supervisor, "whyNotCloud", id)?;
    Some(Why { at: text_of(w, "at").unwrap_or_default(), why: text_of(w, "why").unwrap_or_default(), detail: text_of(w, "detail").filter(|d| !d.is_empty()) })
}

/// The run on a thread, as the fleet keys it: by the thread's id, the newest entry winning
/// (`new Map(running.map(…))`).
pub fn session_of(supervisor: &Supervisor, id: &str) -> Option<Session> {
    let run = supervisor.running.iter().rev().find(|r| r.item_id == id)?;
    let mut facts = run.rest.clone();
    facts.insert("itemId".to_string(), Value::String(run.item_id.clone()));
    facts.insert("product".to_string(), Value::String(run.product.clone()));
    serde_json::from_value(Value::Object(facts)).ok()
}

impl Facts {
    /// What the supervisor says about the thread `id`, read off its lists by that id and
    /// nothing else (`factsFor`). `asking`, `in_progress`, `scheduled_until` and `practice` are
    /// the caller's to say, as `stalled` is where the caller narrows it.
    pub fn of(supervisor: &Supervisor, id: &str, now: f64) -> Facts {
        let s = supervisor;
        Facts {
            session: session_of(s, id),
            queued: s.rest.get("queued").and_then(Value::as_array).map(|ids| ids.iter().filter_map(Value::as_str).map(str::to_string).collect()),
            run_now: listed(s, "runNow", id),
            remote_held: listed(s, "remoteHeld", id),
            sign_in_needed: keyed(s, "signInNeeded", id).map(|tool| tool.as_str().unwrap_or("Claude Code").to_string()),
            sign_in_bar: false,
            at_limit: keyed(s, "atLimit", id).map(|l| (text_of(l, "tool").unwrap_or_default(), l.get("until").and_then(Value::as_f64).unwrap_or(0.0))),
            cannot_start: keyed(s, "cannotStart", id).map(|c| (text_of(c, "why").unwrap_or_default(), text_of(c, "tool").unwrap_or_default())),
            shipping: listed(s, "shipping", id),
            asking: false,
            stalled: listed(s, "stalled", id),
            silent: keyed(s, "silent", id).map(|q| q.get("endedAt").and_then(Value::as_f64).unwrap_or(0.0)),
            paused: truthy(s.rest.get("paused")),
            why_not_cloud: why_not_cloud(s, id),
            in_progress: false,
            scheduled_until: 0.0,
            answering: false,
            practice: false,
            now,
        }
    }
    fn is_queued(&self, id: &str) -> bool {
        self.queued.as_ref().is_some_and(|q| q.iter().any(|q| q == id))
    }
}

/// What the line says: its state, and the whole sentence, which is what it is read aloud as
/// (`LiveLine`).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub state: State,
    pub line: String,
}

/// Has an agent actually written something here. `(withdrawn)` is the app's own word for a
/// field that has been taken back, so it is not something to read.
fn said(text: Option<&str>) -> bool {
    text.map(js::trim).is_some_and(|t| !t.is_empty() && t != "(withdrawn)")
}

/// May this row say that nothing came back (`cameBackEmpty`): the last run ended and wrote
/// nothing, and the row has nothing on it to read.
pub fn came_back_empty(item: &Item, silent: Option<f64>) -> bool {
    silent.is_some() && !said(item.result.as_deref()) && !said(item.string("note"))
}

/// How long a thread just sent reads Queued before the fleet has listed it.
const JUST_SENT_MS: f64 = 30_000.0;

/// Sent or replied to moments ago, and no agent has claimed it or written on it since.
fn just_sent(i: &Item, now: f64) -> bool {
    let created = i.created_at.unwrap_or(0.0);
    let sent_at = created.max(i.wrote_at("answer"));
    if now - sent_at >= JUST_SENT_MS {
        return false;
    }
    if i.created_at == Some(sent_at) && i.number("epoch").unwrap_or(0.0) > 0.0 {
        return false;
    }
    !i.object("wrote").is_some_and(|w| w.values().any(|w| w.get("source").and_then(Value::as_str) == Some("agent") && w.get("ts").and_then(Value::as_f64).is_some_and(|ts| ts >= sent_at)))
}

/// "3:00pm": the clock a reset is said at (`toLocaleTimeString` with two-digit minutes, its
/// spaces taken out and lowered).
fn clock_at(ts: f64) -> String {
    let t = rune::time::LocalTime::at(ts as i64);
    let hour = if t.hour % 12 == 0 { 12 } else { t.hour % 12 };
    format!("{hour}:{:02}{}", t.minute, if t.hour < 12 { "am" } else { "pm" })
}

/// The service an open sign-in waits on (`openSignIn(item).service`).
fn sign_in_service(i: &Item) -> Option<&str> {
    super::kinds::open_sign_in(i).then(|| i.object("signIn").and_then(|s| s.get("service")).and_then(Value::as_str).unwrap_or("it"))
}

/// Whether anything is actually happening on this thread, and the sentence that says so
/// (`liveLine`). None where the line says nothing: a finished thread, a stopped one (its bar
/// says it), one put off (its bar says it), and one out of In progress with nothing to report.
pub fn line(item: &Item, f: &Facts) -> Option<Line> {
    let now = f.now;
    let says = |state: State, line: &str| Some(Line { state, line: line.to_string() });
    // An imported terminal session is not one of ours.
    if item.truthy("agent") {
        return None;
    }
    // The walkthrough's practice rows are answered by the app and never started; while that
    // answer is on its way the thread reads as one at work, or it looks stuck.
    if item.has_label("first-run") {
        return if f.answering { says(State::Working, "The agent just started on this.") } else { None };
    }
    // Finished is finished: the result at the foot is the news.
    if item.status == "done" {
        return None;
    }
    // Above the session: a worker waiting on an approval is a live session in every list.
    if f.asking {
        return match sign_in_service(item) {
            Some(service) => says(State::Asking, &format!("The agent is waiting for you to sign in to {service}.")),
            None => says(State::Asking, "The agent stopped to ask your permission. Nothing happens on this thread until you answer."),
        };
    }
    if let Some(session) = &f.session {
        let elapsed = (now - running_since(session)).max(0.0);
        let helpers = session.helpers.unwrap_or(0);
        if helpers > 0 {
            let many = if helpers == 1 { "one subagent".to_string() } else { format!("{helpers} subagents") };
            return says(State::Working, &if elapsed < 60_000.0 { format!("The agent just started on this, with {many} out.") } else { format!("The agent is working with {many}, {} in.", super::notes::span_label(elapsed)) });
        }
        return says(State::Working, &if elapsed < 60_000.0 { "The agent just started on this.".to_string() } else { format!("The agent is working, {} in.", super::notes::span_label(elapsed)) });
    }
    // Signed out is not queued, and it is said in or out of In progress.
    if let Some(tool) = f.sign_in_needed.as_deref().filter(|_| !f.shipping) {
        let plan = if tool == "Codex" { "ChatGPT" } else { "Claude" };
        let line = if f.sign_in_bar {
            format!("{tool} is signed out. Sign in to {plan} at the top of your inbox, and this starts on its own.")
        } else if tool == "Codex" {
            "Codex is signed out. Open a terminal and run codex login, and this starts on its own.".to_string()
        } else {
            format!("{tool} is signed out. Open a terminal, run claude, and type /login, and this starts on its own.")
        };
        return says(State::SignIn, &line);
    }
    // The two bars over the conversation say these, with the button that fixes them.
    if f.stalled || f.scheduled_until > now {
        return None;
    }
    // A run ended here and wrote nothing down. Above the In progress gate on purpose: such a row
    // sits in the inbox looking untouched. Never under something there is to read.
    if came_back_empty(item, f.silent) {
        let ended = f.silent.filter(|at| *at != 0.0).unwrap_or(now);
        let quiet = (now - ended).max(0.0);
        return says(State::Silent, &if quiet < 60_000.0 {
            "The last agent finished without saying anything. Send it a message to find out what happened.".to_string()
        } else {
            format!("The last agent finished without saying anything, {} ago. Send it a message to find out what happened.", super::notes::span_label(quiet))
        });
    }
    if !f.in_progress {
        return None;
    }
    if f.shipping {
        return says(State::Shipping, &format!("{} is shipping this. It comes back to you once it is live, or goes back to its agent if it fails.", super::settings::NAME));
    }
    // A live claim no session of ours holds.
    if item.status == "claimed" && item.truthy("claim") && !item.truthy("claimExpired") && !f.remote_held {
        return says(State::Elsewhere, "An agent is working on this outside this window, in a terminal or on another computer, so its steps do not show here.");
    }
    if let Some((tool, until)) = &f.at_limit {
        let at = (*until > now).then(|| clock_at(*until));
        let day = at.as_ref().and_then(|_| super::settings::reset_day(*until, now));
        let on = day.map(|d| format!(" on {d}")).unwrap_or_default();
        let at = at.map(|a| format!(" at {a}")).unwrap_or_default();
        return says(State::Limit, &format!("{tool} is at its usage limit, so this starts on its own when it resets{on}{at}."));
    }
    // Queued is a promise that an agent is coming, and for these it is not.
    let queued = f.is_queued(&item.id);
    if let Some(line) = f.cannot_start.as_ref().filter(|_| queued).and_then(|(why, tool)| cannot_start_line(why, tool)) {
        return says(State::Cannot, &line);
    }
    if queued && f.run_now {
        return says(State::Next, "Up next. It starts as soon as an agent is free, and if every agent is busy the least important one is paused to make room.");
    }
    if queued {
        return says(State::Queued, "Queued. An agent starts on it as soon as one is free.");
    }
    if f.paused {
        return says(State::Paused, "Agents are paused, so nothing will start on this.");
    }
    // The window draws a thread just sent before the fleet's next snapshot lists it.
    if just_sent(item, now) {
        return says(State::Queued, "Queued. An agent starts on it as soon as one is free.");
    }
    says(State::Idle, "No agent is on this, and none is waiting to start.")
}

/// Where a working run is, said only where it is news (`livePlace`): a cloud run, or one on
/// this Mac while the cloud takes everything, which carries the card saying why.
#[derive(Clone, Debug, PartialEq)]
pub struct Place {
    pub word: String,
    /// The card's head and body, on a run on this Mac.
    pub why: Option<(String, String)>,
}

pub fn place(f: &Facts) -> Option<Place> {
    let session = f.session.as_ref()?;
    if session.place.as_deref() == Some("cloud") {
        return Some(Place { word: "in the cloud".to_string(), why: None });
    }
    let away = f.why_not_cloud.as_ref().filter(|w| w.at == "mac")?;
    Some(Place { word: "on this Mac".to_string(), why: Some(("Running on this Mac".to_string(), place::why_words(away))) })
}

/// The head of the card a cloud's why is said in (`whyWords`'s head); its body is
/// `place::why_words`.
pub fn why_head(w: &Why) -> &'static str {
    match w.at.as_str() {
        "wait-cloud" => "Queued for the cloud",
        "wait-mac" => "Queued to run locally",
        _ if w.why == "moving" => "Moving to the cloud",
        _ => "Running locally",
    }
}

/* ------------------------------------- try again ------------------------------------- */

/// Whether a press of Try again could put an agent back on a thread nothing will start without
/// a person (`offersTryAgain`). Queued, Up next and Shipping come on their own, signed out waits
/// for a sign-in a press cannot do, and at its limit is offered because switching accounts is
/// exactly when someone wants one more try.
pub fn offers_try_again(item: &Item, f: &Facts) -> bool {
    if item.truthy("agent") || item.status == "done" || f.session.is_some() || f.practice || item.has_label("first-run") {
        return false;
    }
    // Without the fleet this window knows nothing about what is coming.
    let Some(queued) = &f.queued else { return false };
    if f.asking || f.shipping || f.remote_held || f.scheduled_until > f.now {
        return false;
    }
    if f.stalled && f.sign_in_needed.is_none() {
        return true;
    }
    if let Some(live) = line(item, f) {
        return matches!(live.state, State::Idle | State::Silent | State::Limit);
    }
    // Out of In progress, the newest word on it is the app's own note that a run stopped, and no
    // start is queued for it yet. Another try at a model for the other agent is refused the same way.
    item.status == "open"
        && !queued.iter().any(|q| q == &item.id)
        && item.wrote("result").is_some_and(|w| w.source == "agent")
        && super::nowhere::says_a_run_got_nowhere(item.result.as_deref())
        && !super::nowhere::says_the_model_is_for_another_engine(item.result.as_deref())
}

/// The channel Try again writes: Resume, on the one thread (`resumeItems`).
pub const TRY_AGAIN: &str = "zero:resume-items";

/// What Try again writes for the thread `id`.
pub fn try_again(id: &str) -> Value {
    serde_json::json!({ "ids": [id] })
}

/// What a press of Try again says, whatever came back, so a press is never silent
/// (`tryAgainWords`). `None` is no app to ask; a door that failed reads as a start that failed,
/// with the door's reason, as the web window's `catch` turns it.
pub fn try_again_words(answer: Option<&Result<Value, String>>) -> String {
    let r = match answer {
        None => return "Try again did not reach the app.".to_string(),
        Some(Err(why)) => serde_json::json!({ "failed": 1, "why": why }),
        Some(Ok(Value::Null)) => return "Try again did not reach the app.".to_string(),
        Some(Ok(r)) => r.clone(),
    };
    let n = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0) > 0.0;
    if n("resumed") {
        return "Trying again now.".to_string();
    }
    if n("queued") {
        return "Queued.".to_string();
    }
    if n("working") {
        return "Already running.".to_string();
    }
    if n("failed") {
        let why = r.get("why").and_then(|w| w.as_str().map(str::to_string).or_else(|| (!w.is_null()).then(|| w.to_string()))).unwrap_or_else(|| "the app did not say why".to_string());
        return format!("It could not start: {}.", why.strip_suffix('.').unwrap_or(&why));
    }
    if n("missing") {
        return "This thread is gone from this computer.".to_string();
    }
    "It cannot start on this computer: another computer runs it.".to_string()
}

/// Whether Try again is a press this thread can take at all (`tryAgain`'s guard): not an
/// imported session, nor the row the app makes when tasks cannot run, nor its update row.
pub fn can_try_again(item: &Item) -> bool {
    !item.truthy("agent") && item.id != super::nowhere::TROUBLE_ID && item.id != "update"
}

/* ------------------------------- the line, as it is drawn ------------------------------- */

/// What the line at the conversation's foot draws (`Live.tsx`), worked out from the line and the
/// facts. None where it draws nothing.
#[derive(Clone, Debug, PartialEq)]
pub struct Foot {
    pub state: State,
    /// What is running now by its own label, or "Thinking", while it works; the state's word, or
    /// for signed out, at its limit and cannot start the whole sentence, otherwise.
    pub word: String,
    /// "+2": the other calls running beside it, while it works.
    pub more: Option<String>,
    /// How long the run has been up, or how long ago the quiet run ended.
    pub span: Option<String>,
    /// Where it runs, while it works and only where that is news.
    pub place: Option<Place>,
    /// The mark turns: an agent is on it, here or elsewhere.
    pub turning: bool,
    /// Try again is drawn after it.
    pub retry: bool,
    /// What it is read aloud as: the word while it works, else the whole sentence.
    pub announced: String,
}

impl Foot {
    pub fn working(&self) -> bool {
        self.state == State::Working
    }
}

/// The line at the foot, as `Live.tsx` draws it. `retries` says the window can try again (it
/// is handed a way to); Try again is then drawn where `offersTryAgain` says, except while the
/// thread is stopped, whose bar carries it. Out of In progress, where the line says nothing, a
/// thread Try again is offered on gets the one line true of it: nothing running.
pub fn foot(item: &Item, f: &Facts, retries: bool) -> Option<Foot> {
    let retry = retries && !f.stalled && offers_try_again(item, f);
    let Some(live) = line(item, f) else {
        return retry.then(|| Foot { state: State::Idle, word: short_word(State::Idle, 0), more: None, span: None, place: None, turning: false, retry: true, announced: short_word(State::Idle, 0) });
    };
    let working = live.state == State::Working;
    let span = match &f.session {
        Some(session) => Some(short_span(f.now - running_since(session))),
        None if live.state == State::Silent => f.silent.filter(|at| *at != 0.0).map(|ended| short_span(f.now - ended)),
        None => None,
    };
    let activity = f.session.as_ref().map_or(&[][..], |s| s.activity.as_slice());
    let word = if working {
        activity.last().map(|a| a.label.clone()).filter(|l| !l.is_empty()).unwrap_or_else(|| "Thinking".to_string())
    } else if matches!(live.state, State::SignIn | State::Limit | State::Cannot) {
        live.line.clone()
    } else {
        short_word(live.state, f.session.as_ref().and_then(|s| s.helpers).unwrap_or(0))
    };
    Some(Foot {
        state: live.state,
        more: (working && activity.len() > 1).then(|| format!("+{}", activity.len() - 1)),
        span,
        place: if working { place(f) } else { None },
        turning: working || live.state == State::Elsewhere,
        retry,
        announced: if working { word.clone() } else { live.line.clone() },
        word,
    })
}

/// The line is drawn again this often while it counts something: a run up, or a quiet run's
/// end (`TICK_MS`). Its readings are whole minutes, so this only keeps a minute's turn from
/// being late.
pub const TICK_MS: f64 = 15_000.0;

/// Whether the line counts anything, and so is drawn again every `TICK_MS`.
pub fn counting(f: &Facts) -> bool {
    f.session.is_some() || f.silent.is_some_and(|at| at != 0.0)
}

/* --------------------------------- the steps it opens onto -------------------------------- */

/// How many of the steps a run finished since it last said anything the unfolded line lists;
/// the rest are counted (`LIVE_STEPS_SHOWN` in `renderer/src/activity-summary.ts`).
pub const STEPS_SHOWN: usize = 5;

/// One step a run took since it last said anything, as its log has it: when, its verb, what it
/// acted on, and the whole command or path behind that when the words shortened it.
#[derive(Clone, Debug, PartialEq)]
pub struct Took {
    pub at: f64,
    pub verb: String,
    pub subject: String,
    pub full: String,
}

/// A row of what the line unfolds onto (`ActivityRow`): its words, and the command it opens onto.
#[derive(Clone, Debug, PartialEq)]
pub struct StepRow {
    pub label: String,
    pub detail: String,
}

/// What the working line unfolds onto (`Live.tsx`, expanded): how many finished steps came
/// before the newest it lists, and the rows, the finished ones oldest first and then each call
/// running now.
#[derive(Clone, Debug, PartialEq)]
pub struct Steps {
    pub earlier: usize,
    pub rows: Vec<StepRow>,
}

/// The steps of `history` not still running: each call in `running` takes the newest step
/// with its command out of the list, unless that step is older than the call by more than a
/// second (`previousActivity`).
pub fn previous_activity(history: &[Took], running: &[Activity]) -> Vec<Took> {
    let normal = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut previous = history.to_vec();
    for action in running {
        let found = (0..previous.len()).rev().find(|&at| {
            let row = &previous[at];
            let too_old = action.started_at.is_some_and(|started| row.at < started - 1000.0);
            let said = if row.full.is_empty() { &row.subject } else { &row.full };
            !too_old && normal(said) == normal(&action.detail)
        });
        if let Some(at) = found {
            previous.remove(at);
        }
    }
    previous
}

/// What the line unfolds onto, from the steps the run took since it last said anything and the
/// calls running now: the newest five finished, oldest at the top, then what runs now.
pub fn steps(history: &[Took], running: &[Activity]) -> Steps {
    let finished = previous_activity(history, running);
    let recent = &finished[finished.len().saturating_sub(STEPS_SHOWN)..];
    let mut rows: Vec<StepRow> = recent
        .iter()
        .map(|took| {
            let mut verb = took.verb.chars();
            let capital = verb.next().map(|c| c.to_uppercase().collect::<String>() + verb.as_str()).unwrap_or_default();
            StepRow { label: format!("{capital} {}", took.subject).trim().to_string(), detail: if took.full.is_empty() { took.subject.clone() } else { took.full.clone() } }
        })
        .collect();
    rows.extend(running.iter().map(|a| StepRow { label: a.label.clone(), detail: a.detail.clone() }));
    Steps { earlier: finished.len() - recent.len(), rows }
}

/// The line over the rows for the finished steps not listed.
pub fn earlier_steps(earlier: usize) -> String {
    format!("{earlier} earlier {} since it last said something", if earlier == 1 { "step" } else { "steps" })
}

/// What the steps say when the run's log could not be read.
pub const HISTORY_UNREAD: &str = "Couldn’t read the command history.";

/// What they say with nothing finished and nothing running.
pub const NO_COMMANDS: &str = "No commands yet.";

/* --------------------------------- the line under the title -------------------------------- */

/// The words every run of a repeating task is briefed with, as the person wrote them
/// (`instructionOf` in `shared/repeats.mts`).
fn instruction_of(body: &str) -> &str {
    const OPENS: &str = "THIS IS ONE RUN OF A REPEATING TASK.";
    const MARK: &str = "HER INSTRUCTION FOLLOWS.";
    match body.find(&format!("\n{MARK}\n")) {
        Some(at) if body.starts_with(OPENS) => js::trim(&body[at + MARK.len() + 2..]),
        _ => body,
    }
}

re!(PARAGRAPHS, r"\n\s*\n", "");
re!(SPACES, r"\s+", "g");
re!(ENDS_A_SENTENCE, r"[.!?]$", "");
re!(LEAD_IN, r":\s*$", "");
re!(LINK, r"\[([^\]\n]+)\]\([^)\s]+\)", "g");

/// The line under the title while an agent works: the first paragraph of what was asked,
/// unless the title already says it (`askLine`). A paragraph that ends in a colon is a lead-in
/// ("Priya Shah asked:") that says who and not what, so the first one that does not is taken,
/// and a link in it reads as its words.
pub fn ask_line(ask: &str, title: &str) -> Option<String> {
    let flat = |s: &str| js::trim(&SPACES.replace_all(s, |_| " ".to_string())).to_string();
    let paragraphs = PARAGRAPHS.split(js::trim(instruction_of(ask)));
    let at = paragraphs.iter().position(|p| !LEAD_IN.test(p)).unwrap_or(0);
    let first = flat(&LINK.replace_all(paragraphs.get(at).copied().unwrap_or(""), |caps| js::group(caps, 1).to_string()));
    let named = flat(title).to_lowercase();
    let lower = first.to_lowercase();
    // A repeating run's title is its ask with the day after it.
    if first.is_empty() || named == lower || named.starts_with(&format!("{lower} · ")) {
        return None;
    }
    Some(format!("You asked: {}", if ENDS_A_SENTENCE.test(&first) { first } else { format!("{first}.") }))
}

/// The line under a working thread's title: the ask, and its schedule if it repeats
/// (`workingContext`). `repeats` is "Repeats daily at 9:00am" for a run of a repeating task.
pub fn working_context(ask: &str, title: &str, repeats: Option<&str>) -> Option<String> {
    let said: Vec<String> = [ask_line(ask, title), repeats.filter(|r| !r.is_empty()).map(|r| format!("{r}."))].into_iter().flatten().collect();
    (!said.is_empty()).then(|| said.join(" "))
}

/// What a working thread's line under its title is asked of: its body, or its title when it
/// has no body at all, and nothing for a cause's row (`isCause(item) ? '' : item.body ?? item.title`).
pub fn asked_of(item: &Item) -> &str {
    if super::kinds::is_cause(item) {
        return "";
    }
    item.body.as_deref().or(item.title.as_deref()).unwrap_or("")
}

const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];

/// "Repeats daily at 9:00am" (`repeatsLine`, `ruleLabel`): a rule read off `zero:repeats`. A
/// rule of no shape it knows repeats "Once", as the web window says.
pub fn repeats_line(rule: &Value) -> String {
    let at = rule.get("at").and_then(Value::as_str).filter(|at| {
        let b = at.as_bytes();
        b.len() == 5 && b[2] == b':' && at[..2].parse::<u32>().is_ok_and(|h| h < 24) && at[3..].parse::<u32>().is_ok_and(|m| m < 60) && b[..2].iter().chain(&b[3..]).all(u8::is_ascii_digit)
    });
    let every = rule.get("every").and_then(Value::as_str);
    let on = rule.get("on").and_then(Value::as_f64).filter(|d| d.fract() == 0.0 && (0.0..=6.0).contains(d));
    let label = match (at, every, on) {
        (Some(at), Some("day"), _) => format!("daily at {}", clock_label(at)),
        (Some(at), Some("weekday"), _) => format!("weekdays at {}", clock_label(at)),
        (Some(at), Some("week"), Some(on)) => format!("{}s at {}", DAYS[on as usize], clock_label(at)),
        _ => "Once".to_string(),
    };
    format!("Repeats {label}")
}

/// "9:00am" (`clockLabel`): a rule's hour and minute, as the system's clock writes them.
fn clock_label(at: &str) -> String {
    let (h, m) = (at[..2].parse::<u32>().unwrap_or(0), at[3..].parse::<u32>().unwrap_or(0));
    let hour = if h % 12 == 0 { 12 } else { h % 12 };
    format!("{hour}:{m:02}{}", if h < 12 { "am" } else { "pm" })
}

/// The rule a run of a repeating task belongs to, among the rules `zero:repeats` lists.
pub fn rule_for<'a>(item: &Item, rules: &'a [Value]) -> Option<&'a Value> {
    let id = super::kinds::rule_id_of(item)?;
    rules.iter().find(|r| r.get("product").and_then(Value::as_str) == Some(item.product.as_str()) && r.get("id").and_then(Value::as_str) == Some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_800_000_000_000.0;

    fn session(started: f64) -> Session {
        Session { item_id: "w-1".into(), product: "p".into(), started_at: started, ..Session::default() }
    }

    fn item(over: Value) -> Item {
        let mut v = json!({ "id": "w-1", "product": "agentbox", "productName": "Mercury", "status": "open", "title": "A task", "kind": "directive", "labels": ["founder"], "priority": 5, "epoch": 1, "claim": null, "createdAt": NOW - 600_000.0, "updatedAt": NOW - 600_000.0 });
        for (k, value) in over.as_object().unwrap() {
            v[k] = value.clone();
        }
        serde_json::from_value(v).unwrap()
    }

    fn facts() -> Facts {
        Facts { now: NOW, ..Facts::default() }
    }

    fn in_progress() -> Facts {
        Facts { in_progress: true, queued: Some(vec![]), ..facts() }
    }

    fn state(i: &Item, f: &Facts) -> Option<State> {
        line(i, f).map(|l| l.state)
    }

    /// `tests/is-anything-happening.test.mts`.
    #[test]
    fn a_span_is_never_a_seconds_counter() {
        assert_eq!(short_span(0.0), "just now");
        assert_eq!(short_span(41_000.0), "just now");
        assert_eq!(short_span(59_999.0), "just now");
        assert_eq!(short_span(60_000.0), "1m");
        assert_eq!(short_span(4.0 * 60_000.0), "4m");
        assert_eq!(short_span(59.0 * 60_000.0), "59m");
        assert_eq!(short_span(2.0 * 3_600_000.0 + 7.0 * 60_000.0), "2h 7m");
        assert_eq!(short_span(-5_000.0), "just now");
    }

    #[test]
    fn a_worker_on_it_says_so_in_whole_minutes_and_never_in_seconds() {
        let claimed = item(json!({ "status": "claimed" }));
        let working = Facts { session: Some(session(NOW - 240_000.0)), ..in_progress() };
        assert_eq!(line(&claimed, &working), Some(Line { state: State::Working, line: "The agent is working, 4 minutes in.".into() }));
        let fresh = Facts { session: Some(session(NOW - 12_000.0)), ..in_progress() };
        assert_eq!(line(&claimed, &fresh).unwrap().line, "The agent just started on this.");
        // A live session is a fact about the row wherever it is listed.
        assert_eq!(state(&item(json!({})), &Facts { session: Some(session(NOW - 240_000.0)), ..facts() }), Some(State::Working));
    }

    #[test]
    fn nothing_on_it_yet_says_queued_paused_or_admits_nothing_is_coming() {
        let i = item(json!({}));
        let queued = Facts { queued: Some(vec!["w-1".into()]), ..in_progress() };
        assert_eq!(line(&i, &queued), Some(Line { state: State::Queued, line: "Queued. An agent starts on it as soon as one is free.".into() }));
        assert_eq!(short_word(State::Queued, 0), "Queued");
        let paused = Facts { paused: true, ..in_progress() };
        assert!(line(&i, &paused).unwrap().line.contains("paused"));
        let other = Facts { queued: Some(vec!["w-other".into()]), ..in_progress() };
        assert_eq!(line(&i, &other), Some(Line { state: State::Idle, line: "No agent is on this, and none is waiting to start.".into() }));
    }

    #[test]
    fn it_refuses_to_say_twice_what_a_bar_says_and_says_nothing_out_of_in_progress() {
        let i = item(json!({ "answer": "go on" }));
        assert_eq!(line(&i, &Facts { stalled: true, ..in_progress() }), None);
        assert_eq!(line(&i, &Facts { scheduled_until: NOW + 3_600_000.0, ..in_progress() }), None);
        assert_eq!(line(&i, &facts()), None);
        assert_eq!(line(&item(json!({ "status": "done" })), &in_progress()), None);
        let imported = item(json!({ "agent": { "pid": 4, "sessionId": "s", "cwd": "/tmp", "name": "x" } }));
        assert_eq!(line(&imported, &Facts { session: Some(session(NOW)), ..in_progress() }), None);
    }

    #[test]
    fn one_word_for_every_state_and_the_subagents_inside_the_working_one() {
        assert_eq!(short_word(State::Working, 0), "Working");
        assert_eq!(short_word(State::Paused, 0), "Paused");
        assert_eq!(short_word(State::Idle, 0), "Nothing running");
        assert_eq!(short_word(State::Working, 4), "4 Subagents Working");
        assert_eq!(short_word(State::Working, 1), "1 Subagent Working");
        assert_eq!(short_word(State::Queued, 4), "Queued");
        assert_eq!(short_word(State::Silent, 4), "Nothing came back");
        assert_eq!(short_word(State::Idle, 4), "Nothing running");
        let claimed = item(json!({ "status": "claimed" }));
        let mut s = session(NOW - 240_000.0);
        s.helpers = Some(4);
        assert_eq!(line(&claimed, &Facts { session: Some(s.clone()), ..in_progress() }).unwrap().line, "The agent is working with 4 subagents, 4 minutes in.");
        s.helpers = Some(1);
        assert_eq!(line(&claimed, &Facts { session: Some(s), ..in_progress() }).unwrap().line, "The agent is working with one subagent, 4 minutes in.");
    }

    #[test]
    fn a_run_that_ended_and_wrote_nothing_says_so_in_any_tab_and_never_over_something_to_read() {
        let quiet = Some(NOW - 600_000.0);
        let i = item(json!({}));
        assert_eq!(
            line(&i, &Facts { silent: quiet, ..in_progress() }),
            Some(Line { state: State::Silent, line: "The last agent finished without saying anything, 10 minutes ago. Send it a message to find out what happened.".into() })
        );
        assert_eq!(state(&i, &Facts { silent: quiet, ..facts() }), Some(State::Silent));
        assert_eq!(state(&item(json!({ "status": "claimed" })), &Facts { silent: quiet, session: Some(session(NOW - 240_000.0)), ..in_progress() }), Some(State::Working));
        assert_eq!(line(&i, &Facts { silent: quiet, stalled: true, ..in_progress() }), None);
        assert_eq!(line(&i, &Facts { silent: quiet, scheduled_until: NOW + 60_000.0, ..in_progress() }), None);
        assert_eq!(state(&i, &in_progress()), Some(State::Idle));
        assert_ne!(state(&item(json!({ "result": "Pick a way back out." })), &Facts { silent: quiet, ..in_progress() }), Some(State::Silent));
        assert_ne!(state(&item(json!({ "note": "Half way through the branch." })), &Facts { silent: quiet, ..in_progress() }), Some(State::Silent));
        assert_eq!(state(&item(json!({ "result": "(withdrawn)" })), &Facts { silent: quiet, ..in_progress() }), Some(State::Silent));
        assert_eq!(line(&item(json!({ "status": "done" })), &Facts { silent: quiet, ..in_progress() }), None);
        assert_eq!(line(&i, &Facts { silent: Some(NOW - 12_000.0), ..in_progress() }).unwrap().line, "The last agent finished without saying anything. Send it a message to find out what happened.");
        // `cameBackEmpty`.
        assert!(came_back_empty(&item(json!({ "result": "", "note": "   " })), Some(1.0)));
        assert!(!came_back_empty(&item(json!({ "note": "a checkpoint" })), Some(1.0)));
        assert!(!came_back_empty(&item(json!({})), None));
    }

    /// `tests/a-signed-out-thread-reads-the-same-wherever-it-stopped.test.mts`,
    /// `a-task-waiting-on-a-usage-limit-says-so-not-queued`, `a-thread-another-agent-holds-says-working`.
    #[test]
    fn signed_out_at_its_limit_cannot_start_and_elsewhere_each_say_their_own_sentence() {
        let i = item(json!({}));
        let signed_out = Facts { sign_in_needed: Some("Claude Code".into()), ..facts() };
        assert_eq!(line(&i, &signed_out).unwrap(), Line { state: State::SignIn, line: "Claude Code is signed out. Open a terminal, run claude, and type /login, and this starts on its own.".into() });
        let codex = Facts { sign_in_needed: Some("Codex".into()), ..facts() };
        assert_eq!(line(&i, &codex).unwrap().line, "Codex is signed out. Open a terminal and run codex login, and this starts on its own.");
        let bar = Facts { sign_in_needed: Some("Codex".into()), sign_in_bar: true, ..facts() };
        assert_eq!(line(&i, &bar).unwrap().line, "Codex is signed out. Sign in to ChatGPT at the top of your inbox, and this starts on its own.");
        // Shipping outranks a sign-in, and then is said only In progress.
        assert_eq!(state(&i, &Facts { sign_in_needed: Some("Codex".into()), shipping: true, ..in_progress() }), Some(State::Shipping));
        let limit = Facts { at_limit: Some(("Claude Code".into(), 0.0)), ..in_progress() };
        assert_eq!(line(&i, &limit).unwrap().line, "Claude Code is at its usage limit, so this starts on its own when it resets.");
        let later = NOW + 2.0 * 3_600_000.0;
        let said = line(&i, &Facts { at_limit: Some(("Codex".into(), later)), ..in_progress() }).unwrap().line;
        assert_eq!(said, format!("Codex is at its usage limit, so this starts on its own when it resets{} at {}.", super::super::settings::reset_day(later, NOW).map(|d| format!(" on {d}")).unwrap_or_default(), clock_at(later)));
        let asleep = Facts { queued: Some(vec!["w-1".into()]), cannot_start: Some(("asleep".into(), "Claude Code".into())), ..in_progress() };
        assert_eq!(line(&i, &asleep).unwrap(), Line { state: State::Cannot, line: "This computer looks asleep to the app, so nothing starts until it wakes.".into() });
        // A reason with no sentence reads as queued.
        let odd = Facts { cannot_start: Some(("something-new".into(), "Codex".into())), ..asleep.clone() };
        assert_eq!(state(&i, &odd), Some(State::Queued));
        let elsewhere = item(json!({ "status": "claimed", "claim": { "holder": "mcp-9" } }));
        assert_eq!(state(&elsewhere, &in_progress()), Some(State::Elsewhere));
        assert_eq!(state(&elsewhere, &Facts { remote_held: true, ..in_progress() }), Some(State::Idle));
        let expired = item(json!({ "status": "claimed", "claim": { "holder": "mcp-9" }, "claimExpired": true }));
        assert_eq!(state(&expired, &in_progress()), Some(State::Idle));
        assert_eq!(state(&i, &Facts { queued: Some(vec!["w-1".into()]), run_now: true, ..in_progress() }), Some(State::Next));
        for (why, tool, said) in [("login", "Codex", "Codex has no account that can run right now. Open a terminal and run codex login, and this starts on its own."), ("engine", "Codex", "Codex was not found on this computer, so nothing can start. Install it and this starts on its own."), ("brake", "Claude Code", "Claude Code's last runs stopped before they began, so the app is waiting before it tries again.")] {
            assert_eq!(cannot_start_line(why, tool).as_deref(), Some(said));
        }
    }

    /// `tests/a-thread-that-asks-you-says-so-wherever-it-is-drawn.test.mts`.
    #[test]
    fn a_yes_waited_on_outranks_the_run_behind_it() {
        let i = item(json!({ "status": "claimed" }));
        let asking = Facts { asking: true, session: Some(session(NOW - 240_000.0)), ..in_progress() };
        assert_eq!(line(&i, &asking).unwrap(), Line { state: State::Asking, line: "The agent stopped to ask your permission. Nothing happens on this thread until you answer.".into() });
        let link = item(json!({ "signIn": { "id": "s-1", "service": "Vercel", "url": "https://example.com/login" } }));
        assert_eq!(line(&link, &Facts { asking: true, ..facts() }).unwrap().line, "The agent is waiting for you to sign in to Vercel.");
        // Asking draws its word and the time of the run behind it, with the mark still.
        let drawn = foot(&i, &asking, true).unwrap();
        assert_eq!((drawn.word.as_str(), drawn.span.as_deref(), drawn.turning, drawn.retry), ("Asking", Some("4m"), false, false));
    }

    /// `tests/a-thread-just-sent-reads-queued-not-nothing-running.test.mts`,
    /// `the-practice-thread-shows-its-agent-thinking-until-it-answers`.
    #[test]
    fn a_thread_just_sent_reads_queued_and_a_practice_thread_works_while_it_is_answered() {
        let fresh = item(json!({ "createdAt": NOW - 5_000.0, "epoch": 0 }));
        assert_eq!(state(&fresh, &in_progress()), Some(State::Queued));
        let started = item(json!({ "createdAt": NOW - 5_000.0, "epoch": 1 }));
        assert_eq!(state(&started, &in_progress()), Some(State::Idle));
        let practice = item(json!({ "labels": ["founder", "first-run"] }));
        assert_eq!(line(&practice, &Facts { answering: true, ..in_progress() }).unwrap(), Line { state: State::Working, line: "The agent just started on this.".into() });
        assert_eq!(line(&practice, &in_progress()), None);
    }

    #[test]
    fn a_working_line_says_what_runs_now_how_many_beside_it_and_where() {
        let mut s = session(NOW - 4.0 * 60_000.0);
        let i = item(json!({ "status": "claimed" }));
        let drawn = foot(&i, &Facts { session: Some(s.clone()), ..in_progress() }, true).unwrap();
        assert_eq!((drawn.word.as_str(), drawn.more.as_deref(), drawn.span.as_deref(), drawn.place.as_ref(), drawn.turning, drawn.announced.as_str()), ("Thinking", None, Some("4m"), None, true, "Thinking"));
        s.activity = vec![
            Activity { id: "a".into(), label: "Reading notes.md".into(), detail: "notes.md".into(), started_at: None },
            Activity { id: "b".into(), label: "Running the tests".into(), detail: "npm test".into(), started_at: None },
        ];
        s.place = Some("cloud".into());
        let drawn = foot(&i, &Facts { session: Some(s.clone()), ..in_progress() }, true).unwrap();
        assert_eq!((drawn.word.as_str(), drawn.more.as_deref(), drawn.place.map(|p| p.word)), ("Running the tests", Some("+1"), Some("in the cloud".to_string())));
        // A cloud run's reply is timed from the reply.
        s.turn_started_at = Some(NOW - 30_000.0);
        assert_eq!(foot(&i, &Facts { session: Some(s), ..in_progress() }, true).unwrap().span.as_deref(), Some("just now"));
        // On this Mac only while the cloud takes everything, with the card saying why.
        let here = Facts { session: Some(session(NOW)), why_not_cloud: Some(Why { at: "mac".into(), why: "kept".into(), detail: None }), ..in_progress() };
        assert_eq!(place(&here), Some(Place { word: "on this Mac".into(), why: Some(("Running on this Mac".into(), "You chose to keep this thread on this computer.".into())) }));
        let waits = Facts { why_not_cloud: Some(Why { at: "wait-cloud".into(), why: "cloud-full".into(), detail: None }), ..here.clone() };
        assert_eq!(place(&waits), None);
        // `tests/an-open-thread-waiting-on-your-yes-stops-saying-it-is-working.test.mts`.
        assert_eq!(foot(&i, &Facts { asking: true, ..here }, true).unwrap().state, State::Asking);
    }

    #[test]
    fn the_still_states_draw_their_word_or_their_whole_sentence_and_the_quiet_run_its_age() {
        let i = item(json!({}));
        let drawn = |f: Facts| foot(&i, &f, true).unwrap();
        let signed = drawn(Facts { sign_in_needed: Some("Claude Code".into()), ..in_progress() });
        assert_eq!((signed.word.starts_with("Claude Code is signed out."), signed.turning, signed.span, signed.retry), (true, false, None, false));
        let limit = drawn(Facts { at_limit: Some(("Codex".into(), 0.0)), ..in_progress() });
        assert_eq!((limit.word.as_str(), limit.retry), ("Codex is at its usage limit, so this starts on its own when it resets.", true));
        let silent = drawn(Facts { silent: Some(NOW - 3.0 * 60_000.0), ..in_progress() });
        assert_eq!((silent.word.as_str(), silent.span.as_deref(), silent.retry, silent.announced.starts_with("The last agent finished")), ("Nothing came back", Some("3m"), true, true));
        let idle = drawn(in_progress());
        assert_eq!((idle.word.as_str(), idle.span, idle.retry), ("Nothing running", None, true));
        let queued = drawn(Facts { queued: Some(vec!["w-1".into()]), ..in_progress() });
        assert_eq!((queued.word.as_str(), queued.retry), ("Queued", false));
        let elsewhere = foot(&item(json!({ "status": "claimed", "claim": { "holder": "mcp-9" } })), &in_progress(), true).unwrap();
        assert_eq!((elsewhere.word.as_str(), elsewhere.turning, elsewhere.span), ("Working", true, None));
        // With no way to try again, none is drawn.
        assert!(!foot(&i, &in_progress(), false).unwrap().retry);
        // The clock turns while something is counted.
        assert!(counting(&Facts { silent: Some(NOW - 1.0), ..facts() }));
        assert!(!counting(&Facts { silent: Some(0.0), ..facts() }));
        assert!(!counting(&in_progress()));
    }

    /// `tests/a-stuck-thread-offers-try-again.test.mts`.
    #[test]
    fn try_again_is_offered_where_nothing_will_start_the_thread() {
        let hour = 3_600_000.0;
        let thread = item(json!({}));
        let fleet = Facts { queued: Some(vec![]), ..facts() };
        assert!(offers_try_again(&thread, &Facts { in_progress: true, ..fleet.clone() }));
        assert!(offers_try_again(&thread, &Facts { stalled: true, ..fleet.clone() }));
        assert!(offers_try_again(&thread, &Facts { silent: Some(NOW - 60_000.0), ..fleet.clone() }));
        assert!(offers_try_again(&thread, &Facts { in_progress: true, at_limit: Some(("Claude Code".into(), NOW + hour)), ..fleet.clone() }));
        let limit_note = item(json!({ "result": "Claude Code hit its usage limit partway through, so this task is not finished.", "wrote": { "title": { "ts": NOW - hour, "source": "founder" }, "result": { "ts": NOW - 60_000.0, "source": "agent" } } }));
        assert!(offers_try_again(&limit_note, &fleet));
        // And not where an agent is on it, coming, or waiting on the person.
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, session: Some(session(NOW)), ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, queued: Some(vec!["w-1".into()]), ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, queued: Some(vec!["w-1".into()]), run_now: true, ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, sign_in_needed: Some("Claude Code".into()), ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, asking: true, ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, shipping: true, ..fleet.clone() }));
        let answered = item(json!({ "result": "Here is the plan.", "wrote": { "title": { "ts": NOW - hour, "source": "founder" }, "result": { "ts": NOW - 60_000.0, "source": "agent" } } }));
        assert!(!offers_try_again(&answered, &fleet));
        assert!(!offers_try_again(&item(json!({ "status": "done" })), &Facts { stalled: true, ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, scheduled_until: NOW + hour, ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, practice: true, ..fleet.clone() }));
        assert!(!offers_try_again(&thread, &Facts { in_progress: true, queued: None, ..facts() }));
        // A model for the other agent is refused the same way again.
        let other = item(json!({ "result": "This task chose Codex's model \"gpt-5\", but this computer would run it with Claude Code. Nothing has been done.", "wrote": { "result": { "ts": NOW - 60_000.0, "source": "agent" } } }));
        assert!(!offers_try_again(&other, &fleet));
        // Out of In progress with the app's note on it, the line draws nothing running and Try again.
        let drawn = foot(&limit_note, &fleet, true).unwrap();
        assert_eq!((drawn.state, drawn.word.as_str(), drawn.turning, drawn.retry), (State::Idle, "Nothing running", false, true));
        // Stopped: the bar carries it, not the line.
        assert_eq!(foot(&thread, &Facts { stalled: true, ..fleet }, true), None);
    }

    #[test]
    fn a_press_of_try_again_always_says_what_came_back() {
        assert_eq!(try_again_words(None), "Try again did not reach the app.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "ok": true, "resumed": 1 })))), "Trying again now.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "ok": true, "resumed": 0, "queued": 1 })))), "Queued.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "working": 1 })))), "Already running.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "failed": 1, "why": "The folder is gone." })))), "It could not start: The folder is gone.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "failed": 1 })))), "It could not start: the app did not say why.");
        assert_eq!(try_again_words(Some(&Err("the door did not answer".into()))), "It could not start: the door did not answer.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "missing": 1 })))), "This thread is gone from this computer.");
        assert_eq!(try_again_words(Some(&Ok(json!({ "ok": true, "resumed": 0, "queued": 0, "working": 0, "missing": 0, "failed": 0 })))), "It cannot start on this computer: another computer runs it.");
        assert_eq!(try_again(&"w-1"), json!({ "ids": ["w-1"] }));
        assert!(!can_try_again(&item(json!({ "id": "trouble" }))));
        assert!(!can_try_again(&item(json!({ "id": "update" }))));
        assert!(can_try_again(&item(json!({}))));
    }

    #[test]
    fn the_facts_are_read_off_the_supervisor_by_the_threads_id() {
        let supervisor: Supervisor = serde_json::from_value(json!({
            "running": [{ "itemId": "w-1", "product": "p", "startedAt": 1.0 }, { "itemId": "w-1", "product": "p", "startedAt": 9.0, "where": "cloud" }],
            "queued": ["w-1"], "runNow": ["w-1"], "remoteHeld": [], "signInNeeded": { "w-1": "Codex" }, "atLimit": { "w-1": { "tool": "Codex", "until": 5 } },
            "cannotStart": { "w-1": { "why": "asleep", "tool": "Codex" } }, "shipping": ["w-1"], "stalled": ["w-1"], "silent": { "w-1": { "runs": 1, "endedAt": 7 } },
            "paused": true, "whyNotCloud": { "w-1": { "at": "mac", "why": "kept" } },
        }))
        .unwrap();
        let f = Facts::of(&supervisor, "w-1", NOW);
        assert_eq!(f.session.as_ref().map(|s| (s.started_at, s.place.as_deref())), Some((9.0, Some("cloud"))));
        assert_eq!((f.queued.as_deref(), f.run_now, f.remote_held), (Some(&["w-1".to_string()][..]), true, false));
        assert_eq!((f.sign_in_needed.as_deref(), f.at_limit.clone(), f.cannot_start.clone()), (Some("Codex"), Some(("Codex".to_string(), 5.0)), Some(("asleep".to_string(), "Codex".to_string()))));
        assert_eq!((f.shipping, f.stalled, f.silent, f.paused), (true, true, Some(7.0), true));
        assert_eq!(f.why_not_cloud.as_ref().map(|w| w.why.as_str()), Some("kept"));
        let other = Facts::of(&supervisor, "w-2", NOW);
        assert_eq!((other.session.is_none(), other.shipping, other.silent), (true, false, None));
        assert!(Facts::of(&Supervisor::default(), "w-1", NOW).queued.is_none());
        let run = json!({ "itemId": "w-1", "product": "p", "startedAt": 5.0, "where": "cloud", "activity": [{ "id": "a", "label": "Reading", "detail": "x", "startedAt": 4.0 }], "tail": [] });
        let s: Session = serde_json::from_value(run).unwrap();
        assert_eq!((s.started_at, s.place.as_deref(), s.activity.len()), (5.0, Some("cloud"), 1));
        assert_eq!(why_head(&Why { at: "wait-mac".into(), ..Why::default() }), "Queued to run locally");
        assert_eq!(why_head(&Why { at: "mac".into(), why: "moving".into(), detail: None }), "Moving to the cloud");
    }

    /// `tests/a-repeating-run-reads-as-your-words-on-the-day-it-ran.test.mts`.
    #[test]
    fn the_line_under_a_working_title_is_what_was_asked_and_when_it_repeats() {
        assert_eq!(ask_line("Fix the login button on the settings page", "Fix the login").as_deref(), Some("You asked: Fix the login button on the settings page."));
        assert_eq!(ask_line("Checkout fails.\n\nFind out why.", "Checkout").as_deref(), Some("You asked: Checkout fails."));
        assert_eq!(ask_line("Weekly news", "Weekly news"), None);
        assert_eq!(ask_line("Weekly news", "Weekly news · Thu, Oct 29"), None);
        let repeating = "THIS IS ONE RUN OF A REPEATING TASK.\nbrief\nHER INSTRUCTION FOLLOWS.\nSend the weekly news";
        assert_eq!(ask_line(repeating, "Weekly news").as_deref(), Some("You asked: Send the weekly news."));
        assert_eq!(ask_line("", "x"), None);
        assert_eq!(working_context("Weekly news", "Weekly news", Some("Repeats daily at 9:00am")).as_deref(), Some("Repeats daily at 9:00am."));
        assert_eq!(working_context("Send the news", "Weekly", Some("Repeats Thursdays at 4:30pm")).as_deref(), Some("You asked: Send the news. Repeats Thursdays at 4:30pm."));
        assert_eq!(working_context("Weekly news", "Weekly news", None), None);
        assert_eq!(repeats_line(&json!({ "every": "day", "at": "09:00" })), "Repeats daily at 9:00am");
        assert_eq!(repeats_line(&json!({ "every": "weekday", "at": "13:05" })), "Repeats weekdays at 1:05pm");
        assert_eq!(repeats_line(&json!({ "every": "week", "on": 4, "at": "16:30" })), "Repeats Thursdays at 4:30pm");
        assert_eq!(repeats_line(&json!({ "every": "week", "on": 9, "at": "16:30" })), "Repeats Once");
        assert_eq!(repeats_line(&json!({ "every": "day", "at": "25:00" })), "Repeats Once");
        let run = item(json!({ "labels": ["repeat:r-1"], "product": "news" }));
        let rules = vec![json!({ "id": "r-1", "product": "other" }), json!({ "id": "r-1", "product": "news", "every": "day", "at": "09:00" })];
        assert_eq!(rule_for(&run, &rules).and_then(|r| r.get("at")), Some(&json!("09:00")));
        // A cause's ask is not said.
        let cause = item(json!({ "kind": "cause", "labels": ["cause"], "body": "GitHub is signed out" }));
        assert_eq!(asked_of(&cause), "");
        assert_eq!(asked_of(&item(json!({ "body": "Do it" }))), "Do it");
    }

    /// `tests/a-thread-started-from-an-ask-says-what-was-asked-not-who-asked.test.mts`.
    #[test]
    fn the_line_under_the_title_passes_over_who_asked_and_reads_a_link_as_its_words() {
        const LINK: &str = "https://app.agentbox.ac/t/priya/w-0123456789";
        assert_eq!(ask_line("Priya Shah asked:\n\nCan a declined card leave a hold?\n\nSay whether it looks good.", "Friendlier copy").as_deref(), Some("You asked: Can a declined card leave a hold?"));
        assert_eq!(ask_line(&format!("Priya Shah asked:\n\n[Friendlier payment failed copy]({LINK})\n\nPriya: is it friendly enough?"), "Friendlier payment failed copy"), None);
        assert_eq!(ask_line(&format!("Priya Shah asked:\n\n[Friendlier payment failed copy]({LINK})"), "Payment copy").as_deref(), Some("You asked: Friendlier payment failed copy."));
        assert_eq!(ask_line("Fix this: the totals are off", "Totals").as_deref(), Some("You asked: Fix this: the totals are off."));
        assert_eq!(ask_line("Three fixes:", "Fixes").as_deref(), Some("You asked: Three fixes:."));
        assert_eq!(ask_line("Fix the login bug", "Fix the login bug"), None);
    }

    fn took(at: f64, verb: &str, subject: &str, full: &str) -> Took {
        Took { at, verb: verb.into(), subject: subject.into(), full: full.into() }
    }

    fn call(label: &str, detail: &str, started: Option<f64>) -> Activity {
        Activity { id: label.into(), label: label.into(), detail: detail.into(), started_at: started }
    }

    /// `previousActivity`, with `tests/thinking-expands-to-clean-gray-commands.test.mts`'s case.
    #[test]
    fn a_call_still_running_takes_its_own_step_out_of_the_finished_ones() {
        let rows = [took(0.0, "", "npm test", ""), took(0.0, "", "git status", ""), took(0.0, "", "npm test", "")];
        assert_eq!(previous_activity(&rows, &[call("", "npm test", None)]), rows[..2].to_vec(), "the current command is not called completed, nor an earlier repeat erased");
        assert_eq!(previous_activity(&rows, &[]), rows.to_vec());
        assert_eq!(previous_activity(&rows, &[call("", "npm build", None)]), rows.to_vec());
        let history = [took(1_000.0, "edited", "src/tax.ts", ""), took(2_000.0, "ran", "npm test", "npm  test\n")];
        let left = previous_activity(&history, &[call("Running the tests", "npm test", Some(2_500.0))]);
        assert_eq!(left, vec![history[0].clone()]);
        let left = previous_activity(&history, &[call("Running the tests", "npm test", Some(9_000.0))]);
        assert_eq!(left.len(), 2);
        assert_eq!(previous_activity(&history, &[call("Running the tests", "npm test", None)]).len(), 1);
    }

    #[test]
    fn the_unfolded_line_lists_the_newest_five_finished_and_then_what_runs_now() {
        let history: Vec<Took> = (0..8).map(|k| took(f64::from(k) * 1000.0, "read", &format!("file-{k}.md"), &format!("/repo/file-{k}.md"))).collect();
        let shown = steps(&history, &[call("Running the tests", "npm test -- checkout", Some(10_000.0))]);
        assert_eq!(shown.earlier, 3);
        assert_eq!(shown.rows.len(), 6);
        assert_eq!(shown.rows[0], StepRow { label: "Read file-3.md".into(), detail: "/repo/file-3.md".into() });
        assert_eq!(shown.rows[5], StepRow { label: "Running the tests".into(), detail: "npm test -- checkout".into() });
        assert_eq!(earlier_steps(3), "3 earlier steps since it last said something");
        assert_eq!(earlier_steps(1), "1 earlier step since it last said something");
        assert_eq!(steps(&[took(0.0, "ran", "git status", "")], &[]).rows[0].detail, "git status");
        assert_eq!(steps(&[], &[]), Steps { earlier: 0, rows: vec![] });
    }
}
