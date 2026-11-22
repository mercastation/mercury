//! Settings' rules, ported from the web window's: the pages its menu holds and what its search
//! finds (`renderer/src/settings-search.ts`), the words its rows say about accounts, usage, the
//! cloud, the version and the plans (`account-words.ts`, `accounts-once.ts`, `cloud-menu.ts`,
//! `running-reading.ts`, `update-row.ts`, `cloud-upgrade.ts`), and the order of projects
//! (`project-priority.ts`, `shared/rank.mts`). Every function names the TypeScript it follows;
//! the tests are the web's own cases where it has them.

use rune::time::LocalTime;
use serde_json::Value;

use crate::data::truthy;

/// The app's name, in the middle of a sentence (`NAME` in `shared/product-name.mjs`).
pub const NAME: &str = "Mercury";
/// What this computer is called in a sentence (`MACHINE` in `renderer/src/files/files-shown.ts`).
pub const MACHINE: &str = "Mac";

/* ----------------------------------- pages ---------------------------------- */

/// One page of Settings (`SettingsPageId`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageId {
    General,
    Appearance,
    Shortcuts,
    Phone,
    Alpha,
    Claude,
    Codex,
    Antigravity,
    Running,
    Instructions,
    Projects,
    Files,
    Team,
    Rooms,
    Cloud,
}

/// The headings the menu is drawn under, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Personal,
    Agents,
    Workspace,
    Billing,
}

impl Group {
    pub const ALL: [Group; 4] = [Group::Personal, Group::Agents, Group::Workspace, Group::Billing];
    pub fn label(self) -> &'static str {
        match self {
            Group::Personal => "Personal",
            Group::Agents => "Agents",
            Group::Workspace => "Workspace",
            Group::Billing => "Billing",
        }
    }
}

/// The menu, in the order it is drawn (`SETTINGS_PAGES`).
pub const PAGES: [PageId; 15] = [
    PageId::General,
    PageId::Appearance,
    PageId::Shortcuts,
    PageId::Phone,
    PageId::Alpha,
    PageId::Claude,
    PageId::Codex,
    PageId::Antigravity,
    PageId::Running,
    PageId::Instructions,
    PageId::Projects,
    PageId::Files,
    PageId::Team,
    PageId::Rooms,
    PageId::Cloud,
];

impl PageId {
    /// The id the web window names it by, which the bridge to it carries.
    pub fn id(self) -> &'static str {
        match self {
            PageId::General => "general",
            PageId::Appearance => "appearance",
            PageId::Shortcuts => "shortcuts",
            PageId::Phone => "phone",
            PageId::Alpha => "alpha",
            PageId::Claude => "claude",
            PageId::Codex => "codex",
            PageId::Antigravity => "antigravity",
            PageId::Running => "running",
            PageId::Instructions => "instructions",
            PageId::Projects => "projects",
            PageId::Files => "files",
            PageId::Team => "team",
            PageId::Rooms => "rooms",
            PageId::Cloud => "cloud",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            PageId::General => "General",
            PageId::Appearance => "Appearance",
            PageId::Shortcuts => "Shortcuts",
            PageId::Phone => "Phone",
            PageId::Alpha => "Alpha features",
            PageId::Claude => "Claude Code",
            PageId::Codex => "Codex",
            PageId::Antigravity => "Antigravity",
            PageId::Running => "Running",
            PageId::Instructions => "Instructions",
            PageId::Projects => "Projects",
            PageId::Files => "Files",
            PageId::Team => "Team",
            PageId::Rooms => "Rooms",
            PageId::Cloud => "Plan and Billing",
        }
    }
    pub fn group(self) -> Group {
        match self {
            PageId::General | PageId::Appearance | PageId::Shortcuts | PageId::Phone | PageId::Alpha => Group::Personal,
            PageId::Claude | PageId::Codex | PageId::Antigravity | PageId::Running | PageId::Instructions => Group::Agents,
            PageId::Projects | PageId::Files | PageId::Team | PageId::Rooms => Group::Workspace,
            PageId::Cloud => Group::Billing,
        }
    }
    pub fn from_id(id: &str) -> Option<PageId> {
        PAGES.into_iter().find(|p| p.id() == id)
    }
    /// Whether this window draws the page itself. The rest are the web window's, brought
    /// forward on that page when their row is pressed: a theme's pictures, the list of keys,
    /// the instructions' editor and a team's pages are not used here, and are not cheap to draw.
    pub fn native(self) -> bool {
        !matches!(self, PageId::Appearance | PageId::Shortcuts | PageId::Instructions | PageId::Team | PageId::Rooms)
    }
}

/// The pages this Mac has (`pages` in `Settings.tsx`): Team only on a build with a team,
/// Plan and Billing only where the cloud is offered, Rooms only while Rooms is on.
pub fn pages_shown(team: bool, cloud_plans: bool, rooms: bool) -> Vec<PageId> {
    PAGES
        .into_iter()
        .filter(|p| match p {
            PageId::Team => team,
            PageId::Cloud => cloud_plans,
            PageId::Rooms => rooms,
            _ => true,
        })
        .collect()
}

/* ---------------------------------- search ---------------------------------- */

/// One thing the search can land on (`Entry`): `anchor` is the group it scrolls to, `words`
/// what a person might type that is not in the name.
struct Entry {
    page: PageId,
    label: &'static str,
    anchor: Option<&'static str>,
    words: &'static str,
}

const fn entry(page: PageId, label: &'static str, anchor: Option<&'static str>, words: &'static str) -> Entry {
    Entry { page, label, anchor, words }
}

/// `ENTRIES`, row for row.
const ENTRIES: &[Entry] = &[
    entry(PageId::General, "Your account", Some("account"), "signed in sign out log out logout email"),
    entry(PageId::General, "Public links", Some("public-links"), "share shared sharing link links public web stop"),
    entry(PageId::General, "Show keyboard shortcut hints", Some("keys"), "keys hints"),
    entry(PageId::General, "Shrink the sidebar to icons", Some("sidebar"), "sidebar narrow collapse rail icons"),
    entry(PageId::General, "Counts and crash reports", Some("privacy"), "privacy diagnostics analytics telemetry sends data"),
    entry(PageId::General, "Where things are kept", Some("storage"), "folder store path storage files disk"),
    entry(PageId::General, "Check for updates", Some("updates"), "update updates upgrade new version release restart newest current move applications folder disk image read-only stuck"),
    entry(PageId::Appearance, "Theme", Some("theme"), "dark light colour color look background picture wallpaper"),
    entry(PageId::Appearance, "Blur and darkness", Some("tune"), "tint dim glass transparency"),
    entry(PageId::Shortcuts, "Keyboard shortcuts", None, "keys hotkeys keybindings"),
    entry(PageId::Phone, "Use it on your phone", Some("phone"), "phone mobile qr code scan remote tailscale wifi home screen app"),
    entry(PageId::Alpha, "Rooms", Some("alpha"), "alpha beta early try team teammates friends multiplayer people chat"),
    entry(PageId::Rooms, "Show my full name", Some("rooms-name"), "name last surname initial"),
    entry(PageId::Rooms, "What you are working on", Some("rooms-card"), "card thread private wall"),
    entry(PageId::Rooms, "Your activity on your page", Some("rooms-card"), "activity board inbox profile page hide"),
    entry(PageId::Rooms, "Go invisible", Some("rooms-invisible"), "hide hidden away offline ghost"),
    entry(PageId::Claude, "Usage", Some("usage"), "claude code limits plan left remaining quota"),
    entry(PageId::Claude, "Signed in", Some("usage"), "claude code account accounts login subscription add email this computer local"),
    entry(PageId::Claude, "Cloud accounts", Some("cloud-accounts"), "claude code cloud remote accounts limit"),
    entry(PageId::Claude, "Keys for cloud threads", Some("cloud-keys"), "keys key token tokens secret secrets api saved replace remove supabase stripe vercel sentry expo aws github"),
    entry(PageId::Claude, "Permission mode", Some("permissions"), "claude code permissions bypass ask allow plan accept edits"),
    entry(PageId::Codex, "Usage", Some("usage"), "codex limits plan left remaining quota"),
    entry(PageId::Codex, "Signed in", Some("usage"), "codex account accounts login subscription add email this computer local"),
    entry(PageId::Codex, "Cloud accounts", Some("cloud-accounts"), "codex cloud remote accounts limit"),
    entry(PageId::Codex, "Permission mode", Some("permissions"), "codex permissions sandbox read only full access"),
    entry(PageId::Antigravity, "Signed in", Some("usage"), "antigravity agy gemini google account login sign in"),
    entry(PageId::Files, "Keep files in the cloud", Some("files-cloud"), "storage disk space full upload backup films videos pictures images"),
    entry(PageId::Files, "Take big files off this computer", Some("files-cloud"), "mac days free space disk storage old films videos pictures"),
    entry(PageId::Cloud, "Your plan", Some("cloud-billing"), "agentbox cloud billing stripe payment invoices invoice receipts card cancel subscription manage plan account"),
    entry(PageId::Cloud, "This month", Some("cloud-use"), "cloud usage hours agents running used left"),
    entry(PageId::Cloud, "Change plan", Some("cloud-plans"), "agentbox cloud plan plans pro plus max slots upgrade downgrade switch price pay subscription"),
    entry(PageId::Running, "How many run at once", Some("at-once"), "agents concurrent parallel sessions limit queue capacity automatic"),
    entry(PageId::Running, "Hold heavy work when memory is short", Some("at-once"), "ram tests builds slow memory"),
    entry(PageId::Running, "Stop programs agents leave behind", Some("at-once"), "leftovers left running finished dev servers background jobs processes cleanup kill ram memory"),
    entry(PageId::Running, "Something felt slow", Some("at-once"), "slow fewer agents throttle struggling"),
    entry(PageId::Running, "Agents you started yourself", Some("outside"), "terminal outside sessions inbox your own"),
    entry(PageId::Running, "Bring in new chats", Some("outside"), "import chats conversations automatic"),
    entry(PageId::Instructions, "Instructions for every agent", None, "rules prompt claude.md how agents write to you"),
    entry(PageId::Projects, "Projects", None, "order priority archive new project"),
    entry(PageId::Team, "Team", None, "invite people members teammates"),
];

