//! What kind of row a thread is, told by its labels and a field or two. Each of these is one
//! small rule from a file of its own in `shared/`, named beside it.

use serde_json::Value;

use super::js::{self, re};
use crate::data::Item;

/// A thread put in Later and not begun (`notStarted` in `renderer/src/list-rules.ts`).
pub fn not_started(i: &Item) -> bool {
    i.string("start") == Some("later")
}

/// A question or a review: the two kinds an agent addresses to the person.
pub fn asks_the_person(i: &Item) -> bool {
    matches!(i.kind.as_deref(), Some("question" | "review"))
}

/// A task given to a person (`isTaskRow` in `shared/team-rules.mts`).
pub fn is_task_row(i: &Item) -> bool {
    i.has_label("given-task")
}

/// The repeating task a run belongs to (`ruleIdOf` in `shared/repeats.mts`).
pub fn rule_id_of(i: &Item) -> Option<&str> {
    i.labels.iter().find_map(|l| l.strip_prefix("repeat:")).filter(|id| !id.is_empty())
}

/// A run its worker explicitly marked clean (`isCleanRun` in `shared/repeats.mts`). It exists
/// to hide, never to show, so everything else is news: a run that died saying nothing too.
pub fn is_clean_run(i: &Item) -> bool {
    // A thread sent to a teammate is the person's ask, never its quiet run.
    if i.truthy("routed") {
        return false;
    }
    let a_run = rule_id_of(i).is_some() || i.labels.iter().any(|l| l.starts_with("teammate:") || l == "teammate-offer" || l == "teammate-asks-to-write");
    a_run && i.has_label("clean")
}

const PROTOCOL_OPENS: &str = "THIS IS ONE RUN OF A REPEATING TASK.";
const INSTRUCTION_MARK: &str = "\nHER INSTRUCTION FOLLOWS.\n";

/// A run's body is the app's rules for the agent, then the person's instruction. Screens show
/// the instruction (`instructionOf` in `shared/repeats.mts`).
pub fn instruction_of(body: &str) -> &str {
    match body.find(INSTRUCTION_MARK) {
        Some(at) if body.starts_with(PROTOCOL_OPENS) => js::trim(&body[at + INSTRUCTION_MARK.len()..]),
        _ => body,
    }
}

/// A cause row the app filed (`isCause` in `shared/causes.mts`). An agent writing the label on
/// its own row does not make one.
pub fn is_cause(i: &Item) -> bool {
    i.kind.as_deref() == Some("cause") && i.has_label("cause") && i.wrote("kind").is_none_or(|w| w.source != "agent")
}

/// The threads still waiting on this cause, from every project (`waitingOn` in `shared/causes.mts`).
pub fn waiting_on_cause<'a>(cause: &Item, items: &'a [Item]) -> Vec<&'a Item> {
    items.iter().filter(|i| i.id != cause.id && i.status == "blocked" && i.list("blockedBy").iter().any(|id| id.as_str() == Some(cause.id.as_str()))).collect()
}

/// The open cause a thread waits behind, or none (`causeOf` in `shared/causes.mts`).
pub fn cause_of<'a>(i: &Item, items: &'a [Item]) -> Option<&'a Item> {
    let waits = i.list("blockedBy");
    if waits.is_empty() || i.status != "blocked" {
        return None;
    }
    items.iter().find(|c| is_cause(c) && c.status != "done" && waits.iter().any(|id| id.as_str() == Some(c.id.as_str())))
}

/// What a thread behind a cause says on Later, in the Status and Back columns
/// (`CAUSE_WAIT_WORD` and `CAUSE_BACK_WORDS` in `shared/causes.mts`).
pub const CAUSE_WAIT_WORD: &str = "Waiting";
pub const CAUSE_BACK_WORDS: &str = "When fixed";

/// Where the cause is, for the Project column: the project most of its threads are in, and how
/// many others (`causePlace` in `shared/causes.mts`).
pub fn cause_place(cause: &Item, waiting: &[&Item], name_of: impl Fn(&str) -> String) -> (String, usize) {
    // Insertion order breaks ties: the cause's own project, then the first thread's.
    let mut counts: Vec<(&str, usize)> = Vec::new();
    if !cause.product.is_empty() {
        counts.push((&cause.product, 0));
    }
    for i in waiting.iter().filter(|i| !i.product.is_empty()) {
        match counts.iter_mut().find(|(slug, _)| *slug == i.product) {
            Some((_, n)) => *n += 1,
            None => counts.push((&i.product, 1)),
        }
    }
    let places: Vec<(&str, usize)> = counts.into_iter().filter(|(_, n)| *n > 0).collect();
    let Some(first) = places.iter().copied().reduce(|best, p| if p.1 > best.1 { p } else { best }) else {
        return (if cause.product.is_empty() { String::new() } else { name_of(&cause.product) }, 0);
    };
    (name_of(first.0), places.len() - 1)
}

