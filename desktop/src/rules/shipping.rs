//! A thread whose work never reached main, or did, read off the app's own notes. A port of
//! `shared/not-shipped.mts`. The ship queue and the cloud run write these words, and the
//! agent's last message was written before either, so the notes are what has to be shown.

use super::format::is_shipped_note;
use super::js::{self, group, re};
use crate::data::Item;

const SHIP_FAILED: &str = "It did not ship, and it went back to its agent: ";
const KEPT_ASIDE: &str = "This thread's cloud run finished, but its work was not laid into the task folder: ";
// The sweep's note when it reopens a thread that was closed without being asked.
const CLOSED_UNSHIPPED: &str = "This thread was closed before its work reached main, so it is back. ";
// Written when the person was asked and closed it anyway; ends the matter.
const CLOSED_ANYWAY: &str = "Closed without its work reaching main.";
// From `shared/github-push-sign-in.mts`: a ship whose push needed a GitHub sign-in on this
// computer. The push runs here, so the thread's agent cannot fix it: the app asks for the
// sign-in on the thread (`need.rs`) and ships again once it is saved.
const NOT_PUSHED: &str = "It did not ship because ";
pub const NO_GITHUB_SIGN_IN: &str = "this computer could not sign in to GitHub";
pub const GITHUB_REFUSED_SIGN_IN: &str = "GitHub refused the sign-in saved on this computer";
/// A classic token with the one box a push needs already ticked, named after the app.
pub const GITHUB_TOKEN_PAGE: &str = "https://github.com/settings/tokens/new?scopes=repo&description=Mercury";

// From `shared/notes-for-the-agent.mts`: notes the app writes for the next run, not for the
// person, about trouble it is already seeing to by itself.
const MACHINE_STOPPED: &str = "This thread's cloud machine stopped";
const WAITING_FOR_MEMORY: [&str; 2] = ["Waiting for memory on this computer before shipping.", "Waiting for memory on this computer before pushing."];
const CLEARED: &str = " is cleared. Carry on from where this stopped.";

/// A note of the app's that no thread draws: it stays on the row for the next run to read
/// (`forTheAgent`).
pub fn for_the_agent(note: &str) -> bool {
    note.starts_with(MACHINE_STOPPED) || WAITING_FOR_MEMORY.contains(&note) || note.ends_with(CLEARED)
}

/// Whether a not-shipped reason is one a GitHub sign-in answers (`asksForGitHubSignIn`).
pub fn asks_for_github_sign_in(why: &str) -> bool {
    why == NO_GITHUB_SIGN_IN || why == GITHUB_REFUSED_SIGN_IN
}

/// How the newest answer is drawn at the foot of a thread (`footOf` in
/// `renderer/src/one-thing-at-a-time.ts`): the app's own note as its quiet line, a ship that
/// failed as Not shipped, which asks something of the person, anything else as the agent's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Foot {
    App,
    Unshipped,
    Agent,
}

/// `note` says the answer at the foot is the row's note, and `app` that the app wrote it.
pub fn foot_of(item: &Item, note: bool, app: bool) -> Foot {
    if note && not_shipped(item).is_some() {
        Foot::Unshipped
    } else if note && app {
        Foot::App
    } else {
        Foot::Agent
    }
}

re!(CLOSING_STOP, r"\.$", "");
re!(ON_THE_BRANCH, r"on the branch (\S+?)\.(?:\s|$)", "");

/// Why the work did not reach main, while the latest note says so and the agent has not
/// spoken since (`notShipped`, less the parts only an open thread draws).
pub fn not_shipped(item: &Item) -> Option<String> {
    let note = item.text("note")?;
    if item.wrote_at("result") > item.wrote_at("note") {
        return None;
    }
    let text = note.strip_prefix(CLOSED_UNSHIPPED).unwrap_or(note);
    if let Some(why) = text.strip_prefix(SHIP_FAILED) {
        return Some(js::trim(why).to_string());
    }
    if let Some(why) = text.strip_prefix(NOT_PUSHED) {
        return Some(CLOSING_STOP.strip(js::trim(why)));
    }
    if text.starts_with(KEPT_ASIDE) {
        return Some(match ON_THE_BRANCH.exec(text) {
            Some(m) => format!("the cloud run's work is set aside on {}", group(&m, 1)),
            None => "the cloud run's work is set aside".to_string(),
        });
    }
    None
}

