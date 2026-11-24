//! What a close asks when the thread still has tasks waiting on the person, or its work never
//! reached main: a port of `renderer/src/close-asks-first.ts`, and of
//! `renderer/src/discard-asks-first.ts`, which borrows the same strip for the question before a
//! change is thrown away. Only the words and the keys; `closing.rs` decides when to ask and
//! does what the answer says (`markDone` and `answerCloseAsk` in `renderer/src/App.tsx`).

use super::answering::{DISCARD_ASK, discard_options};
use super::format::Choice;

/// What an answer does (`CloseAct`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Decide,
    Start,
    Drop,
    Ship,
    Close,
    Discard,
    Keep,
}

/// One answer: the option as the strip draws it, and what it does.
#[derive(Clone, Debug, PartialEq)]
pub struct CloseOption {
    pub choice: Choice,
    pub act: Act,
}

/// The question and its answers (`CloseAsk`).
#[derive(Clone, Debug, PartialEq)]
pub struct CloseAsk {
    pub ask: String,
    pub options: Vec<CloseOption>,
}

/// The last numbered option rather than an esc row: a key drawn beside the numbers read as one
/// more thing to work out.
pub const KEEP_OPEN: &str = "Keep this thread open";

const WORDS: [&str; 10] = ["", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];

fn counted(n: usize) -> String {
    WORDS.get(n).map_or_else(|| n.to_string(), |w| w.to_string())
}

fn option(n: f64, text: impl Into<String>, recommended: bool, act: Act) -> CloseOption {
    CloseOption { choice: Choice { n, text: text.into(), recommended }, act }
}

/// The question and its options for `waiting` tasks not yet started, or nothing when there are
/// none and the close should simply go through (`closeAsk`). Every option says who does what
/// to which.
pub fn close_ask(waiting: usize) -> Option<CloseAsk> {
    if waiting == 0 {
        return None;
    }
    if waiting == 1 {
        return Some(CloseAsk {
            ask: "Before this thread closes, what happens to the task it filed?".to_string(),
            options: vec![
                option(1.0, "Start the task in a new thread, then close this one", true, Act::Start),
                option(2.0, "Close both without starting the task", false, Act::Drop),
                option(3.0, KEEP_OPEN, false, Act::Keep),
            ],
        });
    }
    let n = counted(waiting);
    // "both" for two, "all three" past that: the way it would be said.
    let them = if waiting == 2 { "both".to_string() } else { format!("all {n}") };
    Some(CloseAsk {
        ask: format!("Before this thread closes, what happens to the {n} tasks it filed?"),
        options: vec![
            option(1.0, format!("Start {them} in new threads, then close this one"), false, Act::Start),
            option(2.0, format!("Close this thread and {them} tasks without starting them"), false, Act::Drop),
            // Several proposals are several decisions, made on their own rows right above.
            option(3.0, "Keep this thread open and decide each one", true, Act::Decide),
        ],
    })
}

/// For a thread whose work never reached main (`unshippedAsk`).
pub fn unshipped_ask() -> CloseAsk {
    CloseAsk {
        ask: "This thread's work never reached main.".to_string(),
        options: vec![option(1.0, "Have the agent ship it", true, Act::Ship), option(2.0, "Close it anyway", false, Act::Close), option(3.0, KEEP_OPEN, false, Act::Keep)],
    }
}

/// The question before the change is thrown away (`discardAsk`): keeping it sits on 2, the
/// number the discard was picked with, so the press repeated by habit is the safe one.
pub fn discard_ask() -> CloseAsk {
    let acts = [Act::Discard, Act::Keep];
    CloseAsk { ask: DISCARD_ASK.to_string(), options: discard_options().into_iter().zip(acts).map(|(choice, act)| CloseOption { choice, act }).collect() }
}

/// The keys the question answers with while it is up, by the name the browser gives them.
/// Every other key keeps its usual meaning: the question can be off screen, and a key that
/// silently does nothing gives no hint that it is there (`closeAskOwnsKey`).
pub fn close_ask_owns_key(key: &str) -> bool {
    matches!(key, "Escape" | "Enter" | "ArrowUp" | "ArrowDown") || (key.len() == 1 && key.bytes().all(|b| (b'1'..=b'9').contains(&b)))
}

