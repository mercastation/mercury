//! What is left of each coding agent's limits, as the rail's usage row and the card it opens say
//! it: `shared/usage.mts` (which windows a reading still answers for, the headline one, the
//! card's rows and when each starts over, the sentence read aloud), `agentsInUse` in
//! `renderer/src/components/UsagePill.tsx`, `lastUsedEngine` in `renderer/src/usage-last-used.ts`
//! and `agoQuiet` in `renderer/src/format.ts`.
//!
//! A reading is never live: each agent's is taken now and then and carries when (`at`). An
//! agent counts while it has a reading from this week that still answers for some window; a
//! window whose reset has gone by is over, and the one running in its place has never been read.

use serde_json::Value;

use crate::data::Snapshot;
use crate::rules::calendar::Day;

/// A window whose reset is this long past is still drawn, saying "any moment", so a countdown
/// crossing nought does not flicker out before the next reading (`JUST_RESET_MS`).
const JUST_RESET_MS: f64 = 2.0 * 60_000.0;
/// A reading older than this is about windows that have all started over (`WEEK`).
const WEEK_MS: f64 = 7.0 * 24.0 * 60.0 * 60_000.0;
/// Past this much spent the row's figure is in the text's own ink (`CLOSE`).
pub const CLOSE: f64 = 80.0;

/// One window of a reading (`UsageLimit`).
#[derive(Clone, Debug, PartialEq)]
pub struct Limit {
    pub span: String,
    pub qualifier: Option<String>,
    pub name: String,
    /// None where the reading has no figure: a five hour window that started over since.
    pub percent: Option<f64>,
    pub resets_at: Option<f64>,
    pub resets_text: Option<String>,
    pub resets_on: Option<String>,
    pub zone: Option<String>,
    /// A five hour window that started over after this reading: its new figure is unread.
    pub reset: bool,
}

impl Limit {
    fn from(v: &Value) -> Limit {
        let text = |key: &str| v.get(key).and_then(Value::as_str).map(str::to_string);
        let finite = |key: &str| v.get(key).and_then(Value::as_f64).filter(|n| n.is_finite());
        Limit {
            span: text("span").unwrap_or_default(),
            qualifier: text("qualifier"),
            name: text("name").unwrap_or_default(),
            percent: finite("percent"),
            resets_at: finite("resetsAt"),
            resets_text: text("resetsText").filter(|t| !t.is_empty()),
            resets_on: text("resetsOn").filter(|t| !t.is_empty()),
            zone: text("zone"),
            reset: v.get("reset").and_then(Value::as_bool).unwrap_or(false),
        }
    }
}

/// One agent's reading (`Usage`): whose, when it was taken, and its windows.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub engine: String,
    pub at: f64,
    pub limits: Vec<Limit>,
}

impl Reading {
    pub fn from(v: &Value) -> Option<Reading> {
        let v = v.as_object()?;
        let limits = v.get("limits").and_then(Value::as_array).map(|all| all.iter().map(Limit::from).collect()).unwrap_or_default();
        Some(Reading {
            engine: v.get("engine").and_then(Value::as_str).unwrap_or("claude").to_string(),
            at: v.get("at").and_then(Value::as_f64).unwrap_or(0.0),
            limits,
        })
    }
}

/// The windows a reading can still answer for: a reset gone by is over, and one that could not
/// be put on a clock keeps its row (`limitsStillRunning`).
pub fn still_running(limits: &[Limit], now: f64) -> Vec<Limit> {
    limits.iter().filter(|l| l.resets_at.is_none_or(|at| at + JUST_RESET_MS > now)).cloned().collect()
}

/// What the card and the row draw: the windows still running and, while any is, the five hour
/// one that ended, with no figure (`limitsToShow`).
pub fn to_show(limits: &[Limit], now: f64) -> Vec<Limit> {
    let running = still_running(limits, now);
    if running.is_empty() {
        return running;
    }
    limits
        .iter()
        .filter_map(|l| {
            if l.resets_at.is_none_or(|at| at + JUST_RESET_MS > now) {
                Some(l.clone())
            } else if l.span == "session" {
                Some(Limit { percent: None, resets_at: None, resets_text: None, resets_on: None, reset: true, ..l.clone() })
            } else {
                None
            }
        })
        .collect()
}

