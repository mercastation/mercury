//! The Inbox's View and filters (`DisplayMenu` in `renderer/src/threads/Pages.tsx`): what the
//! person chose, what each choice keeps of the lists, and the order the rows then run in. A port
//! of the display half of `renderer/src/threads/page-rules.ts` for one person's page
//! (`Display`, `readDisplay`, `isFiltered`, `keeps`, `projectChoices`, `filteredEmptyWords`),
//! and of what `renderer/src/App.tsx` does with it: the filters thin every list a screen draws
//! from the Inbox (Needs you on Home and on the page, the tabs and their numbers, the rail's
//! count, J and K inside a conversation), Home's In progress card excepted, which lists the
//! whole tab under a number that counts what the filters keep (`homeProgress`, `progressCount`).
//!
//! Whose threads are on the page (People) and who sees a thread (Privacy) are a team's choices:
//! the web window offers neither to one person, and neither is here.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::nowhere::TROUBLE_ID;
use crate::data::{Item, Lists, Priority, Product, Row, Tab, truthy};

/// The list, or the board (`view`). The board is the web window's alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    List,
    Board,
}

/// What Needs you and Later run by (`sort`): the project's place and the level, or when the
/// person last wrote (`updated` in the web window's own words, "Sent" on the menu).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Priority,
    Sent,
}

/// When a thread was last sent, as the Sent line keeps it (`UpdatedWindow`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Window {
    Today,
    Week,
    #[default]
    Any,
}

impl Window {
    pub const ALL: [Window; 3] = [Window::Today, Window::Week, Window::Any];

    pub fn label(self) -> &'static str {
        match self {
            Window::Today => "Today",
            Window::Week => "This week",
            Window::Any => "Any time",
        }
    }

    /// The web window's own name for it (`UpdatedWindow`).
    pub fn id(self) -> &'static str {
        match self {
            Window::Today => "today",
            Window::Week => "week",
            Window::Any => "any",
        }
    }
}

/// The levels the Priority line offers, in its order (`PRIORITIES`).
pub const LEVELS: [Priority; 4] = [Priority::Urgent, Priority::High, Priority::Medium, Priority::Low];

/// The web window's own name for a level (`PriorityId`).
fn level_id(p: Priority) -> &'static str {
    match p {
        Priority::Urgent => "urgent",
        Priority::High => "high",
        Priority::Medium => "medium",
        Priority::Low => "low",
    }
}

/// The word a level's chip says.
pub fn level_word(p: Priority) -> &'static str {
    match p {
        Priority::Urgent => "Urgent",
        Priority::High => "High",
        Priority::Medium => "Medium",
        Priority::Low => "Low",
    }
}

/// One person's display: kept as the web window keeps its own (`threads.display.inbox`), in the
/// same shape, and read as leniently (`readDisplay`): a value it does not know is the default's.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Value", into = "Value")]
pub struct Display {
    pub view: View,
    pub sort: Sort,
    /// The levels picked; none picked keeps every level.
    pub priorities: Vec<Priority>,
    /// The projects picked, by slug; none picked keeps every project.
    pub projects: Vec<String>,
    pub updated: Window,
}

impl From<Value> for Display {
    fn from(v: Value) -> Display {
        let word = |field: &str| v.get(field).and_then(Value::as_str).unwrap_or("");
        let list = |field: &str| v.get(field).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        let mut priorities = Vec::new();
        for p in list("priorities").iter().filter_map(Value::as_str) {
            if let Some(level) = LEVELS.into_iter().find(|l| level_id(*l) == p)
                && !priorities.contains(&level)
            {
                priorities.push(level);
            }
        }
        Display {
            view: if word("view") == "board" { View::Board } else { View::List },
            sort: if word("sort") == "updated" { Sort::Sent } else { Sort::Priority },
            priorities,
            projects: list("projects").iter().filter_map(Value::as_str).map(str::to_string).collect(),
            updated: match word("updated") {
                "today" => Window::Today,
                "week" => Window::Week,
                _ => Window::Any,
            },
        }
    }
}

