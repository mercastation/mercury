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

/// How long something done waits before it is written, so Z can take it back whole
/// (`UNDO_GRACE_MS` in `renderer/src/App.tsx`), and how long the line that says so stays.
pub const UNDO_GRACE_MS: f64 = 3000.0;
pub const TOAST_MS: f64 = 2500.0;

/// What Z says when there is nothing of this window's to take back (`renderer/src/undo-window.ts`).
pub const NOTHING_TO_UNDO: &str = "Nothing to undo yet.";

/// A thread's name as a line about it says it: whole up to `budget` letters, else cut at the
/// end of a sentence or at a word (`clipToSentence` in `renderer/src/list-rules.ts`). JavaScript
/// counts UTF-16 units; letters are counted here, which differs only past the first plane.
pub fn clip_to_sentence(text: &str, budget: usize) -> String {
    let letters: Vec<char> = text.chars().collect();
    if letters.len() <= budget {
        return text.to_string();
    }
    // One letter past the budget, so a stop or a space right at the edge still counts.
    let window = &letters[..=budget];
    let last = |stop: [char; 2]| window.windows(2).rposition(|pair| pair == stop);
    let end = [['.', ' '], ['?', ' '], ['!', ' ']].into_iter().filter_map(last).max();
    if let Some(end) = end.filter(|end| *end > 40) {
        return letters[..=end].iter().collect();
    }
    let space = window.iter().rposition(|c| *c == ' ').filter(|space| *space > 40).unwrap_or(budget);
    let kept: String = letters[..space].iter().collect();
    format!("{}…", kept.trim_end())
}

/// What the line says while a close waits: "Done: " and the thread's name (`markDone`).
pub fn said_done(title: &str) -> String {
    format!("Done: {}", clip_to_sentence(title, 48))
}

/// What the line says while a reply or a pick waits (`answerWith`).
pub fn said_sent(project: &str) -> String {
    format!("Sent → {project}")
}

/// What Z says after taking back something still waiting (`graceUndoneLine`).
pub fn said_undone(said: &str, words_are_back: bool) -> String {
    if words_are_back {
        return "Undone. Your message is back in the reply box.".to_string();
    }
    match said.strip_prefix("Done: ") {
        Some(title) => format!("Undone. Still open: {title}"),
        None => "Undone. Nothing changed.".to_string(),
    }
}

/// Opening a thread again as it was before this window closed it: blocked if it was blocked,
/// so undoing a close starts no run (`wasBeforeClose` in `shared/reopen-as-it-was.mts`).
pub fn reopen_as_it_was(item: &Item) -> Value {
    let mine = item.wrote("status").is_some_and(|w| w.source == "founder");
    json!({ "product": item.product, "id": item.id, "status": "open", "reopenAs": { "status": item.status, "mine": mine } })
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
    fn a_line_names_a_thread_whole_or_cut_at_a_sentence_or_a_word() {
        assert_eq!(clip_to_sentence("Snooze is lost on restart", 48), "Snooze is lost on restart");
        // Cut at the last space inside the budget, with the dots.
        assert_eq!(said_done("Checkout fails for some customers since this morning"), "Done: Checkout fails for some customers since this…");
        // A sentence that ends past the fortieth letter is kept whole, with its stop.
        assert_eq!(clip_to_sentence("The weekly usage total is off by one again. Second sentence here.", 48), "The weekly usage total is off by one again.");
    }

    #[test]
    fn taking_something_back_says_what_is_as_it_was() {
        assert_eq!(said_undone("Done: Snooze is lost on restart", false), "Undone. Still open: Snooze is lost on restart");
        assert_eq!(said_undone("Sent → Storefront", true), "Undone. Your message is back in the reply box.");
        assert_eq!(said_undone("Sent → Storefront", false), "Undone. Nothing changed.");
    }

    #[test]
    fn undoing_a_close_puts_the_thread_back_as_it_was_and_starts_no_run() {
        let waiting: Item = serde_json::from_value(json!({ "id": "w-1", "product": "shop", "status": "blocked", "wrote": { "status": { "ts": 1, "source": "agent" } } })).unwrap();
        assert_eq!(reopen_as_it_was(&waiting), json!({ "product": "shop", "id": "w-1", "status": "open", "reopenAs": { "status": "blocked", "mine": false } }));
    }

    #[test]
    fn a_close_says_so_when_the_work_is_left_off_main_on_purpose() {
        assert_eq!(close(&thread("blocked"), false), json!({ "product": "shop", "id": "w-1", "status": "done" }));
        assert_eq!(close(&thread("blocked"), true), json!({ "product": "shop", "id": "w-1", "status": "done", "closeUnshipped": true }));
        assert_eq!(reopen(&thread("done")), json!({ "product": "shop", "id": "w-1" }));
    }
}
