//! The person's account menu: which of its rows a person sees, what each says, and what each
//! writes. The web window's rules, named beside each: `AccountMenu` and `AccountMenuList` in
//! `renderer/src/team/account-menu.tsx`, how `renderer/src/App.tsx` hands `WorkspaceNavigation`
//! its doors, the status's rules in `shared/team-status.mts` and
//! `renderer/src/team/status-until.ts`, and going invisible in `renderer/src/team/rooms/`.
//!
//! The status's rules are ported and held to the web's cases here, but no row uses them yet:
//! the box a status is written in cannot be drawn the same in this window (see "The account
//! menu" in `docs/parity-sidebar.md`), so its row is left out until it can.
#![allow(dead_code)]

use serde_json::{Map, Value, json};

use crate::data::{Snapshot, truthy};
use crate::rules::calendar::Day;

/// What the status row says while nothing is said (`STATUS_PROMPT` in `team/status.tsx`).
pub const STATUS_PROMPT: &str = "Set a status";

/// A status is one line, cut here (`STATUS_MAX`).
pub const STATUS_MAX: usize = 80;

/// The channels the menu's writes go on, each as the web window calls it (`main/ipc.mts`).
pub const SIGN_OUT: &str = "zero:team-sign-out";
pub const STATUS: &str = "zero:team-status";
pub const ROOMS: &str = "zero:rooms";

fn part<'a>(snap: &'a Snapshot, key: &str) -> Option<&'a Map<String, Value>> {
    snap.rest.get(key).and_then(Value::as_object)
}

/// The rows a person's menu has besides who they are, Settings and Sign out, which every
/// signed-in person's has (`App.tsx` hands `onAccount`, `onSettings` and `onSignOut` always).
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Doors {
    /// What they are up to (`statuses={multiplayer}`).
    pub status: bool,
    /// Go invisible, or Come back while they are (`invisible={roomsUi.invisible}`): rooms are
    /// on, live for this team, and know the person. `Some(true)` while invisible.
    pub invisible: Option<bool>,
    /// Invite people (`onInvite={multiplayer ? … : undefined}`).
    pub invite: bool,
    /// Whether Settings has a Team page to open (`teamPane={snap?.team?.configured ? …}`):
    /// without one, who they are and Invite people land on General (`paneFrom`).
    pub team_page: bool,
}

/// The menu's rows for whoever is signed in, or None with nobody signed in, when the corner
/// draws nothing at all (`corner` in `WorkspaceNavigation.tsx`).
pub fn doors(snap: &Snapshot, now: f64) -> Option<Doors> {
    crate::rules::window::me(snap)?;
    // `snap?.alpha?.multiplayer !== false`.
    let multiplayer = part(snap, "alpha").and_then(|a| a.get("multiplayer")) != Some(&Value::Bool(false));
    let team = part(snap, "team");
    // `useRooms`: `live = on && !!rooms?.on`, and `invisible` only while the rooms know the person.
    let rooms = team.and_then(|t| t.get("rooms")).and_then(Value::as_object);
    let live = part(snap, "alpha").is_some_and(|a| truthy(a.get("rooms"))) && rooms.is_some_and(|r| truthy(r.get("on")));
    let me = rooms.and_then(|r| r.get("me")).filter(|m| truthy(Some(*m)));
    let invisible = (live && me.is_some()).then(|| {
        let until = me.and_then(|m| m.get("invisible_until")).and_then(Value::as_str).and_then(crate::rules::notes::iso_ms);
        until.is_some_and(|u| u > now)
    });
    Some(Doors { status: multiplayer, invisible, invite: multiplayer, team_page: team.is_some_and(|t| truthy(t.get("configured"))) })
}

/// What a person said, as it reads now (`liveStatus`): its words, cut to one line, and the
/// moment it runs out. None when nothing is said or the moment has come.
pub fn said(status: Option<&Value>, now: f64) -> Option<(String, Option<f64>)> {
    let words = clean(status.and_then(|s| s.get("text")).and_then(Value::as_str).unwrap_or(""))?;
    let until = status.and_then(|s| s.get("until")).and_then(Value::as_f64).filter(|u| *u != 0.0);
    if until.is_some_and(|u| u <= now) {
        return None;
    }
    Some((words, until))
}

