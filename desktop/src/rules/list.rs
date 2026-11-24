//! What each list holds. A port of `renderer/src/list-rules.ts`, function for function: the
//! only copy of the inbox membership rule, the In progress rule, and the line a row prints
//! under its title.

use std::collections::{HashMap, HashSet};

use super::answers::{answer_settled, filed_by_the_person, wakes_itself_at, word_waits};
use super::format::{first_real_line, offer_is_live, preview_text};
use super::js::{self, re};
use super::kinds::{asks_the_person, asks_to_close, instruction_of, is_cause, is_clean_run, is_task_row, not_started};
use super::nowhere::says_a_run_got_nowhere;
use super::shipping::not_shipped_line;
use super::waits::{blocked_by, set_by_person};
use crate::data::Item;

/// A row that is still asking the person something: a question or a review, unanswered and
/// not finished. A session on the row, or a ship in flight, may not take the question it is
/// asking away from the person it asked.
pub fn still_asks_you(i: &Item) -> bool {
    asks_the_person(i) && i.status != "done" && i.live_answer().is_none()
}

/// When the person last said something on the thread: its title, body or answer, in their own
/// hand. Taking a word back is not saying one.
fn her_last_word(i: &Item) -> f64 {
    let mut last: f64 = 0.0;
    for field in ["title", "body", "answer"] {
        if field == "answer" && i.string("answer") == Some("(withdrawn)") {
            continue;
        }
        if let Some(w) = i.wrote(field).filter(|w| w.source == "founder") {
            last = last.max(w.at());
        }
    }
    last
}

/// The person's own open row, carrying a result a worker wrote after their last word, with no
/// reply from them yet. It goes to the inbox and leaves In progress, but stays open.
pub fn answered_her_ask(i: &Item) -> bool {
    if i.status != "open" || !filed_by_the_person(i) || i.live_answer().is_some() || i.result_text().is_none() {
        return false;
    }
    i.wrote("result").is_some_and(|r| r.source == "agent" && r.at() > her_last_word(i))
}

/// The app has stopped carrying this row's reply: its own word that a run got nowhere, written
/// after a reply, is written only once it gives up.
pub fn gave_up_on_the_answer(i: &Item) -> bool {
    if i.status != "open" || i.live_answer().is_none() || answer_settled(i) {
        return false;
    }
    let Some(r) = i.wrote("result") else { return false };
    if r.source != "agent" || r.at() <= i.wrote_at("answer") {
        return false;
    }
    says_a_run_got_nowhere(i.result.as_deref())
}

/// The person's last words asked for the close and the agent's done came after them: that done
/// is theirs, as if they had pressed E.
fn closed_as_she_asked(i: &Item) -> bool {
    asks_to_close(i.live_answer()) && i.wrote_at("status") >= i.wrote("answer").and_then(|w| w.ts).unwrap_or(f64::INFINITY)
}

/// An agent parking a row is a reason to show it, because what happens next is the person's
/// call. A future moment an agent wrote stops workers respawning but does not hide the row.
pub fn parked_by_agent(i: &Item, now: f64) -> bool {
    i.status != "done" && i.run_at.unwrap_or(0.0) > now && i.wrote("runAt").is_none_or(|w| w.source != "founder")
}

/// Except a thread its agent will run again itself, asking nothing meanwhile: it waits in
/// Later until the moment and is In progress after it. Returns the moment, or 0.
pub fn runs_again_on_its_own(i: &Item) -> f64 {
    if !filed_by_the_person(i) || js::trim(i.result.as_deref().unwrap_or("")).is_empty() || offer_is_live(i) {
        return 0.0;
    }
    wakes_itself_at(i)
}

/// The moment the person put this row away until. `legacy_snooze` is the old localStorage
/// map, still honoured on read and never written.
fn put_away_by_them(i: &Item, legacy_snooze: f64) -> f64 {
    // Something said after scheduling it runs now, and its answer is theirs to read now.
    let spoke_since = i.wrote_at("answer") > i.wrote_at("runAt");
    let mine = if i.wrote("runAt").is_some_and(|w| w.source == "founder") && !spoke_since { i.run_at.unwrap_or(0.0) } else { 0.0 };
    // A pick lifts Later with no write of its own, so an answer after it lifts it here.
    let answered = i.live_answer().is_some() && i.wrote_at("answer") > i.wrote_at("snoozedUntil");
    let reminder = if i.wrote("snoozedUntil").is_some_and(|w| w.source == "founder") && !answered { i.number("snoozedUntil").unwrap_or(0.0) } else { 0.0 };
    mine.max(reminder).max(legacy_snooze)
}

