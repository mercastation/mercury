//! What answering a thread and stopping its agent do, in the web window's own words and order:
//! the Stop that takes Send's place (`stop` in `DockComposer`, `stopAgent` and `stoppable` in
//! `renderer/src/App.tsx` and `list-rules.ts`), where a reply leaves the person
//! (`renderer/src/advance.ts`), what Z writes to take a written reply or pick back and how old a
//! thing may be before Z asks first (`answerWith`, `pickOption`, `renderer/src/undo-window.ts`),
//! the question before an option throws work away (`renderer/src/discard-asks-first.ts`), and
//! the once-ever warning before a waiting agent is closed or put off
//! (`renderer/src/stopped-agent.ts`). `docs/parity-answering.md` lists every line of it.

use serde_json::{Value, json};

use super::format::{self, Choice};
use super::js::re;
use super::list;
use super::writes::{ANSWER, SCHEDULE, SNOOZE, Write};
use crate::data::Item;

/* ------------------------------------- stopping ------------------------------------- */

/// The channel a stop goes to: it kills the thread's session and parks the row blocked, so no
/// worker takes it up again and it waits in Needs you (`zero:stop-session` in `main/ipc.mts`).
pub const STOP: &str = "zero:stop-session";

/// Whether the thread can be stopped: the In progress tab's own rule, so the tab and the stop
/// never disagree about what is under way (`stoppable` in `list-rules.ts`). `live` is a run on
/// it or one being started, `stalled` the supervisor's word that nothing will start it, and
/// `legacy_snooze` the old moment the web window still honours (`dueAt`). An agent the app did
/// not start, the app's own rows and a finished thread are never stopped.
pub fn stoppable(item: &Item, live: bool, stalled: bool, legacy_snooze: f64, now: f64) -> bool {
    if item.status == "done" || item.product.is_empty() || item.rest.get("agent").is_some_and(|a| !a.is_null()) {
        return false;
    }
    let deferred = list::deferred_until(item).max(legacy_snooze);
    list::belongs_in_progress(item, &list::Facts { deferred_until: deferred, now, live, stopped: stalled, ..list::Facts::default() })
}

/// What the stop writes: the thread, and nothing else.
pub fn stop(item: &Item) -> Write {
    (STOP, json!({ "product": item.product, "id": item.id }))
}

/// The button's title, by whether a run is up (`aria-label` and `title` on `.dock-stop`).
pub fn stop_title(running: bool) -> &'static str {
    if running { "Stop this agent. It goes back to your inbox." } else { "Stop this task. It goes back to your inbox." }
}

/// What is said once the engine has the stop, by whether a run was on the thread at the press.
pub fn stopped_line(was_running: bool) -> &'static str {
    if was_running {
        "Agent stopped. Back in your inbox. Reply to redirect it."
    } else {
        "Stopped before it started. Back in your inbox. Reply to redirect it."
    }
}

/// The command bar's row for it, and the hint it carries when nothing has started yet.
pub fn stop_row_words(running: bool) -> (&'static str, Option<&'static str>) {
    if running { ("Stop Agent (Moves to Inbox)", None) } else { ("Stop Task (Moves to Inbox)", Some("nothing has started on it yet")) }
}

/* --------------------------------- a reply, written --------------------------------- */

/// How long a written copy of something sent stands beside the copy drawn at Send, which it
/// has long since replaced on screen (`LANDED_MS`).
pub const LANDED_MS: f64 = 10_000.0;

/// A reply's moment set for later and waiting on the thread: the moment, or 0
/// (`replyWaitsUntil` in `renderer/src/threads/composer-rules.ts`).
pub fn reply_waits_until(item: &Item, now: f64) -> f64 {
    let run_at = item.run_at.unwrap_or(0.0);
    let (Some(set), Some(said)) = (item.wrote("runAt"), item.wrote("answer")) else {
        return 0.0;
    };
    if run_at <= now || set.source != "founder" {
        return 0.0;
    }
    let answer = item.rest.get("answer").and_then(Value::as_str).unwrap_or("").trim();
    if answer.is_empty() || answer == "(withdrawn)" {
        return 0.0;
    }
    if (said.ts.unwrap_or(0.0) - set.ts.unwrap_or(0.0)).abs() <= 5_000.0 { run_at } else { 0.0 }
}

