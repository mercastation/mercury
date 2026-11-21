//! What the engine hands a window: the snapshot (`zero:snapshot`), and the lists the screens
//! draw from it. The shapes follow `renderer/src/types.ts`; a field this app does not read yet
//! is kept in `rest`, so nothing is lost on the way through.

use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Snapshot {
    pub products: Vec<Product>,
    pub items: Vec<Item>,
    pub supervisor: Supervisor,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Product {
    pub slug: String,
    pub name: String,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Item {
    pub id: String,
    pub product: String,
    pub product_name: Option<String>,
    pub status: String,
    pub kind: Option<String>,
    pub title: Option<String>,
    /// The thread's name by subject, when the app has given it one.
    pub label: Option<String>,
    pub body: Option<String>,
    pub result: Option<String>,
    pub labels: Vec<String>,
    pub priority: Option<f64>,
    pub engine: Option<String>,
    pub created_at: Option<f64>,
    pub updated_at: Option<f64>,
    pub run_at: Option<f64>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Supervisor {
    pub running: Vec<Run>,
    pub product_order: Vec<String>,
    pub hidden_products: Vec<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Run {
    pub item_id: String,
    pub product: String,
    pub engine: Option<String>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

/// A fixture file: the snapshot a reference picture was taken with, and the clock it was taken at,
/// so the native screen can be drawn from the very same data (`tools/reference.mjs` writes it).
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Fixture {
    pub now: Option<f64>,
    pub snapshot: Snapshot,
}

/// How strongly a row asks for attention, as the web app's priority marks draw it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    pub fn of(n: Option<f64>) -> Priority {
        match n.unwrap_or(5.0) {
            n if n >= 9.0 => Priority::Urgent,
            n if n >= 7.0 => Priority::High,
            n if n >= 4.0 => Priority::Medium,
            _ => Priority::Low,
        }
    }
}

/// One row of a list: the same few facts Home and the Inbox both show.
#[derive(Clone, Debug)]
pub struct Row {
    pub id: String,
    pub product: String,
    /// C for Claude Code, X for Codex, A for Antigravity.
    pub letter: &'static str,
    pub title: String,
    pub project: String,
    pub priority: Priority,
    /// When the person last sent it (`sentAt` in `renderer/src/threads/page-rules.ts`).
    pub at: Option<f64>,
    /// The agent's answer, or for a running thread what it is doing.
    pub line: String,
    /// The Inbox table's Status word: Done, Answered, Waiting on you, Working…
    pub status: String,
    pub unread: bool,
    pub pinned: bool,
}

impl Snapshot {
    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    fn product_name(&self, item: &Item) -> String {
        item.product_name.clone().or_else(|| self.products.iter().find(|p| p.slug == item.product).map(|p| p.name.clone())).unwrap_or_else(|| item.product.clone())
    }

    fn is_running(&self, item: &Item) -> bool {
        self.supervisor.running.iter().any(|r| r.item_id == item.id && r.product == item.product)
    }

    fn letter(&self, item: &Item) -> &'static str {
        let engine = self.supervisor.running.iter().find(|r| r.item_id == item.id && r.product == item.product).and_then(|r| r.engine.as_deref()).or(item.engine.as_deref());
        match engine {
            Some("codex") => "X",
            Some("antigravity") => "A",
            _ => "C",
        }
    }

    fn row(&self, item: &Item) -> Row {
        Row {
            id: item.id.clone(),
            product: item.product.clone(),
            letter: self.letter(item),
            title: first_real_line(item.label.as_deref().or(item.title.as_deref()).unwrap_or("")),
            project: self.product_name(item),
            priority: Priority::of(item.priority),
            at: item.created_at.or(item.updated_at),
            line: first_real_line(item.result.as_deref().unwrap_or("")),
            status: match item.status.as_str() {
                "done" => "Done",
                "blocked" => "Waiting on you",
                "claimed" => "Working",
                _ => "Not started",
            }
            .to_string(),
            unread: false,
            pinned: false,
        }
    }

    fn visible(&self, item: &Item) -> bool {
        !self.supervisor.hidden_products.contains(&item.product)
    }

    /// Where a product stands in the person's own order; one they never placed goes last.
    fn product_rank(&self, slug: &str) -> usize {
        self.supervisor.product_order.iter().position(|p| p == slug).unwrap_or(usize::MAX)
    }

    /// The threads waiting on the person: answered and not running, their products in the
    /// person's order, the most pressing first and the newest first among equals.
    pub fn needs_you(&self) -> Vec<Row> {
        let mut items: Vec<&Item> = self.items.iter().filter(|i| self.visible(i) && i.status == "done" && !self.is_running(i)).collect();
        items.sort_by(|a, b| {
            self.product_rank(&a.product)
                .cmp(&self.product_rank(&b.product))
                .then(b.priority.unwrap_or(5.0).total_cmp(&a.priority.unwrap_or(5.0)))
                .then(b.updated_at.unwrap_or(0.0).total_cmp(&a.updated_at.unwrap_or(0.0)))
        });
        items.into_iter().map(|i| self.row(i)).collect()
    }

    /// The threads an agent is working on right now.
    pub fn in_progress(&self) -> Vec<Row> {
        self.supervisor.running.iter().filter_map(|r| self.items.iter().find(|i| i.id == r.item_id && i.product == r.product)).filter(|i| self.visible(i)).map(|i| self.row(i)).collect()
    }
}

/// The first line of a text that says something, without the markdown around it.
pub fn first_real_line(s: &str) -> String {
    s.lines().map(|l| l.trim().trim_start_matches('#').trim().replace("**", "")).find(|l| !l.is_empty()).unwrap_or_default()
}

/// How long ago, as Home says it (`agoShort` in `renderer/src/home/home-model.ts`).
pub fn ago_short(at: f64, now: f64) -> String {
    let m = ((now - at) / 60_000.0).floor().max(0.0) as i64;
    if m < 1 {
        return "now".into();
    }
    if m < 60 {
        return format!("{m} min");
    }
    let h = m / 60;
    if h < 24 { format!("{h} hr") } else { format!("{} d", h / 24) }
}
