//! What this window writes, said the way the web window says it: a reply, a pick and a close
//! are each one call to `zero:answer` (`api.answer` in `renderer/src/api.ts`, answered in
//! `main/ipc.mts`), reopening is one to `zero:reopen`, and the words are the web window's own.

use serde_json::{Value, json};

use crate::data::Item;
use crate::rules::format::{self, Choice};

/// The channel all of these go to.
pub const ANSWER: &str = "zero:answer";

/// A reply opens a thread that is blocked or finished; one a worker is on is left as it is,
/// and the worker reads the reply when it stops (`statusForReply` in `renderer/src/list-rules.ts`).
pub fn status_for_reply(status: &str) -> Option<&'static str> {
    matches!(status, "blocked" | "done").then_some("open")
}

/// Words typed in the reply box.
pub fn reply(item: &Item, words: &str) -> Value {
    let mut payload = json!({ "product": item.product, "id": item.id, "answer": words });
    if let Some(status) = status_for_reply(&item.status) {
        payload["status"] = json!(status);
    }
    payload
}

/// The words a picked option is sent as: its number, then the option as the agent wrote it
/// (`pickOption` in `renderer/src/App.tsx`).
pub fn option_words(choice: &Choice) -> String {
    format!("Option {}: {}", choice.n, choice.text)
}

/// Closing a thread, which E does. `unshipped` says the person knows its work never reached
/// main and closes it anyway; without it the engine refuses such a close.
pub fn close(item: &Item, unshipped: bool) -> Value {
    let mut payload = json!({ "product": item.product, "id": item.id, "status": "done" });
    if unshipped {
        payload["closeUnshipped"] = json!(true);
    }
    payload
}

/// The channel Reopen goes to (`sendBack` in `renderer/src/App.tsx`): the engine opens the
/// thread and hands its agent whatever comments were still waiting.
pub const REOPEN: &str = "zero:reopen";

/// Opening a finished thread again with nothing said.
pub fn reopen(item: &Item) -> Value {
    json!({ "product": item.product, "id": item.id })
}

/// What picking an option does. Most are a reply; a few are something else.
#[derive(Debug, PartialEq)]
pub enum Pick {
    /// Sent to the agent as it stands.
    Reply(String),
    /// "Close this task": done the way E does it, with no run.
    Close,
    /// "Keep it on the branch": a close that leaves the work off main on purpose.
    CloseUnshipped,
    /// "Discard it": the web window asks first, since nothing takes it back.
    AsksFirst,
    /// "Change something first": opens the reply box with these words in it, for the person
    /// to say what.
    NeedsWords(String),
}

/// `pickOption` in `renderer/src/App.tsx`, in its order.
pub fn pick(choice: &Choice) -> Pick {
    if format::closes_the_task(&choice.text) {
        Pick::Close
    } else if format::leaves_it_on_the_branch(&choice.text) {
        Pick::CloseUnshipped
    } else if format::discards_the_change(&choice.text) {
        Pick::AsksFirst
    } else if format::needs_your_words(&choice.text) {
        Pick::NeedsWords(format!("{}\n\n", format::without_recommended(&choice.text)))
    } else {
        Pick::Reply(option_words(choice))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thread(status: &str) -> Item {
        serde_json::from_value(json!({ "id": "w-1", "product": "shop", "status": status })).unwrap()
    }

    fn choice(n: f64, text: &str) -> Choice {
        Choice { n, text: text.to_string(), recommended: text.contains("(recommended)") }
    }

    #[test]
    fn a_reply_opens_a_blocked_or_finished_thread_and_leaves_a_claimed_one_alone() {
        assert_eq!(reply(&thread("blocked"), "Go on."), json!({ "product": "shop", "id": "w-1", "answer": "Go on.", "status": "open" }));
        assert_eq!(reply(&thread("done"), "One more thing."), json!({ "product": "shop", "id": "w-1", "answer": "One more thing.", "status": "open" }));
        assert_eq!(reply(&thread("claimed"), "Wait."), json!({ "product": "shop", "id": "w-1", "answer": "Wait." }));
        assert_eq!(reply(&thread("open"), "Wait."), json!({ "product": "shop", "id": "w-1", "answer": "Wait." }));
    }

    #[test]
    fn a_picked_option_is_sent_by_its_number_in_the_agents_own_words() {
        assert_eq!(option_words(&choice(1.0, "Ship the fix now (recommended)")), "Option 1: Ship the fix now (recommended)");
        assert_eq!(pick(&choice(2.0, "Ship it and retry the orders")), Pick::Reply("Option 2: Ship it and retry the orders".to_string()));
    }

    #[test]
    fn some_options_are_not_replies() {
        assert_eq!(pick(&choice(1.0, "Close this task (recommended)")), Pick::Close);
        assert_eq!(pick(&choice(2.0, "Keep it on the branch")), Pick::CloseUnshipped);
        assert_eq!(pick(&choice(3.0, "Discard it")), Pick::AsksFirst);
        assert_eq!(pick(&choice(2.0, "Change something first")), Pick::NeedsWords("Change something first\n\n".to_string()));
    }

    #[test]
    fn a_close_says_so_when_the_work_is_left_off_main_on_purpose() {
        assert_eq!(close(&thread("blocked"), false), json!({ "product": "shop", "id": "w-1", "status": "done" }));
        assert_eq!(close(&thread("blocked"), true), json!({ "product": "shop", "id": "w-1", "status": "done", "closeUnshipped": true }));
        assert_eq!(reopen(&thread("done")), json!({ "product": "shop", "id": "w-1" }));
    }
}
