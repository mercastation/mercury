//! A new thread: what the words typed into the box become, where it is filed, who runs it and
//! how hard, and what the window says while it goes and once it has. A port of the rules under
//! the web window's New thread card (`renderer/src/threads/composer-rules.ts`) and of the files
//! those read: `message-split.ts`, `priority.ts`, `models.ts`, `engines.ts`, `effort.ts`,
//! `compose-says.ts`, `withdraw-send.ts`, `toast-parts.ts`, `reply-keys.ts`,
//! `keep-in-window.ts` and `compose-project.ts`.
//!
//! What its files are, and how they go, is `rules/attach.rs`. Left out, with the parts of the
//! card this window does not draw yet: a thread to a person or to a teammate, a moment to
//! start at (Later, which has none, is in), a repeating task, a branch, a demand, and who sees
//! it.

use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::data::{Item, Product, Snapshot, Tab};
use crate::rules::js::{self, re};
use crate::rules::writes;

/* -------------------------------- the words ------------------------------- */

/// What a title is allowed to hold (`TITLE_BUDGET` in `renderer/src/message-split.ts`).
pub const TITLE_BUDGET: usize = 120;
/// Short enough to be an abbreviation rather than a sentence.
const FIRST_SENTENCE_MIN: usize = 24;

// A list's mark is about the shape of the text and is never part of a name.
re!(MARK, r"^(?:[-*•]|\d{1,2}[.)])\s+", "");
re!(SENTENCE_END, r"[.!?](\s|$)", "");

/// A thread's name and its message, as the engine is handed them.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub title: String,
    pub body: String,
}

fn unmarked(line: &str) -> String {
    let bare = MARK.strip(line);
    let bare = js::trim(&bare);
    if bare.is_empty() { line.to_string() } else { bare.to_string() }
}

/// A label for a first line too long to be one: its first sentence, else as much as fits, cut
/// at a word (`titleFor`).
fn title_for(head: &str) -> String {
    if let Some((at, _)) = SENTENCE_END.matches(head).first().copied() {
        if (FIRST_SENTENCE_MIN..TITLE_BUDGET).contains(&js::len(&head[..at])) {
            return head[..=at].to_string();
        }
    }
    writes::clip_to_sentence(head, TITLE_BUDGET)
}

/// How a typed message becomes a title and a body (`splitMessage`). The title is a label and
/// may lose words; the body never does. The first line is taken out of the body only when a
/// blank line sets it apart, the way a subject is.
pub fn split_message(message: &str) -> Message {
    let text = js::trim(message);
    let lines: Vec<&str> = text.split('\n').collect();
    let head = unmarked(js::trim(lines[0]));
    if js::len(&head) > TITLE_BUDGET {
        return Message { title: title_for(&head), body: text.to_string() };
    }
    if lines.len() == 1 {
        return Message { title: head, body: String::new() };
    }
    if js::trim(lines[1]).is_empty() {
        return Message { title: head, body: js::trim(&lines[2..].join("\n")).to_string() };
    }
    Message { title: head, body: text.to_string() }
}

/// The title and body of what is in the box, or nothing when there is nothing to send
/// (`threadMessage`, with nobody named). Files with no words are a thread too, named after
/// the first of them; `first_file` is its name when the box holds any.
pub fn thread_message(text: &str, first_file: Option<&str>) -> Option<Message> {
    let trimmed = js::trim(text);
    if trimmed.is_empty() {
        return first_file.map(|name| split_message(&format!("Attached: {name}")));
    }
    Some(split_message(trimmed))
}

/* -------------------------------- priority -------------------------------- */

/// One of the four levels. The card's pick is one of them, or none for Agent Decides.
pub use crate::data::Priority as Level;

/// The four, in the order the menu lists them (`PRIORITIES` in `renderer/src/priority.ts`).
pub const LEVELS: [Level; 4] = [Level::Urgent, Level::High, Level::Medium, Level::Low];

/// What a new thread starts on: no level, so the app's small model picks one. In sentence case,
/// as every other choice on the box is (`AGENT_DECIDES` in `renderer/src/priority.ts`).
pub const AGENT_DECIDES: &str = "Agent decides";

pub fn level_word(level: Level) -> &'static str {
    match level {
        Level::Urgent => "Urgent",
        Level::High => "High",
        Level::Medium => "Medium",
        Level::Low => "Low",
    }
}

/// What the chip says (`choiceLabelOf`).
pub fn choice_label(choice: Option<Level>) -> &'static str {
    choice.map_or(AGENT_DECIDES, level_word)
}

/// The number a card sends. Nought is untagged, which is what the model is asked about
/// (`sentPriority`).
pub fn sent_priority(choice: Option<Level>) -> f64 {
    match choice {
        Some(Level::Urgent) => 9.0,
        Some(Level::High) => 7.0,
        Some(Level::Medium) => 5.0,
        Some(Level::Low) => 2.0,
        None => 0.0,
    }
}

/* -------------------------------- projects -------------------------------- */

/// The person's projects in their own running order, and anything not placed after them,
/// oldest first (`rankProducts` in `shared/rank.mts`). The menu lists them so.
pub fn rank_products<'a>(products: &'a [Product], order: &[String]) -> Vec<&'a Product> {
    let mut ranked: Vec<&Product> = order.iter().filter_map(|slug| products.iter().find(|p| &p.slug == slug)).collect();
    let made = |p: &Product| p.rest.get("createdAt").and_then(Value::as_f64).unwrap_or(0.0);
    let mut rest: Vec<&Product> = products.iter().filter(|p| !order.contains(&p.slug)).collect();
    rest.sort_by(|a, b| made(a).total_cmp(&made(b)).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    ranked.extend(rest);
    ranked
}

/// A project that only carries messages between two people (`isDirect`).
pub fn is_direct(p: &Product) -> bool {
    p.rest.get("team").and_then(|team| team.get("direct")) == Some(&Value::Bool(true))
}

fn is_practice(p: &Product) -> bool {
    p.rest.get("practice") == Some(&Value::Bool(true))
}

/// Every project a thread can be filed into: not the ones that only carry messages between
/// two people (`projectsOffered`).
pub fn projects_offered<'a>(ranked: &[&'a Product]) -> Vec<&'a Product> {
    ranked.iter().copied().filter(|p| !is_direct(p)).collect()
}

/// The project the card shows: the one picked on it, else the one on screen, else the one
/// last sent to, else the first that is not the practice one (`cardProject` and
/// `startingProject`).
pub fn card_project<'a>(offered: &[&'a Product], picked: Option<&str>, on_screen: Option<&str>, remembered: Option<&str>) -> Option<&'a Product> {
    let named = |slug: Option<&str>| slug.and_then(|slug| offered.iter().copied().find(|p| p.slug == slug));
    named(picked).or_else(|| named(on_screen)).or_else(|| named(remembered)).or_else(|| offered.iter().copied().find(|p| !is_practice(p))).or_else(|| offered.first().copied())
}

re!(ABOUT_CODE, r"\b(?:code|page|screen|button|bug|fix|build|restyle|repo|branch|merge|component|css|test suite|browser test|onboarding page)\b", "i");

fn has_code(p: &Product) -> bool {
    p.rest.get("code") == Some(&Value::Bool(true))
}

/// Where words about code go when the box points at a project with no code and nobody picked
/// one: the one other project that has code. Two with code is a guess, so the box stays; and
/// words that name the project it is on stay there too (`codeProjectFor` in
/// `renderer/src/compose-project.ts`).
pub fn code_project_for<'a>(text: &str, offered: &[&'a Product], current: &str) -> Option<&'a Product> {
    let here = offered.iter().find(|p| p.slug == current)?;
    if has_code(here) || !ABOUT_CODE.test(text) || text.to_lowercase().contains(&here.name.to_lowercase()) {
        return None;
    }
    let with_code: Vec<&Product> = offered.iter().copied().filter(|p| has_code(p) && !is_direct(p)).collect();
    (with_code.len() == 1).then(|| with_code[0])
}

/// The note that says so, and the way back to the project the box was on.
pub fn going_to_code(to: &str) -> String {
    format!("Going to {to}, where the code is.")
}

pub fn keep(name: &str) -> String {
    format!("Keep {name}")
}

/// What the menu says after a project's name, when it says anything: whose it is, where the
/// folder is not this Mac's.
pub fn project_note(p: &Product) -> Option<&'static str> {
    let shared = p.rest.get("team").and_then(|team| team.get("projectId")).is_some_and(|id| crate::data::truthy(Some(id)));
    let folder = crate::data::truthy(p.rest.get("repoPath"));
    if shared { Some("Shared with your team") } else if folder { None } else { Some("For personal tasks") }
}

