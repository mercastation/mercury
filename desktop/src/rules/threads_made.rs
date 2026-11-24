//! The threads a thread made, and the words for where each one stands: a port of
//! `renderer/src/threads-made.ts`, with the words of `renderer/src/components/ThreadsMade.tsx`.
//! `made.rs` draws them; `filed_in_thread.rs` says which turn of the conversation each hangs off.

use std::collections::HashMap;

use serde_json::{Value, json};

use super::js;
use super::writes::{ANSWER, Write, status_for_reply};
use super::{answers, kinds};
use crate::data::Item;

/// What the agent is told when its proposal is approved as it stands (`RUN_IT`).
pub use super::palette::RUN_IT;

/// The tab a thread sits in, as the tabs list it (`ThreadStateWord`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Waiting,
    Running,
    Scheduled,
    Done,
}

/// The list's own tab names, so a row says what the tab it sits in says (`stateWord`).
pub fn state_word(state: Option<State>) -> &'static str {
    match state {
        Some(State::Waiting) => "Needs you",
        Some(State::Running) => "In progress",
        Some(State::Scheduled) => "Later",
        Some(State::Done) => "Done",
        None => "",
    }
}

/// Every thread filed straight under this one, in its own project, oldest first. Finished ones
/// stay: "done" is part of the answer to what it made (`threadsMade`).
pub fn threads_made<'a>(items: &'a [Item], thread: &Item) -> Vec<&'a Item> {
    let mut made: Vec<&Item> = items.iter().filter(|i| i.text("parent") == Some(thread.id.as_str()) && i.product == thread.product).collect();
    // A stable sort, as the browser's is.
    made.sort_by(|a, b| a.created_at.unwrap_or(0.0).total_cmp(&b.created_at.unwrap_or(0.0)));
    made
}

/// The rows a thread's page lists, what waits on a press first. A thread that serves a demand
/// lists only those (`madeShown`).
pub fn made_shown<T: Clone>(rows: &[T], can_press: impl Fn(&T) -> bool, in_demand: bool) -> Vec<T> {
    let asks: Vec<T> = rows.iter().filter(|r| can_press(r)).cloned().collect();
    if in_demand {
        return asks;
    }
    asks.into_iter().chain(rows.iter().filter(|r| !can_press(r)).cloned()).collect()
}

/// Which of the threads an agent filed waits on a press from the person (`approvableFiled`): in
/// Needs you or Later, open, not a question or a review, not the person's own, begun, and with
/// no answer that stands. The tab it is in has the last word.
pub fn approvable_filed(i: &Item, state: Option<State>) -> bool {
    if !matches!(state, Some(State::Waiting | State::Scheduled)) {
        return false;
    }
    if i.status != "open" || kinds::asks_the_person(i) || answers::filed_by_the_person(i) || kinds::not_started(i) {
        return false;
    }
    i.live_answer().is_none()
}

/// How many tasks each thread has waiting for a yes, by `product/id` of the thread
/// (`waitingCounts`). Ported with the web's cases; the Inbox's rows count them as the lists are
/// built, where each thread's tab is known (`to_approve` in `rules.rs`).
#[allow(dead_code)]
pub fn waiting_counts(items: &[Item], state_of: impl Fn(&Item) -> Option<State>) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for i in items {
        let Some(parent) = i.text("parent") else { continue };
        if approvable_filed(i, state_of(i)) {
            *out.entry(format!("{}/{parent}", i.product)).or_insert(0) += 1;
        }
    }
    out
}

/// "2 tasks to approve", or nothing (`toApproveWords`).
pub fn to_approve_words(n: usize) -> String {
    if n == 0 { String::new() } else { format!("{n} task{} to approve", if n == 1 { "" } else { "s" }) }
}

/* ------------------------------ the list's own words (ThreadsMade.tsx) ------------------------------ */

pub const START_NEW: &str = "New thread";
pub const DO_HERE: &str = "Do here";
pub const DISMISS: &str = "Dismiss";
pub const START_ALL: &str = "Start all";
pub const DISMISS_ALL: &str = "Dismiss all";
pub const FILED_HERE: &str = "Filed from this thread";

/// "2 proposals waiting on you" (`waitingWords`).
pub fn waiting_words(n: usize) -> String {
    format!("{n} proposal{} waiting on you", if n == 1 { "" } else { "s" })
}

/* ------------------------------ do it here ------------------------------ */

/// How much of a proposal's body goes into the message that hands it back (`DO_IT_HERE_BODY`).
pub const DO_IT_HERE_BODY: usize = 1500;

