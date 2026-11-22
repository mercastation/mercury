//! Which model and level a reply runs on: what the reply box opens carrying, what a pick in its
//! menu keeps, and what the next reply writes. A port of `renderer/src/reply-pick.ts`, with the
//! part of `DockComposer` (`renderer/src/components/Focus.tsx`) that holds the pick.
//!
//! A pick is about the next run. Nothing is written when it is made: it is kept with the box's
//! words, a fold keeps it, and only the reply that follows writes it, the model before the words
//! (`answerItem` in `main/store.mts`), so the run that reply starts reads it.
//!
//! Two words for nothing, as in the web window: the store says "nothing chosen" with an empty
//! string, and the box says "the agent's own default" with no model at all. `pick_to_send` turns
//! the second into what the store needs: Claude Code always gets a model, the other agents only
//! one that is named, since with none named their own settings decide.

use serde_json::{Map, Value, json};

use crate::data::{Item, Snapshot};
use crate::rules::compose::{self, Engines, Models, Pick};

/// A model and a level as the reply box holds them, `None` being the agent's own (`ReplyPick`).
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ReplyPick {
    pub model: Option<String>,
    pub effort: Option<String>,
    /// The agent the pick runs on, once a pick in the menu named one: the thread moves to it
    /// when it is not the one the thread is on (`engineChosen`).
    pub engine: Option<String>,
}

/// What is set on the thread itself, the store's empty string read as nothing (`pickOnTheRow`).
pub fn pick_on_the_row(item: &Item) -> ReplyPick {
    ReplyPick { model: item.text("model").map(str::to_string), effort: item.text("effort").map(str::to_string), engine: None }
}

/// What the box opens on, and whether it is the person's own (`replyOpensOn`): the pick she
/// left in the box, else the pick of a reply still on its way, else the thread's own. Only her
/// own is written with the next reply: a box must not write what it is merely showing.
pub fn reply_opens_on(item: &Item, kept: Option<&ReplyPick>, on_its_way: Option<&ReplyPick>) -> (ReplyPick, bool) {
    match (kept, on_its_way) {
        (Some(kept), _) => (kept.clone(), true),
        (None, Some(going)) => (going.clone(), false),
        (None, None) => (pick_on_the_row(item), false),
    }
}

/// The agent the thread's next reply reaches: its own, else the one Settings names, as the
/// engine hands both over (`runningEngine` in `renderer/src/App.tsx`).
pub fn running_engine(snap: &Snapshot, id: &str) -> Option<String> {
    let engines = snap.rest.get("engines");
    let by_item = engines.and_then(|e| e.get("byItem")).and_then(|b| b.get(id)).and_then(Value::as_str);
    by_item.or_else(|| engines.and_then(|e| e.get("workspace")).and_then(Value::as_str)).map(str::to_string)
}

/// One of the three agents a pick can name, else none (`asHarness`).
fn harness(id: Option<&str>) -> Option<&str> {
    id.filter(|id| matches!(*id, "claude" | "codex" | "antigravity"))
}

/// The agent the thread is on now, and the one the box shows: the same until a pick in the
/// menu names the other (`engineNow`, `engineShown`).
pub fn engines_of(running: Option<&str>, pick: &ReplyPick) -> (String, String) {
    let now = harness(running).unwrap_or("claude").to_string();
    let shown = harness(pick.engine.as_deref()).map_or_else(|| now.clone(), str::to_string);
    (now, shown)
}

/// The pick as the menu draws and walks it: the agent shown and its model, the agent's own
/// default where none is named (`pick` in `DockComposer`).
pub fn menu_pick(pick: &ReplyPick, shown: &str, models: &Models) -> Pick {
    Pick::new(shown, &pick.model.clone().unwrap_or_else(|| compose::default_model_for(shown, models.codex_default.as_deref())))
}

/// The word on the reply box's chip, and on the folded box: the model's own name, the agent's
/// default where none is named (`engineModelLabel`).
pub fn model_word(engine: Option<&str>, model: Option<&str>, models: &Models) -> String {
    let engine = engine.unwrap_or("claude");
    let model = model.map_or_else(|| compose::default_model_for(engine, models.codex_default.as_deref()), str::to_string);
    compose::model_label(&Pick::new(engine, &model), models)
}