#[cfg(test)]
mod tests {
    //! The web's cases, from `tests/closing-a-thread-with-tasks-waiting-asks-first`,
    //! `a-thread-whose-work-never-reached-main-is-not-closed-unasked`,
    //! `a-close-question-you-cannot-see-does-not-eat-your-keys` and
    //! `throwing-the-change-away-asks-first` (`.test.mts`).
    use super::*;

    fn texts(a: &CloseAsk) -> Vec<&str> {
        a.options.iter().map(|o| o.choice.text.as_str()).collect()
    }
    fn recommended(a: &CloseAsk) -> Vec<f64> {
        a.options.iter().filter(|o| o.choice.recommended).map(|o| o.choice.n).collect()
    }
    fn acts(a: &CloseAsk) -> Vec<Act> {
        a.options.iter().map(|o| o.act).collect()
    }

    #[test]
    fn with_no_task_waiting_the_close_goes_through() {
        assert_eq!(close_ask(0), None);
    }

    #[test]
    fn one_task_waiting_offers_to_start_it_close_both_or_keep_the_thread() {
        let a = close_ask(1).unwrap();
        assert_eq!(a.ask, "Before this thread closes, what happens to the task it filed?");
        assert_eq!(texts(&a), ["Start the task in a new thread, then close this one", "Close both without starting the task", "Keep this thread open"]);
        assert_eq!(acts(&a), [Act::Start, Act::Drop, Act::Keep]);
        assert_eq!(recommended(&a), [1.0]);
        assert_eq!(a.options.iter().map(|o| o.choice.n).collect::<Vec<_>>(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn several_say_how_many_and_recommend_deciding_each_one() {
        let two = close_ask(2).unwrap();
        assert_eq!(two.ask, "Before this thread closes, what happens to the two tasks it filed?");
        assert_eq!(texts(&two), ["Start both in new threads, then close this one", "Close this thread and both tasks without starting them", "Keep this thread open and decide each one"]);
        assert_eq!(acts(&two), [Act::Start, Act::Drop, Act::Decide]);
        assert_eq!(recommended(&two), [3.0]);
        let three = close_ask(3).unwrap();
        assert_eq!(three.ask, "Before this thread closes, what happens to the three tasks it filed?");
        assert!(texts(&three).contains(&"Start all three in new threads, then close this one"));
        assert!(texts(&three).contains(&"Close this thread and all three tasks without starting them"));
        let twelve = close_ask(12).unwrap();
        assert_eq!(twelve.ask, "Before this thread closes, what happens to the 12 tasks it filed?");
        assert!(texts(&twelve).contains(&"Start all 12 in new threads, then close this one"));
        for n in [1, 2, 5] {
            let said = texts(&close_ask(n).unwrap()).join(" ").to_lowercase();
            assert!(!said.contains("approve") && !said.contains("drop"), "{said}");
        }
    }

    #[test]
    fn work_that_never_reached_main_recommends_shipping_it() {
        let a = unshipped_ask();
        assert_eq!(a.ask, "This thread's work never reached main.");
        assert_eq!(texts(&a), ["Have the agent ship it", "Close it anyway", "Keep this thread open"]);
        assert_eq!(acts(&a), [Act::Ship, Act::Close, Act::Keep]);
        assert_eq!(recommended(&a), [1.0]);
    }

    #[test]
    fn the_discard_question_keeps_the_change_on_two() {
        let a = discard_ask();
        assert_eq!(a.ask, "Discard the change?");
        assert_eq!(texts(&a), ["Discard it", "Keep it"]);
        assert_eq!(recommended(&a), [2.0]);
        assert_eq!(acts(&a), [Act::Discard, Act::Keep]);
    }

    #[test]
    fn it_takes_only_the_keys_that_answer_it() {
        for k in ["1", "2", "3", "9", "ArrowUp", "ArrowDown", "Enter", "Escape"] {
            assert!(close_ask_owns_key(k), "{k}");
        }
        for k in ["e", "E", "n", "N", "r", "R", "j", "k", "l", "z", "Z", "s", "0", "ArrowLeft", "ArrowRight"] {
            assert!(!close_ask_owns_key(k), "{k}");
        }
    }
}