impl From<Display> for Value {
    fn from(d: Display) -> Value {
        json!({
            "view": if d.view == View::Board { "board" } else { "list" },
            "sort": if d.sort == Sort::Sent { "updated" } else { "priority" },
            "priorities": d.priorities.iter().map(|p| level_id(*p)).collect::<Vec<_>>(),
            "projects": d.projects,
            "updated": d.updated.id(),
        })
    }
}

/// A day and a week, in milliseconds.
const WEEK: f64 = 7.0 * 86_400_000.0;

impl Display {
    /// Whether any filter is on, which is what puts the dot on the display button.
    pub fn is_filtered(&self) -> bool {
        !self.priorities.is_empty() || !self.projects.is_empty() || self.updated != Window::Any
    }

    /// The same display with no filter: the foot's Clear filters, and the emptied tab's.
    pub fn cleared(&self) -> Display {
        Display { priorities: Vec::new(), projects: Vec::new(), updated: Window::Any, ..self.clone() }
    }

    /// A level's chip pressed: picked, or let go when it was (`toggle`).
    pub fn toggled_level(&self, level: Priority) -> Display {
        let mut priorities = self.priorities.clone();
        match priorities.iter().position(|p| *p == level) {
            Some(at) => {
                priorities.remove(at);
            }
            None => priorities.push(level),
        }
        Display { priorities, ..self.clone() }
    }

    /// A project's chip pressed, the same way.
    pub fn toggled_project(&self, slug: &str) -> Display {
        let mut projects = self.projects.clone();
        match projects.iter().position(|p| p == slug) {
            Some(at) => {
                projects.remove(at);
            }
            None => projects.push(slug.to_string()),
        }
        Display { projects, ..self.clone() }
    }

    /// What the filters keep (`keeps`), read off a row as a list draws it: the level it shows,
    /// its project, and when the person last wrote on it. A row the app makes itself belongs to
    /// no project and passes that filter; the one that says nothing is running passes them all
    /// (`isTroubleRow` before `keeps` wherever `App.tsx` filters).
    pub fn keeps(&self, row: &Row, now: f64) -> bool {
        if row.id == TROUBLE_ID && row.product.is_empty() {
            return true;
        }
        if !self.priorities.is_empty() && !self.priorities.contains(&row.priority) {
            return false;
        }
        if !self.projects.is_empty() && !row.product.is_empty() && !self.projects.contains(&row.product) {
            return false;
        }
        let sent = row.at.unwrap_or(0.0);
        match self.updated {
            Window::Today => sent >= super::page::start_of_day(now),
            Window::Week => sent >= now - WEEK,
            Window::Any => true,
        }
    }
}

/// Thins the lists to what `d` keeps, as of `now`, and notes beforehand how many each tab held
/// (`Lists::whole`). Needs you, Later and Done lose what the filters do not keep. In progress
/// keeps every row for Home's card, each marked with whether the filters keep it, which the
/// page's tab and every count of it read.
pub fn show(lists: &mut Lists, d: &Display, now: f64) {
    lists.whole = [lists.needs.len(), lists.progress.len(), lists.later.len(), lists.done.len()];
    lists.behind.clear();
    for (tab, list) in [(Tab::Needs, &mut lists.needs), (Tab::Later, &mut lists.later), (Tab::Done, &mut lists.done)] {
        let behind = &mut lists.behind;
        list.retain(|r| {
            let kept = d.keeps(r, now);
            if !kept {
                behind.push((tab, r.product.clone(), r.id.clone()));
            }
            kept
        });
    }
    for r in &mut lists.progress {
        r.kept = d.keeps(r, now);
    }
}

/// The menu's foot (`Showing 4 of 7`): what the open tab draws, of what it holds.
pub fn showing(lists: &Lists, tab: Tab) -> String {
    format!("Showing {} of {}", lists.under(tab).len(), lists.whole(tab))
}

/// What the Sort by line says beside its choices on a tab whose order they do not change.
pub fn sort_note(tab: Tab) -> Option<&'static str> {
    match tab {
        Tab::Done => Some("Done runs newest first"),
        Tab::Progress => Some("In progress runs newest sent first"),
        _ => None,
    }
}