/// Where a hit opens: a page, or a project's own page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pane {
    Page(PageId),
    Project(String),
}

/// A search result (`SettingsHit`): where it opens, the name drawn in it, the page's name under
/// it, and the group it lands on.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub pane: Pane,
    pub label: String,
    pub page_label: &'static str,
    pub anchor: Option<&'static str>,
}

/// The words of a text as the search reads them: lower case, split on everything that is not
/// a letter, a digit or a dot (`words`).
fn words(s: &str) -> Vec<String> {
    s.to_lowercase().split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.')).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

/// Every typed word is the start of some word in the text (`covers`).
fn covers(have: &[String], want: &[String]) -> bool {
    want.iter().all(|w| have.iter().any(|h| h.starts_with(w.as_str())))
}

/// What the search finds for `query` among the pages this Mac has and its projects, the hits on
/// a name first (`searchSettings`).
pub fn search(query: &str, pages: &[PageId], projects: &[(String, String)]) -> Vec<Hit> {
    let want = words(query);
    if want.is_empty() {
        return Vec::new();
    }
    let (mut named, mut hidden) = (Vec::new(), Vec::new());
    for e in ENTRIES.iter().filter(|e| pages.contains(&e.page)) {
        let page = e.page.label();
        let hit = Hit { pane: Pane::Page(e.page), label: e.label.to_string(), page_label: page, anchor: e.anchor };
        if covers(&words(&format!("{} {page}", e.label)), &want) {
            named.push(hit);
        } else if covers(&words(&format!("{} {page} {}", e.label, e.words)), &want) {
            hidden.push(hit);
        }
    }
    if pages.contains(&PageId::Projects) {
        for (slug, name) in projects {
            if covers(&words(name), &want) {
                named.push(Hit { pane: Pane::Project(slug.clone()), label: name.clone(), page_label: "Projects", anchor: None });
            }
        }
    }
    named.extend(hidden);
    named
}

/* ------------------------------- JSON, read short ------------------------------ */

pub fn text<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}
pub fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}
pub fn on(v: &Value, key: &str) -> bool {
    truthy(v.get(key))
}
pub fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/* --------------------------------- how many --------------------------------- */

/// "9 agents", "1 agent".
fn agents(n: f64) -> String {
    format!("{} {}", n, if n == 1.0 { "agent" } else { "agents" })
}

/// The menu behind "How many run at once": Automatic with the number it settled on, then every
/// number for the whole computer (`agentCountOptions`). Each is `(value, label)`.
pub fn agent_count_options(w: &Value) -> Vec<(String, String)> {
    let manual = w.get("agentsAuto") == Some(&Value::Bool(false));
    let auto = manual.then(|| num(w, "agentsAutoTotal")).flatten().or_else(|| num(w, "agentsTotal")).or_else(|| num(w, "capacity")).unwrap_or(1.0);
    let mut options = vec![("auto".to_string(), format!("Automatic ({})", agents(auto)))];
    let most = num(w, "slotsMax").unwrap_or(12.0) as i64;
    options.extend((1..=most).map(|n| (n.to_string(), agents(n as f64))));
    options
}

/* ---------------------------------- time words --------------------------------- */

/// A moment to the nearest minute (`toMinute`).
fn to_minute(at: f64) -> f64 {
    (at / 60_000.0).round() * 60_000.0
}

const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// "5:10pm", "8pm": the clock as the usage rows print it (`hourWords`).
pub fn hour_words(ts: f64) -> String {
    let t = LocalTime::at(to_minute(ts) as i64);
    let hour = if t.hour % 12 == 0 { 12 } else { t.hour % 12 };
    let minute = if t.minute == 0 { String::new() } else { format!(":{:02}", t.minute) };
    format!("{hour}{minute}{}", if t.hour < 12 { "am" } else { "pm" })
}

/// The day a reset falls on: none today, "Thu" this week, "Tue Oct 27" six or more days out
/// (`resetDay` in `shared/usage.mts`).
pub fn reset_day(at: f64, now: f64) -> Option<String> {
    let day = crate::rules::calendar::Day::of(to_minute(at));
    if day == crate::rules::calendar::Day::of(now) {
        return None;
    }
    let weekday = WEEKDAYS[day.weekday().rem_euclid(7) as usize];
    if at - now < 6.0 * 86_400_000.0 {
        return Some(weekday.to_string());
    }
    let t = LocalTime::at(to_minute(at) as i64);
    Some(format!("{weekday} {} {}", MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day))
}

/// When a limit starts over, in words, or none for a moment gone by or never told
/// (`resetWords`).
pub fn reset_words(limit: &Value, now: f64) -> Option<String> {
    if let Some(at) = num(limit, "resetsAt").filter(|at| *at != 0.0) {
        if at <= now {
            return None;
        }
        return Some(match reset_day(at, now) {
            None => hour_words(at),
            Some(day) => format!("{day}{} {}", if day.contains(' ') { "," } else { "" }, hour_words(at)),
        });
    }
    let said = text(limit, "resetsText")?;
    Some(match text(limit, "resetsOn") {
        Some(on) => format!("{on}, {said}"),
        None => said.to_string(),
    })
}

/// One meter of an account (`Meter`): `percent` is none for a window that has started over
/// since it was read.
#[derive(Clone, Debug, PartialEq)]
pub struct Meter {
    pub key: String,
    pub word: String,
    pub percent: Option<f64>,
    pub resets: Option<String>,
}

fn finite(v: &Value, key: &str) -> bool {
    num(v, key).is_some_and(f64::is_finite)
}

/// The meters every account shows: this session, this week, each model's own week, and Codex's
/// other windows, each with when it starts over (`meterPair`).
pub fn meter_pair(limits: &[Value], now: f64) -> Vec<Meter> {
    let span = |l: &Value| text(l, "span").unwrap_or("").to_string();
    let qualifier = |l: &Value| text(l, "qualifier").map(str::to_string);
    let session = limits.iter().find(|l| span(l) == "session");
    let week = limits.iter().find(|l| span(l) == "week" && qualifier(l).is_none_or(|q| q.to_lowercase().contains("all"))).or_else(|| limits.iter().find(|l| span(l) == "week"));
    let meter = |key: String, word: String, l: &Value| {
        let over = on(l, "reset") || num(l, "resetsAt").is_some_and(|at| at != 0.0 && at <= now);
        Meter { key, word, percent: if over { None } else { num(l, "percent").map(f64::round) }, resets: if over { Some("Reset".to_string()) } else { reset_words(l, now) } }
    };
    let mut out = Vec::new();
    if let Some(s) = session.filter(|s| finite(s, "percent") || on(s, "reset")) {
        out.push(meter("session".into(), "Session".into(), s));
    }
    if let Some(w) = week.filter(|w| finite(w, "percent")) {
        out.push(meter("week".into(), "Week".into(), w));
    }
    for l in limits.iter().filter(|l| span(l) == "week" && !week.is_some_and(|w| std::ptr::eq(*l, w)) && qualifier(l).is_some() && finite(l, "percent")) {
        let q = qualifier(l).unwrap_or_default();
        out.push(meter(format!("week:{q}"), q, l));
    }
    for l in limits.iter().filter(|l| ["month", "window"].contains(&span(l).as_str()) && finite(l, "percent")) {
        let word = if span(l) == "month" { "Month".to_string() } else { text(l, "name").unwrap_or("").to_string() };
        out.push(meter(span(l), word, l));
    }
    out
}

/// "Checked 4 minutes ago" (`usageCheckedAt`).
pub fn usage_checked_at(at: Option<f64>, now: f64) -> String {
    let Some(at) = at.filter(|a| a.is_finite()) else {
        return String::new();
    };
    let minutes = ((now - at) / 60_000.0).floor().max(0.0) as i64;
    if minutes == 0 {
        return "Checked just now".to_string();
    }
    if minutes < 60 {
        return format!("Checked {minutes} {} ago", if minutes == 1 { "minute" } else { "minutes" });
    }
    let hours = minutes / 60;
    format!("Checked {hours} {} ago", if hours == 1 { "hour" } else { "hours" })
}

/* ---------------------------------- the cloud ---------------------------------- */

/// The cloud's logins that count: one added twice counts once (`countedAccounts`).
pub fn counted(accounts: &[Value]) -> Vec<&Value> {
    accounts.iter().filter(|a| text(a, "state") != Some("duplicate")).collect()
}

