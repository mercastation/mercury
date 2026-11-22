//! The cloud button in the Inbox's head strip: whether it is there at all, what it wears, and
//! its menu: on or off, what on means, the runs the cloud could take, the accounts, the month's
//! use and the line at its foot. A port of `CloudButton` and of the line in `HeaderActions`
//! that decides it is drawn (`renderer/src/threads/Pages.tsx`), with `renderer/src/cloud-menu.ts`
//! (its words and numbers, ⌘C's target and note) and the menu's part of
//! `renderer/src/cloud-setup.ts` (`menuSetupLine`, `brokenAccounts`, `buttonAsksSignIn`,
//! `seenAfter`). What it writes is the web window's own calls (`HeaderActions`' props in
//! `renderer/src/App.tsx`).

use rune::time::LocalTime;
use serde::Serialize;
use serde_json::{Value, json};

use super::js;
use crate::data::truthy;

/// The word a button that is not a bare square carries beside its cloud.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Word {
    /// "Cloud", until one account can run there: nothing else says what the icon is.
    Cloud,
    /// "Sign in again", while an account has stopped working and nobody has looked.
    Again,
}

impl Word {
    pub fn said(self) -> &'static str {
        match self {
            Word::Cloud => "Cloud",
            Word::Again => "Sign in again",
        }
    }
}

/// What the button wears (the classes `lit`, `count` and `named` and what is inside it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Button {
    /// The cloud is on and an account can run there: the cloud and its count are orange.
    pub lit: bool,
    /// How many threads run in the cloud now, while any do and nothing is being asked.
    pub count: Option<u64>,
    pub word: Option<Word>,
    /// The small orange dot in its corner: something about the setup wants a look.
    pub dot: bool,
}

fn list<'a>(cloud: &'a Value, field: &str) -> &'a [Value] {
    cloud.get(field).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn text<'a>(v: &'a Value, field: &str) -> &'a str {
    v.get(field).and_then(Value::as_str).unwrap_or("")
}

/// The accounts that stopped working, as `engine:key` (`brokenAccounts`).
fn broken_accounts(cloud: &Value) -> Vec<String> {
    list(cloud, "accounts").iter().filter(|a| text(a, "state") == "broken").map(|a| format!("{}:{}", text(a, "engine"), text(a, "key"))).collect()
}

/// A kind of account, by name (`kindName` in `renderer/src/cloud-setup.ts`).
fn kind_name(engine: &str) -> &'static str {
    match engine {
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        _ => "Claude",
    }
}

/// What the button at the end of the menu's line about the setup does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Go {
    /// "Try again", on an account that was not added: its sign-in starts again, then the setup opens.
    TryAgain,
    /// "Fix", on an account that stopped working: the setup opens.
    Fix,
}

impl Go {
    pub fn said(self) -> &'static str {
        match self {
            Go::TryAgain => "Try again",
            Go::Fix => "Fix",
        }
    }
}

/// The menu's one line about the setup (`menuSetupLine`): an account that did not get added,
/// with the way to try again; one being added, with nothing to press while it is; or one that
/// stopped working.
#[derive(Clone, Debug, PartialEq)]
pub struct SetupLine {
    pub say: String,
    pub go: Option<Go>,
}

/// The sign-in of an account that was not added, the one Try again starts again. One that failed
/// is said whether or not its account is among those already there.
fn failed_sign_in(cloud: &Value) -> Option<&Value> {
    list(cloud, "signIns").iter().find(|s| text(s, "state") == "failed")
}

pub fn setup_line(cloud: &Value) -> Option<SetupLine> {
    if let Some(failed) = failed_sign_in(cloud) {
        let why = failed.get("message").and_then(Value::as_str).filter(|m| !m.is_empty()).map_or(String::new(), |m| format!(" {m}"));
        return Some(SetupLine { say: format!("Your {} account wasn’t added.{why}", kind_name(text(failed, "engine"))), go: Some(Go::TryAgain) });
    }
    if let Some(adding) = list(cloud, "signIns").iter().find(|s| text(s, "state") == "checking") {
        return Some(SetupLine { say: format!("Adding your {} account…", kind_name(text(adding, "engine"))), go: None });
    }
    (!broken_accounts(cloud).is_empty()).then(|| SetupLine { say: BROKEN.to_string(), go: Some(Go::Fix) })
}