/// What a reply writes after its words, in order, and what Z writes back for them first: a
/// reply set for later and waiting is taken with this one (its moment off, and back on with
/// Z); the moment the person set comes off as the reply lands (`laterHerWordLifts`, the store
/// does it) and goes back on with Z; a reminder comes off after it, and back on with Z.
pub struct Around {
    pub after: Vec<Write>,
    pub undo_first: Vec<Write>,
}

pub fn around_a_reply(item: &Item, now: f64) -> Around {
    let at = |channel: &'static str, field: &str, value: Value| (channel, json!({ "product": item.product, "id": item.id, field: value }));
    let prior_run = item.run_at.unwrap_or(0.0);
    let snoozed = item.number("snoozedUntil").unwrap_or(0.0);
    let waiting = reply_waits_until(item, now) > 0.0;
    let lifted = prior_run > now && item.wrote("runAt").is_some_and(|w| w.source == "founder");
    let mut around = Around { after: Vec::new(), undo_first: Vec::new() };
    if waiting {
        around.after.push(at(SCHEDULE, "runAt", json!(0)));
        around.undo_first.push(at(SCHEDULE, "runAt", json!(prior_run)));
    }
    if lifted && !waiting {
        around.undo_first.push(at(SCHEDULE, "runAt", json!(prior_run)));
    }
    if snoozed > now {
        around.after.push(at(SNOOZE, "snoozedUntil", json!(0)));
        around.undo_first.push(at(SNOOZE, "snoozedUntil", json!(snoozed)));
    }
    around
}

/// Taking back a written reply or pick: the session it started stopped, then the answer
/// overwritten and the status put back to the one the reply found, so a withdrawn reply on an
/// open thread does not leave it blocked by the stop (`withdrawReply`). A reply's stop says
/// `withdraw`, a pick's does not.
pub fn withdraw(item: &Item, reply: bool) -> Vec<Write> {
    let mut stop = json!({ "product": item.product, "id": item.id });
    if reply {
        stop["withdraw"] = json!(true);
    }
    vec![(STOP, stop), (ANSWER, json!({ "product": item.product, "id": item.id, "answer": "(withdrawn)", "status": item.status }))]
}

/// The lines a written reply's Z says and asks.
pub const REPLY_WITHDRAWN: &str = "Reply withdrawn, back in your inbox";
pub const REPLY_UNDOES: &str = "take back that reply and stop the agent";
/// What its line says too when the words went back in the box.
pub const BACK_IN_THE_BOX: &str = " · your message is back in the reply box";
/// A reply handed to a run already going cannot be taken back.
pub const ALREADY_WITH_THE_AGENT: &str = "Your message is already with the agent. Send another one to add to it.";
pub const ALREADY_UNDOES: &str = "go back to that conversation";

pub fn pick_withdrawn(n: f64) -> String {
    format!("Option {n} withdrawn, back in your inbox")
}
pub fn pick_undoes(n: f64) -> String {
    format!("take back option {n}")
}

/// What a Z leaves in the thread it undid, as the person's own action (`UndidSpec`).
pub const UNDID_REPLY: &str = "Undid your reply";
pub const UNDID_PICKING: &str = "Undid picking";
pub const UNDID_CLOSING: &str = "Undid closing it";

/// The line a refused reply comes back with while the person is elsewhere.
pub const DID_NOT_REACH: &str = "A message you sent did not reach its agent. Click to open it, your words are back in the reply box.";
/// And while they are on its thread, when the engine gave no words of its own.
pub const COULD_NOT_SEND: &str = "Could not send. Your message is back in the reply box.";

/* ------------------------------------ Z's pile ------------------------------------ */

/// Inside thirty seconds Z acts at once; past them it names what it would undo and only a
/// second Z inside eight seconds does it (`UNDO_STRAIGHT_AWAY_MS`, `UNDO_ASK_STANDS_MS`).
pub const UNDO_STRAIGHT_AWAY_MS: f64 = 30_000.0;
pub const UNDO_ASK_STANDS_MS: f64 = 8_000.0;

pub fn instant(at: f64, now: f64) -> bool {
    now - at <= UNDO_STRAIGHT_AWAY_MS
}

pub fn ask_still_stands(asked_at: f64, now: f64) -> bool {
    now - asked_at <= UNDO_ASK_STANDS_MS
}

pub fn undo_ask(undoes: &str) -> String {
    format!("Press Z again to {undoes}.")
}

