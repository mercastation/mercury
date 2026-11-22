//! What the window's frame shows besides the lists: whether there is a team to show, who is
//! signed in, which buttons the rail's foot has, a project's colour. Each follows the web
//! window's own rule, named beside it, and reads the snapshot's parts by the names it gives them.

use serde_json::{Map, Value};

use crate::data::{Snapshot, truthy};

fn part<'a>(snap: &'a Snapshot, key: &str) -> Option<&'a Map<String, Value>> {
    snap.rest.get(key).and_then(Value::as_object)
}

fn list<'a>(of: Option<&'a Map<String, Value>>, key: &str) -> &'a [Value] {
    of.and_then(|o| o.get(key)).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn text<'a>(of: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    of.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

/// The engine hands over teammates, so Home shows the team where it showed In progress, and the
/// rail has Chats (`teammatesUi.live`, `!!snap.teammates`, in `renderer/src/App.tsx`).
pub fn teammates_live(snap: &Snapshot) -> bool {
    truthy(snap.rest.get("teammates"))
}

/// `teamMode` in `renderer/src/App.tsx`: teammates, or rooms switched on for a team that has them.
pub fn team_mode(snap: &Snapshot) -> bool {
    let rooms = part(snap, "alpha").is_some_and(|a| truthy(a.get("rooms")))
        && part(snap, "team").and_then(|t| t.get("rooms")).and_then(Value::as_object).is_some_and(|r| truthy(r.get("on")));
    teammates_live(snap) || rooms
}

/// Whoever is signed in (`team.signedIn ? team.me : null` in `WorkspaceNavigation.tsx`).
pub fn me(snap: &Snapshot) -> Option<&Map<String, Value>> {
    let team = part(snap, "team")?;
    if !truthy(team.get("signedIn")) {
        return None;
    }
    team.get("me").and_then(Value::as_object)
}

/// Whether anyone besides the person is on Home's team card: a teammate that is an agent, a
/// person of the team, or one of their agents. With nobody the card says so in one line.
pub fn team_has_others(snap: &Snapshot) -> bool {
    let team = part(snap, "team");
    let mine = team.and_then(|t| t.get("me")).and_then(Value::as_object).and_then(|m| text(m, "id"));
    let people = list(team, "people").iter().any(|p| p.get("id").and_then(Value::as_str) != mine);
    !list(part(snap, "teammates"), "list").is_empty() || people || !list(team, "teamMates").is_empty()
}

/// The first word of a person's name, else of their email (`firstName` in `renderer/src/team/company.ts`).
fn first_name(person: &Map<String, Value>) -> &str {
    text(person, "name").or_else(|| text(person, "email")).unwrap_or("Someone").split_whitespace().next().unwrap_or("Someone")
}

/// The name at the head of the open rail (`teamName` in `WorkspaceNavigation.tsx`): the team's,
/// else the app's own.
pub fn team_name(snap: &Snapshot) -> &str {
    part(snap, "team").and_then(|t| t.get("team")).and_then(Value::as_object).and_then(|t| text(t, "name")).unwrap_or("Mercury")
}

/// The two letters on the person's own face (`Face` in `renderer/src/team/people.tsx`).
pub fn my_letters(snap: &Snapshot) -> Option<String> {
    me(snap).map(|m| first_name(m).chars().take(2).collect())
}

/// The name Home greets (`greeting` in `renderer/src/home/home-model.ts`, fed `snap.team?.me?.name`).
/// The web window reads it whether or not the sign-in still stands.
pub fn greeted(snap: &Snapshot) -> Option<&str> {
    let me = part(snap, "team")?.get("me")?.as_object()?;
    text(me, "name")?.split_whitespace().next()
}

/// The foot's Record meeting button (`canRecord` in `renderer/src/meeting-card.ts`).
pub fn can_record(snap: &Snapshot) -> bool {
    part(snap, "meeting").is_some_and(|m| m.get("phase").and_then(Value::as_str) == Some("idle") && m.get("preparing").is_none_or(Value::is_null))
}

