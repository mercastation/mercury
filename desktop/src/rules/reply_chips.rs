//! The reply box's two chips besides the model: the priority the next reply sets, and what it may
//! do. Ported from `DockComposer` in `renderer/src/components/Focus.tsx`, with
//! `renderer/src/priority.ts`, `renderer/src/modes.ts` and `shared/codex-modes.mts`; and the
//! little arrow that folds the agent's choices away (`renderer/src/one-thing-at-a-time.ts`).
//!
//! Neither chip writes anything when it is picked. The box keeps the pick while it is open and
//! the next reply carries it: the priority only once it was picked (an untouched chip shows the
//! thread's own level and must not write it), what it may do always, the thread's own mode or
//! `null` until a pick changes it, `null` clearing a mode set on an earlier reply. Folding the box
//! lets both go, as the web window's box is unmounted: neither is kept with the draft.

use serde_json::{Map, Value, json};

use crate::data::{Item, Snapshot};
use crate::rules::compose::{Level, sent_priority};

/// Claude Code's six, in the order of its own mode table, each with its title there
/// (`MODE_ORDER`, `MODE_WORDS`).
const CLAUDE: [(&str, &str); 6] = [
    ("default", "Manual"),
    ("acceptEdits", "Accept edits"),
    ("plan", "Plan"),
    ("auto", "Auto"),
    ("dontAsk", "Don't Ask"),
    ("bypassPermissions", "Bypass Permissions"),
];
/// Codex's three, by what they let an agent do (`CODEX_MODE_ORDER`, `CODEX_MODES`).
const CODEX: [(&str, &str); 3] = [("read-only", "Read only"), ("auto", "Auto"), ("full-access", "Full access")];
/// Shift+Tab's ring on Claude Code: the four that need no second thought (`MODE_CYCLE`).
const CYCLE: [&str; 4] = ["default", "acceptEdits", "plan", "auto"];

/// Whether the thread reaches Claude Code, which decides whose modes the chip offers: the agent
/// it is on now, not one a model picked in the box would move it to (`claudeCode`). Every other
/// agent is offered Codex's three.
pub fn claude_code(running_engine: Option<&str>) -> bool {
    running_engine.unwrap_or("claude") == "claude"
}

/// The modes the chip's menu lists, as `(value, label)`.
pub fn rows(claude: bool) -> &'static [(&'static str, &'static str)] {
    if claude { &CLAUDE } else { &CODEX }
}

/// The project's own mode, which a thread with none of its own runs in (`runningMode`:
/// `config.permission[product] ?? config.permission['']`).
pub fn project_mode<'a>(snap: &'a Snapshot, item: &Item) -> Option<&'a str> {
    let permission = snap.rest.get("config").and_then(|c| c.get("permission"));
    permission.and_then(|p| p.get(&item.product).filter(|v| !v.is_null()).or_else(|| p.get(""))).and_then(Value::as_str)
}

/// What the next reply runs as: the mode the box holds, else the project's on Claude Code and
/// Auto on any other agent (`running`).
pub fn running(mode: Option<&str>, claude: bool, project: Option<&str>) -> String {
    mode.map_or_else(|| if claude { project.unwrap_or("auto").to_string() } else { "auto".to_string() }, str::to_string)
}

/// Every word's first letter in capitals (`titleCase`): "Accept Edits".
pub fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut start = true;
    for c in s.chars() {
        if start && !c.is_whitespace() {
            out.extend(c.to_uppercase());
        } else {
            out.push(c);
        }
        start = c.is_whitespace();
    }
    out
}

/// What the chip says: the mode's label in title case, or its own word where the list has none.
pub fn chip_word(value: &str, claude: bool) -> String {
    title_case(rows(claude).iter().find(|(v, _)| *v == value).map_or(value, |(_, label)| label))
}

/// Shift+Tab: the next mode round the ring. On Claude Code a mode outside it (Bypass
/// Permissions, Don't Ask, one set by hand) goes to Manual, so no key walks anybody into Bypass
/// (`nextMode`); on any other agent round Codex's three (`nextCodexMode`).
pub fn next(running: &str, claude: bool) -> &'static str {
    if claude {
        return CYCLE.iter().position(|m| *m == running).map_or("default", |at| CYCLE[(at + 1) % CYCLE.len()]);
    }
    let at = CODEX.iter().position(|(m, _)| *m == running).map_or(0, |at| at + 1);
    CODEX[at % CODEX.len()].0
}