/// Why Send is off for want of a project, in two halves: the fact, and the way out
/// (`noProjectYet` in `renderer/src/compose-says.ts`).
pub const NO_PROJECT_REASON: &str = "Nowhere to send this yet.";
pub const NO_PROJECT_ACTION: &str = "Make your first project";

/// Nothing ever runs in the tutorial's practice project, so the card says no before anything
/// is sent there (`practiceRefusal`).
pub const PRACTICE_REFUSAL: &str = "Nothing runs in Practice, it is only somewhere to try things out. Pick one of your own projects and this will start.";

pub fn practice_refusal(product: Option<&Product>) -> Option<&'static str> {
    product.filter(|p| is_practice(p)).map(|_| PRACTICE_REFUSAL)
}

/* ------------------------------ agents, models ---------------------------- */

/// One model a thread can be handed to (`ModelChoice` in `renderer/src/models.ts`).
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Model {
    /// What the agent is sent: an alias for Claude Code, a slug for the others.
    pub id: String,
    pub label: String,
    /// The version a Claude Code alias stands for today.
    pub model: Option<String>,
    /// The reasoning levels a Codex or Antigravity model takes. Claude Code's are the same
    /// five for every model.
    pub levels: Vec<String>,
    pub default_level: Option<String>,
}

/// The models this Mac's coding agents offer. The web window reads them with its settings
/// (`zero:settings-read`), not off the snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct Models {
    pub claude: Vec<Model>,
    pub codex: Vec<Model>,
    /// What the person's own Codex runs when no model is named, when its settings say.
    pub codex_default: Option<String>,
    pub antigravity: Vec<Model>,
}

impl Default for Models {
    fn default() -> Models {
        Models::built()
    }
}

impl Models {
    /// Claude Code's models as the app was built with them, which is what every window draws
    /// until the engine has said otherwise (`shared/claude-models.generated.mts`).
    pub fn built() -> Models {
        let row = |id: &str, model: &str, label: &str, level: Option<&str>| Model { id: id.to_string(), label: label.to_string(), model: Some(model.to_string()), levels: Vec::new(), default_level: level.map(str::to_string) };
        let claude = vec![
            row("opus", "claude-opus-5-5", "Opus 5.5", Some("medium")),
            row("claude-opus-5", "claude-opus-5", "Opus 5", Some("high")),
            row("sonnet", "claude-sonnet-5-5", "Sonnet 5.5", Some("medium")),
            row("claude-sonnet-5", "claude-sonnet-5", "Sonnet 5", Some("high")),
            row("haiku", "claude-haiku-5-5", "Haiku 5.5", Some("medium")),
            row("claude-haiku-4-5", "claude-haiku-4-5", "Haiku 4.5", None),
            row("fable", "claude-fable-5-1", "Fable 5.1", Some("high")),
            row("claude-fable-5", "claude-fable-5", "Fable 5", Some("high")),
        ];
        Models { claude, codex: Vec::new(), codex_default: None, antigravity: Vec::new() }
    }

    /// What a settings read says this Mac offers. A list it does not carry is the built one for
    /// Claude Code and an empty one for the others.
    pub fn from_settings(settings: &Value) -> Models {
        let workspace = settings.get("workspace");
        let rows = |name: &str| -> Vec<Model> {
            let list = workspace.and_then(|w| w.get(name)).and_then(Value::as_array).cloned().unwrap_or_default();
            list.into_iter().filter_map(|row| serde_json::from_value::<Model>(row).ok()).filter(|m| !m.label.is_empty()).collect()
        };
        let claude = rows("claudeModels");
        let named: Vec<Model> = rows("antigravityModels").into_iter().filter(|m| !m.id.is_empty()).collect();
        Models {
            claude: if claude.is_empty() { Models::built().claude } else { claude },
            codex: rows("codexModels"),
            codex_default: workspace.and_then(|w| w.get("codexModelDefault")).and_then(Value::as_str).filter(|d| !d.is_empty()).map(str::to_string),
            antigravity: named,
        }
    }
}

/// The coding agents this Mac offers and the one a new thread starts on, off the snapshot
/// (`engines.choices` and `engines.workspace`). A snapshot that names none means Claude Code
/// alone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Engines {
    /// Each as its id and its name, in the engine's order.
    pub choices: Vec<(String, String)>,
    pub workspace: Option<String>,
}

impl Engines {
    pub fn of(snap: &Snapshot) -> Engines {
        let engines = snap.rest.get("engines");
        let text = |row: &Value, field: &str| row.get(field).and_then(Value::as_str).unwrap_or("").to_string();
        let choices = engines.and_then(|e| e.get("choices")).and_then(Value::as_array).map(|rows| rows.iter().map(|row| (text(row, "id"), text(row, "label"))).collect()).unwrap_or_default();
        Engines { choices, workspace: engines.and_then(|e| e.get("workspace")).and_then(Value::as_str).map(str::to_string) }
    }

    fn named(&self, id: &str) -> bool {
        self.choices.iter().any(|(choice, _)| choice == id)
    }

    /// The rows the card draws from: the choices, or Claude Code where there is no choice
    /// (`engineRows` in `ThreadComposer.tsx`).
    pub fn rows(&self) -> Vec<(String, String)> {
        if self.choices.is_empty() { vec![("claude".to_string(), "Claude Code".to_string())] } else { self.choices.clone() }
    }

    pub fn offers(&self, id: &str) -> bool {
        self.rows().iter().any(|(choice, _)| choice == id)
    }

    pub fn label(&self, id: &str) -> String {
        self.rows().into_iter().find(|(choice, _)| choice == id).map_or_else(|| crate::rules::engine::engine_label(id).to_string(), |(_, label)| label)
    }

    /// Whether each model used lately is tagged with its agent: only where there is more than
    /// one to tell apart (`agentChoices`).
    pub fn tagged(&self) -> bool {
        self.choices.len() >= 2 && self.choices.iter().any(|(id, _)| matches!(id.as_str(), "claude" | "codex" | "antigravity"))
    }
}

/// What a Claude Code run is on when nobody names a model (`DEFAULT_CLAUDE_MODEL`).
pub const DEFAULT_MODEL: &str = "opus";
/// The row that means "send no model and let the agent decide" (`CODEX_OWN`).
pub const AGENTS_OWN: &str = "";

re!(ANTIGRAVITY_MODEL, r"^(gemini|gpt-oss)-[a-z0-9._-]*$", "i");
re!(CODEX_MODEL, r"^(gpt|o\d|codex)[a-z0-9._-]*$", "i");
re!(CODEX_SHAPE, r"^(gpt|o\d|codex)", "i");
re!(EFFORT_WORD, r"^[a-z][a-z0-9]{0,23}$", "");

/// A model only Antigravity runs (`isAntigravityModel` in `shared/engines.mts`).
pub fn is_antigravity_model(model: &str) -> bool {
    ANTIGRAVITY_MODEL.test(model)
}

/// A Codex model's slug (`isCodexModel`).
pub fn is_codex_model(model: &str) -> bool {
    CODEX_MODEL.test(model) && !is_antigravity_model(model)
}

/// One model as the card holds it: where it runs, and the word that runs it (`ModelPick`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pick {
    pub engine: String,
    pub model: String,
}

impl Pick {
    pub fn new(engine: &str, model: &str) -> Pick {
        Pick { engine: engine.to_string(), model: model.to_string() }
    }
}

/// Where a model runs, told from the lists and then from its shape (`harnessOf`).
pub fn harness_of(model: &str, models: &Models) -> &'static str {
    if models.claude.iter().any(|m| m.id == model || m.model.as_deref() == Some(model)) {
        return "claude";
    }
    if is_antigravity_model(model) {
        return "antigravity";
    }
    if models.codex.iter().any(|m| m.id == model) || CODEX_SHAPE.test(model) { "codex" } else { "claude" }
}

/// The agent a new card opens on: the one Settings names, where this Mac offers it; Codex on a
/// Mac that has Codex and no Claude Code; else Claude Code (`startingEngine`).
pub fn starting_engine(engines: &Engines) -> &'static str {
    if engines.named("codex") && !engines.named("claude") {
        return "codex";
    }
    match engines.workspace.as_deref().filter(|id| engines.named(id)) {
        Some("codex") => "codex",
        Some("antigravity") => "antigravity",
        _ => "claude",
    }
}

/// What the word says when nothing was picked: Opus for Claude Code, the person's own default
/// for Codex when its settings name one, and the agent's own choice otherwise
/// (`defaultModelFor`).
pub fn default_model_for(engine: &str, codex_default: Option<&str>) -> String {
    match engine {
        "antigravity" => AGENTS_OWN.to_string(),
        "codex" => codex_default.unwrap_or(AGENTS_OWN).to_string(),
        _ => DEFAULT_MODEL.to_string(),
    }
}

