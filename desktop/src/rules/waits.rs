//! What a person says a thread waits on, kept in the row's `blockedBy`: another thread (its
//! id), a teammate's thread (`card:<person>:<thread>`), a person (`person:<id>`) or anyone at
//! all in plain words (`name:<words>`). A port of the part of `shared/thread-waits.mts` the
//! lists read.

use serde_json::Value;

use crate::data::Item;

/// What one wait is on (`waitParts`): a thread of the person's own, something said in plain
/// words, or a person, whether named or through a thread of theirs.
enum Wait<'a> {
    Thread,
    Words,
    Person(&'a str),
}

fn wait_parts(id: &str) -> Wait<'_> {
    if let Some(person) = id.strip_prefix("person:") {
        return Wait::Person(person);
    }
    if id.starts_with("name:") {
        return Wait::Words;
    }
    if id.starts_with("card:") {
        return Wait::Person(id.split(':').nth(1).unwrap_or(""));
    }
    Wait::Thread
}

/// The ids a row waits on, as text (`String(id)` on each).
pub fn blocked_by(item: &Item) -> Vec<String> {
    item.list("blockedBy").iter().map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string)).collect()
}

/// Waiting on the row's own owner is a question for them, not a wait. And a reply after the
/// agent wrote the wait is answered by the agent's next words, which must reach them.
fn on_others(item: &Item) -> bool {
    if item.wrote("answer").is_some_and(|reply| reply.source == "founder" && reply.at() > item.wrote_at("blockedBy")) {
        return false;
    }
    let owner = item.string("runner").or(item.string("createdBy"));
    blocked_by(item).iter().all(|id| match wait_parts(id) {
        Wait::Thread => false,
        // Plain words name nobody, which is never the owner.
        Wait::Words => true,
        Wait::Person(person) => Some(person) != owner,
    })
}

/// The thread waits because a person said so, or on other people, which only the team sync
/// can see end. An agent's wait on threads is not this: the supervisor resumes those.
pub fn set_by_person(item: &Item) -> bool {
    if item.status == "done" || !matches!(item.rest.get("blockedBy"), Some(Value::Array(ids)) if !ids.is_empty()) {
        return false;
    }
    item.wrote("blockedBy").is_some_and(|w| w.source == "founder") || on_others(item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn a_wait_the_person_set_counts_and_an_agents_wait_on_a_thread_does_not() {
        assert!(set_by_person(&item(json!({ "status": "blocked", "blockedBy": ["w-1"], "wrote": { "blockedBy": { "ts": 5, "source": "founder" } } }))));
        assert!(!set_by_person(&item(json!({ "status": "blocked", "blockedBy": ["w-1"], "wrote": { "blockedBy": { "ts": 5, "source": "agent" } } }))));
        assert!(!set_by_person(&item(json!({ "status": "done", "blockedBy": ["w-1"], "wrote": { "blockedBy": { "ts": 5, "source": "founder" } } }))));
        assert!(!set_by_person(&item(json!({ "status": "blocked", "blockedBy": [] }))));
        assert!(!set_by_person(&item(json!({ "status": "blocked" }))));
    }

    #[test]
    fn an_agents_wait_on_other_people_counts_until_the_person_replies() {
        let waits = json!({ "status": "blocked", "blockedBy": ["name:the print shop"], "wrote": { "blockedBy": { "ts": 5, "source": "agent" } } });
        assert!(set_by_person(&item(waits.clone())));
        let mut replied = waits;
        replied["wrote"]["answer"] = json!({ "ts": 9, "source": "founder" });
        assert!(!set_by_person(&item(replied)));
        // Waiting on the row's own owner is a question for them.
        assert!(!set_by_person(&item(json!({ "status": "blocked", "createdBy": "p-1", "blockedBy": ["person:p-1"] }))));
        assert!(set_by_person(&item(json!({ "status": "blocked", "createdBy": "p-1", "blockedBy": ["person:p-2", "card:p-3:w-9"] }))));
        assert!(!set_by_person(&item(json!({ "status": "blocked", "createdBy": "p-1", "blockedBy": ["person:p-2", "w-9"] }))));
    }
}
