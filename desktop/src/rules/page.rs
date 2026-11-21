//! The Inbox page's own rules: when a thread was last sent, the order a tab draws its rows in,
//! how a time is said, and which Status word a row keeps under a tab. A port of the parts of
//! `renderer/src/threads/page-rules.ts` that one person's page reads, with the display the
//! page opens on (a list, sorted by priority, no filter).

use std::cmp::Ordering;

use rune::time::LocalTime;

use super::format::item_options;
use super::js::re;
use super::rank::{item_priority, place_score, placed_at, placed_score};
use crate::data::{Item, Lists, Priority, Tab};

/// When a person last wrote on the thread: the newest of the opening message, a reply, and
/// starting it from Later. An agent's write never moves it. A thread no person has written on
/// reads when it was made; a row with no record of who wrote what reads when it last changed.
pub fn sent_at(i: &Item) -> f64 {
    let updated = i.updated_at.unwrap_or(0.0);
    if i.object("wrote").is_none() {
        return updated;
    }
    let by = |field: &str| i.wrote(field).filter(|w| w.source != "agent").map_or(0.0, |w| w.at());
    let started = if i.string("start") == Some("now") { by("start") } else { 0.0 };
    let said = by("body").max(by("answer")).max(started);
    [said, i.created_at.unwrap_or(0.0)].into_iter().find(|t| *t != 0.0).unwrap_or(updated)
}

/// When a thread was finished: the moment its status was written as done, which a label or a
/// late note does not move. Any row that is not finished reads when it last changed.
pub fn finished_at(i: &Item) -> f64 {
    let closed = if i.status == "done" { i.wrote("status").and_then(|w| w.ts).filter(|ts| *ts != 0.0) } else { None };
    closed.or(i.updated_at).unwrap_or(0.0)
}

/// The value of the level a row shows: 9, 7, 5 or 2, all under a project's hundred.
fn level_value(i: &Item) -> f64 {
    match Priority::of(Some(item_priority(i))) {
        Priority::Urgent => 9.0,
        Priority::High => 7.0,
        Priority::Medium => 5.0,
        Priority::Low => 2.0,
    }
}

/// The one number Sort by Priority reads: the project's place plus the value of the level the
/// row shows, or the thread's own place, which replaces it.
pub fn rank_score(i: &Item, project_order: &[String]) -> f64 {
    placed_score(i, place_score(project_order, &i.product) + level_value(i))
}

/// A pinned thread comes first in every tab but Done, while it is open.
pub fn pinned_open(i: &Item) -> bool {
    i.truthy("pinned") && i.status != "done"
}

/// `b - a` as a sort reads it: the larger first, and a tie (or a number that is not one) left
/// to whatever comes next.
fn larger_first(a: f64, b: f64) -> Ordering {
    b.partial_cmp(&a).unwrap_or(Ordering::Equal)
}

/// The order a tab draws its rows in (`order`). Done is a history, newest finished first. In
/// progress runs newest sent first, where the person finds what they just sent. Needs you runs
/// by the project order, then the level each row shows, newest first inside a level. On a team
/// a person's thread comes before an agent's at one level; that would sit before the time.
fn order<'a>(tab: &'a str, project_order: &'a [String]) -> impl Fn(&&Item, &&Item) -> Ordering + 'a {
    move |a, b| match tab {
        "done" => larger_first(finished_at(a), finished_at(b)),
        "progress" => pinned_open(b).cmp(&pinned_open(a)).then_with(|| larger_first(sent_at(a), sent_at(b))),
        _ => pinned_open(b)
            .cmp(&pinned_open(a))
            .then_with(|| larger_first(rank_score(a, project_order), rank_score(b, project_order)))
            .then_with(|| larger_first(placed_at(a), placed_at(b))),
    }
}

/// The rows in the order the page draws them (`sorted`). The rows the app makes itself belong
/// to no project and stay at the top; rows that tie keep the order they came in.
pub fn sorted<'a>(rows: &[&'a Item], tab: &str, project_order: &[String]) -> Vec<&'a Item> {
    let (mut made, mut mine): (Vec<&Item>, Vec<&Item>) = rows.iter().partition(|r| r.product.is_empty());
    mine.sort_by(order(tab, project_order));
    made.append(&mut mine);
    made
}