/// A login's name: its address, or the agent's word, numbered when there are several
/// (`accountLabel`, `accountName` in `shared/account-names.mts`).
pub fn account_label(a: &Value, all: &[&Value]) -> String {
    if let Some(email) = text(a, "name") {
        return email.to_string();
    }
    let engine = text(a, "engine").unwrap_or("claude");
    let same: Vec<&&Value> = all.iter().filter(|x| text(x, "engine").unwrap_or("claude") == engine).collect();
    let word = match engine {
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        _ => "Claude",
    };
    let index = same.iter().position(|x| std::ptr::eq(**x, a)).unwrap_or(0);
    if same.len() > 1 { format!("{word} account {}", index + 1) } else { format!("{word} account") }
}

/// The one login the cloud runs an agent on (`cloudAccountInUse`).
pub fn cloud_account_in_use<'a>(accounts: &'a [Value], engine: &str) -> Option<&'a Value> {
    counted(accounts).into_iter().find(|a| text(a, "engine") == Some(engine) && on(a, "inUse"))
}

/// When threads go to the cloud, and on which account (`cloudRunsLine`).
pub fn cloud_runs_line(mode: &str, in_use: Option<&str>) -> String {
    if mode == "mac" {
        return "Every thread runs on this computer.".to_string();
    }
    let when = if mode == "all" { "Threads start there first" } else { "Threads start there when this computer is full" };
    format!("{when}, on {}.", in_use.unwrap_or("your cloud account"))
}

/// What turning the cloud on or off writes (`cloudToggleTarget`): off when it is on, else back
/// to what "on" meant.
pub fn cloud_toggle_target(cloud: &Value) -> &'static str {
    if text(cloud, "mode").is_some_and(|m| m != "mac") {
        return "mac";
    }
    if text(cloud, "whenOn") == Some("all") { "all" } else { "full" }
}

/// "The same as on this computer." (`sameAsHere`).
pub const SAME_AS_HERE: &str = "The same as on this computer.";

/* ------------------------------- the account tiles ------------------------------ */

/// A login on this computer, as an agent's page reads it (`HereLogin`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HereLogin {
    pub profile: String,
    pub email: Option<String>,
    pub plan: Option<String>,
    pub chosen: bool,
    pub signed_in: bool,
    pub trouble: Option<String>,
    pub at_limit: bool,
    pub limit_at: Option<f64>,
    pub limit_resets_at: Option<f64>,
    pub dir: Option<String>,
}

/// Claude Code's logins here, off the settings' `workspace.accounts` (`agentStory`).
pub fn claude_logins(w: &Value) -> Vec<HereLogin> {
    list(w, "accounts")
        .iter()
        .map(|a| {
            let trouble = a.get("trouble").filter(|t| t.is_object());
            HereLogin {
                profile: text(a, "profile").unwrap_or("default").to_string(),
                email: text(a, "email").map(str::to_string),
                plan: a.get("plan").and_then(|p| text(p, "label")).map(str::to_string),
                chosen: on(a, "chosen"),
                signed_in: text(a, "email").is_some(),
                trouble: trouble.and_then(|t| text(t, "note")).map(str::to_string),
                at_limit: trouble.and_then(|t| text(t, "cause")) == Some("at-limit"),
                limit_at: trouble.and_then(|t| num(t, "at")),
                limit_resets_at: trouble.and_then(|t| num(t, "resetsAt")),
                dir: text(a, "dir").map(str::to_string),
            }
        })
        .collect()
}

/// Codex's logins here, off `workspace.codex.accounts`.
pub fn codex_logins(w: &Value) -> Vec<HereLogin> {
    w.get("codex")
        .map(|c| list(c, "accounts"))
        .unwrap_or(&[])
        .iter()
        .map(|a| HereLogin {
            profile: text(a, "profile").unwrap_or("default").to_string(),
            email: text(a, "email").map(str::to_string),
            plan: text(a, "plan").map(str::to_string),
            chosen: on(a, "chosen"),
            signed_in: on(a, "signedIn"),
            ..HereLogin::default()
        })
        .collect()
}

/// Each account once, wherever it is signed in, with one reading (`OneAccount`).
#[derive(Clone, Debug, PartialEq)]
pub struct OneAccount {
    pub key: String,
    pub name: String,
    pub plan: Option<String>,
    pub here: Option<HereLogin>,
    pub cloud: Option<Value>,
    pub limits: Option<Vec<Value>>,
    pub checked_at: Option<f64>,
    pub usage_error: Option<String>,
}

/// A reading's windows in the order `readingsDisagree` keys them.
fn window_key(l: &Value) -> String {
    let span = text(l, "span").unwrap_or("");
    let q = text(l, "qualifier");
    let all = span == "week" && q.is_none_or(|q| q.to_lowercase().starts_with("all"));
    format!("{span}:{}", if all { "all".to_string() } else { q.unwrap_or("").to_string() })
}

/// Two readings taken close together whose resets cannot both be true (`readingsDisagree`).
fn readings_disagree(a: (&[Value], f64), b: (&[Value], f64)) -> bool {
    const SAME_MOMENT_MS: f64 = 15.0 * 60_000.0;
    const SAME_RESET_MS: f64 = 2.0 * 60_000.0;
    if (a.1 - b.1).abs() > SAME_MOMENT_MS {
        return false;
    }
    let later = a.1.max(b.1);
    let reset = |limits: &[Value], key: &str| limits.iter().find(|l| window_key(l) == key).and_then(|l| num(l, "resetsAt")).filter(|r| r.is_finite());
    ["session:", "week:all"].iter().any(|key| match (reset(a.0, key), reset(b.0, key)) {
        (Some(p), Some(q)) => p > later && q > later && (p - q).abs() > SAME_RESET_MS,
        _ => false,
    })
}

/// A login's own reading here, or the snapshot's when that is of the same login; none when
/// main could not confirm whose its reading was (`accountReadingFor`).
fn reading_for(profile: &str, readings: &[Value], current: Option<&Value>) -> Option<(Vec<Value>, f64)> {
    if readings.iter().any(|r| text(r, "profile") == Some(profile) && on(r, "unconfirmed")) {
        return None;
    }
    let own = readings.iter().find(|r| text(r, "profile") == Some(profile) && !list(r, "limits").is_empty() && num(r, "at").is_some_and(|a| a != 0.0)).map(|r| (list(r, "limits").to_vec(), num(r, "at").unwrap_or(0.0)));
    let snap = current.filter(|c| text(c, "profile").unwrap_or("default") == profile).map(|c| (list(c, "limits").to_vec(), num(c, "at").unwrap_or(0.0)));
    [own, snap].into_iter().flatten().max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Every account once: the one in use here first, then the rest of this computer's in their
/// order, then the cloud's signed in nowhere here (`accountsOnce`). `cloud` is the counted
/// logins of this agent, oldest first.
pub fn accounts_once(here: &[HereLogin], cloud: &[Value], readings: &[Value], current: Option<&Value>) -> Vec<OneAccount> {
    let mut paired: Vec<(String, usize)> = Vec::new();
    for (at, c) in cloud.iter().enumerate() {
        if let Some(p) = text(c, "profile") {
            if here.iter().any(|h| h.profile == p) && !paired.iter().any(|(q, _)| q == p) {
                paired.push((p.to_string(), at));
            }
        }
    }
    let picked = here.iter().any(|h| h.chosen);
    let mut mine: Vec<&HereLogin> = here.iter().collect();
    // A stable sort: the chosen one first, the rest as they were.
    mine.sort_by_key(|h| !(picked && h.chosen));
    let cloud_refs: Vec<&Value> = cloud.iter().collect();
    let one = |h: Option<&HereLogin>, c: Option<&Value>| -> OneAccount {
        let local = h.and_then(|h| reading_for(&h.profile, readings, current));
        let remote = c.filter(|c| !list(c, "limits").is_empty() && num(c, "readAt").is_some_and(|a| a != 0.0)).map(|c| (list(c, "limits").to_vec(), num(c, "readAt").unwrap_or(0.0)));
        let from_cloud = match (&remote, &local) {
            (Some(r), None) => !r.0.is_empty(),
            (Some(r), Some(l)) => r.1 > l.1 || readings_disagree((&l.0, l.1), (&r.0, r.1)),
            _ => false,
        };
        let shown = if from_cloud { remote.clone() } else { local.clone() };
        let local_error = h.and_then(|h| readings.iter().find(|r| text(r, "profile") == Some(h.profile.as_str())).and_then(|r| text(r, "error")).map(str::to_string));
        OneAccount {
            key: h.map_or_else(|| format!("cloud:{}", c.and_then(|c| text(c, "key")).unwrap_or("")), |h| h.profile.clone()),
            name: h.and_then(|h| h.email.clone()).or_else(|| c.map(|c| account_label(c, &cloud_refs))).unwrap_or_else(|| "Not signed in".to_string()),
            plan: h.filter(|h| h.signed_in).and_then(|h| h.plan.clone()).or_else(|| c.and_then(|c| text(c, "plan")).map(str::to_string)),
            here: h.cloned(),
            cloud: c.cloned(),
            limits: shown.as_ref().map(|s| s.0.clone()),
            checked_at: shown.as_ref().map(|s| s.1),
            usage_error: if from_cloud || (local.is_none() && h.is_none()) { c.and_then(|c| text(c, "readError")).map(str::to_string) } else { local_error },
        }
    };
    let mut out: Vec<OneAccount> = mine.iter().map(|h| one(Some(h), paired.iter().find(|(p, _)| *p == h.profile).map(|(_, at)| &cloud[*at]))).collect();
    for (at, c) in cloud.iter().enumerate() {
        if !paired.iter().any(|(_, used)| *used == at) {
            out.push(one(None, Some(c)));
        }
    }
    out
}

/// The same for Codex, whose cloud login names its email and no folder (`codexAccountsOnce`).
pub fn codex_accounts_once(here: &[HereLogin], cloud: &[Value], readings: &[Value], current: Option<&Value>) -> Vec<OneAccount> {
    let matched: Vec<Value> = cloud
        .iter()
        .map(|c| {
            let mut c = c.clone();
            if text(&c, "profile").is_none() {
                let by_email = text(&c, "name").and_then(|n| here.iter().find(|h| h.email.as_deref() == Some(n))).map(|h| Value::String(h.profile.clone()));
                if let (Some(fields), Some(p)) = (c.as_object_mut(), by_email) {
                    fields.insert("profile".into(), p);
                }
            }
            c
        })
        .collect();
    let in_use = here.iter().find(|h| h.chosen).map_or_else(|| "default".to_string(), |h| h.profile.clone());
    let current = current.map(|c| {
        let mut c = c.clone();
        if let Some(fields) = c.as_object_mut() {
            fields.insert("profile".into(), Value::String(in_use.clone()));
        }
        c
    });
    accounts_once(here, &matched, readings, current.as_ref())
}

/// Where an account is signed in and working, a tag each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Here,
    Cloud,
}