/// The person's own, from the snapshot.
pub fn my_status(snap: &Snapshot, now: f64) -> Option<(String, Option<f64>)> {
    said(crate::rules::window::me(snap)?.get("status"), now)
}

/// Words as the app keeps them: every run of white space one space, trimmed, eighty characters
/// at most, or None for nothing (`cleanStatus`).
pub fn clean(text: &str) -> Option<String> {
    let one = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!one.is_empty()).then(|| one.chars().take(STATUS_MAX).collect())
}

const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// How long it holds, for the end of a line: "until 6pm today", "until Monday", "until Mar 3",
/// or nothing when it holds until cleared or has passed (`holdsUntil`).
pub fn holds_until(until: Option<f64>, now: f64) -> String {
    let Some(until) = until.filter(|u| *u > now) else {
        return String::new();
    };
    let t = rune::time::LocalTime::at(until as i64);
    if Day::of(until) == Day::of(now) {
        let h = t.hour;
        let mins = if t.minute > 0 { format!(":{:02}", t.minute) } else { String::new() };
        return format!("until {}{mins}{} today", if h % 12 == 0 { 12 } else { h % 12 }, if h < 12 { "am" } else { "pm" });
    }
    // Inside a week a weekday names itself; past it the date is safer.
    if until - now < 7.0 * 86_400_000.0 {
        return format!("until {}", DAYS[Day::of(until).weekday() as usize]);
    }
    format!("until {} {}", MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day)
}

/// The status row's words: what was said and how long it holds, or the prompt
/// (`said ? \`${said.text}${held ? \` · ${held}\` : ''}\` : STATUS_PROMPT`).
pub fn status_row(said: Option<&(String, Option<f64>)>, now: f64) -> String {
    match said {
        Some((words, until)) => {
            let held = holds_until(*until, now);
            if held.is_empty() { words.clone() } else { format!("{words} · {held}") }
        }
        None => STATUS_PROMPT.to_string(),
    }
}

/* -------------------------------- when it ends -------------------------------- */