/// "6 waiting", for the Status column (`causeWord` in `shared/causes.mts`).
pub fn cause_word(n: usize) -> String {
    format!("{n} waiting")
}

/// The sign-in this thread still waits on (`openSignIn` in `shared/sign-in-ask.mts`).
pub fn open_sign_in(i: &Item) -> bool {
    let Some(ask) = i.object("signIn") else { return false };
    let Some(id) = ask.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) else { return false };
    if crate::data::truthy(ask.get("done")) || i.status == "done" {
        return false;
    }
    i.object("signInAnswer").and_then(|a| a.get("id")).and_then(Value::as_str) != Some(id)
}

/// A conversation brought in from Codex, there to be read (`isCodexRow` in
/// `renderer/src/import-row.ts`): the label, and the label that says which conversation.
fn is_codex_row(i: &Item) -> bool {
    i.has_label("codex") && i.labels.iter().find_map(|l| l.strip_prefix("codex:")).is_some_and(|id| !id.is_empty())
}

/// Imported and not opened yet: it sits on top of the inbox until it is (`justImported`).
pub fn just_imported(i: &Item, seen: &[String]) -> bool {
    is_codex_row(i) && i.status != "done" && !seen.contains(&i.id)
}

/// A thread just brought in, owed one run that writes it up (`catchUpDue` in `shared/agent-import.mts`).
pub fn catch_up_due(i: &Item) -> bool {
    i.has_label("catch-up") && i.status == "open"
}

/// Sent to Auto and not given to anyone yet (`isAuto` in `shared/auto-route.mts`).
pub fn is_auto(i: &Item) -> bool {
    i.has_label("auto")
}

/// Auto has asked the person which teammate should take it (`asksWho` in `shared/auto-route.mts`).
pub fn asks_who(i: &Item) -> bool {
    is_auto(i) && i.object("routed").and_then(|r| r.get("ask")).and_then(Value::as_array).is_some_and(|ask| !ask.is_empty())
}

/// The chat a task was asked in, as `product/id` (`askedIn` in `renderer/src/team/agent-mentions.ts`).
pub fn asked_in(i: &Item) -> Option<(&str, &str)> {
    let place = i.labels.iter().find_map(|l| l.strip_prefix("asked-in:")).unwrap_or("");
    let at = place.rfind('/')?;
    (at > 0 && at < place.len() - 1).then(|| (&place[..at], &place[at + 1..]))
}

// The close has to open a clause or follow and/then/please/can you, so "don't close this task"
// and "why did you close this thread" are not asks. Words may follow it, but not "or", which
// makes it a choice.
re!(ASKS_TO_CLOSE, r"(?:^|\b(?:and|then|please|just|can you|could you|you can)\s+)close (?:this|the) (?:task|thread)(?!\s+or\b)(?:\s|$)", "i");
re!(OPTION_NUMBER, r"^option \d+:\s*", "i");
re!(RECOMMENDED_WORDS, r"\s*\(recommended\)\s*", "i");
re!(CLAUSE_ENDS, r"[,;.!?\n]+", "g");

