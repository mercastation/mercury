//! The new version's card at the sidebar's foot, and the card of a copy that cannot update
//! itself (`renderer/src/update-row.ts`, `renderer/src/components/SidebarUpdate.tsx`,
//! `shared/where-it-runs-from.mts`): when each is shown, what it says, and what its × hides.

use serde_json::Value;

use crate::data::{Snapshot, truthy};

/// `SAY` and `SIDE_SAY`: the button, the card's heading and its line, and the titles of its ×.
pub const RESTART: &str = "Restart to update";
pub const TITLE: &str = "New version ready";
pub const UPDATING: &str = "Updating";
pub const HOW: &str = "Restarting takes about a minute.";
pub const UPDATING_SAID: &str = "Updating. Mercury restarts by itself in about a minute.";
pub const CLOSE_TIP: &str = "Hide until the next version";
/// `MOVE_SAY`: the card of a copy that cannot update itself.
pub const STUCK_TITLE: &str = "Updates are stuck";
pub const MOVE: &str = "Move to Applications";
pub const STUCK_CLOSE_TIP: &str = "Hide it · Settings still offers it";
const CANNOT_UPDATE_HERE: &str = "Mercury cannot update itself from where it is.";
const AGAIN: &str = "Pressing it again tries again.";

/// A new version waiting (`update` as `App.tsx` hands it to the sidebar).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Waiting {
    /// The restart was pressed and it is rebuilding.
    pub installing: bool,
    pub version: Option<String>,
    /// The newest changes' own first lines, and how many changes there are in all.
    pub changes: Vec<String>,
    pub behind: Option<f64>,
    /// Why the last restart did not finish, if it did not.
    pub error: Option<String>,
}

fn part<'a>(snap: &'a Snapshot, key: &str) -> Option<&'a Value> {
    snap.rest.get("update")?.get(key).filter(|v| !v.is_null())
}
fn said(snap: &Snapshot, key: &str) -> Option<String> {
    part(snap, key)?.as_str().map(str::to_string)
}

/// `announcesUpdate`: a version is on the disk, or is being built. While it rebuilds the card
/// stays, since one that vanished the moment Restart was pressed would read as the press doing
/// nothing.
pub fn waiting(snap: &Snapshot) -> Option<Waiting> {
    let (ready, installing) = (part(snap, "ready").is_some_and(|v| truthy(Some(v))), part(snap, "installing").is_some_and(|v| truthy(Some(v))));
    if !ready && !installing {
        return None;
    }
    let changes = part(snap, "changes").and_then(Value::as_array).map(|c| c.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default();
    Some(Waiting { installing, version: said(snap, "newVersion"), changes, behind: part(snap, "behind").and_then(Value::as_f64), error: said(snap, "error") })
}

/// `closedHere`: whether the version closed with the card's × hides this card. Never while a
/// restart is under way: the card saying Updating is how a press is seen to have done something.
pub fn closed_here(waiting: &Waiting, closed: Option<&str>) -> bool {
    !waiting.installing && closed.is_some_and(|closed| closed == waiting.version.as_deref().unwrap_or(""))
}

/// What the × keeps (`writeUpdateClosed`): the version, or nothing for one the app could not
/// name, whose closing lasts this run only.
pub fn to_keep(version: Option<&str>) -> Option<String> {
    version.filter(|v| !v.is_empty()).map(str::to_string)
}

/// `changeLines`: what changed, newest first, and how many more there are.
pub fn change_lines(waiting: &Waiting) -> Vec<String> {
    if waiting.changes.is_empty() {
        return Vec::new();
    }
    let more = waiting.behind.unwrap_or(waiting.changes.len() as f64) - waiting.changes.len() as f64;
    let mut lines = waiting.changes.clone();
    if more > 0.0 {
        lines.push(format!("and {more} more"));
    }
    lines
}

/// What pointing at the card names (`changed`): the list of what changed, or nothing.
pub fn changed(waiting: &Waiting) -> String {
    let lines = change_lines(waiting);
    if lines.is_empty() { String::new() } else { format!("What changed:\n{}", lines.iter().map(|c| format!("- {c}")).collect::<Vec<_>>().join("\n")) }
}

/// A sentence ending on one full stop (`replace(/\.?$/, '.')`).
fn stopped(words: &str) -> String {
    format!("{}.", words.strip_suffix('.').unwrap_or(words))
}

/// The card's line: why the last try failed and that a press tries again, else how long a
/// restart takes (`failed || SIDE_SAY.how`).
pub fn line(waiting: &Waiting) -> String {
    waiting.error.as_deref().map_or_else(|| HOW.to_string(), |error| format!("{} {AGAIN}", stopped(error)))
}

/// The row's title (`tip`): what a press does and what it costs, then what changed.
pub fn tip(waiting: &Waiting) -> String {
    if waiting.installing {
        return UPDATING_SAID.to_string();
    }
    let first = waiting.error.as_deref().map_or_else(|| format!("{RESTART}. {HOW}"), |error| format!("{} {AGAIN}", stopped(error)));
    [first, changed(waiting)].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n\n")
}

/// A copy that cannot update itself from where it is (`move` as `App.tsx` hands it over).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stuck {
    /// Which read-only place this is: `disk-image`, `translocated`.
    pub place: Option<String>,
    /// Why the last move did not work, if it did not.
    pub error: Option<String>,
}