/// The model remembered for an agent, when it is one worth remembering: not its default, and
/// under Claude Code not another agent's model left there by an older build (`readLastModel`).
pub fn remembered_model(kept: Option<&str>, engine: &str, codex_default: Option<&str>) -> Option<String> {
    let kept = kept.filter(|m| !m.is_empty() && *m != default_model_for(engine, codex_default))?;
    let other_agents = !matches!(engine, "codex" | "antigravity") && (is_antigravity_model(kept) || is_codex_model(kept));
    (!other_agents).then(|| kept.to_string())
}

/// The model a new card opens on, for the agent it opens on (`remembered` in
/// `ThreadComposer.tsx`). `kept` is the model last picked for an agent.
pub fn starting_pick(engines: &Engines, models: &Models, kept: impl Fn(&str) -> Option<String>) -> Pick {
    let engine = starting_engine(engines);
    let codex_default = models.codex_default.as_deref();
    let model = remembered_model(kept(engine).as_deref(), engine, codex_default).unwrap_or_else(|| default_model_for(engine, codex_default));
    Pick { engine: engine.to_string(), model }
}

/// A pick this Mac can still run. A card holding Codex on a Mac that stopped offering it goes
/// back to Claude Code, and the other way round (the effect under `pick` in `ThreadComposer.tsx`).
pub fn pick_this_mac_runs(pick: Pick, engines: &Engines, models: &Models, kept: impl Fn(&str) -> Option<String>) -> Pick {
    let codex_default = models.codex_default.as_deref();
    let on = |engine: &str| Pick { engine: engine.to_string(), model: remembered_model(kept(engine).as_deref(), engine, codex_default).unwrap_or_else(|| default_model_for(engine, codex_default)) };
    match pick.engine.as_str() {
        "codex" if !engines.offers("codex") => on("claude"),
        "claude" if !engines.offers("claude") && engines.offers("codex") => on("codex"),
        _ => pick,
    }
}

/// An agent's rows with the picked value among them even when its list does not carry it: a
/// control that draws a value it is not running has started lying (`engineModelChoices`).
fn model_rows(engine: &str, id: &str, models: &Models) -> Vec<Model> {
    let own = |label: &str| Model { id: AGENTS_OWN.to_string(), label: label.to_string(), ..Model::default() };
    let itself = || Model { id: id.to_string(), label: id.to_string(), ..Model::default() };
    let (mut rows, own_label) = match engine {
        "antigravity" => (models.antigravity.clone(), "Antigravity’s own"),
        "codex" => (models.codex.clone(), "Codex's own"),
        _ => {
            let mut rows = models.claude.clone();
            if !rows.iter().any(|m| m.id == id) {
                rows.push(itself());
            }
            return rows;
        }
    };
    if id == AGENTS_OWN {
        rows.push(own(own_label));
    } else if !rows.iter().any(|m| m.id == id) {
        rows.push(itself());
    }
    rows
}

/// The word on the chip and on a row of the menu (`engineModelLabel`).
pub fn model_label(pick: &Pick, models: &Models) -> String {
    model_rows(&pick.engine, &pick.model, models).into_iter().find(|m| m.id == pick.model).map_or_else(|| pick.model.clone(), |m| m.label)
}

/// What a send says about who runs it (`harnessFields`). The Claude model is always named, the
/// default included; another agent's default is not, so its own settings decide; the agent is
/// named only where this Mac offered a choice; the level only when one was picked.
pub fn harness_fields(pick: &Pick, effort: Option<&str>, engine_count: usize, codex_default: Option<&str>) -> Map<String, Value> {
    let mut fields = Map::new();
    if engine_count > 1 {
        fields.insert("engine".to_string(), json!(pick.engine));
    }
    if pick.engine == "claude" {
        fields.insert("model".to_string(), json!(if pick.model.is_empty() { DEFAULT_MODEL } else { pick.model.as_str() }));
    } else if !pick.model.is_empty() && pick.model != default_model_for(&pick.engine, codex_default) {
        fields.insert("model".to_string(), json!(pick.model));
    }
    if let Some(effort) = effort.filter(|e| !e.is_empty()) {
        fields.insert("effort".to_string(), json!(effort));
    }
    fields
}

/// How far back "the ones you use" looks: the last forty threads that named a model.
const RECENT_WINDOW: usize = 40;

/// The models under Recent (`recentModels`): the one picked last, then the ones reached for
/// most often lately, topped up to three with Opus, Sonnet and Codex's default.
pub fn recent_models(items: &[Item], models: &Models, engines: &Engines) -> Vec<Pick> {
    const LIMIT: usize = 3;
    let offered = |engine: &str| engines.offers(engine);
    let mut out: Vec<Pick> = Vec::new();
    let mut add = |pick: Pick| {
        if out.len() < LIMIT && !out.contains(&pick) {
            out.push(pick);
        }
    };
    // Dated by when the model was last set, not when the thread began: a model picked on a
    // reply to an older thread is the newest pick.
    let mut used: Vec<(&Item, f64)> = items.iter().filter(|i| i.text("model").is_some()).map(|i| (i, i.created_at.unwrap_or(0.0).max(i.wrote_at("model")))).collect();
    used.sort_by(|a, b| b.1.total_cmp(&a.1));
    used.truncate(RECENT_WINDOW);
    let alias = |model: &str| models.claude.iter().find(|m| m.model.as_deref() == Some(model)).map_or(model.to_string(), |m| m.id.clone());
    let used: Vec<(Pick, f64)> = used
        .into_iter()
        .map(|(i, at)| {
            let model = i.text("model").unwrap_or("");
            // Antigravity's models include Claude's full ids, so only the row's own engine says it.
            let engine = match i.engine.as_deref() {
                Some(engine @ ("codex" | "claude" | "antigravity")) => engine,
                _ => harness_of(model, models),
            };
            (Pick { engine: engine.to_string(), model: if engine == "claude" { alias(model) } else { model.to_string() } }, at)
        })
        .filter(|(pick, _)| offered(&pick.engine))
        .collect();
    let mut tally: Vec<(Pick, usize, f64)> = Vec::new();
    for (pick, at) in &used {
        match tally.iter_mut().find(|row| &row.0 == pick) {
            Some(row) => (row.1, row.2) = (row.1 + 1, row.2.max(*at)),
            None => tally.push((pick.clone(), 1, *at)),
        }
    }
    if let Some((first, _)) = used.first() {
        add(first.clone());
    }
    tally.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.total_cmp(&a.2)));
    for (pick, _, _) in tally {
        add(pick);
    }
    let codex_first = || Pick::new("codex", models.codex_default.as_deref().or(models.codex.first().map(|m| m.id.as_str())).unwrap_or(AGENTS_OWN));
    // A Mac without Claude Code is offered no Claude model: it has nothing to run one in.
    if !offered("claude") {
        if offered("codex") {
            add(codex_first());
            for m in &models.codex {
                add(Pick::new("codex", &m.id));
            }
        }
        return out;
    }
    add(Pick::new("claude", DEFAULT_MODEL));
    add(Pick::new("claude", "sonnet"));
    if offered("codex") {
        add(codex_first());
    }
    for m in &models.claude {
        add(Pick::new("claude", &m.id));
    }
    out
}

/// Everything behind "All models": a column for each agent this Mac offers (`allModels`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AllModels {
    pub claude: Vec<Pick>,
    pub codex: Vec<Pick>,
    pub antigravity: Vec<Pick>,
}

impl AllModels {
    /// The columns that have rows, each with its agent, in the order the menu draws them.
    pub fn columns(&self) -> Vec<(&'static str, &[Pick])> {
        [("claude", self.claude.as_slice()), ("codex", self.codex.as_slice()), ("antigravity", self.antigravity.as_slice())].into_iter().filter(|(_, rows)| !rows.is_empty()).collect()
    }
}

pub fn all_models(models: &Models, engines: &Engines) -> AllModels {
    let column = |engine: &str, rows: &[Model]| if engines.offers(engine) { rows.iter().map(|m| Pick::new(engine, &m.id)).collect() } else { Vec::new() };
    AllModels { claude: column("claude", &models.claude), codex: column("codex", &models.codex), antigravity: column("antigravity", &models.antigravity) }
}

/// The "N more" on the All models row: what is in the columns and not under Recent (`moreCount`).
pub fn more_count(recent: &[Pick], all: &AllModels) -> usize {
    let listed: HashSet<&Pick> = all.claude.iter().chain(&all.codex).chain(&all.antigravity).collect();
    listed.len() - recent.iter().filter(|pick| listed.contains(pick)).count()
}

/* --------------------------------- effort --------------------------------- */

/// Claude Code's five levels, in its own order, as its id and the word on the strip
/// (`EFFORT_LEVELS` in `shared/effort-levels.mts`).
pub const EFFORTS: [(&str, &str); 5] = [("low", "Low"), ("medium", "Medium"), ("high", "High"), ("xhigh", "Extra"), ("max", "Max")];