/// The message that hands a proposal back to the thread that filed it, as the person's own
/// (`doItHereWords`): its title, and its body cut past `DO_IT_HERE_BODY` with a pointer to the
/// rest.
pub fn do_it_here_words(child: &Item) -> String {
    let title = js::trim(child.title.as_deref().unwrap_or("")).to_string();
    let mut body = js::trim(child.body.as_deref().unwrap_or("")).to_string();
    if let Some(rest) = body.strip_prefix(title.as_str()) {
        body = js::trim(rest).to_string();
    }
    let head = format!("Do this here, in this thread, not as a thread of its own: “{title}”");
    if body.is_empty() {
        return head;
    }
    if js::len(&body) > DO_IT_HERE_BODY {
        body = format!("{}…\n\n(The rest is on {}, now closed.)", js::trim_end(js::slice(&body, 0, DO_IT_HERE_BODY)), child.id);
    }
    format!("{head}\n\n{body}")
}

/// The two writes, parent first: if the message does not get through, the proposal stays open
/// rather than vanishing with nobody doing the work. Nothing when the parent given is not the
/// thread that filed this one (`doItHereWrites`).
pub fn do_it_here_writes(child: &Item, parent: Option<&Item>) -> Option<Vec<Write>> {
    let parent = parent?;
    if child.text("parent") != Some(parent.id.as_str()) || child.product != parent.product {
        return None;
    }
    let mut to_parent = json!({ "product": parent.product, "id": parent.id, "answer": do_it_here_words(child) });
    if let Some(status) = status_for_reply(&parent.status) {
        to_parent["status"] = json!(status);
    }
    Some(vec![(ANSWER, to_parent), (ANSWER, json!({ "product": child.product, "id": child.id, "status": "done" }))])
}

/* ------------------------------ start all, dismiss all ------------------------------ */

/// What Start all and Dismiss all do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum All {
    Start,
    Dismiss,
}

/// The writes that answer every proposal in one list, and the writes one Z makes to take them
/// all back (`answerAllWrites`). Stopping whatever already started is the caller's.
pub fn answer_all_writes(kids: &[&Item], act: All) -> (Vec<Write>, Vec<Write>) {
    let at = |k: &Item, more: Value| {
        let mut w = json!({ "product": k.product, "id": k.id });
        w.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap_or_default());
        (ANSWER, w)
    };
    match act {
        All::Start => (kids.iter().map(|k| at(k, json!({ "answer": RUN_IT }))).collect(), kids.iter().map(|k| at(k, json!({ "answer": "(withdrawn)", "status": "open" }))).collect()),
        All::Dismiss => (kids.iter().map(|k| at(k, json!({ "status": "done" }))).collect(), kids.iter().map(|k| at(k, json!({ "status": "open" }))).collect()),
    }
}