/// What the menu, the button's title and its word say of an account that stopped working.
pub const BROKEN: &str = "A cloud account needs to sign in again.";

/// The button for a snapshot's `cloud`, or none on a Mac that is not set up for the cloud and
/// is not offered it. `seen` is the stopped accounts the menu was already opened on
/// (`zero.cloudBrokenSeen`).
pub fn button(cloud: Option<&Value>, seen: &[String]) -> Option<Button> {
    let cloud = cloud.filter(|c| truthy(c.get("setUp")) || truthy(c.get("offered")))?;
    // How many run there, while any do (`cloudButtonCount`).
    let count = cloud.get("running").and_then(Value::as_f64).filter(|n| *n > 0.0).map(|n| n as u64);
    let nobody_ready = nobody_ready(cloud);
    // A stopped account is said in words until the menu has been opened on it.
    let asks = !nobody_ready && broken_accounts(cloud).iter().any(|k| !seen.contains(k));
    Some(Button {
        lit: is_on(cloud) && !nobody_ready,
        count: count.filter(|_| !asks),
        word: if asks {
            Some(Word::Again)
        } else if count.is_none() && nobody_ready {
            Some(Word::Cloud)
        } else {
            None
        },
        dot: asks || (count.is_none() && (nobody_ready || setup_line(cloud).is_some())),
    })
}

/// On is anything but "this Mac only" (`cloudIsOn`).
pub fn is_on(cloud: &Value) -> bool {
    cloud.get("mode").and_then(Value::as_str) != Some("mac")
}

/// The invitation is for a cloud with no account at all, and none on its way.
pub fn nobody_ready(cloud: &Value) -> bool {
    list(cloud, "accounts").is_empty() && list(cloud, "signIns").is_empty()
}

/* -------------------------------- the menu -------------------------------- */

/// Where threads run: this Mac only, or the cloud when it is on and this Mac is full, or the
/// cloud first for everything (`CloudMode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Mac,
    Full,
    All,
}

impl Mode {
    /// The engine's own word for it (`cloudRuns`).
    pub fn id(self) -> &'static str {
        match self {
            Mode::Mac => "mac",
            Mode::Full => "full",
            Mode::All => "all",
        }
    }
}

/// Where threads run now (`mode`).
pub fn mode(cloud: &Value) -> Mode {
    match cloud.get("mode").and_then(Value::as_str) {
        Some("mac") => Mode::Mac,
        Some("all") => Mode::All,
        _ => Mode::Full,
    }
}

/// What on means, picked on the menu's second line (`CLOUD_WHEN_ON`), with what it does.
pub const WHEN_ON: [(Mode, &str, &str); 2] = [
    (Mode::Full, "When this Mac is full", "The cloud takes what won’t fit, and all when you’re away."),
    (Mode::All, "Everything", "Threads start in the cloud first, wherever they can."),
];

/// What on means while it is off, kept so turning it back on goes where it went before.
pub fn when_on(cloud: &Value) -> Mode {
    match cloud.get("whenOn").and_then(Value::as_str) {
        Some("all") => Mode::All,
        Some("full") => Mode::Full,
        _ if cloud.get("mode").and_then(Value::as_str) == Some("all") => Mode::All,
        _ => Mode::Full,
    }
}

/// Where ⌘C takes the cloud: off from either kind of on, back on to what on means
/// (`cloudToggleTarget`).
pub fn toggle_target(cloud: &Value) -> Mode {
    if is_on(cloud) { Mode::Mac } else { when_on(cloud) }
}

/// The line after ⌘C, naming where new threads go now (`cloudToggleNote`).
pub fn toggle_note(mode: Mode) -> &'static str {
    match mode {
        Mode::Mac => "Cloud off: new threads run on this Mac",
        Mode::All => "Cloud on: everything starts in the cloud",
        Mode::Full => "Cloud on: threads that don’t fit here start in the cloud",
    }
}

/// What turning the cloud off by ⌘C asks first, as it was pressed by accident (`CLOUD_OFF_ASK`):
/// its heading, its line, and its two buttons.
pub const OFF_ASK: [&str; 4] = ["Turn the cloud off?", "New threads and replies will run on this Mac.", "Keep it on", "Turn off"];