/// The one window the row is about: the five hour one where there is one, else the fullest
/// (`headlineLimit`).
pub fn headline(limits: &[Limit]) -> Option<&Limit> {
    if let Some(session) = limits.iter().find(|l| l.span == "session") {
        return Some(session);
    }
    limits.iter().filter(|l| l.percent.is_some()).fold(None, |best: Option<&Limit>, l| match best {
        Some(b) if l.percent <= b.percent => Some(b),
        _ => Some(l),
    })
}

/// "4m", "2h 10m", "6d 13h", and "any moment" under a minute (`spanLeft`).
pub fn span_left(ms: f64) -> String {
    let minutes = (ms / 60_000.0).floor().max(0.0) as i64;
    if minutes < 1 {
        return ANY_MOMENT.to_string();
    }
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = minutes / 60;
    if hours >= 24 {
        let (days, rest) = (hours / 24, hours % 24);
        return if rest > 0 { format!("{days}d {rest}h") } else { format!("{days}d") };
    }
    format!("{hours}h {}m", minutes % 60)
}
const ANY_MOMENT: &str = "any moment";

/// When a window starts over, in words: Claude Code's own clock time, with the day when the day
/// is not today; an instant with no words (Codex's) counted down; nothing where there is no reset
/// (`whenItStartsOver`). "Today" is this machine's calendar, the zone a reading's time is placed
/// in only when it is this machine's.
fn when_it_starts_over(l: &Limit, now: f64) -> Option<String> {
    if let Some(said) = &l.resets_text {
        if l.resets_at.is_some_and(|at| at != 0.0 && Day::of(at) == Day::of(now)) {
            return Some(format!("Resets {said}"));
        }
        return Some(match &l.resets_on {
            Some(on) => format!("Resets {on} at {said}"),
            None => format!("Resets {said}"),
        });
    }
    let at = l.resets_at.filter(|at| *at != 0.0)?;
    let soon = span_left(at - now);
    Some(if soon == ANY_MOMENT { format!("Resets {soon}") } else { format!("Resets in {soon}") })
}

/// One row of the card: its name, how much is spent (none for a window that started over), and
/// when it starts over (`limitRows`).
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub name: String,
    pub used: Option<f64>,
    pub when: Option<String>,
}

pub fn rows(limits: &[Limit], now: f64) -> Vec<Row> {
    to_show(limits, now)
        .iter()
        .map(|l| Row { name: l.name.clone(), used: if l.reset { None } else { l.percent }, when: if l.reset { None } else { when_it_starts_over(l, now) } })
        .collect()
}

/// What a row says beside its name: "39% used", or "Reset".
pub fn used_words(row: &Row) -> String {
    row.used.map_or_else(|| "Reset".to_string(), |used| format!("{}% used", number(used)))
}

/// A number as JavaScript prints it in a template: 39, 39.5.
pub fn number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 { format!("{}", n as i64) } else { format!("{n}") }
}

/// The coding agent used last: the newest run going now, else the thread touched last that
/// names the agent it runs on (`lastUsedEngine`).
pub fn last_used_engine(snap: &Snapshot) -> Option<String> {
    let started = |r: &crate::data::Run| r.rest.get("startedAt").and_then(Value::as_f64).unwrap_or(0.0);
    let mut run: Option<&crate::data::Run> = None;
    for r in &snap.supervisor.running {
        if r.engine.as_deref().is_some_and(|e| !e.is_empty()) && run.is_none_or(|was| started(r) > started(was)) {
            run = Some(r);
        }
    }
    if let Some(engine) = run.and_then(|r| r.engine.clone()) {
        return Some(engine);
    }
    let by_item = snap.rest.get("engines").and_then(|e| e.get("byItem"));
    let engine_of = |id: &str| by_item.and_then(|b| b.get(id)).and_then(Value::as_str).filter(|e| !e.is_empty());
    let mut latest: Option<&crate::data::Item> = None;
    for item in &snap.items {
        if engine_of(&item.id).is_some() && latest.is_none_or(|was| item.updated_at.unwrap_or(0.0) > was.updated_at.unwrap_or(0.0)) {
            latest = Some(item);
        }
    }
    latest.and_then(|i| engine_of(&i.id)).map(str::to_string)
}

