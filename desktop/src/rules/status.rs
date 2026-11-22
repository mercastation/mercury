//! What one thread is doing, in one word and one mark: the Inbox table's Status cell. A port
//! of `statusCell` in `renderer/src/status-word.ts`, which reads `liveLine` (`rules::live`, the
//! same line a conversation's foot draws), and of `threadState` in `shared/thread-cards.mts`.
//! The cell also carries what it says under the pointer (its sentence, or the card saying why
//! the cloud does not have it) and whether Try again follows its word (`offersTryAgain`).

use serde_json::Value;

use super::js::re;
use super::kinds::{asks_who, catch_up_due, is_auto};
use super::live::{self, State as Live};
use super::nowhere::says_a_run_got_nowhere;
use super::shipping::{closed_unshipped, not_shipped_line, shipped_by_the_app};
use crate::data::{Item, Mark, Run, Supervisor};

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
    /// The same facts as the line at a conversation's foot reads, for this row (`factsFor`, with
    /// `inProgress` the row's tab, the practice project, and no moment it is put off to, which
    /// the row's facts do not carry). The run is the one the caller found for the row.
    pub fn live(&self, i: &Item) -> live::Facts {
        let mut f = live::Facts::of(self.supervisor, &i.id, self.now);
        if let Some(run) = self.session {
            let mut facts = run.rest.clone();
            facts.insert("itemId".to_string(), Value::String(run.item_id.clone()));
            facts.insert("product".to_string(), Value::String(run.product.clone()));
            f.session = serde_json::from_value(Value::Object(facts)).ok();
        }
        f.asking = self.asking;
        f.in_progress = self.in_progress;
        f.practice = self.practice;
        f
    }
}

/// Whether anything is actually happening on this thread (`liveLine`), with `stalled` handed
/// over as false: the Status cell says Stopped itself.
fn live_line(i: &Item, f: &live::Facts) -> Option<live::Line> {
    live::line(i, &live::Facts { stalled: false, ..f.clone() })
}

re!(SHIPPED_AS, r" as ([0-9a-f]{7,40})\.", "");

/// The Status cell's word and the mark before it. `mark` is drawn only for a word about an
/// agent, now: working, waiting its turn, or held (`StatusCell`, with `agent` folded into it).
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub word: String,
    pub mark: Mark,
    /// An agent is on it this second.
    pub live: bool,
    /// The whole sentence, for the pointer resting on the word, where there is more to say.
    pub title: Option<String>,
    /// Why it is not in the cloud, as a card under the word, which the sentence then gives way to:
    /// its head and its body (`whyWords`).
    pub why: Option<(String, String)>,
    /// Try again follows the word: nothing will start the thread without a person
    /// (`offersTryAgain`, on the row's own facts).
    pub retry: bool,
}

fn says(word: &str, mark: Mark, title: Option<&str>) -> Cell {
    Cell { word: word.to_string(), mark, live: false, title: title.map(str::to_string), why: None, retry: false }
}

/// What one thread is doing (`statusCell`): live-line first, falling back to the thread's own
/// state word. Under Later the state is the tab's, whatever the thread itself would say
/// (`laterCell`): a thread finished while it was still written down there reads as the rest do.
pub fn status_cell(i: &Item, f: &Facts) -> Cell {
    let facts = f.live(i);
    Cell { retry: live::offers_try_again(i, &facts), ..cell(i, f, &facts) }
}