/// The line about the runs on this Mac the cloud could take now (`cloudMoveLine`): with
/// Everything they move by themselves and the line says so; when this Mac is full it offers to
/// move them. None with none, or with the cloud off.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveLine {
    pub say: String,
    pub go: Option<&'static str>,
}

pub fn move_line(mode: Mode, movable: usize) -> Option<MoveLine> {
    if movable == 0 || mode == Mode::Mac {
        return None;
    }
    let threads = format!("{movable} thread{}", if movable == 1 { "" } else { "s" });
    if mode == Mode::All {
        return Some(MoveLine { say: format!("Moving {threads} to the cloud"), go: None });
    }
    Some(MoveLine { say: format!("{threads} could run in the cloud"), go: Some(if movable == 1 { "Move it" } else { "Move them" }) })
}

/// The accounts that count: one added twice counts once (`countedAccounts`).
fn counted(cloud: &Value) -> impl Iterator<Item = &Value> {
    list(cloud, "accounts").iter().filter(|a| text(a, "state") != "duplicate")
}

/// "None yet", "1 account", "3 accounts" (`SETUP.menuAccounts`).
pub fn accounts_line(cloud: &Value) -> String {
    match counted(cloud).count() {
        0 => "None yet".to_string(),
        1 => "1 account".to_string(),
        n => format!("{n} accounts"),
    }
}

/// A number of hours as the web window writes it: to a tenth, with no ".0" (`hours`).
fn hours(n: f64) -> String {
    format!("{}", (n * 10.0).round() / 10.0)
}

/// This month's use, never a price (`cloudMonthLine`): Agentbox Cloud's own count once its
/// server has said it (`plan`, `zero:cloud-account-plan`), else this Mac's ledger of its runs.
pub fn month_line(cloud: &Value, plan: Option<&Value>) -> String {
    const NOTHING: &str = "Nothing in the cloud yet this month";
    if let Some(used) = plan.filter(|p| truthy(p.get("signedIn"))).and_then(|p| p.get("usedSeconds")).and_then(Value::as_f64) {
        return if used > 0.0 { format!("{} agent-hours", hours(used / 3600.0)) } else { NOTHING.to_string() };
    }
    let month = cloud.get("month");
    if !truthy(month.and_then(|m| m.get("runs"))) {
        return NOTHING.to_string();
    }
    format!("{} agent-hours", hours(month.and_then(|m| m.get("agentHours")).and_then(Value::as_f64).unwrap_or(0.0)))
}

/// An hour as the menu's foot says it, "3:05pm" (`clock` in `cloud-menu.ts`).
fn clock(ts: f64) -> String {
    let t = LocalTime::at(ts as i64);
    let hour = if t.hour.is_multiple_of(12) { 12 } else { t.hour % 12 };
    format!("{hour}:{:02}{}", t.minute, if t.hour < 12 { "am" } else { "pm" })
}

/// The hour a limit lifts, with its weekday when that is not today, as a weekly limit's is
/// (`dayAndHour`): "3pm", "Mon 3:30pm".
fn day_and_hour(ts: f64, now: f64) -> String {
    let hour = clock(ts).replacen(":00", "", 1);
    let (t, n) = (LocalTime::at(ts as i64), LocalTime::at(now as i64));
    if (t.year, t.month, t.day) == (n.year, n.month, n.day) {
        return hour;
    }
    let day_of = |ms: f64| ((ms / 1000.0).floor() as i64 + i64::from(LocalTime::at(ms as i64).offset)).div_euclid(86_400);
    let weekday = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][(day_of(ts) + 4).rem_euclid(7) as usize];
    format!("{weekday} {hour}")
}

/// An agent by its name in a sentence (`agentName`).
fn agent_name(engine: &str) -> &'static str {
    match engine {
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        _ => "Claude Code",
    }
}