/// The agent and model the folded box names: the row's, or those of a reply still on its way
/// to it (`modelShown`). The pick kept in the box is not the row's yet, and is not named here.
pub fn model_shown(item: &Item, running: Option<&str>, on_its_way: Option<&ReplyPick>) -> (Option<String>, Option<String>) {
    match on_its_way {
        None => (running.map(str::to_string), pick_on_the_row(item).model),
        Some(going) => (going.engine.clone().or_else(|| running.map(str::to_string)), going.model.clone()),
    }
}

/// A model picked in the menu: the agent and the model, and a level picked for another agent's
/// model is let go, since it is a word that agent may not know (`pickModel`).
pub fn picking_model(was: &ReplyPick, shown: &str, to: &Pick) -> ReplyPick {
    let effort = if to.engine == shown { was.effort.clone() } else { None };
    ReplyPick { model: Some(to.model.clone()).filter(|m| !m.is_empty()), effort, engine: Some(to.engine.clone()) }
}

/// A level pressed in the menu's strip, toggled against her own: pressing the lit default sets
/// it, pressing her own lets it go (`onEffort`).
pub fn picking_effort(was: &ReplyPick, level: &str) -> ReplyPick {
    let effort = if was.effort.as_deref() == Some(level) { None } else { Some(level.to_string()) };
    ReplyPick { effort, ..was.clone() }
}

/// The pick as the thread must record it (`pickToSend`). Claude Code has a name for every model
/// it offers, so nothing is lost by naming its default; Codex and Antigravity have none for
/// theirs, and nothing named lets their own settings decide. The agent is written only when the
/// pick moves the thread to another one, and only where this Mac offers more than one.
pub fn pick_to_send(pick: &ReplyPick, shown: &str, was: &str, engine_count: usize) -> ReplyPick {
    let model = if matches!(shown, "codex" | "antigravity") { pick.model.clone() } else { Some(pick.model.clone().unwrap_or_else(|| compose::DEFAULT_MODEL.to_string())) };
    let moves = engine_count > 1 && !was.is_empty() && !shown.is_empty() && shown != was;
    ReplyPick { model, effort: pick.effort.clone(), engine: moves.then(|| shown.to_string()) }
}

/// What a reply carries of the pick, beside its words: the model and the level always, `null`
/// where there is none, which the store writes as the empty string that clears it, and the agent
/// only when it moves (`...(pick ?? {})` in `answerWith`).
pub fn answer_fields(sent: &ReplyPick) -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert("model".to_string(), sent.model.as_deref().map_or(Value::Null, |m| json!(m)));
    fields.insert("effort".to_string(), sent.effort.as_deref().map_or(Value::Null, |e| json!(e)));
    if let Some(engine) = &sent.engine {
        fields.insert("engine".to_string(), json!(engine));
    }
    fields
}

/// The level beside the model's name on the chip, or nothing: only a level that will really be
/// sent, one this model takes (`effortSaid`). It is read against the agent the thread is on now,
/// as the web window reads it.
pub fn effort_said(pick: &ReplyPick, running: Option<&str>, models: &Models) -> Option<String> {
    let engine = running.unwrap_or("claude");
    let model = pick.model.clone().unwrap_or_else(|| compose::default_model_for(engine, models.codex_default.as_deref()));
    let rows = compose::effort_choices(&Pick::new(engine, &model), models);
    let effort = pick.effort.as_deref().filter(|e| compose::effort_picked(Some(e), &rows))?;
    rows.into_iter().find(|(id, _)| id == effort).map(|(_, label)| label)
}