fn cell(i: &Item, f: &Facts, facts: &live::Facts) -> Cell {
    let state = if f.later { State::Scheduled } else { thread_state(i, f.now) };
    // Said over "Waiting" and "Done", which do not tell a shipped thread from one that never
    // shipped. A person's word that it shipped counts, and their zero takes the app's back.
    let shipped_at = i.number("shippedAt");
    let shipped = if shipped_at.is_some_and(|at| at > 0.0) {
        Some("Marked as shipped".to_string())
    } else if shipped_by_the_app(i) && shipped_at != Some(0.0) {
        Some(match i.string("note").and_then(|note| SHIPPED_AS.exec(note)).map(|caps| super::js::group(&caps, 1).to_string()) {
            Some(sha) if !sha.is_empty() => format!("Shipped to main as {sha}"),
            _ => "Shipped to main".to_string(),
        })
    } else {
        None
    };

    if state == State::Done {
        if let Some(title) = &shipped {
            return says("Shipped", Mark::None, Some(title));
        }
        if closed_unshipped(i) {
            return says("Not shipped", Mark::None, Some("Closed without its work reaching main"));
        }
        return says(state_word_of(i, state), Mark::None, None);
    }
    if catch_up_due(i) {
        return says("Getting ready", Mark::None, Some("This chat is being summarized so you can answer it."));
    }
    // Sent to Auto and not given to anyone yet. Ahead of Stopped: a thread that asks the
    // person is on the supervisor's stalled list, and no agent died on it.
    if is_auto(i) {
        return if asks_who(i) {
            says("Asks you who", Mark::None, Some("Auto could not tell which teammate should take this. Open it to pick."))
        } else {
            says("Assigning", Mark::Queued, Some("Auto is picking who takes this."))
        };
    }
    // Signed out is the reason a stopped thread is stopped, so it is the word.
    if facts.sign_in_needed.is_some()
        && let Some(signed_out) = live_line(i, facts).filter(|l| l.state == Live::SignIn)
    {
        return says(&live::short_word(Live::SignIn, 0), Mark::Held, Some(&signed_out.line));
    }
    if facts.stalled {
        return says("Stopped", Mark::Held, Some("The agent that was on this died without finishing. Send it a message to start it again."));
    }
    if state == State::Scheduled {
        return says(state_word_of(i, state), Mark::None, None);
    }

    let live = live_line(i, facts);
    let away = facts.why_not_cloud.as_ref();
    let why = away.map(|w| (live::why_head(w).to_string(), super::place::why_words(w)));
    // Where the cloud says why, the card says it, and the sentence leads with it.
    let titled = |line: &str| why.as_ref().map_or_else(|| line.to_string(), |(head, body)| format!("{head}. {body}"));
    if let Some(working) = live.as_ref().filter(|l| l.state == Live::Working) {
        let cloud = facts.session.as_ref().and_then(|s| s.place.as_deref()) == Some("cloud");
        // With the cloud taking everything, a run on this Mac says why it is here.
        let local = !cloud && away.is_some_and(|a| a.at == "mac");
        let moving = away.is_some_and(|a| a.why == "moving");
        let word = if !local { live::short_word(Live::Working, 0) } else if moving { "Moving to cloud".to_string() } else { "Working locally".to_string() };
        return Cell {
            word,
            mark: if cloud { Mark::Cloud } else { Mark::Running },
            live: true,
            title: Some(if local { titled(&working.line) } else { working.line.clone() }),
            why: if local { why } else { None },
            retry: false,
        };
    }
    // Over "Waiting": the agent's last word, written before, says it ships.
    if let Some(unshipped) = not_shipped_line(i) {
        return says("Not shipped", Mark::None, Some(&unshipped));
    }
    // "Nothing running" is a claim about the whole fleet, and so is an agent being on it
    // outside this window. Both are only said where the fleet has really been read.
    let fleet_unread = facts.queued.is_none() || f.practice;
    let state_now = live.as_ref().map(|l| l.state);
    if (state_now == Some(Live::Idle) && (fleet_unread || facts.remote_held)) || (state_now == Some(Live::Elsewhere) && fleet_unread) {
        return says(state_word_of(i, state), Mark::None, None);
    }
    // Drawn as one of ours is: the cloud when a cloud run holds it, the half-dot otherwise.
    if let Some(elsewhere) = live.as_ref().filter(|l| l.state == Live::Elsewhere) {
        let cloud = i.object("claim").and_then(|c| c.get("holder")).and_then(Value::as_str).is_some_and(|h| h.starts_with("mcp-cloud-"));
        return Cell { word: live::short_word(Live::Elsewhere, 0), mark: if cloud { Mark::Cloud } else { Mark::Running }, live: true, title: Some(elsewhere.line.clone()), why: None, retry: false };
    }
    if let Some(live) = live {
        // Waiting its turn resolves itself; held does not.
        let waits_its_turn = matches!(live.state, Live::Queued | Live::Next | Live::Shipping);
        // Waiting its turn for the cloud or for this Mac, the card says why.
        let waiting = waits_its_turn && live.state != Live::Shipping && away.is_some_and(|a| a.at == "wait-cloud" || a.at == "wait-mac");
        let cell = says(&live::short_word(live.state, 0), if waits_its_turn { Mark::Queued } else { Mark::Held }, Some(&if waiting { titled(&live.line) } else { live.line.clone() }));
        return if waiting { Cell { why, ..cell } } else { cell };
    }
    if state == State::Waiting
        && let Some(title) = &shipped
    {
        return says("Shipped", Mark::None, Some(title));
    }
    // The app's own note that a run got nowhere is not an answer.
    if state == State::Waiting && i.wrote("result").is_some_and(|w| w.source == "agent") && says_a_run_got_nowhere(i.result.as_deref()) {
        return says("Stopped", Mark::Held, i.result.as_deref().map(|r| r.split('\n').next().unwrap_or("")));
    }
    says(state_word_of(i, state), Mark::None, None)
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