/// The moment a row is out of the inbox until. The inbox and Later both read this one
/// function, so a row cannot show in both tabs.
pub fn hidden_until(i: &Item, legacy_snooze: f64) -> f64 {
    put_away_by_them(i, legacy_snooze).max(runs_again_on_its_own(i))
}

/// What a person said this thread waits on has ended, and they have not acted on it since.
pub fn wait_came_back(i: &Item) -> bool {
    let Some(ended) = i.object("waitEnded") else { return false };
    i.status != "done" && ended.get("at").and_then(|v| v.as_f64()).is_some_and(|at| at > her_last_word(i))
}

/// A teammate's comment waits on the person. On a thread they closed it is news to read once.
fn comments_wait_on_you(i: &Item, seen: f64) -> bool {
    let said = i.list("waitingComments");
    !said.is_empty() && (i.status != "done" || said.iter().any(|c| c.get("ts").and_then(|v| v.as_f64()).is_some_and(|ts| ts > seen)))
}

/// A teammate's words it has not dealt with, or comments on its links: either holds it in
/// Needs you.
pub fn words_wait(i: &Item, seen: f64) -> bool {
    comments_wait_on_you(i, seen) || !i.list("linkComments").is_empty()
}

/// What the lists know about one row besides the row itself.
#[derive(Clone, Copy, Debug, Default)]
pub struct Facts<'a> {
    /// Answers finished before this moment stay in Done.
    pub delivered_through: f64,
    /// The person's own deferral only (`hidden_until`).
    pub hidden_until: f64,
    /// When the row's work is due: its moment, unless a reply after the moment runs now.
    pub deferred_until: f64,
    pub now: f64,
    /// A session is on this row now, or one is being started.
    pub live: bool,
    /// The app's ship queue still owes this row a ship.
    pub shipping: bool,
    /// Its agent is stopped on a permission request, or waits on a sign-in.
    pub asking: bool,
    /// The threads this one waits on that can still finish.
    pub waiting_on: &'a [String],
    /// The supervisor will not start it again on its own.
    pub stopped: bool,
    /// When the person last opened the row.
    pub seen: f64,
}

/// The only copy of the inbox membership rule.
pub fn belongs_in_inbox(i: &Item, f: &Facts) -> bool {
    // A teammate's words wait for you whatever the thread is doing.
    if words_wait(i, f.seen) {
        return true;
    }
    // In Later and not begun: it waits for a person, not a clock.
    if not_started(i) {
        return false;
    }
    // Its agent is stopped on a permission request, so it answers before the status, the
    // schedule and the waits do.
    if f.asking {
        return true;
    }
    // A session is still on it, so it is In progress until it exits, unless it is still asking.
    if f.live && !still_asks_you(i) {
        return false;
    }
    // The ship queue owes it a ship. Same exception.
    if f.shipping && !still_asks_you(i) {
        return false;
    }
    // The person put it away themselves.
    if f.hidden_until > f.now {
        return false;
    }
    if f.stopped {
        return true;
    }
    // A repeating-task run hides only when a worker explicitly marked it clean.
    if is_clean_run(i) {
        return false;
    }
    // Waiting on another thread: it sits in Later, and the supervisor resumes it.
    if waits_in_later(i, f.waiting_on) {
        return false;
    }
    if wait_came_back(i) {
        return true;
    }
    if runs_again_on_its_own(i) != 0.0 {
        return false;
    }
    // An agent stopped: what happens next is the person's.
    if parked_by_agent(i, f.now) {
        return true;
    }
    // The agent finished acting on the answer and left the thread open. That is news, and the
    // branches below would hand an open answered row back to the agent, so this comes first.
    if i.status != "done" && i.status != "claimed" && answer_settled(i) {
        return true;
    }
    if answered_her_ask(i) || gave_up_on_the_answer(i) {
        return true;
    }
    // A closed cause row is filed: the threads behind it carrying on are the news.
    if i.status == "done" && is_cause(i) {
        return false;
    }
    if i.status == "done" {
        // An agent's done on a thread the person wrote or spoke on is news; their own done is
        // the archive.
        if !filed_by_the_person(i) && i.live_answer().is_none() {
            return false;
        }
        if i.wrote("status").is_some_and(|w| w.source == "founder") {
            return false;
        }
        if closed_as_she_asked(i) {
            return false;
        }
        return i.wrote("status").and_then(|w| w.ts).or(i.updated_at).unwrap_or(0.0) > f.delivered_through;
    }
    // A stopped row is in the inbox whatever its kind.
    if i.status == "blocked" {
        return true;
    }
    // An answered question is the agent's again, and an answered review is being enacted.
    if asks_the_person(i) {
        return i.live_answer().is_none();
    }
    // A proposal waits in Later, and the thread that filed it shows it.
    if is_proposal(i) {
        return false;
    }
    // Agent-filed work under no thread is a proposal awaiting approval.
    if i.status == "open" && !filed_by_the_person(i) {
        return i.live_answer().is_none();
    }
    false
}