/// Midnight before `now`, on this machine's clock.
fn start_of_day(now: f64) -> f64 {
    let clock = |ms: f64| LocalTime::at(ms as i64);
    let t = clock(now);
    let into_the_day = f64::from(t.hour * 3600 + t.minute * 60 + t.second) * 1000.0 + now.rem_euclid(1000.0);
    let midnight = now - into_the_day;
    // A day the clocks changed on is an hour longer or shorter than the fields say.
    midnight + f64::from(t.offset - clock(midnight).offset) * 1000.0
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A time in plain words: "just now", "6 min ago", "4 hours ago", "Yesterday", "Sep 28"
/// (`updatedWords`). The web app writes the date in the system's language; this writes it in
/// English, which is the only language the rest of the window speaks.
pub fn updated_words(ts: f64, now: f64) -> String {
    let min = ((now - ts) / 60_000.0).floor().max(0.0);
    if min < 1.0 {
        return "just now".into();
    }
    if min < 60.0 {
        return format!("{min} min ago");
    }
    let hours = (min / 60.0).floor();
    let today = start_of_day(now);
    if ts >= today {
        return format!("{hours} hour{} ago", if hours == 1.0 { "" } else { "s" });
    }
    if ts >= today - 86_400_000.0 {
        return "Yesterday".into();
    }
    let t = LocalTime::at(ts as i64);
    format!("{} {}", MONTHS[t.month as usize - 1], t.day)
}

/// The time column's heading: on Done it is when each thread was finished, on Later when it
/// comes back, and everywhere else when the person last wrote (`timeHeading`).
pub fn time_heading(tab: &str) -> &'static str {
    match tab {
        "done" => "Done",
        "snoozed" => "Back",
        _ => "Sent",
    }
}

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// A coming moment as a day and a clock: "Wed 8:00 AM" inside the coming six days, where the
/// weekday names one day, and "Thu, Nov 5, 8:00 AM" from seven days on or once the moment has
/// gone (`dayAndTime` in `renderer/src/format.ts`). The web app asks the system's language for
/// it; this is the English the rest of the window speaks, with a twelve-hour clock.
pub fn day_and_time(ts: f64, now: f64) -> String {
    let t = LocalTime::at(ts as i64);
    // The day each moment falls on, counted on its own clock, as two local midnights compare.
    let day_of = |ms: f64| ((ms / 1000.0).floor() as i64 + i64::from(LocalTime::at(ms as i64).offset)).div_euclid(86_400);
    let days = day_of(ts) - day_of(now);
    let weekday = DAYS[(day_of(ts) + 4).rem_euclid(7) as usize];
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    let clock = format!("{hour}:{:02} {}", t.minute, if t.hour < 12 { "AM" } else { "PM" });
    if (0..7).contains(&days) { format!("{weekday} {clock}") } else { format!("{weekday}, {} {}, {clock}", MONTHS[t.month as usize - 1], t.day) }
}

/// When a thread in Later comes back, for its time column: a schedule, or a reminder the
/// person set. One written down and never started has neither, and its cell is empty
/// (`backWords`). The old put-off times this window still honours are not asked here, as the
/// web window does not ask them either.
pub fn back_at(i: &Item) -> f64 {
    super::list::hidden_until(i, 0.0).max(i.run_at.unwrap_or(0.0))
}

pub fn back_words(i: &Item, now: f64) -> String {
    let at = back_at(i);
    if at > 0.0 { day_and_time(at, now) } else { String::new() }
}

/// Later is drawn once something is in it, or while it is the open tab: an empty tab on a
/// first day is one more thing to wonder about (`laterTabShown` in `renderer/src/later-tab.ts`).
/// The walkthrough keeps it drawn too, and this window has no walkthrough.
pub fn later_tab_shown(count: usize, open: bool) -> bool {
    count > 0 || open
}

/// The tabs the strip draws while `open` is the one on show: all four, less Later while
/// nothing is in it (`StateTabsRow` in `renderer/src/threads/Pages.tsx`, for one person).
pub fn tabs(lists: &Lists, open: Tab) -> Vec<Tab> {
    Tab::ALL.into_iter().filter(|t| *t != Tab::Later || later_tab_shown(lists.count(Tab::Later), open == Tab::Later)).collect()
}

/// The one line an empty tab carries (`emptyTabWords`). `waiting` is what Needs you holds,
/// because "Nothing is running." alone was read as a task having vanished. The web window's
/// empty Needs you is a page of its own, with a box to write in; this is its heading.
pub fn empty_tab_words(tab: &str, waiting: usize) -> String {
    match tab {
        "progress" => match waiting {
            0 => "Nothing is running.".to_string(),
            1 => "Nothing is running. One thread waits in Needs you.".to_string(),
            n => format!("Nothing is running. {n} threads wait in Needs you."),
        },
        "snoozed" => "Nothing is waiting for later. \"Add it to Later\" on a new thread writes one down without starting it.".to_string(),
        "done" => "Nothing closed yet. A finished thread waits in Needs you until you close it.".to_string(),
        _ => "Nothing needs you.".to_string(),
    }
}