/// The word on the strip for a level: Claude Code's own, else the level with its first letter
/// up, so no name is invented for one only a model's list knows (`effortLabel`).
pub fn effort_label(id: &str) -> Option<String> {
    if let Some((_, label)) = EFFORTS.iter().find(|(known, _)| *known == id) {
        return Some(label.to_string());
    }
    EFFORT_WORD.test(id).then(|| js::upper_first(id))
}

/// The levels the strip draws for one agent and one model (`effortChoicesFor`): Claude Code's
/// five whatever the model, and for the others the levels that model says it takes, which may
/// be none.
pub fn effort_choices(pick: &Pick, models: &Models) -> Vec<(String, String)> {
    let of = |rows: &[Model], id: &str| -> Vec<(String, String)> {
        let levels = rows.iter().find(|m| m.id == id).map(|m| m.levels.clone()).unwrap_or_default();
        levels.into_iter().filter_map(|level| effort_label(&level).filter(|_| EFFORT_WORD.test(&level)).map(|label| (level, label))).collect()
    };
    match pick.engine.as_str() {
        "antigravity" => of(&models.antigravity, &pick.model),
        "codex" if pick.model == AGENTS_OWN => Vec::new(),
        "codex" => of(&models.codex, &pick.model),
        _ => EFFORTS.iter().map(|(id, label)| (id.to_string(), label.to_string())).collect(),
    }
}

/// Whether a remembered level is one the strip's rows carry: only then is it lit as a pick,
/// and sent (`effortPicked`).
pub fn effort_picked(effort: Option<&str>, rows: &[(String, String)]) -> bool {
    effort.is_some_and(|effort| !effort.is_empty() && rows.iter().any(|(id, _)| id == effort))
}

/// The level the picked model thinks at when no level was picked, where its agent says
/// (`defaultEffortFor`).
pub fn default_effort(pick: &Pick, models: &Models) -> Option<String> {
    let of = |rows: &[Model]| rows.iter().find(|m| m.id == pick.model).and_then(|m| m.default_level.clone());
    match pick.engine.as_str() {
        "antigravity" => None,
        "codex" if pick.model == AGENTS_OWN => None,
        "codex" => of(&models.codex),
        _ => of(&models.claude),
    }
}

/// The level the strip lights: the person's pick, else the model's own. It is shown, not sent
/// (`effortShown`).
pub fn effort_shown(effort: Option<&str>, rows: &[(String, String)], fallback: Option<&str>) -> Option<String> {
    if effort_picked(effort, rows) {
        return effort.map(str::to_string);
    }
    fallback.filter(|level| rows.iter().any(|(id, _)| id == level)).map(str::to_string)
}

/// A level worth remembering for an agent: one of Claude Code's five, or for the others
/// anything shaped like a level (`readLastEffort`).
pub fn remembered_effort(kept: Option<&str>, engine: &str) -> Option<String> {
    let ok = |level: &&str| if matches!(engine, "codex" | "antigravity") { EFFORT_WORD.test(level) } else { EFFORTS.iter().any(|(id, _)| id == level) };
    kept.filter(ok).map(str::to_string)
}

/* ------------------------------ what is sent ------------------------------ */

/// The channel a new thread goes to (`api.compose` in `renderer/src/api.ts`, answered in
/// `main/ipc.mts`).
pub const COMPOSE: &str = "zero:compose";

/// A new thread, as `sendTask` in `ThreadComposer.tsx` hands it over. `later` writes it down
/// under Later, where nothing starts it until the person does.
pub fn compose(product: &str, message: &Message, priority: f64, harness: Map<String, Value>, later: bool) -> Value {
    let mut payload = json!({ "product": product, "title": message.title, "kind": "directive", "priority": priority });
    if !message.body.is_empty() {
        payload["body"] = json!(message.body);
    }
    if later {
        payload["start"] = json!("later");
    }
    for (field, value) in harness {
        payload[field] = value;
    }
    payload
}

/// What the line says between Send and the engine's answer (`sendingLine`).
pub fn sending_line(to: &str, held: bool) -> String {
    if held { format!("Adding to Later in {to}…") } else { format!("Starting in {to}…") }
}

/// A coding agent every sign-in of which is out of usage, and when it comes back, in
/// milliseconds; nought when nobody said (`supervisor.atLimit`).
#[derive(Clone, Debug, PartialEq)]
pub struct Limit {
    pub tool: String,
    pub until: f64,
}

/// The new thread's entry in `supervisor.atLimit`, when nothing started because of one.
pub fn limit_of(snapshot: &Value, id: &str) -> Option<Limit> {
    let at = snapshot.get("supervisor")?.get("atLimit")?.get(id)?;
    Some(Limit { tool: at.get("tool")?.as_str()?.to_string(), until: at.get("until").and_then(Value::as_f64).unwrap_or(0.0) })
}

/// The line once the engine has it (`sentLine`): where it went, in the project's own name. The
/// promise of Z rides on its end, and `toast_parts` takes it off for the key drawn instead.
/// `back` is when an agent out of usage has room again, in words, when that is still ahead.
pub fn sent_line(to: &str, held: bool, limit: Option<(&str, Option<&str>)>) -> String {
    if held {
        return format!("Added to Later in {to} · Z to undo");
    }
    match limit {
        Some((tool, Some(back))) => format!("{tool} is out until {back} · Z to undo"),
        Some((tool, None)) => format!("{tool} is out of usage · Z to undo"),
        None => format!("Started in {to} · Z to undo"),
    }
}

re!(UNDO_TAIL, r" · (?:press )?Z to undo$", "");

/// What the line draws, and whether Z is drawn beside it (`toastParts` in
/// `renderer/src/toast-parts.ts`).
pub fn toast_parts(text: &str) -> (String, bool) {
    if UNDO_TAIL.test(text) { (UNDO_TAIL.strip(text), true) } else { (text.to_string(), false) }
}

/// How long a line that names a thread stays up, which is how long O opens that thread
/// (`TOAST_GOES_MS` in `App.tsx`).
pub const TOAST_GOES_MS: f64 = 6000.0;

/// A send the app has not taken yet, and how long the wait runs before the window says so
/// (`renderer/src/not-answering.ts`).
pub const SEND_NOT_TAKEN: &str = "Mercury has not taken this yet. It is not lost.";
pub const NOT_ANSWERING_MS: f64 = 6000.0;

/// A send that did not stand (`sendTask`'s catch), and where the words are when something
/// else has been opened since (`onFailed` in `App.tsx`).
pub fn not_sent(why: &str) -> String {
    format!("Not sent: {why}")
}

pub fn not_sent_words_kept(said: &str) -> String {
    format!("{said} Your words are in the new thread card.")
}

/// Where a send leaves the person (`tabAfterSending`). Alone, on the Inbox page, the tab moves
/// to where the thread landed; on a team, or with something else over the list, they stay.
pub fn tab_after_sending(team: bool, later: bool, tab: Option<Tab>, covered: bool) -> Option<Tab> {
    if team || covered || !matches!(tab, Some(Tab::Needs | Tab::Progress | Tab::Later)) {
        return None;
    }
    Some(if later { Tab::Later } else { Tab::Progress })
}

/* ------------------------------- taking back ------------------------------ */

/// Whether an agent has already written its answer onto the row: then closing it would only
/// hide the answer, and Z is refused (`answeredAlready` in `renderer/src/withdraw-send.ts`).
pub fn answered_already(row: Option<&Item>) -> bool {
    row.is_some_and(|row| row.result.as_deref().is_some_and(|result| !js::trim(result).is_empty()))
}

pub const TOO_LATE_TO_WITHDRAW: &str = "Too late to take it back: the agent already answered. Here it is.";

/// Taking a thread back is closing it, which stops its worker (`noteNewTask` in `App.tsx`).
pub fn withdraw(product: &str, id: &str) -> Value {
    json!({ "product": product, "id": id, "status": "done" })
}

/// What Z says once the thread is withdrawn, and where the words went (`undo` in `App.tsx`).
pub fn said_withdrawn(title: &str, words_are_back: bool) -> String {
    let label = format!("Withdrawn: {}", writes::clip_to_sentence(title, 48));
    if words_are_back { format!("{label} · your thread is back in the new thread card") } else { label }
}

pub fn could_not_undo(why: &str) -> String {
    format!("Could not undo that. {why}")
}

/* -------------------------------- the menus ------------------------------- */

/// One thing in an open menu the keys can land on and Enter presses (`[data-item]`).
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    Project(String),
    NewProject,
    Level(Option<Level>),
    Model(Pick),
    /// The row that turns the model menu to every model.
    AllModels,
    /// The row that turns it back.
    Recent,
    Effort(String),
    /// Under the caret beside Send: write the thread down under Later, where nothing starts it
    /// until the person does, and take that back.
    AddToLater,
    RemoveTheTime,
}