/// Whether a person's reply asks for its thread to be closed (`shared/asks-to-close.mts`).
pub fn asks_to_close(answer: Option<&str>) -> bool {
    let Some(answer) = answer.filter(|a| !a.is_empty()) else { return false };
    let words = RECOMMENDED_WORDS.strip(&OPTION_NUMBER.strip(answer));
    CLAUSE_ENDS.split(&words).iter().any(|clause| ASKS_TO_CLOSE.test(js::trim(clause)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn a_reply_asks_for_the_close_only_where_the_close_opens_a_clause() {
        for yes in ["Close this task", "Looks right, close this task.", "Ship it and then close the thread", "Option 2: close this task (recommended)", "please close this thread when the merge lands"] {
            assert!(asks_to_close(Some(yes)), "{yes}");
        }
        for no in ["Don't close this task", "Why did you close this thread?", "Close this task or keep going", "", "Carry on"] {
            assert!(!asks_to_close(Some(no)), "{no}");
        }
        assert!(!asks_to_close(None));
    }

    #[test]
    fn only_a_run_marked_clean_is_hidden() {
        assert!(is_clean_run(&item(json!({ "labels": ["repeat:r-1", "clean"] }))));
        assert!(is_clean_run(&item(json!({ "labels": ["teammate:t-1", "clean"] }))));
        assert!(!is_clean_run(&item(json!({ "labels": ["repeat:r-1"] }))));
        assert!(!is_clean_run(&item(json!({ "labels": ["clean"] }))));
        assert!(!is_clean_run(&item(json!({ "labels": ["teammate:t-1", "clean"], "routed": { "to": "t-1" } }))));
    }

    #[test]
    fn a_runs_body_shows_the_persons_instruction() {
        assert_eq!(instruction_of("THIS IS ONE RUN OF A REPEATING TASK.\nRules.\nHER INSTRUCTION FOLLOWS.\n Check the bookings. "), "Check the bookings.");
        assert_eq!(instruction_of("Check the bookings."), "Check the bookings.");
    }

    #[test]
    fn a_cause_row_names_where_most_of_its_threads_are() {
        let cause = item(json!({ "id": "w-c", "product": "desk", "kind": "cause", "labels": ["cause"], "status": "open" }));
        assert!(is_cause(&cause));
        let items: Vec<Item> = [("w-1", "shop"), ("w-2", "shop"), ("w-3", "desk")]
            .iter()
            .map(|(id, product)| item(json!({ "id": id, "product": product, "status": "blocked", "blockedBy": ["w-c"] })))
            .collect();
        let behind = waiting_on_cause(&cause, &items);
        assert_eq!(behind.len(), 3);
        assert_eq!(cause_place(&cause, &behind, |slug| slug.to_uppercase()), ("SHOP".to_string(), 1));
        assert_eq!(cause_place(&cause, &[], |slug| slug.to_uppercase()), ("DESK".to_string(), 0));
        let by_agent = item(json!({ "kind": "cause", "labels": ["cause"], "wrote": { "kind": { "ts": 1, "source": "agent" } } }));
        assert!(!is_cause(&by_agent));
    }

    #[test]
    fn a_blocked_thread_names_the_open_cause_it_waits_behind() {
        let all: Vec<Item> = [
            json!({ "id": "w-c", "product": "desk", "kind": "cause", "labels": ["cause"], "status": "open", "title": "GitHub is signed out" }),
            json!({ "id": "w-old", "product": "desk", "kind": "cause", "labels": ["cause"], "status": "done" }),
            json!({ "id": "w-other", "product": "desk", "status": "open" }),
        ]
        .into_iter()
        .map(item)
        .collect();
        let behind = |status: &str, waits: serde_json::Value| cause_of(&item(json!({ "id": "w-1", "status": status, "blockedBy": waits })), &all).map(|c| c.id.clone());
        assert_eq!(behind("blocked", json!(["w-other", "w-c"])).as_deref(), Some("w-c"));
        // A cause already closed holds nothing, a thread that is no cause is an ordinary wait,
        // and a thread no longer blocked has stopped waiting.
        assert_eq!(behind("blocked", json!(["w-old"])), None);
        assert_eq!(behind("blocked", json!(["w-other"])), None);
        assert_eq!(behind("open", json!(["w-c"])), None);
        assert_eq!(behind("blocked", json!([])), None);
    }

    #[test]
    fn a_sign_in_waits_until_it_is_answered() {
        assert!(open_sign_in(&item(json!({ "status": "open", "signIn": { "id": "s-1", "service": "Vercel" } }))));
        assert!(!open_sign_in(&item(json!({ "status": "open", "signIn": { "id": "s-1" }, "signInAnswer": { "id": "s-1" } }))));
        assert!(!open_sign_in(&item(json!({ "status": "open", "signIn": { "id": "s-1", "done": true } }))));
        assert!(!open_sign_in(&item(json!({ "status": "done", "signIn": { "id": "s-1" } }))));
        assert!(!open_sign_in(&item(json!({ "status": "open" }))));
    }

    #[test]
    fn a_task_names_the_chat_it_was_asked_in() {
        assert_eq!(asked_in(&item(json!({ "labels": ["asked-in:direct-1/w-9"] }))), Some(("direct-1", "w-9")));
        assert_eq!(asked_in(&item(json!({ "labels": ["asked-in:w-9"] }))), None);
        assert_eq!(asked_in(&item(json!({ "labels": [] }))), None);
    }
}