/// What a tile's button or link does.
#[derive(Clone, Debug, PartialEq)]
pub enum TileAct {
    /// Switch to this login (`activeAccount`), by its profile.
    Use(String),
    /// Sign this agent in on this computer.
    SignIn,
    /// Sign the account in to the cloud: a login's key to sign it in again, or none to add it.
    CloudSignIn(Option<String>),
}

/// One account as a tile draws it (`PlaceAccount`).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Tile {
    pub key: String,
    pub name: String,
    pub plan: Option<String>,
    pub about: Vec<String>,
    pub in_use: bool,
    pub limits: Option<Vec<Value>>,
    pub checked_at: Option<f64>,
    pub usage_error: Option<String>,
    /// The button in its corner, with its word.
    pub use_: Option<(String, TileAct)>,
    pub places: Vec<Place>,
    /// The cloud sign-in it is missing, a link after its places.
    pub cloud_act: Option<(String, TileAct)>,
    pub trouble: Option<String>,
    pub cmd: Option<String>,
    /// Whether usage can be read at all: no "no reading yet" is promised where it cannot.
    pub meters: bool,
    /// What the meters say before there is a reading.
    pub none: Option<String>,
}

/// An account's one tile (`accountTile`): `many` is whether this computer has more than one
/// login, `cloud` whether a cloud is set up, `cloud_sign_in` whether a tile may offer one.
pub fn account_tile(a: &OneAccount, many: bool, now: f64, cloud: bool, cloud_sign_in: bool) -> Tile {
    let h = a.here.as_ref();
    let c = a.cloud.as_ref();
    let broken = c.is_some_and(|c| text(c, "state") == Some("broken"));
    let signing_in = h.is_some_and(|h| !h.signed_in);
    let mut places = Vec::new();
    if cloud && !signing_in {
        if h.is_some() {
            places.push(Place::Here);
        }
        if c.is_some() && !broken {
            places.push(Place::Cloud);
        }
    }
    let limits = a.limits.clone().unwrap_or_default();
    let limited = h.is_some_and(|h| h.trouble.is_some() && h.at_limit) && !limits.is_empty();
    let trouble = if limited { None } else { h.and_then(|h| h.trouble.clone()) };
    let read_since = h.and_then(|h| h.limit_at).is_some_and(|at| a.checked_at.is_some_and(|c| c >= at));
    let say_limit = limited && !read_since && !meter_pair(&limits, now).iter().any(|m| m.percent.unwrap_or(0.0) >= 100.0);
    let when = h.and_then(|h| h.limit_resets_at).and_then(|at| reset_words(&serde_json::json!({ "resetsAt": at }), now));
    let back = match (&when, h.and_then(|h| h.limit_resets_at)) {
        (None, _) => String::new(),
        (Some(w), Some(at)) if *w == hour_words(at) => format!(", back at {w}"),
        (Some(w), _) => format!(", back {w}"),
    };
    let sign_in = |word: &str, key: Option<String>| cloud_sign_in.then(|| (word.to_string(), TileAct::CloudSignIn(key)));
    let cloud_act = if !cloud || signing_in {
        None
    } else if broken {
        sign_in("Sign in to cloud again", c.and_then(|c| text(c, "key")).map(str::to_string))
    } else if h.is_some() && c.is_none() {
        sign_in("Add to cloud", None)
    } else {
        None
    };
    Tile {
        key: a.key.clone(),
        name: a.name.clone(),
        plan: a.plan.clone(),
        about: if signing_in {
            vec!["Finish signing in to use it".to_string()]
        } else if say_limit {
            vec![format!("At its limit{back}")]
        } else {
            Vec::new()
        },
        in_use: h.is_some_and(|h| h.chosen) && !signing_in,
        limits: a.limits.clone(),
        checked_at: a.checked_at,
        usage_error: a.usage_error.clone(),
        use_: h.filter(|h| h.signed_in && !h.chosen && many).map(|h| ("Switch".to_string(), TileAct::Use(h.profile.clone()))),
        places,
        cloud_act,
        cmd: trouble.as_ref().and(h.and_then(|h| h.dir.clone())).map(|dir| format!("CLAUDE_CONFIG_DIR={dir} claude")),
        trouble,
        meters: !limits.is_empty() || h.is_some_and(|_| !signing_in),
        none: None,
    }
}

/// Antigravity's tiles (`antigravityTiles`): agy holds one sign-in on a Mac, and its cloud
/// login is the one with the same address. agy reports no usage, so no meters are promised.
pub fn antigravity_tiles(here: &Value, cloud: Option<&Value>, cloud_sign_in: bool) -> Vec<Tile> {
    let set_up = cloud.is_some_and(|c| on(c, "setUp"));
    let all: Vec<&Value> = if set_up { counted(cloud.map_or(&[], |c| list(c, "accounts"))).into_iter().filter(|a| text(a, "engine") == Some("antigravity")).collect() } else { Vec::new() };
    let signed_in = here.get("signedIn").and_then(Value::as_bool);
    let email = text(here, "email").map(str::to_lowercase);
    let mine = if signed_in == Some(true) {
        match &email {
            Some(e) => all.iter().find(|a| text(a, "name").map(str::to_lowercase).as_deref() == Some(e.as_str())).copied(),
            None => all.first().copied(),
        }
    } else {
        None
    };
    let broken = |a: &Value| text(a, "state") == Some("broken");
    let sign_in = |word: &str, key: Option<&str>| cloud_sign_in.then(|| (word.to_string(), TileAct::CloudSignIn(key.map(str::to_string))));
    let name = text(here, "email").map(str::to_string).or_else(|| mine.and_then(|m| text(m, "name")).map(str::to_string)).unwrap_or_else(|| "Google account".to_string());
    let mut places = Vec::new();
    if set_up && signed_in == Some(true) {
        places.push(Place::Here);
        if mine.is_some_and(|m| !broken(m)) {
            places.push(Place::Cloud);
        }
    }
    let cloud_act = if !set_up || signed_in != Some(true) {
        None
    } else if let Some(m) = mine.filter(|m| broken(m)) {
        sign_in("Sign in to cloud again", text(m, "key"))
    } else if mine.is_none() {
        sign_in("Add to cloud", None)
    } else {
        None
    };
    let tile = Tile {
        key: "antigravity-here".to_string(),
        name,
        about: match signed_in {
            Some(false) => vec!["Not signed in".to_string()],
            None => vec!["Not checked yet".to_string()],
            Some(true) => Vec::new(),
        },
        in_use: signed_in == Some(true),
        use_: (signed_in != Some(true)).then(|| ("Sign in".to_string(), TileAct::SignIn)),
        places,
        cloud_act,
        meters: false,
        ..Tile::default()
    };
    let mut out = vec![tile];
    for a in all.iter().filter(|a| !mine.is_some_and(|m| std::ptr::eq(**a, m))) {
        out.push(Tile {
            key: text(a, "key").unwrap_or("").to_string(),
            name: account_label(a, &all),
            places: if broken(a) { Vec::new() } else { vec![Place::Cloud] },
            cloud_act: if broken(a) { sign_in("Sign in to cloud again", text(a, "key")) } else { None },
            meters: false,
            ..Tile::default()
        });
    }
    out
}

/// A tile's own reading time, said beside it when it reads differently from the head's, or
/// once it is a quarter of an hour old (`ownTime` in `PlaceTiles`).
pub fn own_time(tile: &Tile, head: &str, now: f64) -> bool {
    tile.checked_at.is_some_and(|at| at != 0.0 && (tile.usage_error.is_some() || now - at >= 15.0 * 60_000.0 || usage_checked_at(Some(at), now) != head))
}

/* ----------------------------------- running ----------------------------------- */