/// The menu's last line (`cloudFootLine`): what runs in the cloud, or why threads run on this
/// Mac instead: the cloud resting after its runs failed, an image too old for this Mac's agent,
/// this Mac's Claude account not signed in there, or the account it runs on at its limit.
pub fn foot_line(cloud: &Value, now: f64) -> String {
    let number = |field: &str| cloud.get(field).and_then(Value::as_f64);
    if text(cloud, "health") == "resting"
        && let Some(until) = number("restingUntil").filter(|u| *u > now)
    {
        let why = cloud.get("lastError").and_then(Value::as_str).filter(|e| !e.is_empty()).unwrap_or("its runs failed before they started");
        return format!("The cloud is failing ({}). Threads run on this Mac until {}.", js::slice(why, 0, 120), clock(until));
    }
    let gap = cloud.get("versionGap").filter(|g| g.is_object());
    let rebuild = |g: &Value| format!("The cloud's {} is {}, this Mac's is {}: rebuild the cloud image", agent_name(text(g, "engine")), text(g, "cloud"), text(g, "mac"));
    if let Some(g) = gap.filter(|g| truthy(g.get("blocks"))) {
        return format!("{}. {} threads run on this Mac until then.", rebuild(g), if text(g, "engine") == "codex" { "Codex" } else { "Claude" });
    }
    let on = is_on(cloud);
    if on && list(cloud, "notInCloud").iter().any(|e| e.as_str() == Some("claude")) {
        return "The Claude account this Mac uses is not signed in to the cloud, so threads run on this Mac. Sign it in to run them there.".to_string();
    }
    if on
        && let Some(moved) = cloud.get("switched").filter(|m| m.get("until").and_then(Value::as_f64).is_some_and(|u| u > now))
    {
        let until = moved.get("until").and_then(Value::as_f64).unwrap_or(0.0);
        return format!("{} is at its limit until {}, so threads run on {}.", text(moved, "from"), day_and_hour(until, now), text(moved, "to"));
    }
    // One account runs in the cloud, the one the app marks in use (`cloudAccountInUse`).
    let limited = counted(cloud).find(|a| text(a, "engine") == "claude" && truthy(a.get("inUse"))).and_then(|a| a.get("limitedUntil")).and_then(Value::as_f64).filter(|u| *u > now);
    if on && let Some(until) = limited {
        return format!("The Claude account the cloud runs on is at its usage limit until {}. Threads run on this Mac until then.", clock(until));
    }
    if let Some(g) = gap {
        return rebuild(g);
    }
    match number("running").filter(|n| *n > 0.0) {
        Some(n) => format!("{n} running in the cloud now"),
        None => "Nothing running in the cloud now".to_string(),
    }
}

/// The ids of the runs on this Mac the cloud could take now (`movable`).
pub fn movable(cloud: &Value) -> Vec<String> {
    list(cloud, "movable").iter().filter_map(Value::as_str).map(str::to_string).collect()
}

/// What the stopped accounts already looked at are once the menu has shown `looked`
/// (`seenAfter`): an account that works again, or is gone, drops out, so it asks again if it
/// stops later. With no accounts known, nothing is dropped.
pub fn seen_after(cloud: &Value, seen: &[String], looked: &[String]) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for k in seen.iter().chain(looked) {
        if !all.contains(k) {
            all.push(k.clone());
        }
    }
    if list(cloud, "accounts").is_empty() {
        return all;
    }
    let broken = broken_accounts(cloud);
    all.into_iter().filter(|k| broken.contains(k)).collect()
}

/// The accounts that stopped working, as the menu shows them while it is open.
pub fn looked_at(cloud: &Value) -> Vec<String> {
    broken_accounts(cloud)
}

/// A write of the cloud's own settings: where threads run (`cloudRuns`), and what on means
/// (`cloudWhenOn`), as `HeaderActions`' `onCloudMode` and `onCloudWhenOn` write them.
pub fn set_mode(mode: Mode) -> (&'static str, Value) {
    ("zero:settings-set-workspace", json!({ "key": "cloudRuns", "value": mode.id() }))
}

pub fn set_when_on(mode: Mode) -> (&'static str, Value) {
    ("zero:settings-set-workspace", json!({ "key": "cloudWhenOn", "value": mode.id() }))
}

/// The runs to carry to the cloud as it has room (`onCloudMove`, `api.moveToCloud(ids)`).
pub fn move_runs(ids: &[String]) -> (&'static str, Value) {
    ("zero:move-to-cloud", json!({ "ids": ids }))
}