/// The Status word a row draws under a tab, or none (`cellUnderTab`). The words a cell would
/// only repeat from the tab are dropped; in Needs you a thread waiting on the person says
/// Answered instead of nothing, so the row says why it is there. `from_people`: someone's
/// words hold it there, and those keep their word.
pub fn cell_under_tab(word: &str, tab: &str, from_people: bool, asks: bool) -> Option<String> {
    if tab == "inbox" && from_people {
        return Some(word.to_string());
    }
    let repeats = match tab {
        "inbox" => word == "Waiting on you" || word == "Waiting",
        "progress" => word == "In progress",
        _ => false,
    };
    if !repeats {
        return Some(word.to_string());
    }
    (tab == "inbox").then(|| if asks { "Waiting on you" } else { "Answered" }.to_string())
}

re!(OPTIONS_HEADING, r"^#{1,6}\s*options\b", "im");
re!(A_QUESTION, r#"\?(?=["'”’)\]*_`]*(\s|$))"#, "");

/// Whether an agent's last word asks anything: a question, or options to pick from
/// (`asksSomething` in `shared/teammates.mts`).
fn asks_something(text: &str) -> bool {
    OPTIONS_HEADING.test(text) || A_QUESTION.test(text)
}

/// The agent's newest word is a question with no options to pick, so only the person's words
/// answer it (`asksForYourWords`).
pub fn asks_for_your_words(i: &Item) -> bool {
    if !item_options(i).is_empty() {
        return false;
    }
    let said = if i.wrote_at("note") > i.wrote_at("result") { i.string("note") } else { i.result.as_deref() };
    asks_something(said.unwrap_or(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn sent_is_when_a_person_last_wrote() {
        let v = json!({ "createdAt": 100, "updatedAt": 900, "wrote": { "body": { "ts": 100, "source": "founder" }, "result": { "ts": 900, "source": "agent" } } });
        assert_eq!(sent_at(&item(v.clone())), 100.0);
        let mut replied = v.clone();
        replied["wrote"]["answer"] = json!({ "ts": 500, "source": "founder" });
        assert_eq!(sent_at(&item(replied)), 500.0);
        // A thread no person wrote on reads when it was made; one with no record, when it changed.
        let agents = json!({ "createdAt": 100, "updatedAt": 900, "wrote": { "body": { "ts": 300, "source": "agent" } } });
        assert_eq!(sent_at(&item(agents)), 100.0);
        assert_eq!(sent_at(&item(json!({ "createdAt": 100, "updatedAt": 900 }))), 900.0);
        // Starting it from Later counts only once it is started.
        let mut started = v;
        started["start"] = json!("now");
        started["wrote"]["start"] = json!({ "ts": 700, "source": "founder" });
        assert_eq!(sent_at(&item(started)), 700.0);
    }

    #[test]
    fn done_is_dated_by_its_close() {
        assert_eq!(finished_at(&item(json!({ "status": "done", "updatedAt": 900, "wrote": { "status": { "ts": 400, "source": "agent" } } }))), 400.0);
        assert_eq!(finished_at(&item(json!({ "status": "done", "updatedAt": 900 }))), 900.0);
        assert_eq!(finished_at(&item(json!({ "status": "open", "updatedAt": 900, "wrote": { "status": { "ts": 400, "source": "agent" } } }))), 900.0);
    }

    fn ids(rows: Vec<&Item>) -> Vec<&str> {
        rows.into_iter().map(|i| i.id.as_str()).collect()
    }

    #[test]
    fn needs_you_runs_by_project_then_level_then_newest() {
        let order = vec!["shop".to_string(), "desk".to_string()];
        let items: Vec<Item> = [
            json!({ "id": "desk-urgent", "product": "desk", "priority": 9, "updatedAt": 50 }),
            json!({ "id": "shop-low", "product": "shop", "priority": 2, "updatedAt": 60 }),
            json!({ "id": "shop-old", "product": "shop", "priority": 5, "updatedAt": 10 }),
            json!({ "id": "shop-new", "product": "shop", "priority": 0, "updatedAt": 70 }),
            // An agent's 8 and a person's 7 are one level, so the newer goes first.
            json!({ "id": "shop-eight", "product": "shop", "priority": 8, "updatedAt": 20 }),
            json!({ "id": "shop-seven", "product": "shop", "priority": 7, "updatedAt": 30 }),
            json!({ "id": "pinned", "product": "lab", "priority": 2, "updatedAt": 1, "pinned": true }),
            json!({ "id": "trouble", "product": "", "priority": 5, "updatedAt": 1 }),
            json!({ "id": "dragged", "product": "lab", "priority": 2, "updatedAt": 1, "place": 206.5 }),
        ]
        .into_iter()
        .map(item)
        .collect();
        let rows: Vec<&Item> = items.iter().collect();
        assert_eq!(ids(sorted(&rows, "inbox", &order)), ["trouble", "pinned", "shop-seven", "shop-eight", "dragged", "shop-new", "shop-old", "shop-low", "desk-urgent"]);
    }

    #[test]
    fn in_progress_runs_newest_sent_first_and_done_newest_finished_first() {
        let items: Vec<Item> = [
            json!({ "id": "a", "product": "shop", "status": "open", "createdAt": 10, "updatedAt": 90, "wrote": {} }),
            json!({ "id": "b", "product": "shop", "status": "open", "createdAt": 30, "updatedAt": 40, "wrote": {} }),
            json!({ "id": "c", "product": "shop", "status": "open", "createdAt": 20, "updatedAt": 50, "wrote": {}, "pinned": true }),
        ]
        .into_iter()
        .map(item)
        .collect();
        let rows: Vec<&Item> = items.iter().collect();
        assert_eq!(ids(sorted(&rows, "progress", &[])), ["c", "b", "a"]);
        assert_eq!(ids(sorted(&rows, "done", &[])), ["a", "c", "b"]);
    }

    #[test]
    fn a_time_is_said_in_plain_words() {
        // Noon on this machine's clock, whatever its zone, on a day far from a clock change.
        let noon = start_of_day(1_784_000_000_000.0) + 12.0 * 3_600_000.0;
        let ago = |min: f64| updated_words(noon - min * 60_000.0, noon);
        assert_eq!(ago(0.5), "just now");
        assert_eq!(ago(6.0), "6 min ago");
        assert_eq!(ago(60.0), "1 hour ago");
        assert_eq!(ago(11.0 * 60.0 + 59.0), "11 hours ago");
        assert_eq!(ago(13.0 * 60.0), "Yesterday");
        assert_eq!(ago(35.0 * 60.0), "Yesterday");
        let earlier = updated_words(noon - 40.0 * 3_600_000.0, noon);
        let t = LocalTime::at((noon - 40.0 * 3_600_000.0) as i64);
        assert_eq!(earlier, format!("{} {}", MONTHS[t.month as usize - 1], t.day));
        assert_eq!(updated_words(noon + 5_000.0, noon), "just now");
    }

    /// The day of the week a date falls on, worked out from the date alone (Sakamoto's rule),
    /// so the test does not lean on the arithmetic it checks.
    fn weekday(at: f64) -> &'static str {
        let t = LocalTime::at(at as i64);
        let (m, d) = (t.month as i32, t.day as i32);
        let y = if m < 3 { t.year - 1 } else { t.year };
        let shift = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4][m as usize - 1];
        DAYS[(y + y / 4 - y / 100 + y / 400 + shift + d).rem_euclid(7) as usize]
    }

    #[test]
    fn a_coming_moment_is_a_day_and_a_clock_and_a_far_one_carries_its_date() {
        const HOUR: f64 = 3_600_000.0;
        let noon = start_of_day(1_784_000_000_000.0) + 12.0 * HOUR;
        let at = |hours: f64| day_and_time(noon + hours * HOUR, noon);
        let named = |hours: f64| weekday(noon + hours * HOUR);
        assert_eq!(at(3.5), format!("{} 3:30 PM", named(0.0)));
        assert_eq!(at(12.0), format!("{} 12:00 AM", named(24.0)));
        assert_eq!(at(21.125), format!("{} 9:07 AM", named(24.0)));
        assert_eq!(at(24.0), format!("{} 12:00 PM", named(24.0)));
        // Inside the coming six days the weekday names one day. From seven on it would name
        // the wrong week, and a moment already gone is no day to come.
        assert_eq!(at(6.0 * 24.0), format!("{} 12:00 PM", named(6.0 * 24.0)));
        let far = LocalTime::at((noon + 7.0 * 24.0 * HOUR) as i64);
        assert_eq!(at(7.0 * 24.0), format!("{}, {} {}, 12:00 PM", named(7.0 * 24.0), MONTHS[far.month as usize - 1], far.day));
        let gone = LocalTime::at((noon - 13.0 * HOUR) as i64);
        assert_eq!(at(-13.0), format!("{}, {} {}, 11:00 PM", named(-13.0), MONTHS[gone.month as usize - 1], gone.day));
    }

    #[test]
    fn a_thread_in_later_says_when_it_comes_back() {
        const HOUR: f64 = 3_600_000.0;
        let noon = start_of_day(1_784_000_000_000.0) + 12.0 * HOUR;
        let back = |v: serde_json::Value| back_words(&item(v), noon);
        let tomorrow = day_and_time(noon + 24.0 * HOUR, noon);
        // Put off by the person, or by its agent: the schedule either way.
        assert_eq!(back(json!({ "runAt": noon + 24.0 * HOUR, "wrote": { "runAt": { "ts": 5, "source": "founder" } } })), tomorrow);
        assert_eq!(back(json!({ "runAt": noon + 24.0 * HOUR, "wrote": { "runAt": { "ts": 5, "source": "agent" } } })), tomorrow);
        // A reminder, and of the two the later moment.
        assert_eq!(back(json!({ "snoozedUntil": noon + 24.0 * HOUR, "wrote": { "snoozedUntil": { "ts": 5, "source": "founder" } } })), tomorrow);
        assert_eq!(back(json!({ "runAt": noon + HOUR, "snoozedUntil": noon + 24.0 * HOUR, "wrote": { "snoozedUntil": { "ts": 5, "source": "founder" } } })), tomorrow);
        // Written down and never started, or proposed: no moment, and no words.
        assert_eq!(back(json!({ "start": "later" })), "");
        assert_eq!(back(json!({ "parent": "w-0", "status": "open" })), "");
        assert_eq!(time_heading("snoozed"), "Back");
        assert_eq!((time_heading("done"), time_heading("inbox"), time_heading("progress")), ("Done", "Sent", "Sent"));
    }

    #[test]
    fn later_is_on_the_strip_only_while_something_is_in_it_or_it_is_open() {
        let labels = |lists: &Lists, open: Tab| tabs(lists, open).into_iter().map(Tab::label).collect::<Vec<_>>();
        let empty = Lists::default();
        assert_eq!(labels(&empty, Tab::Needs), ["Needs you", "In progress", "Done"]);
        assert_eq!(labels(&empty, Tab::Later), ["Needs you", "In progress", "Later", "Done"]);
        assert!(later_tab_shown(1, false) && later_tab_shown(0, true) && !later_tab_shown(0, false));
        assert_eq!(empty_tab_words("snoozed", 3), "Nothing is waiting for later. \"Add it to Later\" on a new thread writes one down without starting it.");
        assert_eq!(empty_tab_words("progress", 0), "Nothing is running.");
        assert_eq!(empty_tab_words("progress", 1), "Nothing is running. One thread waits in Needs you.");
        assert_eq!(empty_tab_words("progress", 4), "Nothing is running. 4 threads wait in Needs you.");
    }

    #[test]
    fn a_word_the_tab_already_says_is_not_said_again() {
        assert_eq!(cell_under_tab("Waiting", "inbox", false, false).as_deref(), Some("Answered"));
        assert_eq!(cell_under_tab("Waiting", "inbox", false, true).as_deref(), Some("Waiting on you"));
        assert_eq!(cell_under_tab("Waiting on you", "inbox", true, false).as_deref(), Some("Waiting on you"));
        assert_eq!(cell_under_tab("Done", "inbox", false, false).as_deref(), Some("Done"));
        assert_eq!(cell_under_tab("In progress", "progress", false, false), None);
        assert_eq!(cell_under_tab("Queued", "progress", false, false).as_deref(), Some("Queued"));
        assert_eq!(cell_under_tab("Waiting", "done", false, false).as_deref(), Some("Waiting"));
    }

    #[test]
    fn a_question_with_no_options_asks_for_words() {
        let asks = |result: &str| asks_for_your_words(&item(json!({ "result": result, "wrote": { "result": { "ts": 5, "source": "agent" } } })));
        assert!(asks("Which clinic should go first?"));
        assert!(asks("I can do either. *Which one?*"));
        assert!(!asks("The export is fixed."));
        assert!(!asks("See https://example.com/a?b=1 for the log."));
        assert!(!asks("Which one?\n\n## Options\n1. The first (recommended)\n2. The second"));
        // A newer checkpoint is the agent's newest word.
        let noted = item(json!({ "result": "Which clinic first?", "note": "Started on Riverside.", "wrote": { "result": { "ts": 5, "source": "agent" }, "note": { "ts": 9, "source": "agent" } } }));
        assert!(!asks_for_your_words(&noted));
    }
}
