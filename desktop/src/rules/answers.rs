//! Has the agent finished acting on what the person said, and whose work is this. A port of
//! `shared/answers.mts`, the one rule the supervisor and the inbox both read.

use crate::data::Item;

const SPOKEN: [&str; 3] = ["result", "title", "body"];

/// An agent wrote something the person can read on the row after `since`.
pub fn agent_spoke_since(item: &Item, since: f64) -> bool {
    SPOKEN.iter().any(|field| item.wrote(field).is_some_and(|w| w.source == "agent" && w.at() > since))
}

/// A session acted on the person's answer and left a word they can read. `answeredThrough` is
/// the time of the answer the last finished session was started to carry.
pub fn answer_settled(item: &Item) -> bool {
    if item.live_answer().is_none() {
        return false;
    }
    let ts = item.wrote_at("answer");
    if !(ts > 0.0 && item.number("answeredThrough").unwrap_or(0.0) >= ts) {
        return false;
    }
    agent_spoke_since(item, ts)
}

/// The person's own work: the `founder` tag, unless the ledger says an agent wrote it. A tag
/// with no recorded author predates `wrote` and keeps counting.
pub fn filed_by_the_person(item: &Item) -> bool {
    item.has_label("founder") && item.wrote("labels").is_none_or(|w| w.source != "agent")
}

fn asks_the_person(item: &Item) -> bool {
    matches!(item.kind.as_deref(), Some("question" | "review"))
}

/// The moment an agent set for its own thread to run again, or 0. It counts while it is later
/// than the thread's newest answer, so it wakes the thread once.
pub fn wakes_itself_at(item: &Item) -> f64 {
    if item.status != "open" || asks_the_person(item) {
        return 0.0;
    }
    let Some(set) = item.wrote("runAt") else { return 0.0 };
    if set.source == "founder" {
        return 0.0;
    }
    let at = item.run_at.unwrap_or(0.0);
    if at > item.wrote_at("result") { at } else { 0.0 }
}

/// A person's word written after the moment was set, and not yet acted on, runs now.
pub fn word_waits(item: &Item) -> bool {
    let (Some(said), Some(set)) = (item.wrote("answer"), item.wrote("runAt")) else { return false };
    if item.live_answer().is_none() {
        return false;
    }
    if said.source != "founder" && said.teammate.is_none() {
        return false;
    }
    // A missing time compares as false.
    matches!((said.ts, set.ts), (Some(said), Some(set)) if said > set && item.number("answeredThrough").unwrap_or(0.0) < said)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn an_answer_is_settled_once_a_session_acted_on_it_and_left_a_word() {
        let base = json!({ "status": "open", "answer": "Ship it", "answeredThrough": 30,
            "wrote": { "answer": { "ts": 30, "source": "founder" }, "result": { "ts": 40, "source": "agent" } } });
        assert!(answer_settled(&item(base.clone())));
        // The session exited having written nothing since the answer: not settled.
        let mut silent = base.clone();
        silent["wrote"]["result"]["ts"] = json!(20);
        assert!(!answer_settled(&item(silent)));
        // A newer answer makes the row live again by itself.
        let mut again = base.clone();
        again["wrote"]["answer"]["ts"] = json!(50);
        assert!(!answer_settled(&item(again)));
        let mut withdrawn = base;
        withdrawn["answer"] = json!("(withdrawn)");
        assert!(!answer_settled(&item(withdrawn)));
    }

    #[test]
    fn the_founder_tag_counts_unless_an_agent_wrote_it() {
        assert!(filed_by_the_person(&item(json!({ "labels": ["founder"] }))));
        assert!(filed_by_the_person(&item(json!({ "labels": ["founder"], "wrote": { "labels": { "ts": 1, "source": "founder" } } }))));
        assert!(!filed_by_the_person(&item(json!({ "labels": ["founder"], "wrote": { "labels": { "ts": 1, "source": "agent" } } }))));
        assert!(!filed_by_the_person(&item(json!({ "labels": [] }))));
    }

    #[test]
    fn a_moment_an_agent_set_wakes_its_thread_once() {
        let parked = json!({ "status": "open", "runAt": 500, "wrote": { "runAt": { "ts": 100, "source": "agent" }, "result": { "ts": 90, "source": "agent" } } });
        assert_eq!(wakes_itself_at(&item(parked.clone())), 500.0);
        let mut hers = parked.clone();
        hers["wrote"]["runAt"]["source"] = json!("founder");
        assert_eq!(wakes_itself_at(&item(hers)), 0.0);
        let mut ran = parked.clone();
        ran["wrote"]["result"]["ts"] = json!(600);
        assert_eq!(wakes_itself_at(&item(ran)), 0.0);
        let mut question = parked;
        question["kind"] = json!("question");
        assert_eq!(wakes_itself_at(&item(question)), 0.0);
    }

    #[test]
    fn a_word_after_the_moment_was_set_runs_now() {
        let waits = json!({ "answer": "Go on", "runAt": 900, "wrote": { "answer": { "ts": 200, "source": "founder" }, "runAt": { "ts": 100, "source": "agent" } } });
        assert!(word_waits(&item(waits.clone())));
        let mut acted_on = waits.clone();
        acted_on["answeredThrough"] = json!(200);
        assert!(!word_waits(&item(acted_on)));
        let mut before = waits;
        before["wrote"]["answer"]["ts"] = json!(50);
        assert!(!word_waits(&item(before)));
    }
}