/// An open thread an agent filed under another thread, waiting on a yes. Not a question or a
/// review, not one the person wrote, not one that is answered, and not one filed under no
/// thread, because nothing may be hidden with nowhere to reach it from.
pub fn is_proposal(i: &Item) -> bool {
    i.status == "open" && !filed_by_the_person(i) && i.live_answer().is_none() && !asks_the_person(i) && i.truthy("parent")
}

/// One rule for the inbox, the rail and Later: blocked, behind a wait that can end, and not
/// stopped by the person.
pub fn waits_in_later(i: &Item, waiting_on: &[String]) -> bool {
    // A blocked task stays where its people read it, saying what blocks it.
    if is_task_row(i) {
        return false;
    }
    if set_by_person(i) {
        return true;
    }
    !waiting_on.is_empty() && i.status == "blocked" && i.wrote("status").is_none_or(|w| w.source != "founder")
}

/// Later holds a put-off or waiting row only once no run is on it or queued for it.
pub fn put_off_in_later(i: &Item, hidden_until: f64, now: f64, waiting_on: &[String], run_on_it: bool) -> bool {
    (hidden_until > now || waits_in_later(i, waiting_on)) && !run_on_it
}

/// The threads this one waits on that can still finish. It fails open, because hiding a thread
/// is the dangerous direction: a blocker that is done, one this window cannot see, the thread
/// itself, and a loop of threads that only wait on each other are not waits.
pub fn waits_that_can_end(i: &Item, by_id: &HashMap<&str, &Item>, path: &HashSet<String>) -> Vec<String> {
    let live = |row: &Item| -> Vec<String> {
        blocked_by(row).into_iter().filter(|id| !id.is_empty() && *id != row.id && by_id.get(id.as_str()).is_some_and(|b| b.status != "done")).collect()
    };
    live(i)
        .into_iter()
        .filter(|id| {
            if path.contains(id) {
                return false;
            }
            let blocker = by_id[id.as_str()];
            if live(blocker).is_empty() {
                return true;
            }
            let mut deeper = path.clone();
            deeper.insert(id.clone());
            !waits_that_can_end(blocker, by_id, &deeper).is_empty()
        })
        .collect()
}

/// How long Later holds a proposal quietly before its thread comes back.
const PROPOSAL_PATIENCE: f64 = 24.0 * 3_600_000.0;

/// The threads whose proposals have waited longer than that. The thread comes back, never the
/// proposal: it already shows every proposal under it.
pub fn threads_owed_an_answer<'a>(items: &'a [Item], by_id: &HashMap<&str, &'a Item>, now: f64) -> HashSet<&'a str> {
    let mut owed = HashSet::new();
    for i in items {
        if !is_proposal(i) || now - i.created_at.unwrap_or(0.0) <= PROPOSAL_PATIENCE {
            continue;
        }
        let Some(parent) = i.text("parent").and_then(|id| by_id.get(id)) else { continue };
        if parent.status == "done" && parent.wrote("status").is_some_and(|w| w.source == "founder") {
            continue;
        }
        // In Later: it comes back when Later ends. Only a person's put-off counts.
        if put_away_by_them(parent, 0.0) > now {
            continue;
        }
        if parent.status == "claimed" || (parent.live_answer().is_some() && !answer_settled(parent)) {
            continue;
        }
        owed.insert(parent.id.as_str());
    }
    owed
}