#[cfg(test)]
mod tests {
    //! The web's cases, from `tests/a-thread-shows-the-threads-it-made`,
    //! `the-threads-a-thread-filed-carry-an-approve-button`,
    //! `a-thread-in-a-demand-lists-only-what-waits-on-you`,
    //! `the-inbox-says-how-many-tasks-wait-to-be-approved`,
    //! `a-proposed-thread-can-be-done-in-the-thread-that-raised-it`,
    //! `one-press-starts-every-proposal-in-the-list` and
    //! `a-proposal-is-a-dot-with-its-answers-always-beside-it` (`.test.mts`).
    use super::*;

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }
    fn ids(rows: &[&Item]) -> Vec<String> {
        rows.iter().map(|i| i.id.clone()).collect()
    }

    #[test]
    fn a_thread_lists_what_it_filed_in_its_own_project_oldest_first() {
        let t = 1_000_000.0;
        let row = |id: &str, more: Value| {
            let mut v = json!({ "id": id, "product": "team", "title": id, "status": "open", "createdAt": t, "updatedAt": t });
            v.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap());
            item(v)
        };
        let items = vec![
            row("w-parent", json!({})),
            row("w-second", json!({ "createdAt": t + 2000.0, "parent": "w-parent" })),
            row("w-first", json!({ "createdAt": t + 1000.0, "parent": "w-parent" })),
            row("w-finished", json!({ "createdAt": t + 3000.0, "status": "done", "parent": "w-parent" })),
            row("w-grandchild", json!({ "createdAt": t + 4000.0, "parent": "w-first" })),
            row("w-elsewhere", json!({ "product": "other", "createdAt": t + 5000.0, "parent": "w-parent" })),
            row("w-unrelated", json!({ "createdAt": t + 6000.0 })),
        ];
        assert_eq!(ids(&threads_made(&items, &items[0])), ["w-first", "w-second", "w-finished"]);
        assert!(threads_made(&items, &row("w-unrelated", json!({}))).is_empty());
    }

    #[test]
    fn a_row_says_the_tab_it_sits_in() {
        assert_eq!([State::Waiting, State::Running, State::Scheduled, State::Done].map(|s| state_word(Some(s))), ["Needs you", "In progress", "Later", "Done"]);
        assert_eq!(state_word(None), "");
    }

    #[test]
    fn an_open_task_an_agent_filed_waits_on_a_press_while_its_tab_says_so() {
        let filed = |more: Value| {
            let mut v = json!({ "id": "w-kid", "product": "p", "status": "open", "kind": "task", "labels": [] });
            v.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap());
            item(v)
        };
        assert!(approvable_filed(&filed(json!({})), Some(State::Waiting)));
        assert!(!approvable_filed(&filed(json!({ "answer": "Approved. Run it." })), Some(State::Waiting)));
        assert!(approvable_filed(&filed(json!({ "answer": "(withdrawn)" })), Some(State::Waiting)));
        for status in ["claimed", "done", "blocked"] {
            assert!(!approvable_filed(&filed(json!({ "status": status })), Some(State::Waiting)));
        }
        for kind in ["question", "review"] {
            assert!(!approvable_filed(&filed(json!({ "kind": kind })), Some(State::Waiting)));
        }
        assert!(!approvable_filed(&filed(json!({ "labels": ["founder"] })), Some(State::Waiting)));
        assert!(!approvable_filed(&filed(json!({ "start": "later" })), Some(State::Scheduled)));
        for state in [Some(State::Running), Some(State::Done), None] {
            assert!(!approvable_filed(&filed(json!({})), state));
        }
    }

    #[test]
    fn what_waits_is_listed_first_and_alone_in_a_demand() {
        let rows = [("w-running", false), ("w-ask-1", true), ("w-done", false), ("w-ask-2", true)];
        let press = |r: &(&str, bool)| r.1;
        fn names(v: Vec<(&'static str, bool)>) -> Vec<&'static str> {
            v.into_iter().map(|r| r.0).collect()
        }
        assert_eq!(names(made_shown(&rows, press, true)), ["w-ask-1", "w-ask-2"]);
        assert!(made_shown(&[("w-running", false), ("w-done", false)], press, true).is_empty());
        assert_eq!(names(made_shown(&rows, press, false)), ["w-ask-1", "w-ask-2", "w-running", "w-done"]);
        assert_eq!(names(made_shown(&[("w-running", false), ("w-done", false)], press, false)), ["w-running", "w-done"]);
    }

    #[test]
    fn the_inbox_counts_the_tasks_waiting_under_each_thread() {
        assert_eq!([0, 1, 2].map(to_approve_words), ["", "1 task to approve", "2 tasks to approve"]);
        let kid = |id: &str, parent: &str, more: Value| {
            let mut v = json!({ "id": id, "parent": parent, "product": "p", "status": "open", "kind": "task", "labels": [], "createdAt": 1 });
            v.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap());
            item(v)
        };
        let items = vec![
            item(json!({ "id": "w-a", "product": "p", "status": "open", "kind": "directive", "labels": ["founder"] })),
            kid("w-1", "w-a", json!({})),
            kid("w-2", "w-a", json!({})),
            kid("w-3", "w-a", json!({ "answer": "Approved. Run it." })),
            kid("w-4", "w-a", json!({ "kind": "review" })),
            kid("w-5", "w-b", json!({})),
            kid("w-6", "w-a", json!({ "product": "q" })),
        ];
        let counts = waiting_counts(&items, |_| Some(State::Scheduled));
        assert_eq!((counts.get("p/w-a"), counts.get("q/w-a"), counts.get("p/w-b")), (Some(&2), Some(&1), Some(&1)));
        assert!(waiting_counts(&items, |_| Some(State::Running)).is_empty());
    }

    fn kid(more: Value) -> Item {
        let mut v = json!({ "id": "w-kid", "product": "p", "parent": "w-mum", "status": "open", "kind": "task", "labels": [], "title": "Fix the meter on the usage page", "body": "The meter reads 0% after a reset. Read it from the limit, not the cache." });
        v.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap());
        item(v)
    }
    fn mum(more: Value) -> Item {
        let mut v = json!({ "id": "w-mum", "product": "p", "status": "open" });
        v.as_object_mut().unwrap().extend(more.as_object().cloned().unwrap());
        item(v)
    }

    #[test]
    fn doing_it_here_names_the_proposal_and_carries_what_it_said() {
        let head = "Do this here, in this thread, not as a thread of its own: “Fix the meter on the usage page”";
        assert_eq!(do_it_here_words(&kid(json!({}))), format!("{head}\n\nThe meter reads 0% after a reset. Read it from the limit, not the cache."));
        assert_eq!(do_it_here_words(&kid(json!({ "body": null }))), head);
        assert_eq!(do_it_here_words(&kid(json!({ "body": "Fix the meter on the usage page\n\nIt reads 0%." }))), format!("{head}\n\nIt reads 0%."));
        let whole = "a".repeat(1500);
        assert_eq!(do_it_here_words(&kid(json!({ "body": whole }))), format!("{head}\n\n{whole}"));
        let long = "a".repeat(1501);
        assert_eq!(do_it_here_words(&kid(json!({ "body": long }))), format!("{head}\n\n{whole}…\n\n(The rest is on w-kid, now closed.)"));
    }

    #[test]
    fn doing_it_here_writes_the_parent_first_and_wakes_it_as_a_reply_does() {
        let w = do_it_here_writes(&kid(json!({})), Some(&mum(json!({})))).unwrap();
        assert_eq!(w, vec![(ANSWER, json!({ "product": "p", "id": "w-mum", "answer": do_it_here_words(&kid(json!({}))) })), (ANSWER, json!({ "product": "p", "id": "w-kid", "status": "done" }))]);
        for status in ["done", "blocked"] {
            assert_eq!(do_it_here_writes(&kid(json!({})), Some(&mum(json!({ "status": status })))).unwrap()[0].1["status"], json!("open"));
        }
        for status in ["claimed", "open"] {
            assert!(do_it_here_writes(&kid(json!({})), Some(&mum(json!({ "status": status })))).unwrap()[0].1.get("status").is_none());
        }
        assert_eq!(do_it_here_writes(&kid(json!({})), Some(&mum(json!({ "id": "w-other" })))), None);
        assert_eq!(do_it_here_writes(&kid(json!({ "parent": null })), Some(&mum(json!({})))), None);
        assert_eq!(do_it_here_writes(&kid(json!({})), Some(&mum(json!({ "product": "q" })))), None);
        assert_eq!(do_it_here_writes(&kid(json!({})), None), None);
    }

    #[test]
    fn start_all_and_dismiss_all_write_once_per_proposal_and_take_it_all_back() {
        assert_eq!(RUN_IT, "Approved. Run it.");
        assert_eq!((START_ALL, DISMISS_ALL), ("Start all", "Dismiss all"));
        let (a, b) = (item(json!({ "product": "p", "id": "w-1" })), item(json!({ "product": "p", "id": "w-2" })));
        let (writes, undo) = answer_all_writes(&[&a, &b], All::Start);
        assert_eq!(writes, vec![(ANSWER, json!({ "product": "p", "id": "w-1", "answer": "Approved. Run it." })), (ANSWER, json!({ "product": "p", "id": "w-2", "answer": "Approved. Run it." }))]);
        assert_eq!(undo, vec![(ANSWER, json!({ "product": "p", "id": "w-1", "answer": "(withdrawn)", "status": "open" })), (ANSWER, json!({ "product": "p", "id": "w-2", "answer": "(withdrawn)", "status": "open" }))]);
        let (writes, undo) = answer_all_writes(&[&a, &b], All::Dismiss);
        assert_eq!(writes, vec![(ANSWER, json!({ "product": "p", "id": "w-1", "status": "done" })), (ANSWER, json!({ "product": "p", "id": "w-2", "status": "done" }))]);
        assert_eq!(undo, vec![(ANSWER, json!({ "product": "p", "id": "w-1", "status": "open" })), (ANSWER, json!({ "product": "p", "id": "w-2", "status": "open" }))]);
        assert_eq!(answer_all_writes(&[], All::Start), (vec![], vec![]));
    }

    #[test]
    fn the_count_line_counts_what_the_dots_count() {
        assert_eq!((waiting_words(2).as_str(), waiting_words(1).as_str()), ("2 proposals waiting on you", "1 proposal waiting on you"));
    }
}