/// The caret's menu, as far as this window takes it: its heading, the way to take a pick back,
/// and the one pick with no clock in it (`laterMenu` in `ThreadComposer.tsx`).
pub const SEND_LATER: &str = "Send later";
pub const ADD_TO_LATER: &str = "Add it to Later";
pub const REMOVE_THE_TIME: &str = "Remove the time";
/// What the caret says once Later is picked (`whenWords`).
pub const LATER_WORD: &str = "Later";

/// That menu's entries: "Remove the time" only while something is picked.
pub fn later_entries(later: bool) -> Vec<Entry> {
    if later { vec![Entry::RemoveTheTime, Entry::AddToLater] } else { vec![Entry::AddToLater] }
}

/// The project menu's entries: the projects, then the way to make one.
pub fn project_entries(offered: &[&Product]) -> Vec<Entry> {
    offered.iter().map(|p| Entry::Project(p.slug.clone())).chain([Entry::NewProject]).collect()
}

/// The priority menu's: Agent Decides, then the four levels.
pub fn level_entries() -> Vec<Entry> {
    [None].into_iter().chain(LEVELS.into_iter().map(Some)).map(Entry::Level).collect()
}

/// The model menu's, on either of its pages, with the strip of levels last.
pub fn model_entries(all_page: bool, recent: &[Pick], all: &AllModels, efforts: &[(String, String)]) -> Vec<Entry> {
    let mut entries = Vec::new();
    if all_page {
        entries.push(Entry::Recent);
        entries.extend(all.columns().into_iter().flat_map(|(_, rows)| rows.iter().cloned().map(Entry::Model)));
    } else {
        entries.extend(recent.iter().cloned().map(Entry::Model));
        entries.push(Entry::AllModels);
    }
    entries.extend(efforts.iter().map(|(id, _)| Entry::Effort(id.clone())));
    entries
}

/// The entry a menu opens with the keys on: the one already picked, else its first
/// (`[data-item].on` in `ThreadComposer.tsx`).
pub fn entry_lit_first(entries: &[Entry], picked: impl Fn(&Entry) -> bool) -> usize {
    entries.iter().position(picked).unwrap_or(0)
}

/// One step through a menu's entries, around either end (`menuKeys`).
pub fn entry_step(at: usize, len: usize, down: bool) -> usize {
    if len == 0 {
        return 0;
    }
    if down { (at + 1) % len } else if at == 0 { len - 1 } else { at - 1 }
}

/// Where the stylesheet hangs a menu from the control that opens it: below it or above it, so
/// far from it, and so far along it (`.tc-drop` and `.tc-rise` in `thread-composer.css`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hang {
    pub below: bool,
    pub gap: f32,
    pub dx: f32,
}

/// Where a menu ends up: its left and top in the window, and how tall it may be before it
/// scrolls inside itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub left: f32,
    pub top: f32,
    pub max_height: Option<f32>,
}

/// Keeps a menu inside the window (`fitMenu` in `renderer/src/keep-in-window.ts`). Where the
/// stylesheet put it stands if it fits there. Otherwise it opens on whichever side of its
/// control has more room, and scrolls inside itself if even that side is too short; and one
/// that leaves the strip it can be seen in, side to side, slides back inside. `anchor` is the
/// control as `[x, y, w, h]`, `size` the menu's own width and height, `room` the window's
/// height and `sides` the strip.
pub fn fit_menu(anchor: [f32; 4], hang: Hang, size: (f32, f32), room: f32, sides: (f32, f32)) -> Fit {
    const EDGE: f32 = 8.0;
    const GAP: f32 = 8.0;
    // The window's top 44 points are its title bar's, where no press reaches a page.
    const TOP: f32 = 44.0 + EDGE;
    let [x, y, _, h] = anchor;
    let (wide, tall) = size;
    let (anchor_top, anchor_bottom) = (y, y + h);
    let mut left = x + hang.dx;
    if left < sides.0 + EDGE || left + wide > sides.1 - EDGE {
        let slid = (sides.0 + EDGE).max(left.min(sides.1 - EDGE - wide));
        left = x + (slid - x).round();
    }
    let top = if hang.below { anchor_bottom + hang.gap } else { anchor_top - hang.gap - tall };
    if top >= TOP && top + tall <= room - EDGE {
        return Fit { left, top, max_height: None };
    }
    let below = room - anchor_bottom - GAP - EDGE;
    let above = anchor_top - GAP - TOP;
    let down = below >= tall || (above < tall && below >= above);
    let space = (if down { below } else { above }).floor();
    let max_height = (tall > space).then(|| space.max(96.0));
    let shown = max_height.map_or(tall, |most| tall.min(most));
    Fit { left, top: if down { anchor_bottom + GAP } else { anchor_top - GAP - shown }, max_height }
}

/* ------------------------------- the keyboard ----------------------------- */

/// What has the keyboard while the card is drawn: its box, one of its own controls, or nothing
/// of its own (`Keyboard` in `composer-rules.ts`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Keyboard {
    #[default]
    Nothing,
    Field,
    Card,
}

/// Whether a key pressed while the card is drawn is typed into its message
/// (`keyOnTheNewThreadCard`). A character with no chord on it is, wherever the keyboard is
/// sitting on the card, so no control under it is ever pressed by a letter. Not with a menu
/// open, whose rows take the keys; not in the box, which types by itself; and the box on the
/// page does not own the screen, so a key with nothing focused is the page's.
pub fn key_is_typed(character: bool, chord: bool, menu_open: bool, on: Keyboard, inline: bool) -> bool {
    if chord || !character || menu_open || on == Keyboard::Field {
        return false;
    }
    !(inline && on == Keyboard::Nothing)
}

/// Whether ⌘Return heard outside the box sends the card (`modEnterSendsTheCard`). The dialog
/// owns the keyboard, so it always does; the box on the page only when nothing else has the
/// keyboard and it holds words.
pub fn mod_enter_sends(inline: bool, on: Keyboard, has_words: bool) -> bool {
    !inline || (on == Keyboard::Nothing && has_words)
}