/// What the Running page leads with (`RunningFigures`).
#[derive(Clone, Debug, PartialEq)]
pub struct Figures {
    pub cells: Vec<(String, String)>,
    pub used_pct: Option<f64>,
    pub hot: bool,
    pub holding: Option<String>,
    pub tally: Option<String>,
}

/// The figures, or none with the memory check off (`runningFigures`).
pub fn running_figures(running: Option<f64>, capacity: Option<f64>, gate: Option<&Value>) -> Option<Figures> {
    let r = gate.filter(|g| on(g, "on")).and_then(|g| g.get("reading")).filter(|r| r.is_object())?;
    let pressure = text(r, "pressure");
    let word = match pressure {
        Some("critical") => Some("Very short"),
        Some("tight") => Some("Tight"),
        Some(_) => Some("Fine"),
        None => None,
    };
    let mut cells = Vec::new();
    if let Some(n) = running.filter(|n| n.is_finite()) {
        cells.push((n.to_string(), match capacity.filter(|c| *c != 0.0) {
            Some(c) => format!("of {c} running"),
            None => "running".to_string(),
        }));
    }
    if let Some(n) = num(r, "heavy") {
        cells.push((n.to_string(), "heavy at once".to_string()));
    }
    if let Some(n) = num(r, "waiting") {
        cells.push((n.to_string(), "waiting on memory".to_string()));
    }
    if let Some(w) = word {
        cells.push((w.to_string(), "memory now".to_string()));
    }
    let tally = num(r, "asked").filter(|a| *a != 0.0).map(|asked| {
        let refused = num(r, "refused").filter(|n| *n != 0.0).map_or_else(|| "none turned away".to_string(), |n| format!("{n} turned away"));
        format!("Today: {asked} commands asked, {} waited, {refused}.", num(r, "waited").unwrap_or(0.0))
    });
    let holding = on(r, "holding").then(|| "New agents wait until memory frees up. Urgent tasks still start.".to_string());
    if cells.is_empty() && tally.is_none() && holding.is_none() {
        return None;
    }
    Some(Figures { cells, used_pct: num(r, "usedPct"), hot: pressure != Some("normal"), holding, tally })
}

/* ----------------------------------- updates ---------------------------------- */

/// What the Updates row says and its button does (`UpdateLook`).
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateLook {
    pub sentence: String,
    pub button: &'static str,
    pub restart: bool,
    pub move_: bool,
    pub busy: bool,
}

/// `updateLook`, for the states the row meets. `checking` is the row's own press, still out.
pub fn update_look(state: Option<&Value>, checking: bool) -> UpdateLook {
    let empty = Value::Null;
    let state = state.unwrap_or(&empty);
    let installing = on(state, "installing");
    let phase = if checking && !installing { "checking" } else { text(state, "phase").unwrap_or("idle") };
    let ready = phase == "ready";
    let source = on(state, "source");
    let busy = phase == "checking" || phase == "downloading" || installing;
    let move_ = on(state, "canMove") && !ready && !busy;
    let button = if move_ {
        "Move to Applications"
    } else if ready {
        "Restart to update"
    } else if installing {
        "Updating"
    } else if phase == "checking" {
        "Checking"
    } else if phase == "downloading" {
        "Downloading"
    } else {
        "Check for updates"
    };
    let sentence = if move_ {
        match text(state, "moveError") {
            Some(e) => format!("{}. Pressing it again tries again.", e.trim_end_matches('.')),
            None => format!("{NAME} cannot update itself from where it is. Move it to Applications and it updates as it should."),
        }
    } else if installing {
        format!("Updating. {NAME} restarts by itself in about a minute.")
    } else if ready {
        match text(state, "error") {
            Some(e) => format!("{}. Pressing it again tries again.", e.trim_end_matches('.')),
            None => {
                let how = if source { "Restarting takes about a minute." } else { "Restarting takes a few seconds." };
                let what = if source {
                    match num(state, "behind").filter(|b| *b > 0.0) {
                        Some(b) => format!("{b} new {}.", if b == 1.0 { "change" } else { "changes" }),
                        None => "A new version is ready.".to_string(),
                    }
                } else {
                    text(state, "newVersion").map_or_else(|| "A new version has downloaded.".to_string(), |v| format!("Version {v} has downloaded."))
                };
                format!("{what} {how}")
            }
        }
    } else if phase == "downloading" {
        num(state, "percent").map_or_else(|| "Downloading the new version.".to_string(), |p| format!("Downloading the new version, {p}%."))
    } else if phase == "checking" {
        "Looking for a new version.".to_string()
    } else if phase == "current" {
        format!("{NAME} is up to date.")
    } else if phase == "error" || phase == "unsupported" {
        text(state, "error").unwrap_or("Could not check for updates.").to_string()
    } else {
        format!("{NAME} looks for a new version on its own.")
    };
    UpdateLook { sentence, button, restart: ready, move_, busy }
}

/* ------------------------------------ plans ------------------------------------ */

/// A plan as a card draws it (`PlanCard`).
#[derive(Clone, Debug, PartialEq)]
pub struct PlanCard {
    pub id: &'static str,
    pub name: &'static str,
    pub price: String,
    pub hours: String,
    pub current: bool,
    pub button: Option<String>,
}

/// The paid plans, with their price and hours (`PLANS` in `cloud/gateway/plans.mjs`).
const PAID: [(&str, &str, u32, u32); 3] = [("pro", "Pro", 40, 400), ("plus", "Plus", 100, 1200), ("max", "Max", 200, 3000)];

/// A number as en-US writes it, with commas.
pub fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The three plans as cards (`planCards`).
pub fn plan_cards(plan: Option<&Value>) -> Vec<PlanCard> {
    let mine = plan.and_then(|p| text(p, "plan")).filter(|p| PAID.iter().any(|(id, ..)| id == p));
    let verb = if mine.is_some() && plan.and_then(|p| text(p, "billing")) != Some("given") { "Switch to" } else { "Choose" };
    let offer = if mine.is_some() { false } else { plan.is_some_and(|p| p.get("trialOffer").is_some_and(|o| o.is_object())) };
    PAID.iter()
        .map(|(id, name, dollars, hours)| PlanCard {
            id,
            name,
            price: format!("${dollars}"),
            hours: format!("{} hours a month", grouped(u64::from(*hours))),
            current: mine == Some(*id),
            button: if mine == Some(*id) {
                None
            } else if offer {
                Some(format!("Try {name} free"))
            } else {
                Some(format!("{verb} {name}"))
            },
        })
        .collect()
}

/// Whether a plan is one of the paid ones, which has a summary of its own on the page.
pub fn paid_plan(plan: Option<&Value>) -> bool {
    plan.and_then(|p| text(p, "plan")).is_some_and(|p| PAID.iter().any(|(id, ..)| *id == p))
}

/* ------------------------------------ phone ------------------------------------ */

/// What the phone's projects picker says, and which projects are on (`PhoneProjectsPicker`):
/// `all` is `phoneAll`, `projects` each slug and whether its threads go to the phone.
pub fn phone_word(all: bool, projects: &[(String, bool)]) -> String {
    let kept: Vec<&String> = if all { projects.iter().filter(|(_, on)| !on).map(|(s, _)| s).collect() } else { projects.iter().map(|(s, _)| s).collect() };
    if all && kept.is_empty() {
        return "Every project".to_string();
    }
    if !all || kept.len() == projects.len() {
        return "None".to_string();
    }
    format!("{} of {}", projects.len() - kept.len(), projects.len())
}

/// What ticking one project writes (`tick`): `(all, except)`.
pub fn phone_tick(all: bool, projects: &[(String, bool)], slug: &str) -> (bool, Vec<String>) {
    let mut kept: Vec<String> = if all { projects.iter().filter(|(_, on)| !on).map(|(s, _)| s.clone()).collect() } else { projects.iter().map(|(s, _)| s.clone()).collect() };
    if kept.iter().any(|s| s == slug) {
        kept.retain(|s| s != slug);
    } else {
        kept.push(slug.to_string());
    }
    if kept.len() < projects.len() { (true, kept) } else { (false, Vec::new()) }
}

/* ----------------------------------- projects ---------------------------------- */

/// An order with no empty or repeated slug (`normalizeOrder`).
fn normalize(order: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in order {
        if !s.is_empty() && !out.contains(s) {
            out.push(s.clone());
        }
    }
    out
}

/// `slug` moved to `to` in the order (`moveProduct`).
pub fn move_product(order: &[String], slug: &str, to: usize) -> Vec<String> {
    let mut base: Vec<String> = normalize(order).into_iter().filter(|s| s != slug).collect();
    base.insert(to.min(base.len()), slug.to_string());
    base
}

/// `slug` put just before `before`, or last; none when nothing would change (`placeBefore`).
pub fn place_before(full: &[String], slug: &str, before: Option<&str>) -> Option<Vec<String>> {
    if before == Some(slug) {
        return None;
    }
    let rest: Vec<String> = full.iter().filter(|s| *s != slug).cloned().collect();
    let at = before.and_then(|b| rest.iter().position(|s| s == b)).unwrap_or(rest.len());
    let next = move_product(&rest, slug, at);
    (next != full).then_some(next)
}