/// Whether the latest note says the work did not reach main and the agent has not spoken since
/// (`!!notShipped(item)`): a close then asks first.
pub fn is_not_shipped(item: &Item) -> bool {
    not_shipped(item).is_some()
}

/// The branch a cloud run set the work aside on, when its note names one (`notShipped`'s
/// `branch`).
fn kept_aside_on(item: &Item) -> Option<String> {
    let note = item.text("note")?;
    let text = note.strip_prefix(CLOSED_UNSHIPPED).unwrap_or(note);
    if !text.starts_with(KEPT_ASIDE) {
        return None;
    }
    ON_THE_BRANCH.exec(text).map(|m| group(&m, 1).to_string())
}

/// What the agent is asked when the person picks "Have the agent ship it" (`shipItAsk`).
pub fn ship_it_ask(item: &Item) -> Option<String> {
    not_shipped(item)?;
    Some(match kept_aside_on(item) {
        Some(branch) => format!("Bring the work on {branch} into this folder, check it against current main, then ship it."),
        None => "Fix what stopped the ship, then ship it.".to_string(),
    })
}

/// The row's line: "Not shipped. <why>".
pub fn not_shipped_line(item: &Item) -> Option<String> {
    not_shipped(item).map(|why| format!("Not shipped. {}", js::upper_first(&why)))
}

/// The app's own latest note says it shipped the work and nobody has written on the thread
/// since (`shippedAs`). An agent saying it shipped does not count: only the app ships.
pub fn shipped_by_the_app(item: &Item) -> bool {
    if !is_shipped_note(item.text("note")) || item.wrote("note").is_none_or(|w| w.source != "system") {
        return false;
    }
    let noted = item.wrote_at("note");
    !(item.wrote_at("result") > noted || item.wrote_at("answer") > noted)
}