/// In progress promises that a worker is on this row or about to be, so it never holds a row
/// whose moment has not arrived.
pub fn belongs_in_progress(i: &Item, f: &Facts) -> bool {
    // A thread in Later waits for the person to say go, so no worker is coming.
    if not_started(i) {
        return false;
    }
    // A live session keeps the promise whatever the status says, and so does a ship owed.
    if f.live || f.shipping {
        return true;
    }
    if f.deferred_until > f.now {
        return false;
    }
    // Nothing will start it, so it is in Needs you.
    if f.stopped {
        return false;
    }
    if i.status == "claimed" {
        return true;
    }
    if set_by_person(i) {
        return false;
    }
    // A session acted on the answer and finished, so nothing is coming.
    if answer_settled(i) {
        return false;
    }
    // Its moment has come: the queue starts it.
    if runs_again_on_its_own(i) != 0.0 {
        return true;
    }
    if answered_her_ask(i) || gave_up_on_the_answer(i) {
        return false;
    }
    // Answered or approved, spawn pending.
    if i.status == "open" && i.live_answer().is_some() {
        return true;
    }
    // The person's own work, waiting its turn.
    i.status == "open" && !asks_the_person(i) && filed_by_the_person(i)
}

/// When the window treats a thread's work as due: its moment, unless a reply after the moment
/// is running now, as the supervisor does.
pub fn deferred_until(i: &Item) -> f64 {
    if word_waits(i) { 0.0 } else { i.run_at.unwrap_or(0.0) }
}

re!(NOT_WORDS, r"[^\p{L}\p{N}']+", "gu");

fn same_words(s: &str) -> String {
    js::trim(&NOT_WORDS.replace_all(&s.to_lowercase(), |_| " ".into())).to_string()
}

/// What a row says after its title, unless that only says the title again.
pub fn aside_besides(title: &str, aside: Option<&str>) -> Option<String> {
    let aside = aside.filter(|a| !a.is_empty())?;
    let (t, a) = (same_words(title), same_words(aside));
    if t.is_empty() {
        return Some(aside.to_string());
    }
    if a == t {
        return None;
    }
    if t.split(' ').count() >= 3 && (a.starts_with(&format!("{t} ")) || a.ends_with(&format!(" {t}"))) {
        return None;
    }
    Some(aside.to_string())
}

re!(CUT_MARK, r"…$", "");

// On a message written as one paragraph the title is cut from the body, so the summary starts
// after it rather than repeating it.
fn body_after_title(i: &Item) -> &str {
    let body = instruction_of(i.body.as_deref().unwrap_or(""));
    let label = CUT_MARK.strip(i.title.as_deref().unwrap_or(""));
    let label = js::trim(&label);
    match body.strip_prefix(label) {
        Some(rest) if !label.is_empty() => js::trim(rest),
        _ => body,
    }
}

/// The summary keeps whole sentences up to the budget, and clips mid-sentence with an ellipsis
/// only when the first sentence is longer than the budget.
const SUMMARY_BUDGET: usize = 112;

pub fn clip_to_sentence(text: &str) -> String {
    if js::len(text) <= SUMMARY_BUDGET {
        return text.to_string();
    }
    let window = js::slice(text, 0, SUMMARY_BUDGET + 1);
    let end = [". ", "? ", "! "].iter().filter_map(|stop| js::last_index_of(window, stop)).max();
    if let Some(end) = end.filter(|end| *end > 40) {
        return js::slice(text, 0, end + 1).to_string();
    }
    let space = js::last_index_of(window, " ").filter(|space| *space > 40).unwrap_or(SUMMARY_BUDGET);
    format!("{}…", js::trim_end(js::slice(text, 0, space)))
}

re!(BLANK_LINE, r"\n\s*\n", "");