/// `slug` one step up or down among the rows shown (`nudge`).
pub fn nudge(full: &[String], shown: &[String], slug: &str, up: bool) -> Option<Vec<String>> {
    let i = shown.iter().position(|s| s == slug)?;
    let j = if up { i.checked_sub(1)? } else { i + 1 };
    let target = shown.get(j)?;
    let rest: Vec<String> = full.iter().filter(|s| *s != slug).cloned().collect();
    let k = rest.iter().position(|s| s == target)?;
    Some(move_product(&rest, slug, if up { k } else { k + 1 }))
}

/// A path a person can read in a narrow column: the home folder as ~, and an account's long
/// id cut to the part that tells it apart (`shortPath`).
pub fn short_path(full: &str, home: Option<&str>) -> String {
    let home = home.map(|h| h.trim_end_matches('/')).filter(|h| full.starts_with(&format!("{h}/")));
    let users = || {
        let rest = full.strip_prefix("/Users/")?;
        let end = rest.find('/').unwrap_or(rest.len());
        (end > 0).then(|| &full[..7 + end])
    };
    let at = home.or_else(users);
    let short = match at {
        Some(at) => format!("~{}", &full[at.len()..]),
        None => full.to_string(),
    };
    // `([0-9a-f]{8})-[0-9a-f-]{20,}`, case blind, the first one only.
    let re = regex::Regex::new(r"(?i)([0-9a-f]{8})-[0-9a-f-]{20,}").expect("the account id's shape");
    re.replacen(&short, 1, "$1…").into_owned()
}

/// The header line while selecting, its button, and the line just after (`project-archive.ts`).
pub fn selection_line(count: usize) -> String {
    if count > 0 { format!("{count} selected") } else { "Select the projects to archive".to_string() }
}
fn projects_word(n: usize) -> String {
    format!("{n} project{}", if n == 1 { "" } else { "s" })
}
pub fn archive_label(count: usize) -> String {
    if count > 0 { format!("Archive {}", projects_word(count)) } else { "Archive".to_string() }
}
pub fn archived_line(count: usize) -> String {
    format!("Archived {}", projects_word(count))
}

/// What a project's sites row says (`sitesLine`).
pub fn sites_line(sites: &[String]) -> String {
    if sites.is_empty() {
        "List a site and this project's agents get a browser of their own for it. It holds only the sign-ins you make in it, never your own browser's.".to_string()
    } else {
        sites.join(", ")
    }
}

/// The sign-in row's sentence (`signInLine`).
pub fn sign_in_line(browser: Option<&Value>, busy: bool) -> String {
    let can_run = browser.is_none_or(|b| on(b, "canRun"));
    if busy {
        return if browser.is_some() && !can_run { "Downloading Chrome for Testing, which this browser runs on. This happens once.".to_string() } else { "Opening the window.".to_string() };
    }
    if let Some(e) = browser.filter(|b| b.get("ok") == Some(&Value::Bool(false))).and_then(|b| text(b, "error")) {
        return e.to_string();
    }
    let state = browser.and_then(|b| text(b, "state"));
    if state == Some("signing-in") {
        return "The window is open on this project's sites. Sign in, then close it. Agents wait until you do.".to_string();
    }
    if let Some(n) = browser.filter(|_| state == Some("open")).and_then(|b| num(b, "threads")).filter(|n| *n > 0.0) {
        let them = if n == 1.0 { "1 thread is".to_string() } else { format!("{n} threads are") };
        return format!("{them} using it now. Signing in closes their pages first.");
    }
    let first = if browser.is_some() && !can_run { " The first time, it downloads Chrome for Testing (about 170 MB)." } else { "" };
    format!("Opens this project's browser for you to sign in by hand. Its agents then read these sites as you.{first}")
}

/// The sign-in row's button: its word, and whether pressing it means Done (`signInButton`).
pub fn sign_in_button(browser: Option<&Value>) -> (&'static str, bool) {
    if browser.and_then(|b| text(b, "state")) == Some("signing-in") { ("Done", true) } else { ("Sign in", false) }
}

/// "an empty page", "1,240 characters" (`sizeLabel`).
pub fn size_label(chars: usize) -> String {
    if chars == 0 { "an empty page".to_string() } else { format!("{} characters", grouped(chars as u64)) }
}

/// A size as a person reads it: 625 MB, 2.1 MB, 4 KB (`sizeWords`).
pub fn size_words(bytes: f64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    let (mut n, mut u) = (bytes / 1024.0, 0);
    while n >= 1000.0 && u < UNITS.len() - 1 {
        n /= 1024.0;
        u += 1;
    }
    if n < 1.0 {
        return "1 KB".to_string();
    }
    let shown = if n >= 100.0 || u == 0 { n.round() } else { (n * 10.0).round() / 10.0 };
    format!("{shown} {}", UNITS[u])
}

/* ----------------------------------- the modes ---------------------------------- */

/// Claude Code's six modes, in Anthropic's own row order, with their words and sentences
/// (`MODE_ORDER`, `MODE_WORDS`, `MODE_SENTENCE` in `renderer/src/modes.ts`).
pub const CLAUDE_MODES: [(&str, &str, &str); 6] = [
    ("default", "Manual", "Runs without asking: reads only."),
    ("acceptEdits", "Accept edits", "Runs without asking: reads, file edits, and common filesystem commands like mkdir, mv and cp."),
    ("plan", "Plan", "Runs without asking: reads, plus commands the classifier approves when auto mode is available."),
    ("auto", "Auto", "Runs without asking: everything, with background safety checks."),
    ("dontAsk", "Don't Ask", "Runs without asking: only pre-approved tools. Nothing else is put in front of you, so nothing waits."),
    ("bypassPermissions", "Bypass Permissions", "Runs without asking: everything."),
];
pub const CUSTOM_SENTENCE: &str = "Set by hand in Mercury’s settings file on this computer, so it is not changed here.";
pub const RULES_ALWAYS_APPLY: &str = "A mode is only the starting point. Your own rules sit on top of it: an allow rule can let something through without asking, and your deny rules still apply. A few dangerous actions are never auto-approved, whatever you pick.";

/// Codex's three (`CODEX_MODES` in `shared/codex-modes.mts`), least to most an agent may do.
pub const CODEX_MODES: [(&str, &str, &str); 3] = [
    ("read-only", "Read only", "Reads and searches. Writes nothing and runs nothing that changes anything."),
    ("auto", "Auto", "Works in the project folder. Codex automatically reviews requests for access outside it or to the network."),
    ("full-access", "Full access", "Anything, anywhere on this computer, without asking."),
];
/// Full access in the cloud is a throwaway machine, not this computer (`CODEX_CLOUD_WHAT`).
pub const CODEX_CLOUD_FULL: &str = "Anything on its own throwaway cloud machine, without asking.";

/// Antigravity's five, in agy's own names (`shared/antigravity-modes.mts`).
pub const AGY_MODES: [(&str, &str, &str); 5] = [
    ("auto", "Auto", "Edits and the commands it judges safe go ahead. Antigravity decides what to ask about."),
    ("accept-edits", "Accept edits", "Edits in the folder go ahead. It asks before running commands."),
    ("default", "Default", "It asks before every edit and every command."),
    ("plan", "Plan", "It plans the task carefully before doing it."),
    ("full-access", "Full access", "Anything, anywhere on this computer, without asking."),
];