/// A project that is a conversation between two people, which no project list shows (`isDirect`).
fn is_direct(p: &Product) -> bool {
    p.rest.get("team").and_then(|t| t.get("direct")).is_some_and(|d| truthy(Some(d)))
}

/// Two names in the order a person reads them (`localeCompare`), near enough: by their letters
/// whatever their case, and by their case only when the letters tie.
fn by_name(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase().cmp(&b.to_lowercase()).then_with(|| a.cmp(b))
}

/// The projects the Project line offers, most recently used first, and the rest behind its
/// "Show all" (`projectChoices`): the eight with the newest threads, ties by name; a picked
/// project is never out of sight, so the picked come first and as many as there are stand.
/// Conversations between people and the practice project are not projects to pick.
pub fn project_choices<'a>(products: &'a [Product], items: &[Item], picked: &[String], limit: usize) -> (Vec<&'a Product>, Vec<&'a Product>) {
    let mut last: HashMap<&str, f64> = HashMap::new();
    for i in items.iter().filter(|i| !i.truthy("agent")) {
        let was = last.get(i.product.as_str()).copied().unwrap_or(0.0);
        if let Some(at) = i.updated_at.filter(|at| *at > was) {
            last.insert(i.product.as_str(), at);
        }
    }
    let at = |p: &Product| last.get(p.slug.as_str()).copied().unwrap_or(0.0);
    let mut ranked: Vec<&Product> = products.iter().filter(|p| !is_direct(p) && !truthy(p.rest.get("practice"))).collect();
    ranked.sort_by(|a, b| at(b).partial_cmp(&at(a)).unwrap_or(std::cmp::Ordering::Equal).then_with(|| by_name(&a.name, &b.name)));
    let (mut ordered, off): (Vec<&Product>, Vec<&Product>) = ranked.into_iter().partition(|p| picked.contains(&p.slug));
    let keep = limit.max(ordered.len());
    ordered.extend(off);
    let rest = ordered.split_off(keep.min(ordered.len()));
    (ordered, rest)
}

/// How many the Project line shows before "Show all" (`limit` in `projectChoices`).
pub const PROJECTS_SHOWN: usize = 8;

/// What a tab the filters emptied says (`filteredEmptyWords`): the half that is true either way
/// as its heading, and how many threads the filters hold back under it.
pub fn filtered_empty_words(tab: Tab, hidden: usize) -> (&'static str, String) {
    let head = match tab {
        Tab::Progress => "Nothing in your filter is running",
        Tab::Later => "Nothing in your filter is in Later",
        Tab::Done => "Nothing in your filter is closed",
        Tab::Needs => "Nothing in your filter needs you",
    };
    let line = if hidden == 1 { "One more thread is behind your filters.".to_string() } else { format!("{hidden} more threads are behind your filters.") };
    (head, line)
}

