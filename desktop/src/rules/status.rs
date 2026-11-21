//! What one thread is doing, in one word and one mark: the Inbox table's Status cell. A port
//! of `statusCell` in `renderer/src/status-word.ts`, of `liveLine` in `renderer/src/live-line.ts`
//! (which of its states holds; its sentence is only read aloud), and of `threadState` in
//! `shared/thread-cards.mts`.

use serde_json::{Map, Value};

use super::kinds::{asks_who, catch_up_due, is_auto};
use super::nowhere::says_a_run_got_nowhere;
use super::shipping::{closed_unshipped, not_shipped_line, shipped_by_the_app};
use crate::data::{Item, Mark, Run, Supervisor, truthy};

/// One of four states and only four.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Waiting,
    Running,
    Scheduled,
    Done,
}

/// Which of the four states a thread is in (`threadState`).
pub fn thread_state(i: &Item, now: f64) -> State {
    if i.status == "done" {
        return State::Done;
    }
    // A thread in Later is not running and nobody has to act on it: the scheduled family.
    if i.string("start") == Some("later") || i.run_at.is_some_and(|at| at.is_finite() && at > now) {
        return State::Scheduled;
    }
    if i.status == "claimed" && i.truthy("claim") && !i.truthy("claimExpired") {
        return State::Running;
    }
    // Sent to an agent and not yet picked up: it is about to run, so it waits on nobody. A
    // blocked one waits on the person.
    let person_holds = i.string("assignee").is_some_and(|a| a != "agent");
    let sent_to_an_agent = i.status != "blocked" && i.has_label("founder") && i.result_text().is_none() && i.text("note").is_none() && !matches!(i.kind.as_deref(), Some("question" | "review")) && !person_holds;
    if sent_to_an_agent { State::Running } else { State::Waiting }
}

/// The word a thread shows: the state's word, unless the thread has no moment at all, when it
/// reads "Not started" (`stateWordOf` in `renderer/src/threads/summary-rules.ts`).
fn state_word_of(i: &Item, state: State) -> &'static str {
    if i.string("start") == Some("later") {
        return "Not started";
    }
    match state {
        State::Waiting => "Waiting",
        State::Running => "In progress",
        State::Scheduled => "Scheduled",
        State::Done => "Done",
    }
}

/// What the supervisor says about one thread (`factsFor` in `renderer/src/fleet.ts`, with what
/// `ThreadCells` adds).
pub struct Facts<'a> {
    pub supervisor: &'a Supervisor,
    /// The run on this thread, when an agent is on it now.
    pub session: Option<&'a Run>,
    /// An approval names this thread, or it waits on a sign-in.
    pub asking: bool,
    /// The row is drawn under the In progress tab: the words about agents waiting their turn
    /// are only ever true there.
    pub in_progress: bool,
    /// The row is in the tutorial's practice project, where the supervisor never starts anything.
    pub practice: bool,
    /// The row is drawn under the Later tab, where the page already knows its state: one word
    /// for everything that comes back at the time beside it (`laterCell`).
    pub later: bool,
    pub now: f64,
}

impl Facts<'_> {
    fn listed(&self, list: &str, id: &str) -> bool {
        self.supervisor.rest.get(list).and_then(Value::as_array).is_some_and(|ids| ids.iter().any(|v| v.as_str() == Some(id)))
    }
    fn keyed(&self, map: &str, id: &str) -> Option<&Value> {
        self.supervisor.rest.get(map).and_then(Value::as_object).and_then(|m| m.get(id)).filter(|v| truthy(Some(v)))
    }
}

/// Which of the things that can be true of a thread's agent is true (`LiveState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Live {
    Asking,
    Working,
    Elsewhere,
    Shipping,
    Next,
    Queued,
    Cannot,
    SignIn,
    Limit,
    Paused,
    Silent,
    Idle,
}

/// The one word each state gets (`shortWord`). A run's subagents are counted in the open
/// thread's own line, not in the table.
fn short_word(state: Live) -> &'static str {
    match state {
        Live::Asking => "Asking",
        // The same word for a run of ours and one outside this window; the mark says where.
        Live::Working | Live::Elsewhere => "Working",
        Live::Shipping => "Shipping",
        Live::Next => "Up next",
        Live::Queued => "Queued",
        Live::Cannot => "Not started",
        Live::SignIn => "Signed out",
        Live::Limit => "At its limit",
        Live::Paused => "Paused",
        Live::Silent => "Nothing came back",
        Live::Idle => "Nothing running",
    }
}