/// The line a row prints under its title: whichever of body and result was written last, so a
/// blocked row with an answer shows the answer, and a finished row shows its result.
/// `trouble_body` is the body of the row the app makes itself, which prints only its first line.
pub fn row_summary(i: &Item, trouble_body: Option<&str>) -> String {
    if let Some(body) = trouble_body {
        return clip_to_sentence(js::trim(body.split('\n').next().unwrap_or("")));
    }
    // Over the agent's last word, which was written first and says it ships.
    if let Some(unshipped) = not_shipped_line(i) {
        return clip_to_sentence(&unshipped);
    }
    // Once someone is signed in every line carries its writer, and then the newest answer is
    // the news, whoever wrote it. A teammate's hand-back opens on a line the status already
    // says; their own words follow it.
    if let (Some(answer), Some(said)) = (i.text("answer"), i.wrote("answer").filter(|w| w.by.is_some()))
        && said.ts.is_some_and(|ts| ts >= i.wrote_at("body").max(i.wrote_at("result")))
    {
        let theirs = if said.teammate.is_some() { js::trim(&BLANK_LINE.split(answer)[1..].join("\n\n")).to_string() } else { String::new() };
        return clip_to_sentence(&first_real_line(if theirs.is_empty() { answer } else { &theirs }));
    }
    let finished = i.status == "done";
    let result_is_newer = i.wrote_at("result") > i.wrote_at("body");
    match i.result_text().filter(|_| finished || result_is_newer) {
        Some(result) => clip_to_sentence(&preview_text(result)),
        None => clip_to_sentence(&preview_text(body_after_title(i))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    const NOW: f64 = 1_000_000.0;

    fn facts() -> Facts<'static> {
        Facts { now: NOW, ..Facts::default() }
    }

    /// The person's own thread, answered by an agent and left open.
    fn answered() -> Value {
        json!({ "id": "w-1", "status": "open", "kind": "directive", "labels": ["founder"], "title": "Fix the export", "body": "Fix the export", "result": "Fixed.",
            "wrote": { "title": { "ts": 10, "source": "founder" }, "body": { "ts": 10, "source": "founder" }, "result": { "ts": 20, "source": "agent" } } })
    }

    #[test]
    fn an_answer_to_the_persons_ask_is_in_the_inbox_and_not_in_progress() {
        let i = item(answered());
        assert!(answered_her_ask(&i));
        assert!(belongs_in_inbox(&i, &facts()));
        assert!(!belongs_in_progress(&i, &facts()));
    }

    #[test]
    fn her_own_unanswered_work_is_in_progress_and_not_in_the_inbox() {
        let mut v = answered();
        v.as_object_mut().unwrap().remove("result");
        v["wrote"].as_object_mut().unwrap().remove("result");
        let i = item(v);
        assert!(!belongs_in_inbox(&i, &facts()));
        assert!(belongs_in_progress(&i, &facts()));
    }

    #[test]
    fn a_session_on_a_row_takes_it_out_of_the_inbox_unless_it_still_asks() {
        let i = item(answered());
        assert!(!belongs_in_inbox(&i, &Facts { live: true, ..facts() }));
        assert!(belongs_in_progress(&i, &Facts { live: true, ..facts() }));
        assert!(!belongs_in_inbox(&i, &Facts { shipping: true, ..facts() }));
        let question = item(json!({ "status": "claimed", "kind": "question", "labels": [], "title": "Which clinic first?" }));
        assert!(still_asks_you(&question));
        assert!(belongs_in_inbox(&question, &Facts { live: true, ..facts() }));
        // A permission request answers before the session does.
        assert!(belongs_in_inbox(&i, &Facts { live: true, asking: true, ..facts() }));
    }

    #[test]
    fn a_finished_thread_is_news_only_when_an_agent_finished_the_persons_own() {
        let mut v = answered();
        v["status"] = json!("done");
        v["wrote"]["status"] = json!({ "ts": 30, "source": "agent" });
        assert!(belongs_in_inbox(&item(v.clone()), &facts()));
        // Finished before answers started being delivered: it stays in Done.
        assert!(!belongs_in_inbox(&item(v.clone()), &Facts { delivered_through: 40.0, ..facts() }));
        // The person archived it themselves.
        let mut archived = v.clone();
        archived["wrote"]["status"]["source"] = json!("founder");
        assert!(!belongs_in_inbox(&item(archived), &facts()));
        // Agent work the person never touched is filed.
        let mut untouched = v.clone();
        untouched["labels"] = json!([]);
        assert!(!belongs_in_inbox(&item(untouched), &facts()));
        // Their last words asked for the close.
        let mut asked = v;
        asked["answer"] = json!("Looks right, close this task");
        asked["wrote"]["answer"] = json!({ "ts": 25, "source": "founder" });
        assert!(!belongs_in_inbox(&item(asked), &facts()));
    }

    #[test]
    fn what_the_person_put_away_is_hidden_and_what_an_agent_parked_is_shown() {
        let mut hers = answered();
        hers["runAt"] = json!(NOW + 500.0);
        hers["wrote"]["runAt"] = json!({ "ts": 30, "source": "founder" });
        let i = item(hers);
        assert_eq!(hidden_until(&i, 0.0), NOW + 500.0);
        assert!(!belongs_in_inbox(&i, &Facts { hidden_until: hidden_until(&i, 0.0), ..facts() }));
        // An agent parking an unanswered question may not hide it.
        let parked = item(json!({ "status": "open", "kind": "question", "labels": [], "title": "Which clinic first?", "runAt": NOW + 500.0,
            "wrote": { "runAt": { "ts": 30, "source": "agent" } } }));
        assert_eq!(hidden_until(&parked, 0.0), 0.0);
        assert!(parked_by_agent(&parked, NOW));
        assert!(belongs_in_inbox(&parked, &facts()));
        // The old snooze map is still honoured.
        assert_eq!(hidden_until(&item(answered()), NOW + 9.0), NOW + 9.0);
    }

    #[test]
    fn a_thread_its_agent_will_run_again_waits_out_of_the_inbox() {
        let mut v = answered();
        v["runAt"] = json!(NOW + 500.0);
        v["wrote"]["runAt"] = json!({ "ts": 30, "source": "agent" });
        let i = item(v.clone());
        assert_eq!(runs_again_on_its_own(&i), NOW + 500.0);
        assert!(!belongs_in_inbox(&i, &Facts { hidden_until: hidden_until(&i, 0.0), ..facts() }));
        // An answer that offers a choice keeps it in Needs you.
        v["result"] = json!("Fixed.\n\n## Options\n1. Ship it (recommended)\n2. Change the copy first");
        let offers = item(v);
        assert_eq!(runs_again_on_its_own(&offers), 0.0);
        assert!(belongs_in_inbox(&offers, &facts()));
    }

    #[test]
    fn a_stopped_or_blocked_row_needs_the_person() {
        let mut v = answered();
        v.as_object_mut().unwrap().remove("result");
        v["wrote"].as_object_mut().unwrap().remove("result");
        assert!(belongs_in_inbox(&item(v.clone()), &Facts { stopped: true, ..facts() }));
        assert!(!belongs_in_progress(&item(v.clone()), &Facts { stopped: true, ..facts() }));
        v["status"] = json!("blocked");
        assert!(belongs_in_inbox(&item(v.clone()), &facts()));
        assert!(!belongs_in_progress(&item(v), &facts()));
    }

    #[test]
    fn a_proposal_waits_in_later_and_work_filed_under_nothing_waits_in_the_inbox() {
        let proposal = item(json!({ "id": "w-2", "status": "open", "kind": "directive", "labels": [], "parent": "w-1", "title": "Add a retry" }));
        assert!(is_proposal(&proposal));
        assert!(!belongs_in_inbox(&proposal, &facts()));
        let orphan = item(json!({ "id": "w-3", "status": "open", "kind": "directive", "labels": [], "title": "Add a retry" }));
        assert!(!is_proposal(&orphan));
        assert!(belongs_in_inbox(&orphan, &facts()));
        let approved = item(json!({ "id": "w-3", "status": "open", "kind": "directive", "labels": [], "title": "Add a retry", "answer": "Approved. Run it." }));
        assert!(!belongs_in_inbox(&approved, &facts()));
        assert!(belongs_in_progress(&approved, &facts()));
    }

    #[test]
    fn a_thread_whose_proposals_waited_a_day_comes_back() {
        let day = 24.0 * 3_600_000.0;
        let now = 10.0 * day;
        let parent = json!({ "id": "w-1", "status": "done", "labels": ["founder"], "wrote": { "status": { "ts": 5, "source": "agent" } } });
        let old = json!({ "id": "w-2", "status": "open", "labels": [], "parent": "w-1", "createdAt": now - 2.0 * day });
        let fresh = json!({ "id": "w-3", "status": "open", "labels": [], "parent": "w-9", "createdAt": now - 0.5 * day });
        let items: Vec<Item> = [parent.clone(), old.clone(), fresh].into_iter().map(item).collect();
        let by_id: HashMap<&str, &Item> = items.iter().map(|i| (i.id.as_str(), i)).collect();
        assert_eq!(threads_owed_an_answer(&items, &by_id, now), HashSet::from(["w-1"]));
        // A thread the person closed stays closed.
        let mut closed = parent;
        closed["wrote"]["status"]["source"] = json!("founder");
        let items: Vec<Item> = [closed, old].into_iter().map(item).collect();
        let by_id: HashMap<&str, &Item> = items.iter().map(|i| (i.id.as_str(), i)).collect();
        assert!(threads_owed_an_answer(&items, &by_id, now).is_empty());
    }

    #[test]
    fn only_a_wait_that_can_end_counts() {
        let items: Vec<Item> = [
            json!({ "id": "a", "status": "blocked", "blockedBy": ["b", "gone", "a", "d"] }),
            json!({ "id": "b", "status": "open" }),
            json!({ "id": "d", "status": "done" }),
            json!({ "id": "x", "status": "blocked", "blockedBy": ["y"] }),
            json!({ "id": "y", "status": "blocked", "blockedBy": ["x"] }),
        ]
        .into_iter()
        .map(item)
        .collect();
        let by_id: HashMap<&str, &Item> = items.iter().map(|i| (i.id.as_str(), i)).collect();
        let waits = |id: &str| waits_that_can_end(by_id[id], &by_id, &HashSet::from([id.to_string()]));
        assert_eq!(waits("a"), ["b"]);
        // A loop of threads that only wait on each other is not a wait: nothing would resume them.
        assert!(waits("x").is_empty());
        let agent_wait = item(json!({ "id": "a", "status": "blocked", "blockedBy": ["b"], "wrote": { "status": { "ts": 1, "source": "agent" } } }));
        assert!(waits_in_later(&agent_wait, &["b".to_string()]));
        assert!(!waits_in_later(&agent_wait, &[]));
    }

    #[test]
    fn the_summary_keeps_whole_sentences() {
        let long = "The export now writes one file per clinic. It also names each file by the week it covers, so nothing overwrites last week's numbers when the job runs twice.";
        assert_eq!(clip_to_sentence(long), "The export now writes one file per clinic.");
        let one = "The export now writes one file per clinic and names each file by the week it covers so that nothing overwrites last week's numbers";
        assert_eq!(clip_to_sentence(one), "The export now writes one file per clinic and names each file by the week it covers so that nothing overwrites…");
        assert_eq!(clip_to_sentence("Fixed."), "Fixed.");
    }

    #[test]
    fn the_line_is_whichever_of_ask_and_answer_was_written_last() {
        assert_eq!(row_summary(&item(answered()), None), "Fixed.");
        let mut v = answered();
        v["body"] = json!("Fix the export so it writes one file per clinic.");
        v["wrote"]["body"]["ts"] = json!(30);
        assert_eq!(row_summary(&item(v.clone()), None), "so it writes one file per clinic.");
        // A finished row shows its result whenever it was written.
        v["status"] = json!("done");
        assert_eq!(row_summary(&item(v), None), "Fixed.");
    }

    #[test]
    fn once_someone_is_signed_in_the_newest_answer_is_the_line() {
        let mut v = answered();
        v["answer"] = json!("Reply:\nNow do the second clinic.");
        v["wrote"]["answer"] = json!({ "ts": 30, "source": "founder", "by": "p-1" });
        assert_eq!(row_summary(&item(v.clone()), None), "Now do the second clinic.");
        // An answer the agent has spoken past is no longer the news.
        v["wrote"]["result"]["ts"] = json!(40);
        assert_eq!(row_summary(&item(v.clone()), None), "Fixed.");
        // With nobody signed in no line carries a writer, and the rule above stands.
        v["wrote"]["result"]["ts"] = json!(20);
        v["wrote"]["answer"] = json!({ "ts": 30, "source": "founder" });
        assert_eq!(row_summary(&item(v.clone()), None), "Fixed.");
        // A teammate's hand-back: their own words, after the line that says who handed it back.
        v["answer"] = json!("Theo handed this back.\n\nThe second clinic needs its own key.");
        v["wrote"]["answer"] = json!({ "ts": 30, "source": "system", "by": "p-2", "teammate": "p-2" });
        assert_eq!(row_summary(&item(v), None), "The second clinic needs its own key.");
    }

    #[test]
    fn an_aside_that_only_repeats_the_title_is_dropped() {
        assert_eq!(aside_besides("Fix the export", Some("2 tasks to approve")).as_deref(), Some("2 tasks to approve"));
        assert_eq!(aside_besides("Fix the export", Some("fix the export!")), None);
        assert_eq!(aside_besides("Fix the export", Some("“Fix the export” finished")), None);
        assert_eq!(aside_besides("Fix", Some("Fix finished")).as_deref(), Some("Fix finished"));
        assert_eq!(aside_besides("Fix the export", None), None);
    }
}