/// What a press on Board, or B, says: the board is the web window's.
pub const BOARD_NOT_HERE: &str = "The board is in Mercury's own window.";

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(id: &str, product: &str, priority: Priority, at: f64) -> Row {
        Row {
            id: id.to_string(),
            product: product.to_string(),
            letter: "C",
            title: id.to_string(),
            project: String::new(),
            priority,
            at: Some(at),
            line: String::new(),
            status: String::new(),
            unread: false,
            pinned: false,
            mark: crate::data::Mark::None,
            aside: String::new(),
            held: false,
            place: String::new(),
            place_more: 0,
            engine: "Claude Code",
            when: String::new(),
            shown_at: at,
            kept: true,
        }
    }

    #[test]
    fn a_display_reads_as_the_web_window_keeps_it_and_anything_odd_reads_as_the_default() {
        let kept = json!({ "view": "board", "sort": "updated", "priorities": ["urgent", "low", "nonsense", "urgent"], "projects": ["shop", 7], "updated": "week", "v": 2 });
        let d = Display::from(kept);
        assert_eq!(d, Display { view: View::Board, sort: Sort::Sent, priorities: vec![Priority::Urgent, Priority::Low], projects: vec!["shop".to_string()], updated: Window::Week });
        assert_eq!(Value::from(d.clone()), json!({ "view": "board", "sort": "updated", "priorities": ["urgent", "low"], "projects": ["shop"], "updated": "week" }));
        assert_eq!(Display::from(Value::from(d.clone())), d);
        assert_eq!(Display::from(json!("not a display")), Display::default());
        assert_eq!(Display::from(json!({ "sort": "newest", "updated": "month" })), Display::default());
        // Kept beside the window's other picks, it goes through serde the same way.
        let text = serde_json::to_string(&d).unwrap();
        assert_eq!(serde_json::from_str::<Display>(&text).unwrap(), d);
    }

    #[test]
    fn a_filter_is_on_until_it_is_cleared_and_a_chip_pressed_twice_lets_go() {
        let d = Display::default();
        assert!(!d.is_filtered());
        let urgent = d.toggled_level(Priority::Urgent);
        assert!(urgent.is_filtered());
        assert!(!urgent.toggled_level(Priority::Urgent).is_filtered());
        let shop = urgent.toggled_project("shop").toggled_project("desk").toggled_project("shop");
        assert_eq!(shop.projects, ["desk"]);
        let week = Display { updated: Window::Week, sort: Sort::Sent, ..shop };
        assert!(week.is_filtered());
        // Clearing keeps the sort and the view, which are not filters.
        assert_eq!(week.cleared(), Display { sort: Sort::Sent, ..Display::default() });
    }

    #[test]
    fn the_filters_keep_a_level_a_project_and_when_it_was_sent() {
        const NOW: f64 = 1_784_000_000_000.0;
        const HOUR: f64 = 3_600_000.0;
        let urgent_shop = row("a", "shop", Priority::Urgent, NOW - HOUR);
        let low_desk = row("b", "desk", Priority::Low, NOW - 3.0 * 24.0 * HOUR);
        let old = row("c", "shop", Priority::Medium, NOW - 10.0 * 24.0 * HOUR);
        let trouble = row(TROUBLE_ID, "", Priority::Medium, NOW - 30.0 * 24.0 * HOUR);
        let levels = Display { priorities: vec![Priority::Urgent, Priority::Low], ..Display::default() };
        assert!(levels.keeps(&urgent_shop, NOW) && levels.keeps(&low_desk, NOW) && !levels.keeps(&old, NOW));
        let shop = Display { projects: vec!["shop".to_string()], ..Display::default() };
        assert!(shop.keeps(&urgent_shop, NOW) && !shop.keeps(&low_desk, NOW) && shop.keeps(&old, NOW));
        let week = Display { updated: Window::Week, ..Display::default() };
        assert!(week.keeps(&urgent_shop, NOW) && week.keeps(&low_desk, NOW) && !week.keeps(&old, NOW));
        let today = Display { updated: Window::Today, ..Display::default() };
        assert!(!today.keeps(&low_desk, NOW));
        // The row that says nothing is running stays whatever the filters say.
        let everything = Display { priorities: vec![Priority::High], projects: vec!["desk".to_string()], updated: Window::Today, ..Display::default() };
        assert!(everything.keeps(&trouble, NOW));
    }

    #[test]
    fn the_lists_lose_what_the_filters_do_not_keep_and_remember_what_they_held() {
        const NOW: f64 = 1_784_000_000_000.0;
        let mut lists = Lists {
            needs: vec![row("a", "shop", Priority::Urgent, NOW), row("b", "desk", Priority::Low, NOW)],
            progress: vec![row("c", "shop", Priority::Medium, NOW), row("d", "desk", Priority::Medium, NOW)],
            later: vec![row("e", "desk", Priority::High, NOW)],
            done: vec![row("f", "shop", Priority::Low, NOW), row("g", "shop", Priority::High, NOW)],
            ..Lists::default()
        };
        show(&mut lists, &Display { projects: vec!["shop".to_string()], ..Display::default() }, NOW);
        assert_eq!(lists.whole, [2, 2, 1, 2]);
        assert_eq!(lists.needs.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["a"]);
        // Home's In progress card keeps every row; the tab and the counts keep the filtered.
        assert_eq!(lists.progress.len(), 2);
        assert_eq!(lists.in_progress_tab().iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["c"]);
        assert_eq!((lists.count(Tab::Needs), lists.count(Tab::Progress), lists.count(Tab::Later)), (1, 1, 0));
        assert_eq!((lists.whole(Tab::Later), lists.done.len()), (1, 2));
        assert_eq!(showing(&lists, Tab::Needs), "Showing 1 of 2");
        assert_eq!(showing(&lists, Tab::Later), "Showing 0 of 1");
        // A thread the filters took out is still in its tab, for the strip over it.
        assert!(lists.holds(Tab::Needs, "b", "desk") && lists.holds(Tab::Later, "e", "desk") && lists.holds(Tab::Progress, "d", "desk"));
        assert!(!lists.holds(Tab::Done, "b", "desk"));
        // Shown again with no filter, nothing is behind.
        show(&mut lists, &Display::default(), NOW);
        assert!(lists.behind.is_empty());
    }

    #[test]
    fn the_projects_offered_are_the_most_recently_used_and_a_picked_one_is_never_hidden() {
        let product = |slug: &str, name: &str| Product { slug: slug.to_string(), name: name.to_string(), ..Product::default() };
        let mut products: Vec<Product> = (0..10).map(|n| product(&format!("p{n}"), &format!("Project {n}"))).collect();
        products.push(serde_json::from_value(json!({ "slug": "dm", "name": "Theo", "team": { "direct": true } })).unwrap());
        products.push(serde_json::from_value(json!({ "slug": "practice", "name": "Practice", "practice": true })).unwrap());
        // p3 has the newest thread, then p7; an outside agent's row does not count.
        let items: Vec<Item> = [json!({ "product": "p3", "updatedAt": 30 }), json!({ "product": "p7", "updatedAt": 20 }), json!({ "product": "p9", "updatedAt": 99, "agent": true })]
            .into_iter()
            .map(|v| serde_json::from_value(v).unwrap())
            .collect();
        let slugs = |ps: &[&Product]| ps.iter().map(|p| p.slug.clone()).collect::<Vec<_>>();
        let (shown, rest) = project_choices(&products, &items, &[], PROJECTS_SHOWN);
        assert_eq!(slugs(&shown), ["p3", "p7", "p0", "p1", "p2", "p4", "p5", "p6"]);
        assert_eq!(slugs(&rest), ["p8", "p9"]);
        // A picked project comes first, so it is never behind Show all; it takes one of the eight.
        let (shown, rest) = project_choices(&products, &items, &["p9".to_string()], PROJECTS_SHOWN);
        assert_eq!(slugs(&shown)[0], "p9");
        assert_eq!((shown.len(), slugs(&rest)), (8, vec!["p6".to_string(), "p8".to_string()]));
        let picked: Vec<String> = (0..10).map(|n| format!("p{n}")).collect();
        assert_eq!(project_choices(&products, &items, &picked, PROJECTS_SHOWN).0.len(), 10);
    }

    #[test]
    fn an_emptied_tab_says_its_filters_hold_threads_back() {
        assert_eq!(filtered_empty_words(Tab::Needs, 9), ("Nothing in your filter needs you", "9 more threads are behind your filters.".to_string()));
        assert_eq!(filtered_empty_words(Tab::Later, 1), ("Nothing in your filter is in Later", "One more thread is behind your filters.".to_string()));
        assert_eq!(filtered_empty_words(Tab::Progress, 2).0, "Nothing in your filter is running");
        assert_eq!(filtered_empty_words(Tab::Done, 2).0, "Nothing in your filter is closed");
        assert_eq!((sort_note(Tab::Done), sort_note(Tab::Progress), sort_note(Tab::Needs)), (Some("Done runs newest first"), Some("In progress runs newest sent first"), None));
    }
}