/// The foot's Restart row: a new version is on the disk, or is being built
/// (`announcesUpdate` in `renderer/src/update-row.ts`). The web window also keeps, for itself,
/// which version's card the person closed; this window has no such memory yet.
pub fn update_waits(snap: &Snapshot) -> bool {
    part(snap, "update").is_some_and(|u| truthy(u.get("ready")) || truthy(u.get("installing")))
}

/// How many agents are held on a yes or a no from the person (`approvals` on the snapshot,
/// drawn by the web window as a card over the page). This window draws no such card yet, so
/// it says that one waits: an agent stopped on a question nobody is shown stays stopped.
pub fn approvals_waiting(snap: &Snapshot) -> usize {
    snap.rest.get("approvals").and_then(Value::as_array).map_or(0, Vec::len)
}

/// The foot's Feedback button: there unless the engine says this build takes none
/// (`openFeedback` in `renderer/src/App.tsx`).
pub fn takes_feedback(snap: &Snapshot) -> bool {
    snap.rest.get("feedback") != Some(&Value::Bool(false))
}

/// A window whose reset is this long past has started over (`JUST_RESET_MS` in `shared/usage.mts`).
const JUST_RESET_MS: f64 = 2.0 * 60_000.0;
const WEEK_MS: f64 = 7.0 * 24.0 * 60.0 * 60_000.0;

/// `limitsStillRunning` in `shared/usage.mts`: a window with no clock on its reset keeps its row.
fn still_running(limit: &Value, now: f64) -> bool {
    limit.get("resetsAt").and_then(Value::as_f64).filter(|at| at.is_finite()).is_none_or(|at| at + JUST_RESET_MS > now)
}

/// `headlineLimit`: the five hour window where there is one, else any window with a figure.
fn has_headline<'a>(limits: impl Iterator<Item = &'a Value>) -> bool {
    limits.into_iter().any(|l| l.get("span").and_then(Value::as_str) == Some("session") || l.get("percent").and_then(Value::as_f64).is_some_and(f64::is_finite))
}