/// Has an agent actually written something here. `(withdrawn)` is the app's own word for a
/// field that has been taken back.
fn said(text: Option<&str>) -> bool {
    text.map(crate::rules::js::trim).is_some_and(|t| !t.is_empty() && t != "(withdrawn)")
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
    let wrote: Option<&Map<String, Value>> = i.object("wrote");
    !wrote.is_some_and(|w| w.values().any(|w| w.get("source").and_then(Value::as_str) == Some("agent") && w.get("ts").and_then(Value::as_f64).is_some_and(|ts| ts >= sent_at)))
}

/// The reasons a queued row cannot start that the app has a sentence for (`cannotStartLine`).
const CANNOT_START: [&str; 9] = ["setup", "asleep", "cloud-carried", "engine", "login", "memory", "slow-folder", "error", "brake"];

/// Whether anything is actually happening on this thread (`liveLine`). `stalled` is always
/// handed over as false by the Status cell, which says Stopped itself.
fn live_line(i: &Item, f: &Facts) -> Option<Live> {
    // An imported session is not one of ours, and the walkthrough's practice rows are
    // answered by the app and never started.
    if i.truthy("agent") || i.has_label("first-run") {
        return None;
    }
    // Finished is finished.
    if i.status == "done" {
        return None;
    }
    // Above the session: a worker waiting on an approval is a live session in every list.
    if f.asking {
        return Some(Live::Asking);
    }
    if f.session.is_some() {
        return Some(Live::Working);
    }
    let shipping = f.listed("shipping", &i.id);
    // Signed out is not queued, and it is said in or out of In progress.
    if f.keyed("signInNeeded", &i.id).is_some() && !shipping {
        return Some(Live::SignIn);
    }
    // A run ended here and wrote nothing down. Above the In progress gate on purpose: such a
    // row sits in the inbox looking untouched. Never under something there is to read.
    if f.keyed("silent", &i.id).is_some() && !said(i.result.as_deref()) && !said(i.string("note")) {
        return Some(Live::Silent);
    }
    if !f.in_progress {
        return None;
    }
    // The app is shipping it: no agent is on it and none is coming.
    if shipping {
        return Some(Live::Shipping);
    }
    let remote_held = f.listed("remoteHeld", &i.id);
    // A live claim that no session of ours holds.
    if i.status == "claimed" && i.truthy("claim") && !i.truthy("claimExpired") && !remote_held {
        return Some(Live::Elsewhere);
    }
    if f.keyed("atLimit", &i.id).is_some() {
        return Some(Live::Limit);
    }
    // Queued is a promise that an agent is coming, and for these it is not.
    let queued = f.listed("queued", &i.id);
    let why = f.keyed("cannotStart", &i.id).and_then(|c| c.get("why")).and_then(Value::as_str);
    if queued && why.is_some_and(|why| CANNOT_START.contains(&why)) {
        return Some(Live::Cannot);
    }
    if queued && f.listed("runNow", &i.id) {
        return Some(Live::Next);
    }
    if queued {
        return Some(Live::Queued);
    }
    if truthy(f.supervisor.rest.get("paused")) {
        return Some(Live::Paused);
    }
    // The window draws a thread just sent before the fleet's next snapshot lists it.
    if just_sent(i, f.now) {
        return Some(Live::Queued);
    }
    Some(Live::Idle)
}

/// The Status cell's word and the mark before it. `mark` is drawn only for a word about an
/// agent, now: working, waiting its turn, or held (`StatusCell`, with `agent` folded into it).
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub word: String,
    pub mark: Mark,
    /// An agent is on it this second.
    pub live: bool,
}

fn says(word: &str, mark: Mark) -> Cell {
    Cell { word: word.to_string(), mark, live: false }
}