/// What a Z that could not be done says, with the engine's reason.
pub fn could_not_undo(why: &str) -> String {
    format!("Could not undo that. {why}")
}

/* ------------------------------- where a reply leaves you ------------------------------- */

/// Whether a reply moves the person on to the next thread waiting: when the thread was opened
/// from Needs you on the Inbox, and was in it, and not from Home or come back to with O
/// (`replyMovesOn`).
pub fn reply_moves_on(from_needs_you: bool, in_the_list: bool, from_home: bool, came_back: bool) -> bool {
    from_needs_you && in_the_list && !from_home && !came_back
}

/// A pick answers and moves on, unless the talk plainly carries on here: its words promise
/// more, or an agent is on the thread now (`pickStaysPut`).
pub fn pick_stays_put(option: &str, live: bool) -> bool {
    live || MORE_COMING.test(&format::without_recommended(option))
}

re!(MORE_COMING, r"\?\s*$|\bask me\b|\bI(?:'ll| will) (?:explain|say|tell|describe)\b|\blet me (?:explain|say|describe)\b", "i");

/// What the corner says a pick sent, and where (`pickSentWords`).
pub fn pick_sent_words(option: &str, title: &str) -> String {
    let words = format::without_recommended(option);
    let words = words.trim();
    if words == "Don't send it" {
        return format!("Not sent to {title}");
    }
    let letters: Vec<char> = words.chars().collect();
    let cut = if letters.len() > 48 { format!("{}…", letters[..47].iter().collect::<String>()) } else { words.to_string() };
    format!("Sent “{cut}” to {title}")
}

/// The first words of the line over the reply box of the thread a send moved the person to
/// (`sentLineSays`).
pub fn sent_line_says(said: &str) -> &'static str {
    if said.starts_with("Not sent") { "Not sent to" } else { "Sent to" }
}

/// How long that line stands, its ring running out (`SENT_LINE_MS`).
pub const SENT_LINE_MS: f64 = 5_000.0;

/// A press arriving within this long of the app moving the person onto a thread, or of the
/// options moving under a still pointer, was aimed at what was there before (`PICK_SETTLES_MS`).
pub const PICK_SETTLES_MS: f64 = 800.0;
pub const AIMED_AT_THE_THREAD_YOU_LEFT: &str = "That number was for the thread you just left. Press it again to answer this one.";
pub const THE_OPTIONS_MOVED: &str = "The options moved under your pointer. Click the one you want again.";
/// How far the options must move under the pointer to count (`MOVED_PX`).
pub const MOVED_PX: f32 = 4.0;

/* ------------------------------ the question before a discard ------------------------------ */

/// The question an option that throws the change away asks first, and its two answers: keeping
/// it sits on 2, the number the discard was picked with (`discardAsk`).
pub const DISCARD_ASK: &str = "Discard the change?";

pub fn discard_options() -> Vec<Choice> {
    vec![Choice { n: 1.0, text: "Discard it".to_string(), recommended: false }, Choice { n: 2.0, text: "Keep it".to_string(), recommended: true }]
}

/* --------------------------------- the stopped agent --------------------------------- */

/// An agent stopped on this thread and waiting on the person: its offer is live and not every
/// choice is to close (`stoppedWaiting`).
pub fn stopped_waiting(item: &Item) -> bool {
    format::offer_is_live(item) && !format::item_options(item).iter().all(|o| format::closes_the_task(&o.text))
}