/// When a status ends: one of the four holds, or a moment chosen (`StatusEnd`).
#[derive(Debug, PartialEq, Clone)]
pub enum End {
    Hold(&'static str),
    Until(f64),
}

/// The holds, in the order the menu draws them, in the sentence's words (`UNTIL_PRESETS`).
pub const PRESETS: [(&str, &str); 4] = [("today", "the end of today"), ("tomorrow", "tomorrow night"), ("week", "the end of the week"), ("open", "I clear it")];

/// When a hold runs out: the end of today, of tomorrow, of the coming Sunday (never today), or
/// never (`holdEnds`).
pub fn hold_ends(id: &str, now: f64) -> Option<f64> {
    if id == "open" {
        return None;
    }
    let today = Day::of(now);
    let day = match id {
        "tomorrow" => today.plus(1),
        "week" => {
            let to_sunday = (7 - today.weekday()) % 7;
            today.plus(if to_sunday == 0 { 7 } else { to_sunday })
        }
        _ => today,
    };
    // 23:59:59.999 on that day's clock.
    Some(day.at(23, 59) + 59_999.0)
}

/// A time typed in words, as a moment still ahead and the words for it (`readUntil`).
pub fn read_until(phrase: &str, now: f64) -> Option<(f64, String)> {
    crate::rules::later::parse_when(phrase.trim(), now).filter(|(ts, _)| *ts > now)
}

/// The sentence's last words (`untilWords`).
pub fn until_words(end: &End, now: f64) -> String {
    match end {
        End::Until(at) => {
            let held = holds_until(Some(*at), now);
            match held.strip_prefix("until ") {
                Some(rest) if !rest.is_empty() => rest.to_string(),
                _ => "I clear it".to_string(),
            }
        }
        End::Hold(hold) => PRESETS.iter().find(|(id, _)| id == hold).map_or("the end of today", |(_, w)| w).to_string(),
    }
}

/// Where the sentence starts when it opens: the end already chosen, never a guess; a new one
/// on the end of today (`endOf`).
pub fn end_of(said: Option<&(String, Option<f64>)>, now: f64) -> End {
    match said {
        None => End::Hold("today"),
        Some((_, None)) => End::Hold("open"),
        Some((_, Some(until))) => PRESETS
            .iter()
            .find(|(id, _)| *id != "open" && hold_ends(id, now) == Some(*until))
            .map_or(End::Until(*until), |(id, _)| End::Hold(id)),
    }
}

/// The write that saves a status (`save` in `StatusComposer`): the words and the hold, with
/// the moment when one was chosen; or, cleared, nothing said and held open.
pub fn status_payload(words: &str, end: &End, clear: bool) -> Value {
    if clear {
        return json!({ "text": "", "hold": "open" });
    }
    match end {
        End::Hold(hold) => json!({ "text": words, "hold": hold }),
        End::Until(at) => json!({ "text": words, "hold": "open", "until": at }),
    }
}

/// What the box keeps of what is typed: every run of white space one space, and no more than
/// eighty characters (`typed`).
pub fn typed(raw: &str) -> String {
    let mut one = String::new();
    let mut space = false;
    for c in raw.chars() {
        if c.is_whitespace() {
            space = true;
            continue;
        }
        if space {
            one.push(' ');
            space = false;
        }
        one.push(c);
    }
    if space {
        one.push(' ');
    }
    one.chars().take(STATUS_MAX).collect()
}

/* ------------------------------- going invisible ------------------------------- */

/// The three ends Go invisible offers, in its words, and how many minutes each is
/// (`GoInvisible`: an hour, until midnight, until turned off).
pub fn invisible_ends(now: f64) -> [(&'static str, Option<f64>); 3] {
    [("For an hour", Some(60.0)), ("Until tomorrow", Some(minutes_to_midnight(now))), ("Until I turn it off", None)]
}

/// Minutes left until this computer's midnight, one at least (`minutesToMidnight`).
pub fn minutes_to_midnight(now: f64) -> f64 {
    let midnight = Day::of(now).plus(1).at(0, 0);
    ((midnight - now) / 60_000.0).round().max(1.0)
}

/// What the rooms are asked (`api.rooms(op, ...args)` sends `{ op, args }`).
pub fn go_invisible(minutes: Option<f64>) -> Value {
    json!({ "op": "goInvisible", "args": [minutes] })
}
pub fn come_back() -> Value {
    json!({ "op": "comeBack", "args": [] })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: f64 = 3_600_000.0;
    const DAY: f64 = 24.0 * HOUR;

    fn snap(rest: Value) -> Snapshot {
        serde_json::from_value(rest).unwrap()
    }

    /// A Wednesday at 10am on this computer's clock (`new Date(2026, 9, 7, 10)`).
    fn wednesday() -> f64 {
        Day::real(2026, 9, 7).unwrap().at(10, 0)
    }

    fn me() -> Value {
        json!({ "id": "u-1", "email": "ada@example.test", "name": "Ada Lovelace", "avatarUrl": null })
    }

    #[test]
    fn nobody_signed_in_has_no_menu_and_everyone_signed_in_has_who_settings_and_sign_out() {
        assert_eq!(doors(&snap(json!({})), 0.0), None);
        assert_eq!(doors(&snap(json!({ "team": { "signedIn": false, "me": me() } })), 0.0), None);
        // Multiplayer on, as it is unless turned off: the status and Invite people too.
        let on = doors(&snap(json!({ "team": { "signedIn": true, "me": me(), "configured": true } })), 0.0).unwrap();
        assert_eq!(on, Doors { status: true, invisible: None, invite: true, team_page: true });
    }

    #[test]
    fn with_multiplayer_off_there_is_no_status_and_no_invite_people() {
        // `the account menu with Multiplayer off`: who you are, Settings and Sign out.
        let off = doors(&snap(json!({ "alpha": { "multiplayer": false }, "team": { "signedIn": true, "me": me(), "configured": false } })), 0.0).unwrap();
        assert_eq!(off, Doors { status: false, invisible: None, invite: false, team_page: false });
    }

    #[test]
    fn going_invisible_is_offered_only_while_rooms_are_live_and_know_the_person() {
        let now = wednesday();
        let with = |alpha: bool, on: bool, me_there: bool, until: Option<&str>| {
            let rooms = json!({ "on": on, "me": if me_there { json!({ "id": "u-1", "invisible_until": until }) } else { Value::Null } });
            doors(&snap(json!({ "alpha": { "rooms": alpha }, "team": { "signedIn": true, "me": me(), "rooms": rooms } })), now).unwrap().invisible
        };
        assert_eq!(with(false, true, true, None), None);
        assert_eq!(with(true, false, true, None), None);
        assert_eq!(with(true, true, false, None), None);
        assert_eq!(with(true, true, true, None), Some(false));
        // Invisible while the moment is ahead (`invisibleUntil > now`), and not once it has come.
        let ahead = "2099-01-01T00:00:00.000Z";
        assert_eq!(with(true, true, true, Some(ahead)), Some(true));
        assert_eq!(with(true, true, true, Some("2001-01-01T00:00:00.000Z")), Some(false));
        assert_eq!(with(true, true, true, Some("not a moment")), Some(false));
    }

    #[test]
    fn a_status_shows_with_how_long_it_holds_and_asks_again_once_it_ran_out() {
        let now = wednesday();
        let said_now = |status: Value| said(Some(&status), now);
        // `shows the status you set, with how long it holds`.
        let two_days = said_now(json!({ "text": "In meetings until Thursday", "until": now + 2.0 * DAY }));
        assert_eq!(status_row(two_days.as_ref(), now), "In meetings until Thursday · until Friday");
        // `asks again once a status has run out`.
        let gone = said_now(json!({ "text": "Away last week", "until": now - 60_000.0 }));
        assert_eq!(gone, None);
        assert_eq!(status_row(gone.as_ref(), now), STATUS_PROMPT);
        // `reads as nothing a second after, and still reads a second before`; gone on the moment.
        assert!(said_now(json!({ "text": "In meetings", "until": now + 1000.0 })).is_some());
        assert!(said_now(json!({ "text": "In meetings", "until": now })).is_none());
        assert!(said_now(json!({ "text": "In meetings", "until": now + 1.0 })).is_some());
        // Held for good with no moment; nothing when nothing was said.
        assert_eq!(said_now(json!({ "text": "In meetings", "until": null })), Some(("In meetings".to_string(), None)));
        assert_eq!(status_row(Some(&("In meetings".to_string(), None)), now), "In meetings");
        assert_eq!(said_now(json!({ "text": "  ", "until": now + DAY })), None);
        assert_eq!(said(None, now), None);
        // One line of eighty characters.
        let long = "word ".repeat(30);
        assert_eq!(said_now(json!({ "text": long, "until": null })).unwrap().0.chars().count(), STATUS_MAX);
    }

    #[test]
    fn how_long_it_holds_is_the_hour_today_a_weekday_inside_a_week_and_a_date_past_it() {
        let wed9 = Day::real(2026, 9, 7).unwrap().at(9, 0);
        let on = |month: i64, day: i64, h: i64, m: i64| Day::real(2026, month, day).unwrap().at(h, m);
        assert_eq!(holds_until(Some(on(9, 7, 18, 0)), wed9), "until 6pm today");
        assert_eq!(holds_until(Some(on(9, 7, 9, 30)), wed9), "until 9:30am today");
        assert_eq!(holds_until(Some(on(9, 12, 18, 0)), wed9), "until Monday");
        assert_eq!(holds_until(Some(on(9, 15, 18, 0)), wed9), "until Oct 15");
        assert_eq!(holds_until(None, wed9), "");
        assert_eq!(holds_until(Some(wed9 - 1.0), wed9), "");
    }

    #[test]
    fn the_sentence_says_when_it_ends_and_reopens_on_the_end_picked() {
        let wed = wednesday();
        // `offers the four presets in plain words`.
        assert_eq!(PRESETS.map(|(_, w)| w), ["the end of today", "tomorrow night", "the end of the week", "I clear it"]);
        assert_eq!(until_words(&End::Hold("today"), wed), "the end of today");
        assert_eq!(until_words(&End::Hold("open"), wed), "I clear it");
        assert_eq!(until_words(&End::Until(wed + 2.0 * DAY), wed), "Friday");
        assert_eq!(until_words(&End::Until(wed + 20.0 * DAY), wed), "Oct 27");
        // `reopens on the end you picked`.
        let x = |until: Option<f64>| ("x".to_string(), until);
        assert_eq!(end_of(Some(&x(hold_ends("week", wed))), wed), End::Hold("week"));
        assert_eq!(end_of(Some(&x(None)), wed), End::Hold("open"));
        assert_eq!(end_of(Some(&x(Some(wed + 9.0 * DAY))), wed), End::Until(wed + 9.0 * DAY));
        assert_eq!(end_of(None, wed), End::Hold("today"));
    }

    #[test]
    fn today_and_tomorrow_end_at_the_end_of_the_day_and_the_week_on_the_coming_sunday() {
        let wed = wednesday();
        let end = |id: &str| hold_ends(id, wed).map(|t| rune::time::LocalTime::at(t as i64));
        let today = end("today").unwrap();
        assert_eq!((today.day, today.hour, today.minute, today.second), (7, 23, 59, 59));
        assert_eq!(end("tomorrow").unwrap().day, 8);
        let sunday = end("week").unwrap();
        assert_eq!((sunday.day, Day::of(hold_ends("week", wed).unwrap()).weekday()), (11, 0));
        // On a Sunday "this week" is the next one, never tonight.
        let sun = Day::real(2026, 9, 11).unwrap().at(10, 0);
        assert_eq!(rune::time::LocalTime::at(hold_ends("week", sun).unwrap() as i64).day, 18);
        assert_eq!(hold_ends("open", wed), None);
    }

    #[test]
    fn a_time_typed_reads_ahead_or_not_at_all() {
        let wed = wednesday();
        let (at, _) = read_until("friday 5pm", wed).unwrap();
        let t = rune::time::LocalTime::at(at as i64);
        assert_eq!((Day::of(at).weekday(), t.hour), (5, 17));
        let (at, _) = read_until("in 3 days", wed).unwrap();
        assert_eq!(Day::of(at), Day::of(wed + 3.0 * DAY));
        assert!(at > wed + 2.0 * DAY);
        assert_eq!(read_until("banana", wed), None);
        assert_eq!(read_until("", wed), None);
    }

    #[test]
    fn a_status_is_written_with_its_hold_or_its_moment_and_cleared_as_nothing_held_open() {
        assert_eq!(status_payload("In meetings", &End::Hold("today"), false), json!({ "text": "In meetings", "hold": "today" }));
        assert_eq!(status_payload("On holiday", &End::Until(1_800_000_000_000.0), false), json!({ "text": "On holiday", "hold": "open", "until": 1_800_000_000_000.0 }));
        assert_eq!(status_payload("In meetings", &End::Hold("week"), true), json!({ "text": "", "hold": "open" }));
        // What is typed is kept to one line of eighty characters, a trailing space kept for the next word.
        assert_eq!(typed("in   meetings\n"), "in meetings ");
        assert_eq!(typed(&"x".repeat(90)).chars().count(), STATUS_MAX);
    }

    #[test]
    fn going_invisible_asks_the_rooms_for_so_many_minutes_and_coming_back_for_nothing() {
        assert_eq!(go_invisible(Some(60.0)), json!({ "op": "goInvisible", "args": [60.0] }));
        assert_eq!(go_invisible(None), json!({ "op": "goInvisible", "args": [null] }));
        assert_eq!(come_back(), json!({ "op": "comeBack", "args": [] }));
        let ten_pm = Day::real(2026, 9, 7).unwrap().at(22, 0);
        assert_eq!(minutes_to_midnight(ten_pm), 120.0);
        assert_eq!(minutes_to_midnight(Day::real(2026, 9, 8).unwrap().at(0, 0) - 1000.0), 1.0);
        assert_eq!(invisible_ends(ten_pm).map(|(w, m)| (w, m)), [("For an hour", Some(60.0)), ("Until tomorrow", Some(120.0)), ("Until I turn it off", None)]);
    }
}