/// Every agent with a reading from this week that still answers for a window, the one used
/// last first (`agentsInUse`): `recent` is the agent used last, `sent` the reading the snapshot
/// calls its own.
pub fn agents_in_use(all: &[Reading], recent: Option<&str>, sent: Option<&Reading>, now: f64) -> Vec<Reading> {
    let fresh: Vec<&Reading> = all.iter().filter(|u| now - u.at < WEEK_MS && headline(&still_running(&u.limits, now)).is_some()).collect();
    let last = fresh.iter().find(|u| Some(u.engine.as_str()) == recent).or_else(|| fresh.iter().find(|u| sent.is_some_and(|s| s.engine == u.engine))).or(fresh.first()).copied();
    let Some(last) = last else {
        return Vec::new();
    };
    std::iter::once(last).chain(fresh.into_iter().filter(|u| !std::ptr::eq(*u, last))).cloned().collect()
}

/// What the rail's usage row and its card stand on: the agents in use, the one used last
/// first, or the snapshot's own reading alone where it names no agents. None while there is
/// nothing true to draw (`UsagePill`, `UsageOne`).
#[derive(Clone, Debug, PartialEq)]
pub struct Usage {
    /// The reading the row is about, the one used last.
    pub last: Reading,
    /// Every agent in use; more than one draws the card with each under a hairline.
    pub agents: Vec<Reading>,
}

impl Usage {
    pub fn of(snap: &Snapshot, now: f64) -> Option<Usage> {
        let sent = snap.rest.get("usage").and_then(Reading::from);
        // `all={snap.usageByEngine ?? []}`: a snapshot with no list names no agents.
        let all: Vec<Reading> = snap.rest.get("usageByEngine").and_then(Value::as_array).map(|list| list.iter().filter_map(Reading::from).collect()).unwrap_or_default();
        let recent = last_used_engine(snap);
        let agents = agents_in_use(&all, recent.as_deref(), sent.as_ref(), now);
        let last = agents.first().cloned().or(sent)?;
        headline(&to_show(&last.limits, now))?;
        Some(Usage { last, agents })
    }

    /// Whether the card lists each agent under its own heading (`joint`).
    pub fn joint(&self) -> bool {
        self.agents.len() > 1
    }

    /// The figure on the row: the headline window's spent share, none for a five hour window
    /// that started over since the reading.
    pub fn used(&self, now: f64) -> Option<f64> {
        let shown = to_show(&self.last.limits, now);
        let headline = headline(&shown)?;
        if headline.reset { None } else { headline.percent.map(|p| p.clamp(0.0, 100.0)) }
    }

    /// The agent the row names (`engineLabel(last.engine)`).
    pub fn word(&self) -> &'static str {
        crate::rules::engine::engine_label(&self.last.engine)
    }

    /// The whole of it in a sentence, for whatever reads the row aloud (`usageSentence`).
    pub fn sentence(&self, now: f64) -> String {
        if self.joint() {
            return self.agents.iter().filter_map(|a| sentence(&a.limits, now, crate::rules::engine::engine_label(&a.engine))).collect::<Vec<_>>().join(" ");
        }
        sentence(&self.last.limits, now, self.word()).unwrap_or_default()
    }
}

/// `usageSentence`: each window, how much is spent and how long until it starts over.
fn sentence(limits: &[Limit], now: f64, word: &str) -> Option<String> {
    let live = to_show(limits, now);
    if live.is_empty() {
        return None;
    }
    let said: Vec<String> = live
        .iter()
        .map(|l| {
            if l.reset {
                return format!("{}: reset since the last reading.", l.name);
            }
            let when = match (l.resets_at.filter(|at| *at != 0.0), &l.resets_text) {
                (Some(at), _) => {
                    let span = span_left(at - now);
                    format!(", {}", if span == ANY_MOMENT { span } else { format!("{span} left") })
                }
                (None, Some(text)) => format!(", resets {text}"),
                _ => String::new(),
            };
            format!("{}: {}% used{when}.", l.name, number(l.percent.unwrap_or(0.0)))
        })
        .collect();
    Some(format!("{word}. {}", said.join(" ")))
}

/// How long ago a reading was taken, said quietly: "Read just now" under a minute, then "Read
/// 4m ago", "Read 1d ago" (`agoQuiet`, `ago`).
pub fn read_words(at: f64, now: f64) -> String {
    if ((now - at) / 60_000.0).floor() < 1.0 {
        return "Read just now".to_string();
    }
    format!("Read {} ago", crate::rules::files_shown::ago(at, now))
}