/// The one lesson, for the key that would leave it stuck; E's is held and says to press again,
/// L's opens its picker with the sentence inside (`stoppedWarning`).
pub fn stopped_warning(closing: bool) -> String {
    if closing {
        "This agent is waiting on your answer, and marking it done leaves it stuck. Press again to go ahead.".to_string()
    } else {
        "This agent is waiting on your answer, and putting it off leaves it stuck.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    const NOW: f64 = 1_000_000_000.0;

    #[test]
    fn a_thread_can_be_stopped_while_it_is_in_progress_and_never_once_done() {
        // A run on it, whatever its status says.
        let claimed = item(json!({ "id": "w-1", "product": "shop", "status": "claimed" }));
        assert!(stoppable(&claimed, false, false, 0.0, NOW));
        let blocked = item(json!({ "id": "w-1", "product": "shop", "status": "blocked" }));
        assert!(stoppable(&blocked, true, false, 0.0, NOW));
        assert!(!stoppable(&blocked, false, false, 0.0, NOW));
        // Answered and waiting for a worker: stoppable before anything has started.
        let answered = item(json!({ "id": "w-1", "product": "shop", "status": "open", "kind": "question", "answer": "Go on.", "wrote": { "answer": { "ts": 5, "source": "founder" } } }));
        assert!(stoppable(&answered, false, false, 0.0, NOW));
        // The supervisor's stalled list says nothing will start it.
        assert!(!stoppable(&answered, false, true, 0.0, NOW));
        // Set for later, or put off on the old snooze.
        let later = item(json!({ "id": "w-1", "product": "shop", "status": "claimed", "runAt": NOW + 60_000.0 }));
        assert!(!stoppable(&later, false, false, 0.0, NOW));
        assert!(!stoppable(&claimed, false, false, NOW + 1.0, NOW));
        // Done, the app's own rows, a terminal agent: never.
        assert!(!stoppable(&item(json!({ "id": "w-1", "product": "shop", "status": "done" })), true, false, 0.0, NOW));
        assert!(!stoppable(&item(json!({ "id": "trouble", "product": "", "status": "claimed" })), true, false, 0.0, NOW));
        assert!(!stoppable(&item(json!({ "id": "a-1", "product": "shop", "status": "claimed", "agent": { "pid": 4 } })), true, false, 0.0, NOW));
    }

    #[test]
    fn the_stop_says_what_it_stopped_in_the_web_windows_words() {
        assert_eq!(stop(&item(json!({ "id": "w-1", "product": "shop", "status": "claimed" }))), (STOP, json!({ "product": "shop", "id": "w-1" })));
        assert_eq!(stop_title(true), "Stop this agent. It goes back to your inbox.");
        assert_eq!(stop_title(false), "Stop this task. It goes back to your inbox.");
        assert_eq!(stopped_line(true), "Agent stopped. Back in your inbox. Reply to redirect it.");
        assert_eq!(stopped_line(false), "Stopped before it started. Back in your inbox. Reply to redirect it.");
        assert_eq!(stop_row_words(true), ("Stop Agent (Moves to Inbox)", None));
        assert_eq!(stop_row_words(false), ("Stop Task (Moves to Inbox)", Some("nothing has started on it yet")));
    }

    #[test]
    fn taking_a_written_reply_back_stops_its_run_and_puts_the_status_back() {
        let open = item(json!({ "id": "w-1", "product": "shop", "status": "open" }));
        assert_eq!(withdraw(&open, true), vec![(STOP, json!({ "product": "shop", "id": "w-1", "withdraw": true })), (ANSWER, json!({ "product": "shop", "id": "w-1", "answer": "(withdrawn)", "status": "open" }))]);
        let blocked = item(json!({ "id": "w-1", "product": "shop", "status": "blocked" }));
        assert_eq!(withdraw(&blocked, false), vec![(STOP, json!({ "product": "shop", "id": "w-1" })), (ANSWER, json!({ "product": "shop", "id": "w-1", "answer": "(withdrawn)", "status": "blocked" }))]);
    }

    #[test]
    fn a_reply_lifts_a_reminder_and_takes_a_waiting_reply_with_it_and_z_puts_both_back() {
        let plain = item(json!({ "id": "w-1", "product": "shop", "status": "blocked" }));
        let around = around_a_reply(&plain, NOW);
        assert!(around.after.is_empty() && around.undo_first.is_empty());
        let put_off = item(json!({ "id": "w-1", "product": "shop", "status": "blocked", "snoozedUntil": NOW + 5000.0 }));
        let around = around_a_reply(&put_off, NOW);
        assert_eq!(around.after, vec![(SNOOZE, json!({ "product": "shop", "id": "w-1", "snoozedUntil": 0 }))]);
        assert_eq!(around.undo_first, vec![(SNOOZE, json!({ "product": "shop", "id": "w-1", "snoozedUntil": NOW + 5000.0 }))]);
        // A reply set for later and waiting: its moment comes off now and back with Z.
        let waiting = item(json!({ "id": "w-1", "product": "shop", "status": "open", "answer": "Later, this.", "runAt": NOW + 9000.0,
            "wrote": { "runAt": { "ts": 100, "source": "founder" }, "answer": { "ts": 102, "source": "founder" } } }));
        assert_eq!(reply_waits_until(&waiting, NOW), NOW + 9000.0);
        let around = around_a_reply(&waiting, NOW);
        assert_eq!(around.after, vec![(SCHEDULE, json!({ "product": "shop", "id": "w-1", "runAt": 0 }))]);
        assert_eq!(around.undo_first, vec![(SCHEDULE, json!({ "product": "shop", "id": "w-1", "runAt": NOW + 9000.0 }))]);
        // The person's own moment the store lifts as the reply lands: only Z writes it.
        let set = item(json!({ "id": "w-1", "product": "shop", "status": "open", "runAt": NOW + 9000.0, "wrote": { "runAt": { "ts": 100, "source": "founder" } } }));
        let around = around_a_reply(&set, NOW);
        assert!(around.after.is_empty());
        assert_eq!(around.undo_first, vec![(SCHEDULE, json!({ "product": "shop", "id": "w-1", "runAt": NOW + 9000.0 }))]);
    }

    #[test]
    fn z_acts_at_once_for_thirty_seconds_and_then_asks() {
        assert!(instant(NOW, NOW + 30_000.0));
        assert!(!instant(NOW, NOW + 30_001.0));
        assert!(ask_still_stands(NOW, NOW + 8_000.0) && !ask_still_stands(NOW, NOW + 8_001.0));
        assert_eq!(undo_ask(REPLY_UNDOES), "Press Z again to take back that reply and stop the agent.");
        assert_eq!(undo_ask(&pick_undoes(2.0)), "Press Z again to take back option 2.");
        assert_eq!(pick_withdrawn(2.0), "Option 2 withdrawn, back in your inbox");
    }

    #[test]
    fn a_reply_moves_on_only_while_needs_you_is_being_cleared() {
        assert!(reply_moves_on(true, true, false, false));
        assert!(!reply_moves_on(false, true, false, false));
        assert!(!reply_moves_on(true, false, false, false));
        assert!(!reply_moves_on(true, true, true, false));
        assert!(!reply_moves_on(true, true, false, true));
    }

    #[test]
    fn a_pick_stays_when_its_words_promise_more_or_a_run_is_up() {
        assert!(!pick_stays_put("Ship the fix now (recommended)", false));
        assert!(pick_stays_put("Ship the fix now", true));
        assert!(pick_stays_put("Which one did you mean?", false));
        assert!(pick_stays_put("Ask me before each refund", false));
        assert!(pick_stays_put("I'll explain what I need", false));
        assert!(pick_stays_put("Let me describe it", false));
        assert!(!pick_stays_put("Tell support first", false));
    }

    #[test]
    fn a_picks_line_names_the_option_and_the_thread() {
        assert_eq!(pick_sent_words("Ship the fix now (recommended)", "Checkout fails"), "Sent “Ship the fix now” to Checkout fails");
        assert_eq!(pick_sent_words("Don't send it", "Checkout fails"), "Not sent to Checkout fails");
        let long = "Ship the fix and let the nightly job retry every order";
        assert_eq!(pick_sent_words(long, "X"), "Sent “Ship the fix and let the nightly job retry ever…” to X");
        assert_eq!(sent_line_says("Sent “a” to X"), "Sent to");
        assert_eq!(sent_line_says("Not sent to X"), "Not sent to");
    }

    #[test]
    fn an_agent_waits_on_the_person_while_its_offer_stands_and_is_not_only_to_close() {
        let asking = item(json!({ "id": "w-1", "product": "shop", "status": "blocked", "note": "Which way?\n\n1. Ship it\n2. Hold it",
            "wrote": { "note": { "ts": 5, "source": "agent" } } }));
        assert!(stopped_waiting(&asking));
        let closing = item(json!({ "id": "w-1", "product": "shop", "status": "blocked", "note": "Done.\n\n1. Close this task",
            "wrote": { "note": { "ts": 5, "source": "agent" } } }));
        assert!(!stopped_waiting(&closing));
        assert_eq!(stopped_warning(true), "This agent is waiting on your answer, and marking it done leaves it stuck. Press again to go ahead.");
        assert_eq!(stopped_warning(false), "This agent is waiting on your answer, and putting it off leaves it stuck.");
    }

    #[test]
    fn the_discard_question_keeps_the_change_on_two() {
        let options = discard_options();
        assert_eq!(options.iter().map(|o| (o.n, o.text.as_str(), o.recommended)).collect::<Vec<_>>(), [(1.0, "Discard it", false), (2.0, "Keep it", true)]);
        assert_eq!(DISCARD_ASK, "Discard the change?");
    }
}