/// A closed thread whose close was told its work never reached main (`closedUnshipped`).
pub fn closed_unshipped(item: &Item) -> bool {
    item.status == "done" && item.text("note").is_some_and(|n| n.starts_with(CLOSED_ANYWAY))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn a_failed_ship_is_said_until_the_agent_speaks_again() {
        let failed = json!({ "note": "It did not ship, and it went back to its agent: the tests on main are red", "wrote": { "note": { "ts": 20, "source": "system" }, "result": { "ts": 10, "source": "agent" } } });
        assert_eq!(not_shipped_line(&item(failed.clone())).as_deref(), Some("Not shipped. The tests on main are red"));
        let mut answered = failed;
        answered["wrote"]["result"]["ts"] = json!(30);
        assert_eq!(not_shipped_line(&item(answered)), None);
        let aside = json!({ "note": "This thread's cloud run finished, but its work was not laid into the task folder: it is on the branch cloud/w-7. Ask for it.", "wrote": { "note": { "ts": 20, "source": "system" } } });
        assert_eq!(not_shipped_line(&item(aside)).as_deref(), Some("Not shipped. The cloud run's work is set aside on cloud/w-7"));
        let unpushed = json!({ "note": "This thread was closed before its work reached main, so it is back. It did not ship because GitHub refused the sign-in.", "wrote": { "note": { "ts": 20, "source": "system" } } });
        assert_eq!(not_shipped_line(&item(unpushed)).as_deref(), Some("Not shipped. GitHub refused the sign-in"));
    }

    #[test]
    fn a_push_that_could_not_sign_in_asks_for_a_github_sign_in_and_nothing_else_does() {
        let noted = |note: &str| item(json!({ "status": "open", "note": note, "wrote": { "note": { "ts": 20, "source": "system" }, "result": { "ts": 10, "source": "agent" } } }));
        let none = not_shipped(&noted("It did not ship because this computer could not sign in to GitHub.")).unwrap();
        assert_eq!(none, NO_GITHUB_SIGN_IN);
        assert!(asks_for_github_sign_in(&none));
        let refused = not_shipped(&noted("It did not ship because GitHub refused the sign-in saved on this computer.")).unwrap();
        assert_eq!(refused, GITHUB_REFUSED_SIGN_IN);
        assert!(asks_for_github_sign_in(&refused));
        // A red test run went back to its agent: there is nothing for a sign-in to answer.
        let red = not_shipped(&noted("It did not ship, and it went back to its agent: the tests on main are red")).unwrap();
        assert!(!asks_for_github_sign_in(&red));
    }

    #[test]
    fn the_foot_of_a_thread_is_the_apps_quiet_line_a_ship_that_failed_or_the_agents_word() {
        let shipped = item(json!({ "note": "Shipped to main as 0a1b2c3.", "wrote": { "note": { "ts": 20, "source": "system" }, "result": { "ts": 10, "source": "agent" } } }));
        assert_eq!(foot_of(&shipped, true, true), Foot::App);
        // A failed ship is Not shipped whoever is taken to have written the note.
        let failed = item(json!({ "note": "It did not ship because this computer could not sign in to GitHub.", "wrote": { "note": { "ts": 20, "source": "system" }, "result": { "ts": 10, "source": "agent" } } }));
        assert_eq!(foot_of(&failed, true, true), Foot::Unshipped);
        // The agent's own checkpoint, and its result, are the agent's.
        let checked = item(json!({ "note": "Halfway there.", "wrote": { "note": { "ts": 20, "source": "agent" } } }));
        assert_eq!(foot_of(&checked, true, false), Foot::Agent);
        assert_eq!(foot_of(&shipped, false, false), Foot::Agent);
    }

    #[test]
    fn trouble_the_app_is_seeing_to_itself_is_for_the_next_run() {
        assert!(for_the_agent("This thread's cloud machine stopped two minutes in. The thread picks up from where it was before that run."));
        assert!(for_the_agent("Waiting for memory on this computer before shipping."));
        assert!(for_the_agent("Waiting for memory on this computer before pushing."));
        assert!(for_the_agent("The red main is cleared. Carry on from where this stopped."));
        assert!(!for_the_agent("Shipped to main as 0a1b2c3."));
        assert!(!for_the_agent("Waiting for memory on this computer before shipping. More."));
    }

    #[test]
    fn only_the_apps_own_note_says_shipped() {
        let shipped = json!({ "note": "Shipped to main as 0a1b2c3.", "wrote": { "note": { "ts": 20, "source": "system" }, "result": { "ts": 10, "source": "agent" } } });
        assert!(shipped_by_the_app(&item(shipped.clone())));
        let mut by_agent = shipped.clone();
        by_agent["wrote"]["note"]["source"] = json!("agent");
        assert!(!shipped_by_the_app(&item(by_agent)));
        let mut replied = shipped;
        replied["wrote"]["answer"] = json!({ "ts": 30, "source": "founder" });
        assert!(!shipped_by_the_app(&item(replied)));
        assert!(closed_unshipped(&item(json!({ "status": "done", "note": "Closed without its work reaching main." }))));
    }

    /// `tests/a-thread-whose-work-never-reached-main-is-not-closed-unasked.test.mts`.
    #[test]
    fn the_agent_is_asked_to_ship_it_in_words_that_name_where_the_work_is() {
        let conflict = item(json!({ "status": "open", "note": "It did not ship, and it went back to its agent: CONFLICT (content): Merge conflict in cloud/sandbox/server.mjs", "wrote": { "note": { "ts": 200, "source": "system" }, "result": { "ts": 100 } } }));
        assert!(is_not_shipped(&conflict));
        assert_eq!(ship_it_ask(&conflict).as_deref(), Some("Fix what stopped the ship, then ship it."));
        let aside = item(json!({ "status": "open", "note": "This thread's cloud run finished, but its work was not laid into the task folder: it is on the branch cloud/w-21b638d821. Nothing is lost.", "wrote": { "note": { "ts": 200, "source": "system" } } }));
        assert_eq!(ship_it_ask(&aside).as_deref(), Some("Bring the work on cloud/w-21b638d821 into this folder, check it against current main, then ship it."));
        let spoke_since = item(json!({ "status": "open", "note": "It did not ship, and it went back to its agent: x", "wrote": { "note": { "ts": 200 }, "result": { "ts": 300 } } }));
        assert!(!is_not_shipped(&spoke_since) && ship_it_ask(&spoke_since).is_none());
        assert!(!is_not_shipped(&item(json!({ "status": "open", "note": "Shipped to main as d202b199.", "wrote": { "note": { "ts": 200 } } }))));
        assert!(!is_not_shipped(&item(json!({ "status": "open" }))));
    }
}