/// What the agent's own status line says while a mode is on (`MODE_STATUS`, `CODEX_MODE_STATUS`).
fn status(mode: &str, claude: bool) -> &'static str {
    match (claude, mode) {
        (true, "default") => "manual mode on",
        (true, "acceptEdits") => "accept edits on",
        (true, "plan") => "plan mode on",
        (true, "auto") => "auto mode on",
        (true, "dontAsk") => "don't ask on",
        (true, "bypassPermissions") => "bypass permissions on",
        (true, "custom") => "set by hand",
        (false, "read-only") => "read only on",
        (false, "auto") => "auto on",
        (false, "full-access") => "full access on",
        // A word the agent does not know reads as JavaScript prints a missing entry.
        _ => "undefined",
    }
}

/// The line a change of mode says, since nothing else on screen says it (`announce`): the mode
/// for this thread, or, cleared, what the project's own setting runs as.
pub fn announce(mode: Option<&str>, claude: bool, project: Option<&str>) -> String {
    match mode {
        Some(mode) => format!("Permissions · {} for this thread", status(mode, claude)),
        None => format!("Permissions · back to this project's setting, {}", status(&running(None, claude, project), claude)),
    }
}

/// The words in the box a mode pick clears: a slash and one word, the slash menu's query
/// (`slashQuery`). Anything else she typed stays.
pub fn clears_the_words(text: &str) -> bool {
    text.strip_prefix('/').is_some_and(|rest| !rest.chars().any(char::is_whitespace))
}

/// The level the chip shows untouched: the one the thread runs at, an untagged one as Medium
/// (`priorityIdOf(shownPriority(item))`).
pub fn shown_level(item: &Item) -> Level {
    Level::of(item.priority)
}

/// What a reply carries besides its words and its model: the priority picked, and what it may
/// do. The mode always goes, `null` included, which clears one set on an earlier reply
/// (`...(priority != null ? { priority } : {})`, `permissionMode: mode`).
#[derive(Clone, Debug, PartialEq)]
pub struct Said {
    pub level: Option<Level>,
    pub mode: Option<String>,
}

impl Said {
    /// Untouched chips: no priority, and the thread's own mode.
    pub fn untouched(item: &Item) -> Said {
        Said { level: None, mode: item.text("answerMode").map(str::to_string) }
    }

    pub fn fields(&self) -> Map<String, Value> {
        let mut fields = Map::new();
        if let Some(level) = self.level {
            fields.insert("priority".to_string(), json!(sent_priority(Some(level))));
        }
        fields.insert("permissionMode".to_string(), self.mode.as_deref().map_or(Value::Null, |m| json!(m)));
        fields
    }
}

/* -------------------------- the choices' little arrow -------------------------- */

/// How many folds are kept (`FOLDS_KEPT`).
const FOLDS_KEPT: usize = 100;

/// Which choices a fold holds: one thread's current round of them. Another thread, or the same
/// thread's next round, opens unfolded (`foldKey`).
pub fn fold_key(item: &Item) -> String {
    let field = crate::rules::format::options_from(item);
    let ts = item.wrote(field).and_then(|w| w.ts).unwrap_or(0.0);
    format!("{}:{}:{}", item.product, item.id, crate::rules::usage::number(ts))
}

