//! What the engine hands a window: the snapshot (`zero:snapshot`), and the lists the screens
//! draw from it. The shapes follow `renderer/src/types.ts`; a field with no name here is kept
//! in `rest`, so nothing is lost on the way through. Which thread goes in which list, in what
//! order and with which words is `rules`.

use serde::{Deserialize, Serialize};
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

/// Who last set one field of a thread, and when (`wrote` in `renderer/src/types.ts`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Wrote<'a> {
    /// Absent on a line folded before times were kept; the rules then read it as 0.
    pub ts: Option<f64>,
    /// `founder`, `agent` or `system`.
    pub source: &'a str,
    /// The teammate whose words these are, on a team.
    pub teammate: Option<&'a str>,
    /// The person who wrote it, once someone is signed in. With nobody signed in no line carries one.
    pub by: Option<&'a str>,
}

impl Wrote<'_> {
    pub fn at(&self) -> f64 {
        self.ts.unwrap_or(0.0)
    }
}

/// The fields the list rules read besides the ones above. They stay in `rest`, where a screen
/// may already be reading them, and are read through here by the name the web app gives them.
impl Item {
    /// A text field, when it holds any text: JavaScript reads an empty one as absent.
    pub fn text(&self, field: &str) -> Option<&str> {
        self.rest.get(field).and_then(Value::as_str).filter(|s| !s.is_empty())
    }
    /// A text field as it came, empty or not: what `??` in the web app falls through on is
    /// only a missing one.
    pub fn string(&self, field: &str) -> Option<&str> {
        self.rest.get(field).and_then(Value::as_str)
    }
    pub fn number(&self, field: &str) -> Option<f64> {
        self.rest.get(field).and_then(Value::as_f64)
    }
    /// Whether JavaScript would call the field truthy.
    pub fn truthy(&self, field: &str) -> bool {
        truthy(self.rest.get(field))
    }
    pub fn object(&self, field: &str) -> Option<&Map<String, Value>> {
        self.rest.get(field).and_then(Value::as_object)
    }
    pub fn list(&self, field: &str) -> &[Value] {
        self.rest.get(field).and_then(Value::as_array).map_or(&[], Vec::as_slice)
    }
    pub fn wrote(&self, field: &str) -> Option<Wrote<'_>> {
        let w = self.object("wrote")?.get(field)?.as_object()?;
        let text = |field: &str| w.get(field).and_then(Value::as_str).filter(|s| !s.is_empty());
        Some(Wrote { ts: w.get("ts").and_then(Value::as_f64), source: w.get("source").and_then(Value::as_str).unwrap_or(""), teammate: text("teammate"), by: text("by") })
    }
    /// When `field` was last written, or 0 (`wrote?.[field]?.ts ?? 0`).
    pub fn wrote_at(&self, field: &str) -> f64 {
        self.wrote(field).map_or(0.0, |w| w.at())
    }
    pub fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|l| l == label)
    }
    /// The answer, unless it was taken back (`liveAnswer` in `renderer/src/list-rules.ts`).
    pub fn live_answer(&self) -> Option<&str> {
        self.text("answer").filter(|a| *a != "(withdrawn)")
    }
    pub fn result_text(&self) -> Option<&str> {
        self.result.as_deref().filter(|s| !s.is_empty())
    }
}

/// JavaScript's truthiness, for a value read off the snapshot.
pub fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|n| n != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// A fixture file: the snapshot a reference picture was taken with, and the clock it was taken at,
/// so the native screen can be drawn from the very same data (`tools/reference.mjs` writes it).
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Fixture {
    pub now: Option<f64>,
    pub snapshot: Snapshot,
    pub local: Local,
    /// Whatever else the file carries, for the screen that knows what it is (a thread's words).
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

/// What the web window keeps for itself, outside the snapshot (its localStorage): what counts
/// as already delivered and already read. The native window will keep its own; a fixture
/// carries the page's, so both screens decide from the same facts.
#[derive(Deserialize, Default, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Local {
    /// `zero.deliveredThrough`: a thread finished before this was never news here.
    pub delivered_through: f64,
    /// `zero.unreadSince`: everything older counts as read.
    pub unread_since: f64,
    /// `zero.seenAt`: when each thread was last opened, by id.
    pub seen_at: std::collections::HashMap<String, f64>,
    /// `zero.seen`: the threads opened at least once. An imported conversation stays on top until it is.
    pub seen: Vec<String>,
    /// `zero.snooze`: the old put-off times, by id. Never written any more, still honoured.
    pub snooze: std::collections::HashMap<String, f64>,
    /// `zero.troubleClosed`: the pile of stuck tasks the person dismissed, named by when it began.
    pub trouble_closed: f64,
}

