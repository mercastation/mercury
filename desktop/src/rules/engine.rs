//! Which coding agent a row is on, and the letter and the name it wears. A port of `rowEngine`
//! and `engineLetter` in `renderer/src/home/home-model.ts`, with `harnessOf` from
//! `renderer/src/box-filter.ts` and `engineLabel` from `shared/engines.mts` behind them.

use serde_json::Value;

use crate::data::{Item, Snapshot};

/// The labels that mark a conversation as Codex's. Exact words, so a label that merely
/// contains "codex" is not one.
const CODEX_LABELS: [&str; 3] = ["codex", "codex-import", "not-imported"];

/// A row that names no other engine and carries a Codex label is Codex's (`harnessOf`, as Home
/// asks it: of the labels and the engine alone).
fn is_codex(item: &Item) -> bool {
    match item.engine.as_deref() {
        Some("codex") => true,
        Some("antigravity") => false,
        _ => item.labels.iter().any(|l| CODEX_LABELS.contains(&l.as_str()) || l.starts_with("codex:")),
    }
}

/// Which coding agent a row is on: the run that really started, else a Codex conversation,
/// else the engine's answer for its next run. The row's own `engine` is only what it asked
/// for, and is absent when it takes the workspace's default.
pub fn row_engine<'a>(item: &'a Item, snap: &'a Snapshot) -> &'a str {
    let run = snap.supervisor.running.iter().find(|r| r.item_id == item.id && r.product == item.product);
    if let Some(engine) = run.and_then(|r| r.engine.as_deref()).filter(|e| !e.is_empty()) {
        return engine;
    }
    if is_codex(item) {
        return "codex";
    }
    let engines = snap.rest.get("engines");
    let for_item = engines.and_then(|e| e.get("byItem")).and_then(|by| by.get(&item.id)).and_then(Value::as_str);
    let workspace = engines.and_then(|e| e.get("workspace")).and_then(Value::as_str);
    for_item.or(workspace).or(item.engine.as_deref()).unwrap_or("claude")
}

/// The harness a thread of the person's own runs on, as the Inbox table's mark names it
/// (`harnessOn` in `renderer/src/fleet.ts`, read as the row's byline reads it): a conversation
/// brought in from Codex, whatever the workspace runs; else what runs on it now; else what it
/// would start on (`byItem[id] ?? workspace`); else Claude Code, where a run naming none starts.
/// Unlike Home's `row_engine`, the run is the thread's by its id alone (the fleet keeps its runs
/// in a map by id, the last one written standing), and the row's own ask counts for nothing.
/// The answer is the harness's id: `claude`, `codex`, `antigravity`, or whatever else a run or
/// the engine named, which has no mark of its own (`harness_mark`).
pub fn harness_on<'a>(item: &'a Item, snap: &'a Snapshot) -> &'a str {
    if item.labels.iter().any(|l| CODEX_LABELS.contains(&l.as_str())) {
        return "codex";
    }
    if let Some(engine) = snap.supervisor.running.iter().rev().find(|r| r.item_id == item.id).and_then(|r| r.engine.as_deref()) {
        return engine;
    }
    let engines = snap.rest.get("engines");
    let for_item = engines.and_then(|e| e.get("byItem")).and_then(|by| by.get(&item.id)).and_then(Value::as_str);
    let workspace = engines.and_then(|e| e.get("workspace")).and_then(Value::as_str);
    for_item.or(workspace).unwrap_or("claude")
}

/// Which mark a harness wears in its tile (`HarnessMark` in `renderer/src/components/
/// EngineLogos.tsx`): its maker's, or `face`, the agent's two eyes, for one with no mark.
pub fn harness_mark(harness: &str) -> &'static str {
    match harness {
        "claude" => "claude",
        "codex" => "codex",
        "antigravity" => "antigravity",
        _ => "face",
    }
}

/// The letter on a row's tile: C for Claude Code, X for Codex, A for Antigravity, so they read apart.
pub fn engine_letter(engine: &str) -> &'static str {
    match engine {
        "codex" => "X",
        "antigravity" => "A",
        _ => "C",
    }
}