/// `announcesMove`: the copy can be moved, and no restart is running.
pub fn stuck(snap: &Snapshot) -> Option<Stuck> {
    let can = part(snap, "canMove").is_some_and(|v| truthy(Some(v)));
    let installing = part(snap, "installing").is_some_and(|v| truthy(Some(v)));
    (can && !installing).then(|| Stuck { place: said(snap, "place"), error: said(snap, "moveError") })
}

/// Whether the place closed with the stuck card's × hides this card (`moveClosed !== (move.place
/// ?? '')`). A place the app could not name is closed for this run only, as `to_keep` keeps
/// nothing for it.
pub fn stuck_closed_here(stuck: &Stuck, closed: Option<&str>) -> bool {
    closed.is_some_and(|closed| closed == stuck.place.as_deref().unwrap_or(""))
}

/// The stuck row's title: what a press does, and why it is asked (`${MOVE_SAY.button}. ${why}`).
pub fn stuck_tip(stuck: &Stuck) -> String {
    format!("{MOVE}. {}", why(stuck))
}

/// The stuck card's line (`why`): the fact in half the words, since the button beside it says
/// the fix; or why the last move failed and that a press tries again.
pub fn why(stuck: &Stuck) -> String {
    if let Some(error) = &stuck.error {
        return format!("{} {AGAIN}", stopped(error));
    }
    match stuck.place.as_deref() {
        Some("disk-image") => "Mercury can’t update itself from the disk image.".to_string(),
        Some("translocated") => "Mercury can’t update itself from the copy macOS made.".to_string(),
        _ => CANNOT_UPDATE_HERE.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn snap(update: Value) -> Snapshot {
        serde_json::from_value(json!({ "update": update })).unwrap()
    }
    fn v() -> Waiting {
        Waiting { version: Some("abc1234".to_string()), ..Waiting::default() }
    }

    #[test]
    fn a_version_waits_while_it_is_ready_or_being_built() {
        assert_eq!(waiting(&snap(json!({}))), None);
        assert_eq!(waiting(&snap(json!({ "ready": false, "installing": false }))), None);
        assert_eq!(waiting(&snap(json!({ "ready": true, "newVersion": "abc1234" }))), Some(v()));
        let building = waiting(&snap(json!({ "ready": false, "installing": true, "changes": ["One", "Two"], "behind": 4, "error": null }))).expect("still up while it rebuilds");
        assert_eq!((building.installing, building.changes.len(), building.behind, building.error), (true, 2, Some(4.0), None));
    }

    // `tests/a-waiting-update-card-closes-until-the-next-version.test.mts`.
    #[test]
    fn the_x_hides_the_card_for_that_version_only_and_never_while_it_rebuilds() {
        assert!(!closed_here(&v(), None));
        assert!(closed_here(&v(), Some("abc1234")));
        assert!(!closed_here(&Waiting { version: Some("def5678".to_string()), ..v() }, Some("abc1234")), "a newer version brings it back");
        assert!(!closed_here(&Waiting { installing: true, ..v() }, Some("abc1234")));
        // A version the app could not name is not kept: closing it lasts this run.
        assert_eq!(to_keep(Some("abc1234")), Some("abc1234".to_string()));
        assert_eq!(to_keep(Some("")), None);
        assert_eq!(to_keep(None), None);
    }

    #[test]
    fn it_says_how_long_a_restart_takes_or_why_the_last_try_failed() {
        assert_eq!(line(&v()), HOW);
        let failed = Waiting { error: Some("The new code would not build: boom".to_string()), ..v() };
        assert_eq!(line(&failed), "The new code would not build: boom. Pressing it again tries again.");
        assert_eq!(line(&Waiting { error: Some("It stopped.".to_string()), ..v() }), "It stopped. Pressing it again tries again.");
        assert_eq!(tip(&v()), "Restart to update. Restarting takes about a minute.");
        assert_eq!(tip(&Waiting { installing: true, ..v() }), UPDATING_SAID);
    }

    #[test]
    fn pointing_at_it_names_what_changed_and_how_many_more() {
        let w = Waiting { changes: vec!["The inbox reads faster".to_string(), "Fix sign-in".to_string()], behind: Some(4.0), ..v() };
        assert_eq!(change_lines(&w), ["The inbox reads faster", "Fix sign-in", "and 2 more"]);
        assert_eq!(changed(&w), "What changed:\n- The inbox reads faster\n- Fix sign-in\n- and 2 more");
        assert_eq!(tip(&w), "Restart to update. Restarting takes about a minute.\n\nWhat changed:\n- The inbox reads faster\n- Fix sign-in\n- and 2 more");
        // An installed app has no titles, and all of them named leaves nothing more to count.
        assert!(change_lines(&v()).is_empty());
        assert_eq!(changed(&v()), "");
        assert_eq!(change_lines(&Waiting { changes: vec!["One".to_string()], behind: Some(1.0), ..v() }), ["One"]);
    }

    #[test]
    fn a_copy_that_cannot_update_itself_says_where_it_is_or_why_the_move_failed() {
        assert_eq!(stuck(&snap(json!({ "canMove": false }))), None);
        assert_eq!(stuck(&snap(json!({ "canMove": true, "installing": true }))), None, "not while a restart is running");
        let on_the_image = stuck(&snap(json!({ "canMove": true, "place": "disk-image" }))).expect("stuck");
        assert_eq!(why(&on_the_image), "Mercury can’t update itself from the disk image.");
        assert_eq!(why(&Stuck { place: Some("translocated".to_string()), error: None }), "Mercury can’t update itself from the copy macOS made.");
        assert_eq!(why(&Stuck::default()), "Mercury cannot update itself from where it is.");
        assert_eq!(why(&Stuck { place: None, error: Some("It could not be moved".to_string()) }), "It could not be moved. Pressing it again tries again.");
        assert_eq!(stuck_tip(&on_the_image), "Move to Applications. Mercury can’t update itself from the disk image.");
    }

    #[test]
    fn the_stuck_cards_x_hides_it_for_that_place() {
        let on_the_image = Stuck { place: Some("disk-image".to_string()), error: None };
        assert!(!stuck_closed_here(&on_the_image, None));
        assert!(stuck_closed_here(&on_the_image, Some("disk-image")));
        assert!(!stuck_closed_here(&on_the_image, Some("translocated")));
        // A place with no name is closed by the empty word, which is never kept.
        assert!(stuck_closed_here(&Stuck::default(), Some("")));
        assert!(!stuck_closed_here(&Stuck::default(), None));
        assert_eq!(to_keep(Some("disk-image")), Some("disk-image".to_string()));
    }
}