/// A mode's sentence in one of those tables.
pub fn mode_what(table: &[(&'static str, &'static str, &'static str)], id: &str) -> Option<&'static str> {
    table.iter().find(|(m, ..)| *m == id).map(|(_, _, what)| *what)
}

/// Who "Agents you started yourself" lets into the Inbox (`AGENT_OPTIONS`, `AGENT_SENTENCE`).
pub const OUTSIDE: [(&str, &str, &str); 3] = [
    ("all", "All of them", "Each one is a row in your inbox. Closing it only takes it out of the inbox."),
    ("waiting", "Only the stuck ones", "Only the ones waiting for you to type. The rest run without interrupting you."),
    ("off", "None", "Each is still in its project’s sidebar."),
];

/// How long files stay on this computer once kept in the cloud (`OFF_AFTER`).
pub const OFF_AFTER: [(&str, &str); 4] = [("7", "After 7 days"), ("30", "After 30 days"), ("90", "After 90 days"), ("0", "Never")];

/* ------------------------------- an agent's state ------------------------------- */

/// Whether an agent is here and working (`connectionState`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    Connected,
    Trouble,
    Missing,
    Unsure,
}

pub fn connection_state(found: bool, certain: bool, trouble: bool) -> Connection {
    match (found, trouble, certain) {
        (true, true, _) => Connection::Trouble,
        (true, false, _) => Connection::Connected,
        (false, _, true) => Connection::Missing,
        (false, _, false) => Connection::Unsure,
    }
}

/// The word beside the dot (`STATE_WORD`).
pub fn connection_word(c: Connection) -> &'static str {
    match c {
        Connection::Connected => "connected",
        Connection::Trouble => "not working",
        Connection::Missing => "not found",
        Connection::Unsure => "not sure",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The instant a clock on this machine names, as `new Date(y, m, d, h, min)` does (`m` from 0).
    fn local(y: i64, m: i64, d: i64, h: i64, min: i64) -> f64 {
        crate::rules::calendar::Day::real(y, m, d).expect("a real day").at(h, min)
    }

    fn every() -> Vec<PageId> {
        PAGES.to_vec()
    }

    fn labels(hits: &[Hit]) -> Vec<String> {
        hits.iter()
            .map(|h| match &h.pane {
                Pane::Page(p) => format!("{}: {}", p.id(), h.label),
                Pane::Project(slug) => format!("project:{slug}: {}", h.label),
            })
            .collect()
    }

    #[test]
    fn the_search_finds_a_setting_by_the_word_you_would_use() {
        let all = every();
        let one_person: Vec<PageId> = all.iter().copied().filter(|p| !matches!(p, PageId::Codex | PageId::Team)).collect();
        assert!(search("", &all, &[]).is_empty());
        assert!(search("   ", &all, &[]).is_empty());
        assert_eq!(labels(&search("agents at once", &all, &[]))[0], "running: How many run at once");
        assert_eq!(labels(&search("theme", &all, &[]))[0], "appearance: Theme");
        assert!(labels(&search("dark", &all, &[])).contains(&"appearance: Theme".to_string()));
        assert!(labels(&search("log out", &all, &[])).contains(&"general: Your account".to_string()));
        assert!(labels(&search("ram", &all, &[])).contains(&"running: Hold heavy work when memory is short".to_string()));
        assert!(labels(&search("privacy", &all, &[])).contains(&"general: Counts and crash reports".to_string()));
        // At the start of a word and nowhere else.
        assert!(labels(&search("mode", &all, &[])).contains(&"claude: Permission mode".to_string()));
        assert!(search("ode", &all, &[]).is_empty());
        assert!(search("uto", &all, &[]).is_empty());
        // Every word, not any one.
        assert_eq!(labels(&search("codex permission", &all, &[])), ["codex: Permission mode"]);
        assert!(search("theme zebra", &all, &[]).is_empty());
        // A name ahead of a hidden word.
        let hits = labels(&search("signed", &all, &[]));
        assert_eq!(hits[0], "claude: Signed in");
        let at = |l: &str| hits.iter().position(|h| h == l).expect(l);
        assert!(at("general: Your account") > at("codex: Signed in"));
        assert!(labels(&search("account", &one_person, &[])).contains(&"claude: Signed in".to_string()));
        // Never a page this Mac does not have.
        assert!(search("codex", &one_person, &[]).is_empty());
        assert!(search("invite", &one_person, &[]).is_empty());
        assert!(labels(&search("invite", &all, &[])).contains(&"team: Team".to_string()));
        // A project by its name, onto its own page.
        let projects = [("kestrel".to_string(), "Kestrel".to_string()), ("onboard".to_string(), "Onboard".to_string())];
        let hits = search("kes", &all, &projects);
        assert_eq!(hits.iter().map(|h| h.pane.clone()).collect::<Vec<_>>(), [Pane::Project("kestrel".into())]);
        assert_eq!(hits[0].page_label, "Projects");
        assert_eq!(labels(&search("CLAUDE-CODE usage", &all, &[])), ["claude: Usage"]);
    }

    #[test]
    fn automatic_names_the_number_it_settled_on_and_every_number_is_for_the_whole_computer() {
        let options = agent_count_options(&json!({ "agentsAuto": true, "agentsTotal": 9, "slotsMax": 12 }));
        assert_eq!(options[0], ("auto".to_string(), "Automatic (9 agents)".to_string()));
        assert_eq!(options.len(), 13);
        assert_eq!(options[1], ("1".to_string(), "1 agent".to_string()));
        assert_eq!(options[12], ("12".to_string(), "12 agents".to_string()));
        // Set by hand, Automatic names what it would run.
        let options = agent_count_options(&json!({ "agentsAuto": false, "agentsTotal": 4, "agentsAutoTotal": 6 }));
        assert_eq!(options[0].1, "Automatic (6 agents)");
    }

    fn limit(span: &str, percent: f64, extra: Value) -> Value {
        let mut l = json!({ "span": span, "qualifier": if span == "week" { json!("all models") } else { Value::Null }, "name": span, "percent": percent, "resetsText": null, "resetsOn": null, "resetsAt": null, "zone": null });
        if let (Some(l), Some(extra)) = (l.as_object_mut(), extra.as_object()) {
            l.extend(extra.clone());
        }
        l
    }

    #[test]
    fn every_account_meter_says_when_it_resets() {
        let now = local(2026, 9, 20, 14, 30);
        assert_eq!(reset_words(&limit("session", 9.0, json!({ "resetsAt": local(2026, 9, 20, 18, 19) })), now).as_deref(), Some("6:19pm"));
        assert_eq!(reset_words(&limit("week", 37.0, json!({ "resetsAt": local(2026, 9, 22, 16, 0) })), now).as_deref(), Some("Thu 4pm"));
        assert_eq!(reset_words(&limit("session", 9.0, json!({ "resetsAt": local(2026, 9, 21, 0, 30) })), now).as_deref(), Some("Wed 12:30am"));
        assert_eq!(reset_words(&limit("session", 9.0, json!({ "resetsText": "8pm" })), now).as_deref(), Some("8pm"));
        assert_eq!(reset_words(&limit("week", 37.0, json!({ "resetsText": "4pm", "resetsOn": "Oct 22" })), now).as_deref(), Some("Oct 22, 4pm"));
        assert_eq!(reset_words(&limit("session", 9.0, json!({ "resetsAt": local(2026, 9, 20, 14, 0) })), now), None);
        assert_eq!(reset_words(&limit("session", 9.0, json!({ "resetsAt": now })), now), None);
        assert_eq!(reset_words(&limit("session", 0.0, json!({})), now), None);
        let meters = meter_pair(&[limit("session", 100.0, json!({ "resetsText": "5:09pm" })), limit("week", 15.0, json!({ "resetsAt": local(2026, 9, 26, 11, 59) }))], now);
        assert_eq!(meters.iter().map(|m| (m.percent, m.resets.clone())).collect::<Vec<_>>(), [(Some(100.0), Some("5:09pm".into())), (Some(15.0), Some("Mon 11:59am".into()))]);
        // Six days out or more names the date.
        assert_eq!(reset_day(local(2026, 9, 27, 9, 0), now).as_deref(), Some("Tue Oct 27"));
        // A window past its reset shows no number.
        let past = meter_pair(&[limit("session", 40.0, json!({ "resetsAt": local(2026, 9, 20, 13, 0) }))], now);
        assert_eq!((past[0].percent, past[0].resets.as_deref()), (None, Some("Reset")));
        // A model's own week, after the week of all of them.
        let three = meter_pair(&[limit("session", 1.0, json!({})), limit("week", 2.0, json!({})), json!({ "span": "week", "qualifier": "Fable", "percent": 3 })], now);
        assert_eq!(three.iter().map(|m| m.word.as_str()).collect::<Vec<_>>(), ["Session", "Week", "Fable"]);
    }

    #[test]
    fn a_reading_says_how_long_ago_it_was_checked() {
        let now = 1_000_000_000.0;
        assert_eq!(usage_checked_at(None, now), "");
        assert_eq!(usage_checked_at(Some(now - 20_000.0), now), "Checked just now");
        assert_eq!(usage_checked_at(Some(now - 60_000.0), now), "Checked 1 minute ago");
        assert_eq!(usage_checked_at(Some(now - 4.0 * 60_000.0), now), "Checked 4 minutes ago");
        assert_eq!(usage_checked_at(Some(now - 125.0 * 60_000.0), now), "Checked 2 hours ago");
    }

    #[test]
    fn each_claude_account_is_drawn_once_wherever_it_is_signed_in() {
        let w = json!({ "accounts": [
            { "profile": "default", "email": "a@example.com", "chosen": false, "plan": { "label": "Max" } },
            { "profile": "studio", "email": "b@example.com", "chosen": true, "plan": { "label": "Pro" } },
        ] });
        let here = claude_logins(&w);
        let read = |p: f64, at: f64| json!([{ "span": "session", "percent": p, "resetsAt": at + 3_600_000.0 }]);
        let cloud = vec![
            json!({ "engine": "claude", "key": "c1", "name": "a@example.com", "profile": "default", "plan": "Max", "state": "ready", "limits": read(50.0, 2000.0), "readAt": 2000.0 }),
            json!({ "engine": "claude", "key": "c3", "name": "c@example.com", "plan": "Pro", "state": "ready" }),
        ];
        let readings = vec![json!({ "profile": "default", "limits": read(10.0, 1000.0), "at": 1000.0 }), json!({ "profile": "studio", "limits": read(20.0, 1500.0), "at": 1500.0 })];
        let once = accounts_once(&here, &cloud, &readings, None);
        // The one in use first, then the rest here, then the cloud's alone.
        assert_eq!(once.iter().map(|a| a.key.as_str()).collect::<Vec<_>>(), ["studio", "default", "cloud:c3"]);
        // The cloud's reading of the same account is newer, and wins.
        assert_eq!(once[1].checked_at, Some(2000.0));
        assert_eq!(once[0].checked_at, Some(1500.0));
        let tile = account_tile(&once[1], true, 0.0, true, true);
        assert_eq!(tile.places, [Place::Here, Place::Cloud]);
        assert_eq!(tile.use_, Some(("Switch".to_string(), TileAct::Use("default".to_string()))));
        assert!(!tile.in_use);
        let tile = account_tile(&once[0], true, 0.0, true, true);
        assert!(tile.in_use && tile.use_.is_none());
        // Here and not in the cloud: the tile offers to add it.
        assert_eq!(tile.cloud_act, Some(("Add to cloud".to_string(), TileAct::CloudSignIn(None))));
        // One account alone offers no switch, and with no cloud, no places.
        let lone = accounts_once(&here[..1], &[], &readings, None);
        assert_eq!(account_tile(&lone[0], false, 0.0, false, true).use_, None);
        assert!(account_tile(&lone[0], false, 0.0, false, true).places.is_empty());
    }

    #[test]
    fn an_account_not_signed_in_is_never_in_use_and_says_to_finish() {
        let here = vec![HereLogin { profile: "codex-2".into(), chosen: true, signed_in: false, ..HereLogin::default() }];
        let tile = account_tile(&accounts_once(&here, &[], &[], None)[0], true, 0.0, true, true);
        assert!(!tile.in_use);
        assert_eq!(tile.about, ["Finish signing in to use it"]);
        assert!(tile.places.is_empty() && tile.cloud_act.is_none());
    }

    #[test]
    fn antigravity_has_one_sign_in_and_its_cloud_login_goes_by_its_address() {
        let cloud = json!({ "setUp": true, "accounts": [{ "engine": "antigravity", "key": "g1", "name": "Sam@Example.com", "state": "ready" }] });
        let tiles = antigravity_tiles(&json!({ "signedIn": true, "email": "sam@example.com" }), Some(&cloud), true);
        assert_eq!(tiles.len(), 1);
        assert!(tiles[0].in_use && !tiles[0].meters);
        assert_eq!(tiles[0].places, [Place::Here, Place::Cloud]);
        let tiles = antigravity_tiles(&json!({ "signedIn": false }), Some(&cloud), true);
        assert_eq!(tiles[0].about, ["Not signed in"]);
        assert_eq!(tiles[0].use_, Some(("Sign in".to_string(), TileAct::SignIn)));
        // The login nobody here is signed in as is a tile of its own.
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[1].places, [Place::Cloud]);
    }

    #[test]
    fn the_running_page_leads_with_what_it_measured_and_nothing_it_did_not() {
        let gate = json!({ "on": true, "reading": { "heavy": null, "waiting": null, "pressure": null, "usedPct": null, "asked": null, "waited": null, "refused": null, "holding": false } });
        let f = running_figures(Some(2.0), Some(9.0), Some(&gate)).expect("a figure");
        assert_eq!(f.cells, [("2".to_string(), "of 9 running".to_string())]);
        assert_eq!((f.used_pct, f.holding, f.tally), (None, None, None));
        assert_eq!(running_figures(Some(2.0), Some(9.0), Some(&json!({ "on": false, "reading": {} }))), None);
        let busy = json!({ "on": true, "reading": { "heavy": 3, "waiting": 1, "pressure": "tight", "usedPct": 81, "asked": 12, "waited": 4, "refused": 0, "holding": true } });
        let f = running_figures(Some(6.0), Some(6.0), Some(&busy)).expect("figures");
        assert_eq!(f.cells.iter().map(|(n, of)| format!("{n} {of}")).collect::<Vec<_>>(), ["6 of 6 running", "3 heavy at once", "1 waiting on memory", "Tight memory now"]);
        assert_eq!(f.tally.as_deref(), Some("Today: 12 commands asked, 4 waited, none turned away."));
        assert!(f.hot && f.holding.is_some());
    }

    #[test]
    fn check_for_updates_says_what_the_last_check_found() {
        let current = json!({ "phase": "current", "currentVersion": "0.1.22", "source": true });
        let look = update_look(Some(&current), false);
        assert_eq!((look.sentence.as_str(), look.button), ("Mercury is up to date.", "Check for updates"));
        assert_eq!(update_look(Some(&current), true).button, "Checking");
        assert!(update_look(Some(&current), true).busy);
        let ready = json!({ "phase": "ready", "source": true, "behind": 3 });
        let look = update_look(Some(&ready), false);
        assert_eq!((look.sentence.as_str(), look.button, look.restart), ("3 new changes. Restarting takes about a minute.", "Restart to update", true));
        assert_eq!(update_look(None, false).sentence, "Mercury looks for a new version on its own.");
    }

    #[test]
    fn a_cloud_of_ones_own_is_offered_the_three_plans_to_choose() {
        let mine = json!({ "plan": "self", "billing": "given", "trialOffer": null });
        let cards = plan_cards(Some(&mine));
        assert_eq!(cards.iter().map(|c| c.button.clone().unwrap_or_default()).collect::<Vec<_>>(), ["Choose Pro", "Choose Plus", "Choose Max"]);
        assert_eq!((cards[1].price.as_str(), cards[1].hours.as_str()), ("$100", "1,200 hours a month"));
        assert!(!paid_plan(Some(&mine)));
        let pro = json!({ "plan": "pro", "billing": "stripe" });
        let cards = plan_cards(Some(&pro));
        assert!(cards[0].current && cards[0].button.is_none());
        assert_eq!(cards[1].button.as_deref(), Some("Switch to Plus"));
        let offered = json!({ "plan": "none", "trialOffer": { "days": 7, "hours": 50 } });
        assert_eq!(plan_cards(Some(&offered))[2].button.as_deref(), Some("Try Max free"));
    }

    #[test]
    fn the_phone_picker_says_every_project_none_or_how_many() {
        let p = |on: [bool; 3]| vec![("a".to_string(), on[0]), ("b".to_string(), on[1]), ("c".to_string(), on[2])];
        assert_eq!(phone_word(true, &p([true, true, true])), "Every project");
        assert_eq!(phone_word(false, &p([true, true, true])), "None");
        assert_eq!(phone_word(true, &p([true, false, true])), "2 of 3");
        assert_eq!(phone_word(true, &p([false, false, false])), "None");
        // Ticking one off leaves the rest; ticking the last one off is None.
        assert_eq!(phone_tick(true, &p([true, true, true]), "b"), (true, vec!["b".to_string()]));
        assert_eq!(phone_tick(true, &p([true, false, false]), "a"), (false, vec![]));
        assert_eq!(phone_tick(false, &p([true, true, true]), "a"), (true, vec!["b".to_string(), "c".to_string()]));
    }

    #[test]
    fn a_project_moves_in_the_running_order_as_the_web_window_moves_it() {
        let full: Vec<String> = ["a", "b", "c", "d"].iter().map(|s| s.to_string()).collect();
        let s = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(place_before(&full, "c", Some("a")), Some(s(&["c", "a", "b", "d"])));
        assert_eq!(place_before(&full, "a", Some("a")), None);
        assert_eq!(place_before(&full, "a", Some("b")), None);
        assert_eq!(place_before(&full, "b", None), Some(s(&["a", "c", "d", "b"])));
        // A step among the rows shown, the hidden ones keeping their places.
        assert_eq!(nudge(&full, &s(&["a", "c", "d"]), "c", true), Some(s(&["c", "a", "b", "d"])));
        assert_eq!(nudge(&full, &s(&["a", "c", "d"]), "c", false), Some(s(&["a", "b", "d", "c"])));
        assert_eq!(nudge(&full, &full, "a", true), None);
        assert_eq!(nudge(&full, &full, "d", false), None);
    }

    #[test]
    fn a_path_reads_short_in_a_narrow_column() {
        assert_eq!(short_path("/Users/sam/dev/storefront", Some("/Users/sam")), "~/dev/storefront");
        assert_eq!(short_path("/Users/sam/dev/storefront", None), "~/dev/storefront");
        assert_eq!(short_path("/home/shared/repos", None), "/home/shared/repos");
        assert_eq!(short_path("/Users/sam/Library/Application Support/Mercury/accounts/3f2a8c1e-7b4d-4e9a-9c11-5d0e6f7a8b9c/x", None), "~/Library/Application Support/Mercury/accounts/3f2a8c1e…/x");
    }

    #[test]
    fn the_cloud_rows_say_when_threads_go_there_and_on_which_account() {
        assert_eq!(cloud_runs_line("full", Some("sam@example.com")), "Threads start there when this computer is full, on sam@example.com.");
        assert_eq!(cloud_runs_line("all", None), "Threads start there first, on your cloud account.");
        assert_eq!(cloud_runs_line("mac", None), "Every thread runs on this computer.");
        assert_eq!(cloud_toggle_target(&json!({ "mode": "full", "whenOn": "full" })), "mac");
        assert_eq!(cloud_toggle_target(&json!({ "mode": "mac", "whenOn": "all" })), "all");
        assert_eq!(cloud_toggle_target(&json!({ "mode": "mac" })), "full");
        let all = [json!({ "engine": "claude", "name": null }), json!({ "engine": "claude", "name": null })];
        let refs: Vec<&Value> = all.iter().collect();
        assert_eq!(account_label(&all[1], &refs), "Claude account 2");
    }

    #[test]
    fn numbers_and_sizes_read_as_the_web_window_writes_them() {
        assert_eq!(grouped(1_200), "1,200");
        assert_eq!(grouped(1_000_000), "1,000,000");
        assert_eq!(size_words(0.0), "1 KB");
        assert_eq!(size_words(2.1 * 1024.0 * 1024.0), "2.1 MB");
        assert_eq!(size_words(625.0 * 1024.0 * 1024.0), "625 MB");
        assert_eq!(size_label(0), "an empty page");
        assert_eq!(size_label(1240), "1,240 characters");
    }
}