/// The line under the words when the pick moves the thread to another agent: its session cannot
/// cross, so what runs next starts fresh with the thread (`freshNote`).
pub fn fresh_note(engines: &Engines, now: &str, shown: &str) -> Option<String> {
    (now != shown).then(|| format!("{} starts fresh with this thread.", engines.rows().into_iter().find(|(id, _)| id == shown).map_or_else(|| "It".to_string(), |(_, label)| label)))
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

    fn pick(model: Option<&str>, effort: Option<&str>, engine: Option<&str>) -> ReplyPick {
        ReplyPick { model: model.map(str::to_string), effort: effort.map(str::to_string), engine: engine.map(str::to_string) }
    }

    #[test]
    fn the_box_opens_on_her_pick_else_one_on_its_way_else_the_threads_own() {
        let row = thread(json!({ "model": "sonnet", "effort": "" }));
        // The store's empty string is nothing.
        assert_eq!(reply_opens_on(&row, None, None), (pick(Some("sonnet"), None, None), false));
        let going = pick(Some("haiku"), Some("low"), None);
        assert_eq!(reply_opens_on(&row, None, Some(&going)), (going.clone(), false));
        // A kept pick of the agent's own default is hers, and does not fall through to the row's.
        let kept = pick(None, None, Some("codex"));
        assert_eq!(reply_opens_on(&row, Some(&kept), Some(&going)), (kept, true));
    }

    #[test]
    fn a_send_names_claudes_default_and_leaves_the_others_to_their_settings() {
        // Picking Claude Code's default pins it, rather than unpinning the thread.
        assert_eq!(pick_to_send(&pick(None, Some("high"), None), "claude", "claude", 1), pick(Some("opus"), Some("high"), None));
        assert_eq!(pick_to_send(&pick(Some("sonnet"), None, Some("claude")), "claude", "claude", 2), pick(Some("sonnet"), None, None));
        // Codex's own default is said by naming none.
        assert_eq!(pick_to_send(&pick(None, None, Some("codex")), "codex", "claude", 2), pick(None, None, Some("codex")));
        // The agent is written only where it moves the thread and this Mac has two to move between.
        assert_eq!(pick_to_send(&pick(Some("gpt-6-astra"), None, Some("codex")), "codex", "claude", 1).engine, None);
        assert_eq!(pick_to_send(&pick(Some("gpt-6-astra"), None, Some("codex")), "codex", "codex", 2).engine, None);
    }

    #[test]
    fn a_reply_carries_the_model_and_the_level_and_clears_with_null() {
        let fields = answer_fields(&pick(Some("sonnet"), None, None));
        assert_eq!(Value::Object(fields), json!({ "model": "sonnet", "effort": null }));
        let fields = answer_fields(&pick(None, Some("low"), Some("codex")));
        assert_eq!(Value::Object(fields), json!({ "model": null, "effort": "low", "engine": "codex" }));
    }

    #[test]
    fn a_model_picked_keeps_the_level_on_its_own_agent_and_drops_it_on_another() {
        let was = pick(Some("opus"), Some("high"), None);
        assert_eq!(picking_model(&was, "claude", &Pick::new("claude", "sonnet")), pick(Some("sonnet"), Some("high"), Some("claude")));
        assert_eq!(picking_model(&was, "claude", &Pick::new("codex", "gpt-6-astra")), pick(Some("gpt-6-astra"), None, Some("codex")));
        // Codex's own is the agent's default, named by no model.
        assert_eq!(picking_model(&was, "claude", &Pick::new("codex", "")).model, None);
        // A level is toggled against her own: pressed again, it is let go.
        assert_eq!(picking_effort(&was, "high").effort, None);
        assert_eq!(picking_effort(&was, "max").effort.as_deref(), Some("max"));
    }

    #[test]
    fn the_folded_box_names_the_rows_model_or_the_one_on_its_way() {
        let models = Models::built();
        let row = thread(json!({ "model": "sonnet" }));
        let (engine, model) = model_shown(&row, None, None);
        assert_eq!(model_word(engine.as_deref(), model.as_deref(), &models), "Sonnet 5.5");
        let (engine, model) = model_shown(&thread(json!({})), None, None);
        assert_eq!(model_word(engine.as_deref(), model.as_deref(), &models), "Opus 5.5");
        let going = pick(Some("haiku"), None, None);
        let (engine, model) = model_shown(&row, Some("claude"), Some(&going));
        assert_eq!(model_word(engine.as_deref(), model.as_deref(), &models), "Haiku 5.5");
    }

    #[test]
    fn the_chip_says_a_level_only_when_one_will_be_sent() {
        let models = Models::built();
        assert_eq!(effort_said(&pick(None, Some("xhigh"), None), None, &models).as_deref(), Some("Extra"));
        assert_eq!(effort_said(&pick(None, None, None), None, &models), None);
        assert_eq!(effort_said(&pick(None, Some("ultra"), None), None, &models), None);
    }

    #[test]
    fn moving_to_another_agent_says_it_starts_fresh() {
        let engines = Engines { choices: vec![("claude".to_string(), "Claude Code".to_string()), ("codex".to_string(), "Codex".to_string())], workspace: None };
        assert_eq!(fresh_note(&engines, "claude", "codex").as_deref(), Some("Codex starts fresh with this thread."));
        assert_eq!(fresh_note(&engines, "claude", "claude"), None);
        assert_eq!(engines_of(Some("codex"), &pick(None, None, None)), ("codex".to_string(), "codex".to_string()));
        assert_eq!(engines_of(None, &pick(None, None, Some("antigravity"))), ("claude".to_string(), "antigravity".to_string()));
    }

}