/// What one thread is doing (`statusCell`): live-line first, falling back to the thread's own
/// state word. Under Later the state is the tab's, whatever the thread itself would say
/// (`laterCell`): a thread finished while it was still written down there reads as the rest do.
pub fn status_cell(i: &Item, f: &Facts) -> Cell {
    let state = if f.later { State::Scheduled } else { thread_state(i, f.now) };
    // Said over "Waiting" and "Done", which do not tell a shipped thread from one that never
    // shipped. A person's word that it shipped counts, and their zero takes the app's back.
    let shipped_at = i.number("shippedAt");
    let shipped = shipped_at.is_some_and(|at| at > 0.0) || (shipped_by_the_app(i) && shipped_at != Some(0.0));

    if state == State::Done {
        if shipped {
            return says("Shipped", Mark::None);
        }
        if closed_unshipped(i) {
            return says("Not shipped", Mark::None);
        }
        return says(state_word_of(i, state), Mark::None);
    }
    if catch_up_due(i) {
        return says("Getting ready", Mark::None);
    }
    // Sent to Auto and not given to anyone yet. Ahead of Stopped: a thread that asks the
    // person is on the supervisor's stalled list, and no agent died on it.
    if is_auto(i) {
        return if asks_who(i) { says("Asks you who", Mark::None) } else { says("Assigning", Mark::Queued) };
    }
    // Signed out is the reason a stopped thread is stopped, so it is the word.
    if f.keyed("signInNeeded", &i.id).is_some() && live_line(i, f) == Some(Live::SignIn) {
        return says(short_word(Live::SignIn), Mark::Held);
    }
    if f.listed("stalled", &i.id) {
        return says("Stopped", Mark::Held);
    }
    if state == State::Scheduled {
        return says(state_word_of(i, state), Mark::None);
    }

    let live = live_line(i, f);
    let away = f.keyed("whyNotCloud", &i.id);
    if live == Some(Live::Working) {
        let cloud = f.session.and_then(|s| s.rest.get("where")).and_then(Value::as_str) == Some("cloud");
        // With the cloud taking everything, a run on this Mac says why it is here.
        let local = !cloud && away.and_then(|a| a.get("at")).and_then(Value::as_str) == Some("mac");
        let moving = away.and_then(|a| a.get("why")).and_then(Value::as_str) == Some("moving");
        let word = if !local { short_word(Live::Working) } else if moving { "Moving to cloud" } else { "Working locally" };
        return Cell { word: word.to_string(), mark: if cloud { Mark::Cloud } else { Mark::Running }, live: true };
    }
    // Over "Waiting": the agent's last word, written before, says it ships.
    if not_shipped_line(i).is_some() {
        return says("Not shipped", Mark::None);
    }
    // "Nothing running" is a claim about the whole fleet, and so is an agent being on it
    // outside this window. Both are only said where the fleet has really been read.
    let fleet_unread = !f.supervisor.rest.get("queued").is_some_and(Value::is_array) || f.practice;
    if (live == Some(Live::Idle) && (fleet_unread || f.listed("remoteHeld", &i.id))) || (live == Some(Live::Elsewhere) && fleet_unread) {
        return says(state_word_of(i, state), Mark::None);
    }
    // Drawn as one of ours is: the cloud when a cloud run holds it, the half-dot otherwise.
    if live == Some(Live::Elsewhere) {
        let cloud = i.object("claim").and_then(|c| c.get("holder")).and_then(Value::as_str).is_some_and(|h| h.starts_with("mcp-cloud-"));
        return Cell { word: short_word(Live::Elsewhere).to_string(), mark: if cloud { Mark::Cloud } else { Mark::Running }, live: true };
    }
    if let Some(live) = live {
        // Waiting its turn resolves itself; held does not.
        let waits_its_turn = matches!(live, Live::Queued | Live::Next | Live::Shipping);
        return says(short_word(live), if waits_its_turn { Mark::Queued } else { Mark::Held });
    }
    if state == State::Waiting && shipped {
        return says("Shipped", Mark::None);
    }
    // The app's own note that a run got nowhere is not an answer.
    if state == State::Waiting && i.wrote("result").is_some_and(|w| w.source == "agent") && says_a_run_got_nowhere(i.result.as_deref()) {
        return says("Stopped", Mark::Held);
    }
    says(state_word_of(i, state), Mark::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_000_000_000.0;

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    fn word(i: Value, supervisor: Value, tab: &str) -> (String, Mark) {
        let i = item(i);
        let supervisor: Supervisor = serde_json::from_value(supervisor).unwrap();
        let session = supervisor.running.iter().find(|r| r.item_id == i.id);
        let asking = false;
        let cell = status_cell(&i, &Facts { supervisor: &supervisor, session, asking, in_progress: tab == "progress", practice: false, later: tab == "snoozed", now: NOW });
        (cell.word, cell.mark)
    }

    fn open() -> Value {
        json!({ "id": "w-1", "product": "shop", "status": "open", "kind": "directive", "labels": ["founder"], "title": "Fix the export", "createdAt": 1000, "updatedAt": 1000 })
    }

    #[test]
    fn the_four_states() {
        assert_eq!(thread_state(&item(json!({ "status": "done" })), NOW), State::Done);
        assert_eq!(thread_state(&item(json!({ "status": "open", "start": "later" })), NOW), State::Scheduled);
        assert_eq!(thread_state(&item(json!({ "status": "open", "runAt": NOW + 1.0 })), NOW), State::Scheduled);
        assert_eq!(thread_state(&item(json!({ "status": "claimed", "claim": { "holder": "x" } })), NOW), State::Running);
        assert_eq!(thread_state(&item(json!({ "status": "claimed", "claim": { "holder": "x" }, "claimExpired": true })), NOW), State::Waiting);
        // Sent to an agent and not yet picked up: about to run.
        assert_eq!(thread_state(&item(open()), NOW), State::Running);
        let mut answered = open();
        answered["result"] = json!("Fixed.");
        assert_eq!(thread_state(&item(answered), NOW), State::Waiting);
        let mut question = open();
        question["kind"] = json!("question");
        assert_eq!(thread_state(&item(question), NOW), State::Waiting);
    }

    #[test]
    fn a_running_thread_says_working_and_where() {
        let here = json!({ "running": [{ "itemId": "w-1", "product": "shop", "startedAt": 5 }], "queued": [] });
        assert_eq!(word(open(), here, "progress"), ("Working".into(), Mark::Running));
        let cloud = json!({ "running": [{ "itemId": "w-1", "product": "shop", "startedAt": 5, "where": "cloud" }], "queued": [] });
        assert_eq!(word(open(), cloud, "progress"), ("Working".into(), Mark::Cloud));
        let kept = json!({ "running": [{ "itemId": "w-1", "product": "shop", "startedAt": 5 }], "queued": [], "whyNotCloud": { "w-1": { "at": "mac", "why": "needs-mac" } } });
        assert_eq!(word(open(), kept, "progress"), ("Working locally".into(), Mark::Running));
    }

    #[test]
    fn a_waiting_thread_under_in_progress_says_what_it_waits_for() {
        assert_eq!(word(open(), json!({ "queued": ["w-1"] }), "progress"), ("Queued".into(), Mark::Queued));
        assert_eq!(word(open(), json!({ "queued": ["w-1"], "runNow": ["w-1"] }), "progress"), ("Up next".into(), Mark::Queued));
        assert_eq!(word(open(), json!({ "queued": ["w-1"], "cannotStart": { "w-1": { "why": "asleep", "tool": "Claude Code" } } }), "progress"), ("Not started".into(), Mark::Held));
        assert_eq!(word(open(), json!({ "queued": [], "shipping": ["w-1"] }), "progress"), ("Shipping".into(), Mark::Queued));
        assert_eq!(word(open(), json!({ "queued": [], "paused": true }), "progress"), ("Paused".into(), Mark::Held));
        assert_eq!(word(open(), json!({ "queued": [], "atLimit": { "w-1": { "tool": "Claude Code", "until": 0 } } }), "progress"), ("At its limit".into(), Mark::Held));
        assert_eq!(word(open(), json!({ "queued": [] }), "progress"), ("Nothing running".into(), Mark::Held));
        // A surface drawn without the fleet knows nothing, and says only the state.
        assert_eq!(word(open(), json!({}), "progress"), ("In progress".into(), Mark::None));
        // Off In progress the words about waiting a turn are not said.
        assert_eq!(word(open(), json!({ "queued": ["w-1"] }), "inbox"), ("In progress".into(), Mark::None));
        let mut claimed = open();
        claimed["status"] = json!("claimed");
        claimed["claim"] = json!({ "holder": "mcp-cloud-7" });
        assert_eq!(word(claimed, json!({ "queued": [] }), "progress"), ("Working".into(), Mark::Cloud));
    }

    #[test]
    fn a_thread_just_sent_reads_queued_before_the_fleet_lists_it() {
        let mut fresh = open();
        fresh["createdAt"] = json!(NOW - 5_000.0);
        assert_eq!(word(fresh.clone(), json!({ "queued": [] }), "progress"), ("Queued".into(), Mark::Queued));
        fresh["epoch"] = json!(1);
        assert_eq!(word(fresh, json!({ "queued": [] }), "progress"), ("Nothing running".into(), Mark::Held));
    }

    #[test]
    fn a_stopped_signed_out_or_silent_thread_says_so_in_any_tab() {
        assert_eq!(word(open(), json!({ "queued": [], "stalled": ["w-1"] }), "inbox"), ("Stopped".into(), Mark::Held));
        assert_eq!(word(open(), json!({ "queued": [], "stalled": ["w-1"], "signInNeeded": { "w-1": "Claude Code" } }), "inbox"), ("Signed out".into(), Mark::Held));
        assert_eq!(word(open(), json!({ "queued": [], "silent": { "w-1": { "runs": 1, "endedAt": 5 } } }), "inbox"), ("Nothing came back".into(), Mark::Held));
        let mut answered = open();
        answered["result"] = json!("Fixed.");
        assert_eq!(word(answered, json!({ "queued": [], "silent": { "w-1": { "runs": 1, "endedAt": 5 } } }), "inbox"), ("Waiting".into(), Mark::None));
        let mut dead = open();
        dead["result"] = json!("Claude Code could not sign in, so nothing has been done on this task.");
        dead["wrote"] = json!({ "result": { "ts": 5, "source": "agent" } });
        assert_eq!(word(dead, json!({ "queued": [] }), "inbox"), ("Stopped".into(), Mark::Held));
    }

    #[test]
    fn a_finished_thread_says_whether_it_shipped() {
        let mut done = open();
        done["status"] = json!("done");
        assert_eq!(word(done.clone(), json!({}), "done"), ("Done".into(), Mark::None));
        let mut shipped = done.clone();
        shipped["note"] = json!("Shipped to main as 0a1b2c3.");
        shipped["wrote"] = json!({ "note": { "ts": 9, "source": "system" } });
        assert_eq!(word(shipped.clone(), json!({}), "done"), ("Shipped".into(), Mark::None));
        // The person's own zero takes the app's word back.
        shipped["shippedAt"] = json!(0);
        assert_eq!(word(shipped, json!({}), "done"), ("Done".into(), Mark::None));
        let mut marked = done.clone();
        marked["shippedAt"] = json!(77);
        assert_eq!(word(marked, json!({}), "done"), ("Shipped".into(), Mark::None));
        let mut unshipped = done;
        unshipped["note"] = json!("Closed without its work reaching main.");
        assert_eq!(word(unshipped, json!({}), "done"), ("Not shipped".into(), Mark::None));
    }

    #[test]
    fn later_and_auto_have_their_own_words() {
        let mut later = open();
        later["start"] = json!("later");
        assert_eq!(word(later, json!({ "queued": [] }), "inbox"), ("Not started".into(), Mark::None));
        let mut scheduled = open();
        scheduled["runAt"] = json!(NOW + 60_000.0);
        assert_eq!(word(scheduled, json!({ "queued": [] }), "inbox"), ("Scheduled".into(), Mark::None));
        let mut auto = open();
        auto["labels"] = json!(["founder", "auto"]);
        assert_eq!(word(auto.clone(), json!({ "queued": [] }), "progress"), ("Assigning".into(), Mark::Queued));
        auto["routed"] = json!({ "ask": ["t-1", "t-2"] });
        assert_eq!(word(auto, json!({ "queued": [] }), "inbox"), ("Asks you who".into(), Mark::None));
    }

    #[test]
    fn under_later_every_thread_that_comes_back_says_scheduled() {
        // A proposal would say Waiting anywhere else, and a thread waiting its turn In progress.
        let mut proposal = open();
        proposal["labels"] = json!([]);
        proposal["parent"] = json!("w-0");
        assert_eq!(word(proposal.clone(), json!({ "queued": [] }), "inbox"), ("Waiting".into(), Mark::None));
        assert_eq!(word(proposal, json!({ "queued": [] }), "snoozed"), ("Scheduled".into(), Mark::None));
        assert_eq!(word(open(), json!({ "queued": ["w-1"] }), "snoozed"), ("Scheduled".into(), Mark::None));
        // One with no moment says so, and goes on saying so once its agent has written Done on
        // it, under Later and under Done alike: the word follows `start`, not the state.
        let mut written_down = open();
        written_down["start"] = json!("later");
        assert_eq!(word(written_down.clone(), json!({ "queued": [] }), "snoozed"), ("Not started".into(), Mark::None));
        written_down["status"] = json!("done");
        assert_eq!(word(written_down.clone(), json!({ "queued": [] }), "snoozed"), ("Not started".into(), Mark::None));
        assert_eq!(word(written_down.clone(), json!({ "queued": [] }), "done"), ("Not started".into(), Mark::None));
        // A finished thread put off by its agent is Scheduled here and Done there.
        let mut finished = open();
        finished["status"] = json!("done");
        assert_eq!(word(finished.clone(), json!({ "queued": [] }), "snoozed"), ("Scheduled".into(), Mark::None));
        assert_eq!(word(finished, json!({ "queued": [] }), "done"), ("Done".into(), Mark::None));
        // What stops a thread for good is still said over the tab's word.
        assert_eq!(word(open(), json!({ "queued": [], "stalled": ["w-1"] }), "snoozed"), ("Stopped".into(), Mark::Held));
        assert_eq!(word(open(), json!({ "queued": [], "stalled": ["w-1"], "signInNeeded": { "w-1": "Claude Code" } }), "snoozed"), ("Signed out".into(), Mark::Held));
    }
}