/// Folds or unfolds one round, keeping the newest hundred (`toggleFold`).
pub fn toggle_fold(list: &[String], key: &str) -> Vec<String> {
    if list.iter().any(|k| k == key) {
        return list.iter().filter(|k| *k != key).cloned().collect();
    }
    let mut out = list.to_vec();
    out.push(key.to_string());
    let extra = out.len().saturating_sub(FOLDS_KEPT);
    out.split_off(extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thread(extra: Value) -> Item {
        let mut v = json!({ "id": "w-1", "product": "shop", "status": "blocked" });
        for (k, value) in extra.as_object().unwrap() {
            v[k] = value.clone();
        }
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn claude_codes_modes_are_offered_only_to_a_thread_on_claude_code() {
        assert!(claude_code(None) && claude_code(Some("claude")));
        assert!(!claude_code(Some("codex")) && !claude_code(Some("antigravity")));
        assert_eq!(rows(true).iter().map(|(_, l)| *l).collect::<Vec<_>>(), ["Manual", "Accept edits", "Plan", "Auto", "Don't Ask", "Bypass Permissions"]);
        assert_eq!(rows(false).iter().map(|(_, l)| *l).collect::<Vec<_>>(), ["Read only", "Auto", "Full access"]);
    }

    #[test]
    fn the_chip_says_the_mode_the_next_reply_runs_in_title_cased() {
        let snap: Snapshot = serde_json::from_value(json!({ "config": { "permission": { "": "auto", "shop": "bypassPermissions" } } })).unwrap();
        let project = project_mode(&snap, &thread(json!({})));
        assert_eq!(project, Some("bypassPermissions"));
        assert_eq!(chip_word(&running(None, true, project), true), "Bypass Permissions");
        assert_eq!(chip_word(&running(Some("acceptEdits"), true, project), true), "Accept Edits");
        assert_eq!(project_mode(&snap, &thread(json!({ "product": "desk" }))), Some("auto"));
        // Codex runs Auto with nothing picked, whatever the project says for Claude Code.
        assert_eq!(chip_word(&running(None, false, project), false), "Auto");
        assert_eq!(chip_word("custom", true), "Custom");
        assert_eq!(chip_word("read-only", true), "Read-only");
    }

    #[test]
    fn shift_tab_walks_the_ring_and_never_into_bypass() {
        assert_eq!(next("default", true), "acceptEdits");
        assert_eq!(next("auto", true), "default");
        assert_eq!(next("bypassPermissions", true), "default");
        assert_eq!(next("dontAsk", true), "default");
        assert_eq!(next("read-only", false), "auto");
        assert_eq!(next("full-access", false), "read-only");
        assert_eq!(next("plan", false), "read-only");
    }

    #[test]
    fn a_change_of_mode_says_what_the_thread_now_runs_as() {
        assert_eq!(announce(Some("plan"), true, None), "Permissions · plan mode on for this thread");
        assert_eq!(announce(Some("full-access"), false, None), "Permissions · full access on for this thread");
        assert_eq!(announce(None, true, Some("bypassPermissions")), "Permissions · back to this project's setting, bypass permissions on");
        assert_eq!(announce(None, false, Some("bypassPermissions")), "Permissions · back to this project's setting, auto on");
    }

    #[test]
    fn a_reply_carries_a_picked_priority_and_always_its_mode() {
        let untouched = Said::untouched(&thread(json!({})));
        assert_eq!(Value::Object(untouched.fields()), json!({ "permissionMode": null }));
        assert_eq!(Value::Object(Said::untouched(&thread(json!({ "answerMode": "plan" }))).fields()), json!({ "permissionMode": "plan" }));
        let picked = Said { level: Some(Level::High), mode: Some("acceptEdits".into()) };
        assert_eq!(Value::Object(picked.fields()), json!({ "priority": 7.0, "permissionMode": "acceptEdits" }));
        // Untagged and an agent's own 8 show as they run.
        assert_eq!(shown_level(&thread(json!({}))), Level::Medium);
        assert_eq!(shown_level(&thread(json!({ "priority": 8 }))), Level::High);
    }

    #[test]
    fn a_pick_clears_only_a_slash_query() {
        assert!(clears_the_words("/pl") && clears_the_words("/"));
        assert!(!clears_the_words("/plan and then") && !clears_the_words("Ship it."));
    }

    /// `tests/a-little-arrow-folds-the-choices-at-the-foot.test.mts`.
    #[test]
    fn a_fold_holds_one_threads_current_round_of_choices() {
        let at = |id: &str, ts: f64| thread(json!({ "id": id, "product": "p", "wrote": { "result": { "ts": ts } }, "result": "## Options\n1. A\n2. B" }));
        assert_eq!(fold_key(&at("w-1", 5.0)), "p:w-1:5");
        let folded = toggle_fold(&[], &fold_key(&at("w-1", 5.0)));
        assert!(folded.contains(&fold_key(&at("w-1", 5.0))));
        assert!(!folded.contains(&fold_key(&at("w-2", 5.0))));
        assert!(!folded.contains(&fold_key(&at("w-1", 9.0))));
        assert!(toggle_fold(&folded, &fold_key(&at("w-1", 5.0))).is_empty());
        let mut list = Vec::new();
        for i in 0..150 {
            list = toggle_fold(&list, &format!("p:w-{i}:1"));
        }
        assert_eq!(list.len(), 100);
        assert_eq!(list.last().map(String::as_str), Some("p:w-149:1"));
    }
}