/// The Settings page a press on the row opens: Codex's for a Codex reading, Claude Code's for
/// any other (`openAgentSettings`).
pub fn settings_page(engine: &str) -> crate::rules::settings::PageId {
    if engine == "codex" { crate::rules::settings::PageId::Codex } else { crate::rules::settings::PageId::Claude }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_784_000_000_000.0;
    const MIN: f64 = 60_000.0;
    const HOUR: f64 = 60.0 * MIN;

    fn snap(rest: Value) -> Snapshot {
        serde_json::from_value(rest).unwrap()
    }

    fn reading(engine: &str, at: f64, limits: Value) -> Value {
        json!({ "engine": engine, "at": at, "limits": limits })
    }

    fn limit(span: &str, name: &str, percent: f64, resets_at: Option<f64>) -> Value {
        json!({ "span": span, "qualifier": null, "name": name, "percent": percent, "resetsAt": resets_at, "resetsText": null, "resetsOn": null, "zone": null })
    }

    #[test]
    fn the_headline_is_the_five_hour_window_else_the_fullest() {
        let r = Reading::from(&reading("codex", NOW, json!([limit("week", "This week", 12.0, None), limit("week", "Other", 30.0, None)]))).unwrap();
        assert_eq!(headline(&r.limits).map(|l| l.name.as_str()), Some("Other"));
        let r = Reading::from(&reading("claude", NOW, json!([limit("week", "This week", 90.0, None), limit("session", "This session", 4.0, None)]))).unwrap();
        assert_eq!(headline(&r.limits).map(|l| l.name.as_str()), Some("This session"));
        assert_eq!(headline(&[]), None);
    }

    #[test]
    fn a_window_gone_by_is_dropped_but_the_five_hour_one_says_reset() {
        let r = Reading::from(&reading("claude", NOW - 5.0 * HOUR, json!([limit("session", "This session", 71.0, Some(NOW - 30.0 * MIN)), limit("week", "This week", 39.0, Some(NOW + 50.0 * HOUR))]))).unwrap();
        let shown = to_show(&r.limits, NOW);
        assert_eq!(shown.len(), 2);
        assert!(shown[0].reset && shown[0].percent.is_none());
        let shown = rows(&r.limits, NOW);
        assert_eq!(used_words(&shown[0]), "Reset");
        assert_eq!(shown[0].when, None);
        assert_eq!(used_words(&shown[1]), "39% used");
        // Nothing still running: nothing to show at all.
        let gone = Reading::from(&reading("claude", NOW - 9.0 * HOUR, json!([limit("week", "This week", 39.0, Some(NOW - HOUR))]))).unwrap();
        assert!(to_show(&gone.limits, NOW).is_empty());
        // Inside two minutes of its reset a window is still drawn.
        let just = Reading::from(&reading("claude", NOW - HOUR, json!([limit("session", "This session", 50.0, Some(NOW - MIN))]))).unwrap();
        assert_eq!(rows(&just.limits, NOW)[0].when.as_deref(), Some("Resets any moment"));
    }

    #[test]
    fn a_reset_is_claude_codes_clock_with_the_day_unless_today_and_codexs_counted_down() {
        let today = json!([{ "span": "session", "name": "This session", "percent": 4.0, "resetsAt": NOW + 10.0 * MIN, "resetsText": "6:10pm", "resetsOn": "Mon", "zone": null }]);
        assert_eq!(rows(&Reading::from(&reading("claude", NOW, today)).unwrap().limits, NOW)[0].when.as_deref(), Some("Resets 6:10pm"));
        let later = json!([{ "span": "week", "name": "This week", "percent": 39.0, "resetsAt": NOW + 3.0 * 24.0 * HOUR, "resetsText": "7am", "resetsOn": "Mon", "zone": null }]);
        assert_eq!(rows(&Reading::from(&reading("claude", NOW, later)).unwrap().limits, NOW)[0].when.as_deref(), Some("Resets Mon at 7am"));
        // A stamp with no instant says itself whole.
        let stamp = json!([{ "span": "week", "name": "This week", "percent": 39.0, "resetsAt": null, "resetsText": "Sep 3 at 3:59pm", "resetsOn": null, "zone": null }]);
        assert_eq!(rows(&Reading::from(&reading("claude", NOW, stamp)).unwrap().limits, NOW)[0].when.as_deref(), Some("Resets Sep 3 at 3:59pm"));
        let codex = json!([limit("week", "This week", 12.0, Some(NOW + 6.0 * 24.0 * HOUR + 13.0 * HOUR + 30_000.0))]);
        assert_eq!(rows(&Reading::from(&reading("codex", NOW, codex)).unwrap().limits, NOW)[0].when.as_deref(), Some("Resets in 6d 13h"));
        assert_eq!(span_left(2.0 * HOUR + 10.0 * MIN), "2h 10m");
        assert_eq!(span_left(4.0 * MIN), "4m");
        assert_eq!(span_left(3.0 * 24.0 * HOUR), "3d");
        assert_eq!(span_left(20_000.0), "any moment");
    }

    #[test]
    fn the_row_names_the_agent_used_last_and_lists_the_others_after_it() {
        let week = |p: f64| json!([limit("week", "This week", p, Some(NOW + 50.0 * HOUR))]);
        let s = snap(json!({
            "usage": reading("claude", NOW - MIN, week(40.0)),
            "usageByEngine": [reading("claude", NOW - MIN, week(40.0)), reading("codex", NOW - 2.0 * HOUR, week(9.0))],
            "engines": { "byItem": { "w-1": "codex", "w-2": "claude" } },
            "items": [{ "id": "w-1", "product": "p", "status": "open", "updatedAt": NOW - MIN }, { "id": "w-2", "product": "p", "status": "open", "updatedAt": NOW - HOUR }],
        }));
        assert_eq!(last_used_engine(&s).as_deref(), Some("codex"));
        let usage = Usage::of(&s, NOW).unwrap();
        assert_eq!(usage.word(), "Codex");
        assert_eq!(usage.agents.iter().map(|a| a.engine.as_str()).collect::<Vec<_>>(), ["codex", "claude"]);
        assert!(usage.joint());
        assert_eq!(usage.used(NOW), Some(9.0));
        // A run going now outranks the thread touched last.
        let mut running = s.clone();
        running.supervisor.running = serde_json::from_value(json!([{ "itemId": "w-2", "product": "p", "engine": "claude", "startedAt": NOW - MIN }])).unwrap();
        assert_eq!(Usage::of(&running, NOW).unwrap().word(), "Claude Code");
        // A reading a week old has reset everywhere and is not an agent in use.
        let stale = snap(json!({ "usageByEngine": [reading("claude", NOW - MIN, week(40.0)), reading("codex", NOW - 8.0 * 24.0 * HOUR, week(9.0))] }));
        assert!(!Usage::of(&stale, NOW).unwrap().joint());
    }

    #[test]
    fn with_no_list_of_agents_the_snapshots_own_reading_is_the_card() {
        let s = snap(json!({ "usage": reading("claude", NOW - MIN, json!([limit("session", "This session", 4.0, Some(NOW + HOUR))])) }));
        let usage = Usage::of(&s, NOW).unwrap();
        assert!(usage.agents.is_empty() && !usage.joint());
        assert_eq!(usage.word(), "Claude Code");
        assert_eq!(usage.sentence(NOW), "Claude Code. This session: 4% used, 1h 0m left.");
        assert!(Usage::of(&snap(json!({})), NOW).is_none());
        // A session that started over has no figure on the row.
        let over = snap(json!({ "usage": reading("claude", NOW - 6.0 * HOUR, json!([limit("session", "This session", 71.0, Some(NOW - HOUR)), limit("week", "This week", 39.0, Some(NOW + 50.0 * HOUR))])) }));
        assert_eq!(Usage::of(&over, NOW).unwrap().used(NOW), None);
    }

    #[test]
    fn a_reading_says_how_old_it_is_quietly() {
        assert_eq!(read_words(NOW - 30_000.0, NOW), "Read just now");
        assert_eq!(read_words(NOW - 4.0 * MIN, NOW), "Read 4m ago");
        assert_eq!(read_words(NOW - 26.0 * HOUR, NOW), "Read 1d ago");
        assert_eq!(number(39.0), "39");
        assert_eq!(number(39.5), "39.5");
    }

    #[test]
    fn a_press_opens_codexs_page_for_codex_and_claude_codes_for_any_other() {
        assert_eq!(settings_page("codex"), crate::rules::settings::PageId::Codex);
        assert_eq!(settings_page("claude"), crate::rules::settings::PageId::Claude);
        assert_eq!(settings_page("antigravity"), crate::rules::settings::PageId::Claude);
    }
}