/// Whether a letter pressed on a list, with no box holding the keyboard, is one of the list's
/// own keys (`keyOnTheList` in `renderer/src/reply-keys.ts`). Any other letter is somebody
/// typing, and where the page has a box to start a thread in, it goes there.
pub fn key_is_the_lists(letter: &str) -> bool {
    const LIST_KEYS: [&str; 13] = ["j", "k", "r", "n", "z", "o", "g", "b", "i", "1", "2", "e", "l"];
    let mut letters = letter.chars();
    let one = matches!((letters.next(), letters.next()), (Some(c), None) if c != ' ');
    !one || LIST_KEYS.contains(&letter.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(title: &str, body: &str) -> Message {
        Message { title: title.to_string(), body: body.to_string() }
    }

    fn thread(rest: Value) -> Item {
        serde_json::from_value(rest).unwrap()
    }

    fn project(rest: Value) -> Product {
        serde_json::from_value(rest).unwrap()
    }

    fn two_agents() -> (Engines, Models) {
        let engines = Engines { choices: vec![("claude".to_string(), "Claude Code".to_string()), ("codex".to_string(), "Codex".to_string())], workspace: Some("claude".to_string()) };
        let codex = |id: &str, label: &str, levels: &[&str], level: &str| Model { id: id.to_string(), label: label.to_string(), model: None, levels: levels.iter().map(|l| l.to_string()).collect(), default_level: Some(level.to_string()) };
        let models = Models { codex: vec![codex("gpt-6-astra", "GPT-6 Astra", &["low", "medium", "high", "ultra"], "medium"), codex("gpt-6-astra-mini", "GPT-6 Astra Mini", &["low", "medium"], "low")], codex_default: Some("gpt-6-astra".to_string()), ..Models::built() };
        (engines, models)
    }

    fn used(at: f64, model: &str, engine: Option<&str>) -> Item {
        let mut row = json!({ "id": format!("w-{at}"), "product": "shop", "createdAt": at, "model": model });
        if let Some(engine) = engine {
            row["engine"] = json!(engine);
        }
        thread(row)
    }

    #[test]
    fn a_short_message_is_its_own_title_and_a_subject_is_set_apart_by_a_blank_line() {
        assert_eq!(split_message("Fix the sign in loop"), message("Fix the sign in loop", ""));
        assert_eq!(split_message("Fix the sign in loop\n\nIt goes back to the start on a wrong password."), message("Fix the sign in loop", "It goes back to the start on a wrong password."));
        // With no blank line the first line is not lifted out: the body keeps every line.
        assert_eq!(split_message("Fix the sign in loop\nIt goes back to the start."), message("Fix the sign in loop", "Fix the sign in loop\nIt goes back to the start."));
        assert_eq!(split_message("  \n\nWater the plants.\n\n   One fern, one cactus.  \n\n"), message("Water the plants.", "One fern, one cactus."));
    }

    #[test]
    fn a_long_paragraph_keeps_every_word_and_is_titled_by_its_first_sentence() {
        let long = "Where did the export button go? It used to sit beside the filter and now the whole toolbar is one row shorter than the page it belongs to, so give it back.";
        let split = split_message(long);
        assert_eq!(split.title, "Where did the export button go?");
        assert_eq!(split.body, long);
        // No sentence to stop at: as much as fits, cut at a word, and never past the budget.
        let wall = format!("{}end", "word ".repeat(200));
        let split = split_message(&wall);
        assert!(split.title.ends_with('…') && split.title.chars().count() <= TITLE_BUDGET + 1);
        assert_eq!(split.body, wall);
    }

    #[test]
    fn a_pasted_lists_title_loses_its_mark_and_nothing_else_does() {
        let pasted = "- Fix the failing test in the converter\n- Add a --version flag";
        assert_eq!(split_message(pasted), message("Fix the failing test in the converter", pasted));
        for marked in ["* Ship the pricing page", "• Ship the pricing page", "1. Ship the pricing page", "2) Ship the pricing page"] {
            assert_eq!(split_message(marked).title, "Ship the pricing page");
        }
        for kept in ["-5 degrees is what the test expects", "--version is missing", "Fix the converter - the order is wrong"] {
            assert_eq!(split_message(kept).title, kept);
        }
    }

    // tests/threads-the-composer-addresses-a-thread-like-an-email.test.mts
    #[test]
    fn nothing_typed_is_nothing_to_send_and_files_alone_are_named_after_the_first() {
        assert_eq!(thread_message("  \n ", None), None);
        assert_eq!(thread_message(" Ship it ", None), Some(message("Ship it", "")));
        assert_eq!(thread_message("   ", Some("shot.png")), Some(message("Attached: shot.png", "")));
        // Words name the thread themselves, whatever is attached.
        assert_eq!(thread_message("Draft the renewal\n\nAt 8% over last year.", Some("shot.png")), Some(message("Draft the renewal", "At 8% over last year.")));
    }

    #[test]
    fn a_card_sends_its_level_as_a_number_and_agent_decides_as_none() {
        assert_eq!(sent_priority(None), 0.0);
        assert_eq!(LEVELS.map(|level| sent_priority(Some(level))), [9.0, 7.0, 5.0, 2.0]);
        assert_eq!(choice_label(None), "Agent decides");
        assert_eq!(choice_label(Some(Level::Medium)), "Medium");
    }

    #[test]
    fn the_card_opens_on_the_project_on_screen_then_the_last_one_then_the_first_real_one() {
        let products = [project(json!({ "slug": "practice", "name": "Practice", "practice": true })), project(json!({ "slug": "shop", "name": "Storefront" })), project(json!({ "slug": "app", "name": "Mobile" })), project(json!({ "slug": "dm", "name": "Ada and you", "team": { "direct": true } }))];
        let ranked = rank_products(&products, &["app".to_string(), "shop".to_string()]);
        // Placed ones in the person's order, then the rest, oldest first and by name.
        assert_eq!(ranked.iter().map(|p| p.slug.as_str()).collect::<Vec<_>>(), ["app", "shop", "dm", "practice"]);
        let offered = projects_offered(&ranked);
        // A project that only carries two people's messages is never filed into.
        assert_eq!(offered.len(), 3);
        let on = |picked, on_screen, remembered| card_project(&offered, picked, on_screen, remembered).map(|p| p.slug.as_str());
        assert_eq!(on(Some("shop"), Some("app"), Some("app")), Some("shop"));
        assert_eq!(on(None, Some("shop"), Some("app")), Some("shop"));
        assert_eq!(on(None, None, Some("shop")), Some("shop"));
        // A remembered project that is gone falls back; the practice one is only chosen on purpose.
        assert_eq!(on(None, None, Some("gone")), Some("app"));
        assert_eq!(card_project(&projects_offered(&rank_products(&products[..1], &[])), None, None, None).map(|p| p.slug.as_str()), Some("practice"));
        assert_eq!(card_project(&[], None, None, None).map(|p| p.slug.as_str()), None);
        assert_eq!(practice_refusal(Some(&products[0])), Some(PRACTICE_REFUSAL));
        assert_eq!(practice_refusal(Some(&products[1])), None);
    }

    #[test]
    fn words_about_code_on_a_project_with_none_go_to_the_one_project_that_has_code() {
        let products = [
            project(json!({ "slug": "mine", "name": "My Workspace", "code": false })),
            project(json!({ "slug": "shop", "name": "Storefront", "code": true })),
            project(json!({ "slug": "notes", "name": "Notes", "code": false })),
        ];
        let offered: Vec<&Product> = products.iter().collect();
        let to = |text: &str, current: &str| code_project_for(text, &offered, current).map(|p| p.slug.as_str());
        assert_eq!(to("Fix the checkout button", "mine"), Some("shop"));
        // Not about code, already on code, or naming the project it is on: it stays.
        assert_eq!(to("Book the flights for March", "mine"), None);
        assert_eq!(to("Fix the checkout button", "shop"), None);
        assert_eq!(to("Fix the bug in my workspace list", "mine"), None);
        // A word that only starts like one of them is not one ("debugging", "pages").
        assert_eq!(to("Write about debugging habits", "mine"), None);
        // Two projects with code is a guess, so nothing moves.
        let two = [project(json!({ "slug": "mine", "name": "Mine", "code": false })), project(json!({ "slug": "a", "name": "A", "code": true })), project(json!({ "slug": "b", "name": "B", "code": true }))];
        assert!(code_project_for("Fix the build", &two.iter().collect::<Vec<_>>(), "mine").is_none());
        assert_eq!(going_to_code("Storefront"), "Going to Storefront, where the code is.");
        assert_eq!(keep("My Workspace"), "Keep My Workspace");
    }

    #[test]
    fn the_caret_offers_later_and_takes_it_back_once_it_is_picked() {
        assert_eq!(later_entries(false), [Entry::AddToLater]);
        assert_eq!(later_entries(true), [Entry::RemoveTheTime, Entry::AddToLater]);
    }

    #[test]
    fn a_project_with_no_folder_here_says_whose_it_is() {
        assert_eq!(project_note(&project(json!({ "slug": "shop", "name": "Storefront", "repoPath": "/code/shop" }))), None);
        assert_eq!(project_note(&project(json!({ "slug": "mine", "name": "My Workspace" }))), Some("For personal tasks"));
        assert_eq!(project_note(&project(json!({ "slug": "ours", "name": "Launch", "team": { "projectId": "p-1" } }))), Some("Shared with your team"));
    }

    #[test]
    fn a_model_runs_where_the_lists_and_its_shape_say() {
        let (_, models) = two_agents();
        assert_eq!(harness_of("opus", &models), "claude");
        assert_eq!(harness_of("claude-opus-5-5", &models), "claude");
        assert_eq!(harness_of("gpt-6-astra", &models), "codex");
        // A GPT slug Codex has not listed is still Codex's, and Antigravity's own come first.
        assert_eq!(harness_of("gpt-7-new", &Models::built()), "codex");
        assert_eq!(harness_of("gpt-oss-120b", &models), "antigravity");
        assert_eq!(harness_of("sonnet", &Models::built()), "claude");
    }

    #[test]
    fn a_new_card_opens_on_the_agent_from_settings_and_that_agents_last_model() {
        let (mut engines, models) = two_agents();
        let nothing = |_: &str| None;
        assert_eq!(starting_pick(&engines, &models, nothing), Pick::new("claude", "opus"));
        assert_eq!(starting_pick(&engines, &models, |engine| (engine == "claude").then(|| "sonnet".to_string())), Pick::new("claude", "sonnet"));
        // Another agent's model left under Claude Code's name is no memory of Claude Code's.
        assert_eq!(starting_pick(&engines, &models, |_| Some("gpt-6-astra".to_string())), Pick::new("claude", "opus"));
        engines.workspace = Some("codex".to_string());
        assert_eq!(starting_pick(&engines, &models, nothing), Pick::new("codex", "gpt-6-astra"));
        // An agent Settings names and this Mac does not offer is not opened on.
        engines.workspace = Some("antigravity".to_string());
        assert_eq!(starting_engine(&engines), "claude");
        // A Mac with Codex and no Claude Code opens on Codex whatever was set.
        let codex_only = Engines { choices: vec![("codex".to_string(), "Codex".to_string())], workspace: None };
        assert_eq!(starting_engine(&codex_only), "codex");
        assert_eq!(starting_engine(&Engines::default()), "claude");
        assert_eq!(pick_this_mac_runs(Pick::new("codex", "gpt-6-astra"), &Engines::default(), &Models::built(), nothing), Pick::new("claude", "opus"));
        assert_eq!(pick_this_mac_runs(Pick::new("claude", "opus"), &codex_only, &models, nothing), Pick::new("codex", "gpt-6-astra"));
    }

    #[test]
    fn the_chip_names_the_model_as_its_agent_does() {
        let (_, models) = two_agents();
        assert_eq!(model_label(&Pick::new("claude", "opus"), &models), "Opus 5.5");
        assert_eq!(model_label(&Pick::new("codex", "gpt-6-astra-mini"), &models), "GPT-6 Astra Mini");
        // A model set by hand that no list carries is shown as itself.
        assert_eq!(model_label(&Pick::new("claude", "claude-opus-4"), &models), "claude-opus-4");
        assert_eq!(model_label(&Pick::new("codex", ""), &Models::built()), "Codex's own");
        assert_eq!(model_label(&Pick::new("antigravity", ""), &Models::built()), "Antigravity’s own");
    }

    #[test]
    fn a_send_names_the_claude_model_always_and_another_agents_only_when_moved_off_its_default() {
        let fields = |pick: Pick, effort, count, default| Value::Object(harness_fields(&pick, effort, count, default));
        assert_eq!(fields(Pick::new("claude", "opus"), None, 1, None), json!({ "model": "opus" }));
        assert_eq!(fields(Pick::new("claude", "sonnet"), None, 2, None), json!({ "engine": "claude", "model": "sonnet" }));
        assert_eq!(fields(Pick::new("codex", "gpt-6-astra"), None, 2, Some("gpt-6-astra")), json!({ "engine": "codex" }));
        assert_eq!(fields(Pick::new("codex", "gpt-6-astra-mini"), Some("low"), 2, Some("gpt-6-astra")), json!({ "engine": "codex", "model": "gpt-6-astra-mini", "effort": "low" }));
        assert_eq!(fields(Pick::new("antigravity", ""), None, 3, None), json!({ "engine": "antigravity" }));
    }

    #[test]
    fn recent_is_the_model_picked_last_then_the_ones_used_most() {
        let (engines, models) = two_agents();
        let items = [used(1.0, "opus", Some("claude")), used(2.0, "opus", Some("claude")), used(3.0, "gpt-6-astra", Some("codex")), used(4.0, "gpt-6-astra", Some("codex")), used(5.0, "opus", Some("claude")), used(6.0, "fable", Some("claude"))];
        assert_eq!(recent_models(&items, &models, &engines), [Pick::new("claude", "fable"), Pick::new("claude", "opus"), Pick::new("codex", "gpt-6-astra")]);
        // Fewer than three are topped up with Opus, Sonnet and Codex's default.
        assert_eq!(recent_models(&[], &models, &engines), [Pick::new("claude", "opus"), Pick::new("claude", "sonnet"), Pick::new("codex", "gpt-6-astra")]);
        assert_eq!(recent_models(&[], &Models::built(), &Engines::default()), [Pick::new("claude", "opus"), Pick::new("claude", "sonnet"), Pick::new("claude", "claude-opus-5")]);
    }

    #[test]
    fn recent_reads_an_agent_off_the_model_names_a_full_id_by_its_alias_and_offers_only_what_runs_here() {
        let (engines, models) = two_agents();
        // A row that never wrote an agent is placed by its model.
        assert_eq!(recent_models(&[used(2.0, "gpt-6-astra-mini", None), used(1.0, "claude-sonnet-5", None)], &models, &engines)[..2], [Pick::new("codex", "gpt-6-astra-mini"), Pick::new("claude", "claude-sonnet-5")]);
        let rows = recent_models(&[used(2.0, "claude-opus-5-5", None), used(1.0, "opus", None)], &Models::built(), &Engines::default());
        assert_eq!(rows.iter().filter(|pick| pick.model == "opus").count(), 1);
        // A row that named no model ran on a default nobody chose, and says nothing.
        let rows = recent_models(&[thread(json!({ "id": "w-3", "product": "shop", "createdAt": 3 })), used(2.0, "haiku", None)], &Models::built(), &Engines::default());
        assert_eq!(rows[0], Pick::new("claude", "haiku"));
        let rows = recent_models(&[used(9.0, "gpt-6-astra", Some("codex"))], &models, &Engines::default());
        assert!(rows.len() == 3 && rows.iter().all(|pick| pick.engine == "claude"));
        // A model picked on a reply to an older thread is the newest pick.
        let replied = thread(json!({ "id": "w-old", "product": "shop", "createdAt": 1, "model": "haiku", "wrote": { "model": { "ts": 50, "source": "founder" } } }));
        assert_eq!(recent_models(&[used(9.0, "sonnet", None), replied], &Models::built(), &Engines::default())[0], Pick::new("claude", "haiku"));
    }

    #[test]
    fn all_models_is_a_column_for_each_agent_offered_and_counts_what_recent_leaves_out() {
        let (engines, models) = two_agents();
        let all = all_models(&models, &engines);
        assert_eq!(all.codex, [Pick::new("codex", "gpt-6-astra"), Pick::new("codex", "gpt-6-astra-mini")]);
        assert_eq!(all.columns().iter().map(|(engine, _)| *engine).collect::<Vec<_>>(), ["claude", "codex"]);
        assert_eq!(all_models(&models, &Engines::default()).codex, []);
        let recent = [Pick::new("claude", "opus"), Pick::new("codex", "gpt-6-astra"), Pick::new("claude", "sonnet")];
        assert_eq!(more_count(&recent, &all), all.claude.len() + all.codex.len() - 3);
        // A model under Recent that is in neither column is not taken off the count.
        assert_eq!(more_count(&[Pick::new("claude", "pinned-by-hand")], &all_models(&Models::built(), &Engines::default())), 8);
    }

    #[test]
    fn the_strip_draws_the_levels_the_model_takes_and_lights_the_pick_or_the_models_own() {
        let (_, models) = two_agents();
        let claude = effort_choices(&Pick::new("claude", "haiku"), &models);
        assert_eq!(claude.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>(), ["Low", "Medium", "High", "Extra", "Max"]);
        let codex = effort_choices(&Pick::new("codex", "gpt-6-astra"), &models);
        assert_eq!(codex.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>(), ["Low", "Medium", "High", "Ultra"]);
        // No list for the model: no strip at all, rather than a guess the run would refuse.
        assert_eq!(effort_choices(&Pick::new("codex", ""), &models), []);
        assert_eq!(effort_choices(&Pick::new("antigravity", "gemini-3"), &models), []);
        // Nothing picked lights the model's own level, which is shown and not sent.
        assert_eq!(default_effort(&Pick::new("claude", "opus"), &models).as_deref(), Some("medium"));
        assert_eq!(default_effort(&Pick::new("claude", "claude-haiku-4-5"), &models), None);
        assert_eq!(effort_shown(None, &claude, Some("medium")).as_deref(), Some("medium"));
        assert_eq!(effort_shown(Some("max"), &claude, Some("medium")).as_deref(), Some("max"));
        // A level remembered for another model's list is neither lit nor sent here.
        assert!(!effort_picked(Some("max"), &codex));
        assert_eq!(effort_shown(Some("max"), &codex, Some("medium")).as_deref(), Some("medium"));
        assert_eq!(effort_shown(None, &claude, None), None);
        assert_eq!(remembered_effort(Some("ultra"), "claude"), None);
        assert_eq!(remembered_effort(Some("ultra"), "codex").as_deref(), Some("ultra"));
        assert_eq!(remembered_effort(Some("Not A Level"), "codex"), None);
    }

    #[test]
    fn the_engine_is_handed_a_thread_with_its_words_its_level_and_who_runs_it() {
        let harness = harness_fields(&Pick::new("claude", "opus"), None, 1, None);
        assert_eq!(compose("shop", &message("Fix the sign in loop", ""), 0.0, harness.clone(), false), json!({ "product": "shop", "title": "Fix the sign in loop", "kind": "directive", "priority": 0.0, "model": "opus" }));
        assert_eq!(compose("shop", &message("Fix the sign in loop", "It loops."), 7.0, harness, true), json!({ "product": "shop", "title": "Fix the sign in loop", "body": "It loops.", "kind": "directive", "priority": 7.0, "start": "later", "model": "opus" }));
    }

    #[test]
    fn the_line_says_where_it_is_going_and_then_where_it_went() {
        assert_eq!(sending_line("Storefront", false), "Starting in Storefront…");
        assert_eq!(sending_line("Storefront", true), "Adding to Later in Storefront…");
        assert_eq!(sent_line("Storefront", false, None), "Started in Storefront · Z to undo");
        assert_eq!(sent_line("Storefront", true, None), "Added to Later in Storefront · Z to undo");
        // Nothing started: every sign-in of its agent is out of usage.
        assert_eq!(sent_line("Storefront", false, Some(("Claude Code", Some("6:10pm")))), "Claude Code is out until 6:10pm · Z to undo");
        assert_eq!(sent_line("Storefront", false, Some(("Codex", None))), "Codex is out of usage · Z to undo");
        assert_eq!(toast_parts("Started in Storefront · Z to undo"), ("Started in Storefront".to_string(), true));
        // A sentence that only mentions Z keeps its words.
        assert_eq!(toast_parts("Press Z to undo a close."), ("Press Z to undo a close.".to_string(), false));
        assert_eq!(not_sent_words_kept(&not_sent("the disk is full.")), "Not sent: the disk is full. Your words are in the new thread card.");
        let snapshot = json!({ "supervisor": { "atLimit": { "w-1": { "tool": "Codex", "until": 5 } } } });
        assert_eq!(limit_of(&snapshot, "w-1"), Some(Limit { tool: "Codex".to_string(), until: 5.0 }));
        assert_eq!(limit_of(&snapshot, "w-2"), None);
    }

    #[test]
    fn alone_on_the_inbox_the_tab_follows_the_thread_and_anywhere_else_the_person_stays() {
        assert_eq!(tab_after_sending(false, false, Some(Tab::Needs), false), Some(Tab::Progress));
        assert_eq!(tab_after_sending(false, true, Some(Tab::Needs), false), Some(Tab::Later));
        assert_eq!(tab_after_sending(true, false, Some(Tab::Needs), false), None);
        // Sent from over something else (Home, an open thread), or from Done.
        assert_eq!(tab_after_sending(false, false, Some(Tab::Needs), true), None);
        assert_eq!(tab_after_sending(false, false, Some(Tab::Done), false), None);
        assert_eq!(tab_after_sending(false, false, None, false), None);
    }

    #[test]
    fn z_takes_a_thread_back_until_its_agent_has_answered() {
        assert!(!answered_already(None));
        assert!(!answered_already(Some(&thread(json!({ "id": "w-1", "product": "shop", "result": "  " })))));
        assert!(answered_already(Some(&thread(json!({ "id": "w-1", "product": "shop", "result": "Done. Merge it?" })))));
        assert_eq!(withdraw("shop", "w-1"), json!({ "product": "shop", "id": "w-1", "status": "done" }));
        assert_eq!(said_withdrawn("Fix the sign in loop", true), "Withdrawn: Fix the sign in loop · your thread is back in the new thread card");
        assert_eq!(said_withdrawn("Fix the sign in loop", false), "Withdrawn: Fix the sign in loop");
    }

    #[test]
    fn a_menu_opens_on_what_is_picked_and_the_keys_walk_it_around_both_ends() {
        let (engines, models) = two_agents();
        let recent = recent_models(&[], &models, &engines);
        let all = all_models(&models, &engines);
        let efforts = effort_choices(&Pick::new("claude", "sonnet"), &models);
        let first = model_entries(false, &recent, &all, &efforts);
        assert_eq!(first.len(), 3 + 1 + 5);
        assert_eq!(first[3], Entry::AllModels);
        let picked = |entry: &Entry| *entry == Entry::Model(Pick::new("claude", "sonnet"));
        assert_eq!(entry_lit_first(&first, picked), 1);
        let every = model_entries(true, &recent, &all, &efforts);
        assert_eq!(every[0], Entry::Recent);
        assert_eq!(every.len(), 1 + 8 + 2 + 5);
        assert_eq!(entry_lit_first(&every, picked), 3);
        // Nothing picked among them: the keys start on the first.
        assert_eq!(entry_lit_first(&level_entries(), |_| false), 0);
        assert_eq!(level_entries()[0], Entry::Level(None));
        assert_eq!((entry_step(0, 5, false), entry_step(4, 5, true), entry_step(2, 5, true), entry_step(0, 0, true)), (4, 0, 3, 0));
        let shop = project(json!({ "slug": "shop", "name": "Storefront" }));
        assert_eq!(project_entries(&[&shop]), [Entry::Project("shop".to_string()), Entry::NewProject]);
    }

    #[test]
    fn a_menu_stays_where_the_stylesheet_hangs_it_while_it_fits() {
        let below = Hang { below: true, gap: 8.0, dx: 0.0 };
        // Under a chip in the middle of a 760 point window, with room to spare.
        assert_eq!(fit_menu([309.0, 385.4, 75.0, 28.0], below, (256.0, 261.5), 760.0, (82.0, 1258.0)), Fit { left: 309.0, top: 421.4, max_height: None });
        // Rising out of a card's bar, twelve points over its chip, and hung a little to the left.
        let above = Hang { below: false, gap: 12.0, dx: -16.0 };
        assert_eq!(fit_menu([434.0, 428.0, 128.0, 28.0], above, (190.0, 211.5), 760.0, (0.0, 1280.0)), Fit { left: 418.0, top: 204.5, max_height: None });
    }

    #[test]
    fn a_menu_with_no_room_turns_to_the_roomier_side_and_scrolls_inside_itself() {
        let below = Hang { below: true, gap: 8.0, dx: 0.0 };
        // Too tall for either side: it stays under, as tall as the room there, to a whole point.
        assert_eq!(fit_menu([309.0, 385.4, 75.0, 28.0], below, (256.0, 431.5), 760.0, (82.0, 1258.0)), Fit { left: 309.0, top: 421.4, max_height: Some(330.0) });
        // Near the foot of the window it opens upward instead, eight points over its control.
        assert_eq!(fit_menu([300.0, 600.0, 80.0, 28.0], below, (256.0, 300.0), 760.0, (0.0, 1280.0)), Fit { left: 300.0, top: 292.0, max_height: None });
        // A menu set to rise that would reach into the title bar's band drops instead.
        let above = Hang { below: false, gap: 12.0, dx: 0.0 };
        assert_eq!(fit_menu([300.0, 230.0, 80.0, 28.0], above, (190.0, 200.0), 760.0, (0.0, 1280.0)), Fit { left: 300.0, top: 266.0, max_height: None });
        // However short the room, it keeps 96 points to scroll in.
        assert_eq!(fit_menu([300.0, 60.0, 80.0, 28.0], below, (190.0, 400.0), 180.0, (0.0, 1280.0)).max_height, Some(96.0));
    }

    #[test]
    fn a_menu_that_would_leave_the_strip_it_is_seen_in_slides_back_inside() {
        let below = Hang { below: true, gap: 8.0, dx: 0.0 };
        assert_eq!(fit_menu([1100.4, 300.0, 80.0, 28.0], below, (370.0, 200.0), 760.0, (82.0, 1258.0)).left, 880.4);
        assert_eq!(fit_menu([70.0, 300.0, 80.0, 28.0], Hang { dx: -16.0, ..below }, (190.0, 200.0), 760.0, (82.0, 1258.0)).left, 90.0);
    }

    #[test]
    fn a_letter_on_the_card_is_typed_and_never_presses_what_is_under_it() {
        // In the dialog, with the keyboard on one of its controls or on nothing at all.
        assert!(key_is_typed(true, false, false, Keyboard::Card, false));
        assert!(key_is_typed(true, false, false, Keyboard::Nothing, false));
        // Not a chord, not an arrow, not with a menu open, and the box types by itself.
        assert!(!key_is_typed(true, true, false, Keyboard::Card, false));
        assert!(!key_is_typed(false, false, false, Keyboard::Card, false));
        assert!(!key_is_typed(true, false, true, Keyboard::Card, false));
        assert!(!key_is_typed(true, false, false, Keyboard::Field, false));
        // The box on the page does not own the screen: with nothing focused a key is the page's.
        assert!(!key_is_typed(true, false, false, Keyboard::Nothing, true));
        assert!(key_is_typed(true, false, false, Keyboard::Card, true));
    }

    #[test]
    fn command_return_sends_the_dialog_always_and_the_box_on_the_page_only_with_words_and_nothing_focused() {
        assert!(mod_enter_sends(false, Keyboard::Card, false));
        assert!(mod_enter_sends(true, Keyboard::Nothing, true));
        assert!(!mod_enter_sends(true, Keyboard::Nothing, false));
        assert!(!mod_enter_sends(true, Keyboard::Card, true));
    }

    #[test]
    fn a_letter_that_is_not_one_of_the_lists_keys_is_somebody_typing() {
        for key in ["j", "K", "n", "z", "e", "1", " ", "Enter", "ArrowDown"] {
            assert!(key_is_the_lists(key), "{key}");
        }
        for key in ["a", "T", "?", "5"] {
            assert!(!key_is_the_lists(key), "{key}");
        }
    }
}
