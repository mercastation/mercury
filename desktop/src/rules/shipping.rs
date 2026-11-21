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
// From `shared/github-push-sign-in.mts`.
const NOT_PUSHED: &str = "It did not ship because ";

re!(CLOSING_STOP, r"\.$", "");
re!(ON_THE_BRANCH, r"on the branch (\S+?)\.(?:\s|$)", "");

/// Why the work did not reach main, while the latest note says so and the agent has not
/// spoken since (`notShipped`, less the parts only an open thread draws).
fn not_shipped(item: &Item) -> Option<String> {
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
}