/// The lists every screen draws, worked out once from a snapshot.
#[derive(Default, Clone, Debug, Serialize)]
pub struct Lists {
    /// The Inbox: what waits on the person, in the order Home and the Inbox page draw it. Home
    /// lists the first eighty and counts them all.
    pub needs: Vec<Row>,
    /// What an agent is working on, or will without being asked, in the order it will run:
    /// Home's order, of which it lists the first sixty. The Inbox page's own tab reads
    /// `in_progress_tab`.
    pub progress: Vec<Row>,
    /// What is finished and put away, newest finished first.
    pub done: Vec<Row>,
}

impl Lists {
    /// In progress as the Inbox page's tab draws it: pinned first, then newest sent first
    /// (`order` in `renderer/src/threads/page-rules.ts`). Rows that tie keep Home's order.
    pub fn in_progress_tab(&self) -> Vec<&Row> {
        let mut rows: Vec<&Row> = self.progress.iter().collect();
        rows.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.at.unwrap_or(0.0).total_cmp(&a.at.unwrap_or(0.0))));
        rows
    }
}

/// How strongly a row asks for attention, as the web app's priority marks draw it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    /// The level a stored number shows as. Nought, or none, is untagged and runs as Medium
    /// (`itemPriority` in `shared/rank.mts`, `priorityIdOf` in `renderer/src/priority.ts`).
    pub fn of(n: Option<f64>) -> Priority {
        match n.filter(|n| n.is_finite() && *n > 0.0).unwrap_or(5.0) {
            n if n >= 9.0 => Priority::Urgent,
            n if n >= 7.0 => Priority::High,
            n if n <= 2.0 => Priority::Low,
            _ => Priority::Medium,
        }
    }
    /// The word the Inbox table writes beside the mark; nothing at Medium, which wears no mark.
    pub fn label(self) -> &'static str {
        match self {
            Priority::Urgent => "Urgent",
            Priority::High => "High",
            Priority::Medium => "",
            Priority::Low => "Low",
        }
    }
}

/// The mark the Inbox table draws before a Status word, which it does only for a word about an
/// agent (`StatusMark` in `renderer/src/threads/Pages.tsx`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mark {
    /// The word is the thread's own state: no mark.
    #[default]
    None,
    /// An agent is on it now, on this Mac: the turning half-dot.
    Running,
    /// An agent is on it now, in the cloud.
    Cloud,
    /// Waiting its turn, which resolves itself.
    Queued,
    /// Nothing starts it until something changes: the accent ring.
    Held,
    /// Someone's words on it wait on the person: the solid dot.
    Need,
}

/// One row of a list: the same few facts Home and the Inbox both show.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub product: String,
    /// C for Claude Code, X for Codex, A for Antigravity.
    pub letter: &'static str,
    pub title: String,
    /// The project's name, as Home's tag says it.
    pub project: String,
    pub priority: Priority,
    /// When the person last sent it (`sentAt` in `renderer/src/threads/page-rules.ts`).
    pub at: Option<f64>,
    /// The line a list prints under the title where it prints one: the agent's answer, or the
    /// ask while nothing has answered it (`rowSummary` in `renderer/src/list-rules.ts`).
    pub line: String,
    /// The Inbox table's Status word: Done, Answered, Waiting on you, Working… Empty where the
    /// tab already says it.
    pub status: String,
    pub unread: bool,
    pub pinned: bool,
    /// The mark before the Status word.
    pub mark: Mark,
    /// Faint words after the title in the Inbox table: a comment on a link, a wait that ended,
    /// how many tasks it filed still wait for a yes.
    pub aside: String,
    /// Written down in Later and not begun: the table says "not started" after the title.
    pub held: bool,
    /// The Inbox table's Project cell. The project's name, except on a cause row, which names
    /// where most of the threads behind it are and counts the other projects in `place_more`.
    pub place: String,
    pub place_more: usize,
    /// The coding agent it is on: Claude Code, Codex or Antigravity. Home's In progress row
    /// says it before the project, "Claude Code · Storefront".
    pub engine: &'static str,
    /// The time both pages write, "6 min ago", "Yesterday", "Sep 28", told from `shown_at`.
    pub when: String,
    /// The moment `when` tells: when it was finished on Done, `at` everywhere else.
    pub shown_at: f64,
}

impl Snapshot {
    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Every list at once, as of `now`. The one door the screens come through; the rules behind
    /// it are `renderer/src/list-rules.ts` and its neighbours, ported in `rules`.
    pub fn lists(&self, now: f64, local: &Local) -> Lists {
        crate::rules::lists(self, now, local)
    }
}