fn limits(usage: &Value) -> &[Value] {
    usage.get("limits").and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// Whether the foot has its usage button: a reading of some agent still answers for a window
/// (`UsagePill` and `agentsInUse` in `renderer/src/components/UsagePill.tsx`).
pub fn usage_shown(snap: &Snapshot, now: f64) -> bool {
    let sent = snap.rest.get("usage").filter(|u| u.is_object());
    let all: Vec<&Value> = match snap.rest.get("usageByEngine").and_then(Value::as_array) {
        Some(every) => every.iter().collect(),
        None => sent.into_iter().collect(),
    };
    let fresh = |u: &&Value| now - u.get("at").and_then(Value::as_f64).unwrap_or(0.0) < WEEK_MS && has_headline(limits(u).iter().filter(|l| still_running(l, now)));
    // Whichever agent is read, the button is the same one here: folded, it has no figure on it.
    let Some(last) = all.into_iter().find(fresh).or(sent) else {
        return false;
    };
    // `limitsToShow`: the windows still running, and with any of them a five hour one that ended.
    let running = limits(last).iter().any(|l| still_running(l, now));
    running && has_headline(limits(last).iter().filter(|l| still_running(l, now) || l.get("span").and_then(Value::as_str) == Some("session")))
}

/// The composer's To chip stands only when someone besides the agent could get the thread
/// (`toOffered` in `renderer/src/threads/composer-rules.ts`): teammates that are agents, or
/// people of a team. Read-only, nobody is picked, so it always says Agent.
pub fn to_offered(snap: &Snapshot) -> bool {
    let team = part(snap, "team");
    // `teamView`: with Multiplayer off the sign-in is kept for the phone only.
    let multiplayer = part(snap, "alpha").and_then(|a| a.get("multiplayer")) != Some(&Value::Bool(false));
    let mine = me(snap).and_then(|m| text(m, "id"));
    let people = multiplayer && mine.is_some() && (list(team, "people").iter().any(|p| p.get("id").and_then(Value::as_str) != mine) || !list(team, "sent").is_empty());
    teammates_live(snap) || people
}

/// A project's colour, read off its slug so it is the same on every Mac
/// (`projectSwatch` in `renderer/src/threads/composer-rules.ts`). JavaScript walks a string by
/// code point here and takes each one's first UTF-16 unit.
pub fn swatch(slug: &str) -> (u8, u8, u8) {
    const SWATCHES: [(u8, u8, u8); 7] = [(0xee, 0x60, 0x18), (0x5b, 0x9b, 0xd5), (0x9a, 0x8f, 0x86), (0x6a, 0xa3, 0x7a), (0xb4, 0x8e, 0xad), (0xd0, 0xa6, 0x5a), (0x4f, 0xa3, 0xa5)];
    let mut h: u32 = 0;
    for c in slug.chars() {
        let mut units = [0u16; 2];
        h = h.wrapping_mul(31).wrapping_add(u32::from(c.encode_utf16(&mut units)[0]));
    }
    SWATCHES[(h % 7) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_784_000_000_000.0;

    fn snap(rest: Value) -> Snapshot {
        serde_json::from_value(rest).unwrap()
    }

    #[test]
    fn a_project_keeps_the_colour_its_slug_gives_it() {
        // The same seven the web window has, by the same sum.
        assert_eq!(swatch("storefront"), (0xee, 0x60, 0x18));
        assert_eq!(swatch("shop"), (0x4f, 0xa3, 0xa5));
        assert_eq!(swatch("desk"), (0x5b, 0x9b, 0xd5));
        assert_eq!(swatch("café"), (0xb4, 0x8e, 0xad));
        assert_eq!(swatch(""), (0xee, 0x60, 0x18));
    }

    #[test]
    fn home_shows_the_team_once_the_engine_hands_over_teammates() {
        assert!(!team_mode(&snap(json!({}))));
        assert!(!team_mode(&snap(json!({ "teammates": null }))));
        assert!(team_mode(&snap(json!({ "teammates": { "list": [] } }))));
        // Rooms count only when switched on and the team has them.
        assert!(!team_mode(&snap(json!({ "alpha": { "rooms": true }, "team": { "rooms": { "on": false } } }))));
        assert!(team_mode(&snap(json!({ "alpha": { "rooms": true }, "team": { "rooms": { "on": true } } }))));
    }

    #[test]
    fn the_team_card_is_empty_until_someone_else_is_on_it() {
        let alone = json!({ "teammates": { "list": [] }, "team": { "signedIn": true, "me": { "id": "p1", "name": "Ada Lovelace" }, "people": [{ "id": "p1" }] } });
        assert!(!team_has_others(&snap(alone)));
        assert!(team_has_others(&snap(json!({ "teammates": { "list": [{ "id": "t1" }] } }))));
        assert!(team_has_others(&snap(json!({ "team": { "me": { "id": "p1" }, "people": [{ "id": "p1" }, { "id": "p2" }] } }))));
    }

    #[test]
    fn the_corner_is_the_person_signed_in_and_nobody_otherwise() {
        let signed = snap(json!({ "team": { "signedIn": true, "me": { "id": "p1", "name": "ada lovelace", "email": "ada@example.com" } } }));
        assert_eq!(my_letters(&signed).as_deref(), Some("ad"));
        assert_eq!(greeted(&signed), Some("ada"));
        let by_email = snap(json!({ "team": { "signedIn": true, "me": { "id": "p1", "name": "", "email": "grace@example.com" } } }));
        assert_eq!(my_letters(&by_email).as_deref(), Some("gr"));
        assert_eq!(greeted(&by_email), None);
        let out = snap(json!({ "team": { "signedIn": false, "me": { "id": "p1", "name": "Ada" } } }));
        assert_eq!(my_letters(&out), None);
        assert_eq!(my_letters(&snap(json!({}))), None);
    }

    #[test]
    fn the_foot_has_record_only_while_no_meeting_is_on_or_getting_ready() {
        assert!(!can_record(&snap(json!({}))));
        assert!(can_record(&snap(json!({ "meeting": { "phase": "idle", "preparing": null } }))));
        assert!(can_record(&snap(json!({ "meeting": { "phase": "idle" } }))));
        assert!(!can_record(&snap(json!({ "meeting": { "phase": "recording", "preparing": null } }))));
        assert!(!can_record(&snap(json!({ "meeting": { "phase": "idle", "preparing": { "at": 1 } } }))));
    }

    #[test]
    fn an_agent_held_on_a_yes_is_counted() {
        assert_eq!(approvals_waiting(&snap(json!({}))), 0);
        assert_eq!(approvals_waiting(&snap(json!({ "approvals": [] }))), 0);
        assert_eq!(approvals_waiting(&snap(json!({ "approvals": [{ "id": "a1", "tool": "Bash" }, { "id": "a2", "tool": "Edit" }] }))), 2);
    }

    #[test]
    fn the_foot_has_restart_while_a_new_version_waits_or_is_being_built() {
        assert!(!update_waits(&snap(json!({}))));
        assert!(!update_waits(&snap(json!({ "update": { "ready": false, "installing": false } }))));
        assert!(update_waits(&snap(json!({ "update": { "ready": true } }))));
        assert!(update_waits(&snap(json!({ "update": { "ready": false, "installing": true } }))));
    }

    #[test]
    fn the_foot_has_feedback_unless_the_engine_says_this_build_takes_none() {
        assert!(takes_feedback(&snap(json!({}))));
        assert!(takes_feedback(&snap(json!({ "feedback": true }))));
        assert!(!takes_feedback(&snap(json!({ "feedback": false }))));
    }

    #[test]
    fn the_foot_has_usage_only_while_a_reading_still_answers_for_a_window() {
        let week = |percent: f64, resets: f64| json!({ "span": "week", "percent": percent, "resetsAt": resets });
        assert!(!usage_shown(&snap(json!({})), NOW));
        // A reading with a window still running.
        assert!(usage_shown(&snap(json!({ "usage": { "engine": "claude", "at": NOW - 60_000.0, "limits": [week(40.0, NOW + 3_600_000.0)] } })), NOW));
        // Every window has started over since: nothing to say.
        assert!(!usage_shown(&snap(json!({ "usage": { "engine": "claude", "at": NOW - 60_000.0, "limits": [week(40.0, NOW - 3_600_000.0)] } })), NOW));
        // A reset with no clock on it keeps its row.
        assert!(usage_shown(&snap(json!({ "usage": { "engine": "claude", "at": NOW - 60_000.0, "limits": [{ "span": "week", "percent": 12.0, "resetsAt": null }] } })), NOW));
        // Another agent's reading answers when the first one's has run out.
        let two = json!({ "usage": { "engine": "claude", "at": NOW - 60_000.0, "limits": [week(40.0, NOW - 3_600_000.0)] },
            "usageByEngine": [{ "engine": "claude", "at": NOW - 60_000.0, "limits": [week(40.0, NOW - 3_600_000.0)] }, { "engine": "codex", "at": NOW - 120_000.0, "limits": [week(9.0, NOW + 3_600_000.0)] }] });
        assert!(usage_shown(&snap(two), NOW));
    }

    #[test]
    fn the_composer_offers_to_only_when_someone_besides_the_agent_could_get_the_thread() {
        assert!(!to_offered(&snap(json!({}))));
        assert!(to_offered(&snap(json!({ "teammates": { "list": [] } }))));
        let team = json!({ "signedIn": true, "me": { "id": "p1" }, "people": [{ "id": "p1" }, { "id": "p2" }] });
        assert!(to_offered(&snap(json!({ "team": team.clone() }))));
        // With Multiplayer off the sign-in is for the phone, and the team is not offered.
        assert!(!to_offered(&snap(json!({ "alpha": { "multiplayer": false }, "team": team }))));
    }
}