/// The engine's name. One the app does not offer reads as Claude Code, the default.
pub fn engine_label(engine: &str) -> &'static str {
    match engine {
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        _ => "Claude Code",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn engine(item: serde_json::Value, snap: serde_json::Value) -> String {
        let snap: Snapshot = serde_json::from_value(snap).unwrap();
        let item: Item = serde_json::from_value(item).unwrap();
        row_engine(&item, &snap).to_string()
    }

    #[test]
    fn the_run_that_started_then_a_codex_conversation_then_the_engines_answer() {
        let row = json!({ "id": "w-1", "product": "shop", "labels": [] });
        assert_eq!(engine(row.clone(), json!({})), "claude");
        assert_eq!(engine(row.clone(), json!({ "engines": { "workspace": "codex" } })), "codex");
        assert_eq!(engine(row.clone(), json!({ "engines": { "workspace": "codex", "byItem": { "w-1": "antigravity" } } })), "antigravity");
        assert_eq!(engine(row.clone(), json!({ "supervisor": { "running": [{ "itemId": "w-1", "product": "shop", "engine": "codex" }] }, "engines": { "workspace": "claude" } })), "codex");
        // A run on a row of the same id in another project is not this row's.
        assert_eq!(engine(row, json!({ "supervisor": { "running": [{ "itemId": "w-1", "product": "desk", "engine": "codex" }] } })), "claude");
        let imported = json!({ "id": "w-2", "product": "shop", "labels": ["codex", "codex:abc"] });
        assert_eq!(engine(imported, json!({ "engines": { "workspace": "claude" } })), "codex");
        let asked = json!({ "id": "w-3", "product": "shop", "labels": ["codex:abc"], "engine": "antigravity" });
        assert_eq!(engine(asked, json!({})), "antigravity");
        assert_eq!((engine_letter("codex"), engine_letter("antigravity"), engine_letter("claude"), engine_letter("opencode")), ("X", "A", "C", "C"));
        assert_eq!((engine_label("codex"), engine_label("opencode")), ("Codex", "Claude Code"));
    }

    fn mark(item: serde_json::Value, snap: serde_json::Value) -> &'static str {
        let snap: Snapshot = serde_json::from_value(snap).unwrap();
        let item: Item = serde_json::from_value(item).unwrap();
        harness_mark(harness_on(&item, &snap))
    }

    // The web's own cases (`tests/your-own-thread-wears-the-mark-of-its-harness.test.mts`).
    #[test]
    fn a_thread_of_your_own_wears_the_mark_of_its_harness() {
        let row = json!({ "id": "w-1", "product": "app", "labels": ["founder"] });
        let on_claude = json!({ "engines": { "workspace": "claude", "byItem": {} } });
        assert_eq!(mark(row.clone(), on_claude.clone()), "claude");
        assert_eq!(mark(row.clone(), json!({ "engines": { "workspace": "codex", "byItem": {} } })), "codex");
        assert_eq!(mark(row.clone(), json!({ "engines": { "workspace": "claude", "byItem": { "w-1": "codex" } } })), "codex");
        // Each row is read on its own: one the supervisor did not name stays on the workspace's.
        let other = json!({ "id": "w-2", "product": "app", "labels": ["founder"] });
        assert_eq!(mark(other, json!({ "engines": { "workspace": "claude", "byItem": { "w-1": "codex" } } })), "claude");
        // What runs on it now over what would run next.
        assert_eq!(mark(row.clone(), json!({ "engines": { "workspace": "claude", "byItem": {} }, "supervisor": { "running": [{ "itemId": "w-1", "product": "app", "engine": "codex" }] } })), "codex");
        // A conversation brought in from Codex, whatever the workspace runs; a label that merely
        // contains the word is not one.
        assert_eq!(mark(json!({ "id": "w-1", "product": "app", "labels": ["codex-import"] }), on_claude.clone()), "codex");
        assert_eq!(mark(json!({ "id": "w-1", "product": "app", "labels": ["codex-notes"] }), on_claude.clone()), "claude");
        // Claude Code when the snapshot names no engine, as a run with none starts there.
        assert_eq!(mark(row.clone(), json!({})), "claude");
        // A harness with no mark keeps the agent's face.
        assert_eq!(mark(row, json!({ "engines": { "workspace": "opencode", "byItem": {} } })), "face");
    }

    #[test]
    fn the_rows_mark_takes_the_codex_label_before_the_run_and_never_the_rows_own_ask() {
        // Home's engine takes the run first and the row's own ask last; the mark does neither.
        let imported = json!({ "id": "w-1", "product": "app", "labels": ["codex-import"] });
        assert_eq!(mark(imported, json!({ "supervisor": { "running": [{ "itemId": "w-1", "product": "app", "engine": "antigravity" }] } })), "codex");
        let asked = json!({ "id": "w-1", "product": "app", "labels": [], "engine": "codex" });
        assert_eq!(mark(asked, json!({ "engines": { "workspace": "claude", "byItem": {} } })), "claude");
        // The run is the thread's by its id, whichever project the run says.
        let row = json!({ "id": "w-1", "product": "app", "labels": [] });
        assert_eq!(mark(row, json!({ "supervisor": { "running": [{ "itemId": "w-1", "product": "desk", "engine": "codex" }] } })), "codex");
    }
}