/// What the window says once the engine has a move (`moveToCloud` with `said`): how many go, or
/// that the cloud took none.
pub fn moved_line(moving: usize) -> String {
    match moving {
        0 => super::place::CLOUD_CANNOT_TAKE.to_string(),
        1 => "Moving this thread to the cloud. It keeps working until the cloud has room.".to_string(),
        n => format!("Moving {n} threads to the cloud. They keep working until the cloud has room."),
    }
}

/// Try again on an account that was not added: its sign-in is let go and started again, on the
/// same account when it is one of the accounts already there (`retry` in `CloudButton`).
pub fn sign_in_again(cloud: &Value) -> Vec<(&'static str, Value)> {
    let Some(failed) = failed_sign_in(cloud) else {
        return Vec::new();
    };
    let (engine, key) = (text(failed, "engine"), text(failed, "key"));
    let mine = list(cloud, "accounts").iter().any(|a| text(a, "key") == key);
    vec![
        ("zero:cloud-signin", json!({ "action": "cancel", "engine": engine, "key": key })),
        ("zero:cloud-signin", json!({ "action": "start", "engine": engine, "key": if mine { Value::from(key) } else { Value::Null } })),
    ]
}

/// The invitation a cloud with no account at all opens on (`SETUP.invite*`): its heading, its
/// line and its one button.
pub const INVITE: [&str; 3] = ["Run your agents in the cloud", "They keep working with your laptop closed, and as many run at once as you need.", "Set up the cloud"];

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn worn(cloud: Value) -> Option<Button> {
        button(Some(&cloud), &[])
    }

    const PLAIN: Button = Button { lit: false, count: None, word: None, dot: false };

    #[test]
    fn there_is_no_button_on_a_mac_the_cloud_was_never_offered_to() {
        assert_eq!(button(None, &[]), None);
        assert_eq!(worn(json!({ "setUp": false, "mode": "full", "running": 0 })), None);
        assert_eq!(worn(json!({})), None);
        assert!(worn(json!({ "setUp": false, "offered": true, "mode": "mac", "running": 0 })).is_some());
    }

    #[test]
    fn it_is_orange_while_the_cloud_is_on_and_counts_what_runs_there() {
        let account = json!([{ "engine": "claude", "key": "a-1", "ready": true, "state": "ready" }]);
        let cloud = |mode: &str, running: u64| json!({ "setUp": true, "mode": mode, "running": running, "accounts": account, "signIns": [] });
        assert_eq!(worn(cloud("full", 0)), Some(Button { lit: true, ..PLAIN }));
        assert_eq!(worn(cloud("all", 3)), Some(Button { lit: true, count: Some(3), ..PLAIN }));
        // Off: a plain square, and what was already running there is still counted.
        assert_eq!(worn(cloud("mac", 0)), Some(PLAIN));
        assert_eq!(worn(cloud("mac", 2)), Some(Button { count: Some(2), ..PLAIN }));
    }

    #[test]
    fn it_says_cloud_until_an_account_is_set_up() {
        let invited = Some(Button { word: Some(Word::Cloud), dot: true, ..PLAIN });
        assert_eq!(worn(json!({ "setUp": false, "offered": true, "mode": "full", "running": 0 })), invited);
        // On with nobody to run it is not on.
        assert_eq!(worn(json!({ "setUp": true, "mode": "all", "running": 0, "accounts": [], "signIns": [] })), invited);
        // One on its way is no longer nobody, and the dot says the menu has a line about it.
        let adding = json!({ "setUp": true, "mode": "full", "running": 0, "accounts": [], "signIns": [{ "engine": "codex", "key": "a-2", "state": "checking" }] });
        assert_eq!(worn(adding), Some(Button { lit: true, dot: true, ..PLAIN }));
        let waiting = json!({ "setUp": true, "mode": "full", "running": 0, "accounts": [], "signIns": [{ "engine": "codex", "key": "a-2", "state": "needs-code" }] });
        assert_eq!(worn(waiting), Some(Button { lit: true, ..PLAIN }));
    }

    #[test]
    fn it_asks_for_a_sign_in_until_the_stopped_account_has_been_looked_at() {
        let cloud = json!({ "setUp": true, "mode": "full", "running": 2, "signIns": [],
            "accounts": [{ "engine": "claude", "key": "a-1", "ready": true, "state": "ready" }, { "engine": "codex", "key": "a-2", "ready": false, "state": "broken" }] });
        // The ask takes the count's place.
        assert_eq!(button(Some(&cloud), &[]), Some(Button { lit: true, word: Some(Word::Again), dot: true, ..PLAIN }));
        // Looked at: the count is back, and no dot while there is a number to read.
        let seen = ["codex:a-2".to_string()];
        assert_eq!(button(Some(&cloud), &seen), Some(Button { lit: true, count: Some(2), ..PLAIN }));
        let mut idle = cloud.clone();
        idle["running"] = json!(0);
        assert_eq!(button(Some(&idle), &seen), Some(Button { lit: true, dot: true, ..PLAIN }));
        assert_eq!(Word::Again.said(), "Sign in again");
    }

    /// A moment on this machine's clock, on the day the web window's own cases use.
    fn at(hour: u32, minute: u32) -> f64 {
        // Midnight of 16 October 2026 here: noon UTC that day, less the hours past midnight here.
        let noon = 1_792_152_000_000.0;
        let t = LocalTime::at(noon as i64);
        let midnight = noon - f64::from(t.hour * 3600 + t.minute * 60 + t.second) * 1000.0;
        midnight + f64::from(hour * 3600 + minute * 60) * 1000.0
    }

    /// The web window's own account (`the-cloud-menu-says-when-its-accounts-are-at-their-limit`).
    fn account(key: &str, limited_until: Option<f64>) -> Value {
        json!({ "engine": "claude", "key": key, "name": null, "ready": limited_until.is_none(), "state": if limited_until.is_some() { "limited" } else { "ready" }, "limitedUntil": limited_until, "inUse": key == "a" })
    }

    fn cloud_of(accounts: Vec<Value>, running: u64, mode: &str) -> Value {
        json!({ "setUp": true, "mode": mode, "whenOn": "all", "running": running, "accounts": accounts, "signIns": [], "month": { "runs": 0, "agentHours": 0 } })
    }

    #[test]
    fn the_foot_says_what_runs_there_or_why_threads_run_on_this_mac() {
        let now = at(20, 13);
        let limited = "The Claude account the cloud runs on is at its usage limit until 10:10pm. Threads run on this Mac until then.";
        assert_eq!(foot_line(&cloud_of(vec![account("a", Some(at(22, 10))), account("b", Some(at(22, 40)))], 0, "all"), now), limited);
        assert_eq!(foot_line(&cloud_of(vec![account("a", None), account("b", Some(at(22, 40)))], 2, "all"), now), "2 running in the cloud now");
        assert_eq!(foot_line(&cloud_of(vec![account("a", None)], 0, "all"), now), "Nothing running in the cloud now");
        assert_eq!(foot_line(&cloud_of(vec![account("a", Some(at(22, 10))), account("b", None)], 2, "all"), now), limited);
        // A limit gone by is forgotten, and none is said with the cloud off.
        assert_eq!(foot_line(&cloud_of(vec![account("a", Some(at(20, 0)))], 0, "all"), now), "Nothing running in the cloud now");
        assert_eq!(foot_line(&cloud_of(vec![account("a", Some(at(22, 10)))], 0, "mac"), now), "Nothing running in the cloud now");
        // An image older than this Mac's agent, that stops its threads or only says so.
        let mut base = cloud_of(vec![account("a", None)], 2, "all");
        base["versionGap"] = json!({ "engine": "claude", "cloud": "2.1", "mac": "2.3", "blocks": false });
        assert_eq!(foot_line(&base, now), "The cloud's Claude Code is 2.1, this Mac's is 2.3: rebuild the cloud image");
        base["versionGap"] = json!({ "engine": "codex", "cloud": "0.9", "mac": "1.0", "blocks": true });
        assert_eq!(foot_line(&base, now), "The cloud's Codex is 0.9, this Mac's is 1.0: rebuild the cloud image. Codex threads run on this Mac until then.");
        base["versionGap"] = Value::Null;
        assert_eq!(foot_line(&base, now), "2 running in the cloud now");
        // Resting after its runs failed, this Mac's Claude not signed in there, a move off a limit.
        let mut resting = base.clone();
        resting["health"] = json!("resting");
        resting["restingUntil"] = json!(at(21, 5));
        assert_eq!(foot_line(&resting, now), "The cloud is failing (its runs failed before they started). Threads run on this Mac until 9:05pm.");
        let mut away = base.clone();
        away["notInCloud"] = json!(["claude"]);
        assert!(foot_line(&away, now).starts_with("The Claude account this Mac uses is not signed in to the cloud"));
        let mut moved = base;
        moved["switched"] = json!({ "from": "one@example.com", "to": "two@example.com", "until": at(22, 0) });
        assert_eq!(foot_line(&moved, now), "one@example.com is at its limit until 10pm, so threads run on two@example.com.");
        moved["switched"]["until"] = json!(at(22, 0) + 3.0 * 86_400_000.0);
        assert!(foot_line(&moved, now).contains(" is at its limit until ") && foot_line(&moved, now).contains(" 10pm, so"), "{}", foot_line(&moved, now));
    }

    #[test]
    fn the_months_use_is_the_clouds_own_count_once_its_server_has_said_it() {
        let mine = json!({ "month": { "runs": 40, "agentHours": 400.24 } });
        assert_eq!(month_line(&mine, Some(&json!({ "signedIn": true, "usedSeconds": 510.4 * 3600.0 }))), "510.4 agent-hours");
        assert_eq!(month_line(&mine, Some(&json!({ "signedIn": true, "usedSeconds": 12 * 3600 }))), "12 agent-hours");
        assert_eq!(month_line(&mine, Some(&json!({ "signedIn": true, "usedSeconds": 0 }))), "Nothing in the cloud yet this month");
        for plan in [None, Some(json!({ "signedIn": false })), Some(json!({ "signedIn": true }))] {
            assert_eq!(month_line(&mine, plan.as_ref()), "400.2 agent-hours");
        }
        assert_eq!(month_line(&json!({ "month": { "runs": 0, "agentHours": 0 } }), None), "Nothing in the cloud yet this month");
        assert_eq!(month_line(&json!({ "month": { "runs": 1, "agentHours": 0.5 } }), None), "0.5 agent-hours");
    }

    #[test]
    fn the_menu_offers_to_move_what_the_cloud_could_take_and_counts_the_accounts() {
        assert_eq!(move_line(Mode::All, 1), Some(MoveLine { say: "Moving 1 thread to the cloud".to_string(), go: None }));
        assert_eq!(move_line(Mode::All, 3).unwrap().say, "Moving 3 threads to the cloud");
        assert_eq!(move_line(Mode::All, 0), None);
        assert_eq!(move_line(Mode::Full, 2), Some(MoveLine { say: "2 threads could run in the cloud".to_string(), go: Some("Move them") }));
        assert_eq!(move_line(Mode::Full, 1).unwrap().go, Some("Move it"));
        assert_eq!(move_line(Mode::Mac, 2), None);
        let accounts = |states: &[&str]| json!({ "accounts": states.iter().map(|s| json!({ "engine": "claude", "key": s, "state": s })).collect::<Vec<_>>() });
        assert_eq!(accounts_line(&accounts(&[])), "None yet");
        assert_eq!(accounts_line(&accounts(&["ready", "duplicate"])), "1 account");
        assert_eq!(accounts_line(&accounts(&["ready", "broken", "limited"])), "3 accounts");
        assert_eq!(moved_line(0), "The cloud can’t take this thread right now");
        assert_eq!(moved_line(2), "Moving 2 threads to the cloud. They keep working until the cloud has room.");
        assert_eq!(move_runs(&["w-1".to_string()]), ("zero:move-to-cloud", json!({ "ids": ["w-1"] })));
    }

    #[test]
    fn command_c_turns_it_off_or_back_on_to_what_on_means() {
        let cloud = |mode: &str, when: Option<&str>| json!({ "setUp": true, "mode": mode, "whenOn": when });
        assert_eq!(toggle_target(&cloud("full", Some("full"))), Mode::Mac);
        assert_eq!(toggle_target(&cloud("all", Some("all"))), Mode::Mac);
        assert_eq!(toggle_target(&cloud("mac", Some("all"))), Mode::All);
        assert_eq!(toggle_target(&cloud("mac", Some("full"))), Mode::Full);
        assert_eq!(toggle_target(&cloud("mac", None)), Mode::Full);
        // What on means, kept while off; read off the mode where none was kept.
        assert_eq!((when_on(&cloud("all", None)), when_on(&cloud("full", None))), (Mode::All, Mode::Full));
        assert_eq!(toggle_note(Mode::Mac), "Cloud off: new threads run on this Mac");
        assert_eq!(toggle_note(Mode::Full), "Cloud on: threads that don’t fit here start in the cloud");
        assert_eq!(set_mode(Mode::Mac), ("zero:settings-set-workspace", json!({ "key": "cloudRuns", "value": "mac" })));
        assert_eq!(set_when_on(Mode::All), ("zero:settings-set-workspace", json!({ "key": "cloudWhenOn", "value": "all" })));
    }

    #[test]
    fn the_setup_line_says_what_was_not_added_what_is_on_its_way_and_what_stopped() {
        let ready = json!({ "engine": "claude", "key": "a-1", "ready": true, "state": "ready" });
        let failed = json!({ "setUp": true, "mode": "full", "accounts": [ready], "signIns": [{ "engine": "codex", "key": "a-2", "state": "failed", "message": "The code had expired." }] });
        assert_eq!(setup_line(&failed), Some(SetupLine { say: "Your Codex account wasn’t added. The code had expired.".to_string(), go: Some(Go::TryAgain) }));
        // Try again lets that sign-in go and starts a new one, not on an account already there.
        assert_eq!(sign_in_again(&failed), vec![
            ("zero:cloud-signin", json!({ "action": "cancel", "engine": "codex", "key": "a-2" })),
            ("zero:cloud-signin", json!({ "action": "start", "engine": "codex", "key": null })),
        ]);
        let again = json!({ "accounts": [ready, { "engine": "codex", "key": "a-2", "state": "broken" }], "signIns": [{ "engine": "codex", "key": "a-2", "state": "failed", "message": null }] });
        assert_eq!(setup_line(&again).unwrap().say, "Your Codex account wasn’t added.");
        assert_eq!(sign_in_again(&again)[1].1["key"], json!("a-2"));
        let adding = json!({ "accounts": [], "signIns": [{ "engine": "antigravity", "key": "a-3", "state": "checking" }] });
        assert_eq!(setup_line(&adding), Some(SetupLine { say: "Adding your Antigravity account…".to_string(), go: None }));
        let broken = json!({ "accounts": [{ "engine": "codex", "key": "a-2", "state": "broken" }], "signIns": [] });
        assert_eq!(setup_line(&broken), Some(SetupLine { say: BROKEN.to_string(), go: Some(Go::Fix) }));
        assert_eq!(setup_line(&json!({ "accounts": [ready], "signIns": [] })), None);
    }

    #[test]
    fn an_account_looked_at_stays_seen_until_it_works_again() {
        let cloud = |accounts: Value| json!({ "accounts": accounts });
        let c = cloud(json!([{ "engine": "claude", "key": "a1", "state": "ready" }, { "engine": "claude", "key": "a2", "state": "broken" }, { "engine": "codex", "key": "a3", "state": "broken" }]));
        assert_eq!(seen_after(&c, &[], &["claude:a2".to_string(), "codex:a3".to_string()]), ["claude:a2", "codex:a3"]);
        let fixed = cloud(json!([{ "engine": "claude", "key": "a2", "state": "ready" }, { "engine": "codex", "key": "a3", "state": "broken" }]));
        assert_eq!(seen_after(&fixed, &["claude:a2".to_string(), "codex:a3".to_string()], &[]), ["codex:a3"]);
        assert_eq!(seen_after(&cloud(json!([])), &["claude:a2".to_string()], &[]), ["claude:a2"]);
        assert_eq!(seen_after(&json!({}), &["claude:a2".to_string()], &[]), ["claude:a2"]);
        assert_eq!(looked_at(&c), ["claude:a2", "codex:a3"]);
    }
}
