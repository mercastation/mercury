//! The command bar ⌘K opens: the rows it offers, which of them a typed query keeps, the
//! threads the same words find, and the order and headings all of it is drawn under. The web
//! window's own rules: `renderer/src/palette-rows.ts`, `palette-pick.ts`, `palette-keys.ts`
//! and `palette-glyphs.ts`, the order in `renderer/src/components/Palette.tsx`, and the rows
//! `renderer/src/App.tsx` hands the bar (`itemCommands`, `findThreads`).
//!
//! A row is offered here only when this window does what it says: go Home, go to the Inbox
//! and to each of its tabs, open a thread, and on the thread the keys are on reply, mark done,
//! reopen, put off and bring back. The web's list is far longer, because the web window does
//! far more; `README.md` says what it offers that this does not.
//!
//! One person, with no team: with a team's tabs on, Later and Done are columns of In progress
//! and the web's bar goes to two pages, not four (`pageRows({ team })`).

use rune::time::LocalTime;

use super::js::{self, re};
use super::{list, search, title};
use crate::data::{Item, Snapshot, Tab};

/// A thread, as `(id, product)`.
pub type Thread = (String, String);

/// What pressing a row does. The window carries it out (`src/palette.rs`); here it is only
/// named, so the rows can be decided and tested with no window.
#[derive(Clone, Debug, PartialEq)]
pub enum Does {
    /// Opens the box a new thread is written in.
    NewThread,
    Home,
    /// The Inbox, on this tab.
    Tab(Tab),
    Open(Thread),
    /// Opens the thread with the caret in its reply box.
    Reply(Thread),
    Done(Thread),
    Reopen(Thread),
    /// Opens the Later picker on it.
    Later(Thread),
    /// Brings it back from Later.
    Back(Thread),
    /// The Files place: All files, or one project's page.
    Files(Option<String>),
}

/// One row of the bar (`Command` in `Palette.tsx`).
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// The web row's own id. Its mark and its place among the matches hang on it.
    pub id: String,
    pub label: String,
    /// What is said at the row's end. Shown, never matched: it is copy, and a search that
    /// leaned on copy would break the next time someone improved the sentence.
    pub hint: Option<String>,
    /// A thread's line under its title: the sentence the query was found in.
    pub sub: Option<String>,
    /// A finished thread, drawn dimmed.
    pub done: bool,
    /// Words a person might type that the label does not say. Matched, never shown.
    pub keywords: &'static str,
    /// What the row was called before a rename: typing one ranks the row as its label would.
    pub old_names: &'static [&'static str],
    /// The key that does the same, drawn at the row's end in place of a hint.
    pub key: Option<&'static str>,
    pub does: Does,
}

impl Row {
    fn command(id: &str, label: &str, does: Does) -> Row {
        Row { id: id.to_string(), label: label.to_string(), hint: None, sub: None, done: false, keywords: "", old_names: &[], key: None, does }
    }
    fn found_by(mut self, keywords: &'static str) -> Row {
        self.keywords = keywords;
        self
    }
    fn on_key(mut self, key: &'static str) -> Row {
        self.key = Some(key);
        self
    }
}

/* --------------------------------- the rows offered --------------------------------- */

/// The row is called Later, after the key. It said "Remind Me (Snooze)" once, and the old
/// names still find it, ahead of any thread they also find (`LATER_KEYWORDS`, `LATER_OLD_NAMES`).
pub const LATER_KEYWORDS: &str = "remind me snooze put off defer schedule";
pub const LATER_OLD_NAMES: &[&str] = &["remind me", "snooze"];

/// No "inbox" among them: typed alone it means going there, and this row would sit above Go
/// to Inbox under the Return key (`REOPEN_KEYWORDS`).
pub const REOPEN_KEYWORDS: &str = "unmark undo undone not done open again bring back put back";

/// New thread, which N also does (`new-thread` in `Palette.tsx`): the row the web's list has
/// second, after New project. It is offered only once this window can start a thread.
pub fn new_thread_row() -> Row {
    let mut row = Row::command("new-thread", "New thread", Does::NewThread).found_by("task compose write message ask start").on_key("N");
    row.hint = Some("ask an agent".to_string());
    row
}

/// Home, then the tabs ⌘K goes to (`view-home` in `Palette.tsx`, and `pageRows`). Home stands
/// above the Inbox, as in the rail, so "home" lands on it although the Inbox answers to it too.
pub fn page_rows() -> Vec<Row> {
    vec![
        Row::command("view-home", "Go to Home", Does::Home).found_by("start overview"),
        Row::command("view-inbox", "Go to Inbox", Does::Tab(Tab::Needs)).found_by("needs you home list threads"),
        Row::command("view-progress", "Go to In progress", Does::Tab(Tab::Progress)).found_by("working running"),
        Row::command("view-later", "Go to Later", Does::Tab(Tab::Later)).found_by(LATER_KEYWORDS),
        Row::command("view-done", "Go to Done", Does::Tab(Tab::Done)).found_by("finished closed"),
    ]
}

/// The words a Files row is found by besides its name (`filePages` in `Palette.tsx`, whose
/// keywords end with the row's title, which a query finds in its name here).
const FILES_KEYWORDS: &str = "files file documents docs pictures images films videos decks slides pdfs designs shared";

/// The Files place and each project's page in it (`filePages` in `App.tsx`, after the tabs
/// there): Go to Files, then Files in each project the place has listed.
pub fn file_rows(projects: &[String]) -> Vec<Row> {
    let mut all = Row::command("files-all", "Go to Files", Does::Files(None)).found_by(FILES_KEYWORDS);
    all.hint = Some("every file you can see".to_string());
    let mut rows = vec![all];
    for p in projects {
        rows.push(Row::command(&format!("files-project-{p}"), &format!("Files in {p}"), Does::Files(Some(p.clone()))).found_by(FILES_KEYWORDS));
    }
    rows
}

/// When a thread that was put off comes back (`returnAt` in `App.tsx`).
fn return_at(item: &Item) -> f64 {
    list::deferred_until(item).max(list::hidden_until(item, 0.0))
}

/// What can be done to the thread the keys are on, in the web's order (`itemCommands` in
/// `App.tsx`), so ⌘K opened over a row cannot mean another thread than E pressed over it.
/// `open` says the thread is the conversation on screen, which has no row to open.
///
/// Left out, because this window cannot do them yet: Remote control, Open terminal, Approve,
/// Run now, Close task, Try again and Resume this agent, Mark read and Mark unread, Stop, Send
/// back to agent, the thread's own menu (rename, pin, where it runs, who sees it), its model,
/// its project and its priority.
pub fn thread_commands(item: &Item, open: bool, now: f64) -> Vec<Row> {
    let thread = || (item.id.clone(), item.product.clone());
    let mut rows = Vec::new();
    // "Mark done" is what the thread's own menu calls it.
    rows.push(if item.status == "done" {
        Row::command("reopen", "Reopen", Does::Reopen(thread())).found_by(REOPEN_KEYWORDS)
    } else {
        Row::command("done", "Mark done", Does::Done(thread())).on_key("E")
    });
    rows.push(Row::command("reply", "Reply", Does::Reply(thread())).on_key("R"));
    if return_at(item) > now {
        // A moment an agent set is the agent's own wait, not the person's reminder.
        let label = if list::parked_by_agent(item, now) { "Let it run now" } else { "Back to Inbox" };
        rows.push(Row::command("unsnooze", label, Does::Back(thread())).on_key("E"));
    }
    let mut later = Row::command("snooze", "Later", Does::Later(thread())).found_by(LATER_KEYWORDS).on_key("L");
    later.old_names = LATER_OLD_NAMES;
    rows.push(later);
    if !open {
        rows.push(Row::command("open", "Open", Does::Open(thread())).on_key("↵"));
    }
    rows
}

/// Every row the bar has before anything is typed, in the web's order: starting a thread,
/// then what acts on the thread under the keys, so every key can be searched for, then the
/// places (`commands` in `Palette.tsx`). `starts_threads` says this window can start one.
pub fn commands(under: Option<&Item>, open: bool, now: f64, starts_threads: bool) -> Vec<Row> {
    let mut rows = Vec::new();
    if starts_threads {
        rows.push(new_thread_row());
    }
    if let Some(item) = under {
        rows.extend(thread_commands(item, open, now));
    }
    rows.extend(page_rows());
    rows
}

/* ------------------------------- what a query keeps ------------------------------- */

/// Every word of the query must land, in any order, anywhere in the label or the keywords:
/// "dark mode" appears as one string in no label, and as two words it finds its row. A word
/// may land inside another, and nothing is fuzzy: a fuzzy match on a list this long puts rows
/// nobody asked for under the Return key (`matchesQuery`). The web's two other gates, a row
/// held back until one word is named and a row left off the untyped list, belong to rows this
/// window does not have.
pub fn matches_query(query: &str, row: &Row) -> bool {
    let query = query.to_lowercase();
    let words = js::words(&query);
    if words.is_empty() {
        return true;
    }
    let hay = format!("{} {}", row.label, row.keywords).to_lowercase();
    words.iter().all(|w| hay.contains(w))
}

/// When two rows answer to the same letters, the one meant goes first. "rem" is Later, which
/// was Remind Me, and not a row used once a month that starts with the same three letters: no
/// rule of strings separates the two, so the winner is named (`FIRST_AMONG_MATCHES`).
const FIRST_AMONG_MATCHES: [&str; 1] = ["snooze"];

/// The matches, with the named winner first, then a row whose name starts with what was
/// typed: "go to" is Go to Inbox, not a row that matched because "logo" holds "go". The rest
/// keep the order the list was put together in (`rankMatches`).
pub fn rank_matches(query: &str, mut rows: Vec<Row>) -> Vec<Row> {
    let q = js::trim(query).to_lowercase();
    if q.is_empty() {
        return rows;
    }
    let rank = |row: &Row| {
        if FIRST_AMONG_MATCHES.contains(&row.id.as_str()) {
            0
        } else if row.label.to_lowercase().starts_with(&q) {
            1
        } else {
            2
        }
    };
    rows.sort_by_key(rank);
    rows
}

/// Words as they are compared: small letters, one space between them (`norm` in `palette-pick.ts`).
fn said(s: &str) -> String {
    js::words(&s.to_lowercase()).join(" ")
}

re!(NOT_A_LETTER, r"[^\p{L}\p{N}]+", "gu");

/// Every word typed starts a word of the name (`namesIt`): "sum up" is not "reSUMe
/// interrUPted agents".
pub fn names_it(query: &str, label: &str) -> bool {
    let (typed, name) = (said(query), said(label));
    let words: Vec<&str> = typed.split(' ').filter(|w| !w.is_empty()).collect();
    let own: Vec<&str> = NOT_A_LETTER.split(&name).into_iter().filter(|w| !w.is_empty()).collect();
    !words.is_empty() && words.iter().all(|w| own.iter().any(|o| o.starts_with(w)))
}

/// The thread whose title is the words first, then one that starts with them, then one that
/// has them; otherwise the order they came in. A thread whose body names the words must not
/// sit above the thread whose title is them (`titleFirst`).
pub fn title_first(mut rows: Vec<Row>, query: &str) -> Vec<Row> {
    let q = said(query);
    if q.is_empty() {
        return rows;
    }
    rows.sort_by_key(|r| {
        let t = said(&r.label);
        if t == q {
            0
        } else if t.starts_with(&q) {
            1
        } else if t.contains(&q) {
            2
        } else {
            3
        }
    });
    rows
}

/* -------------------------------- the threads found -------------------------------- */

/// How many threads the bar lists (`THREADS_SHOWN`). The web's bar ends them on a row that
/// opens its search page on the rest; this window has no search page, so the best few are all.
pub const THREADS_SHOWN: usize = 5;

/// The bar's short age for a thread under a day old (`agoShort` in `renderer/src/home/home-model.ts`).
pub fn ago_short(at: f64, now: f64) -> String {
    let m = ((now - at) / 60_000.0).floor().max(0.0);
    if m < 1.0 {
        return "now".to_string();
    }
    if m < 60.0 {
        return format!("{m} min");
    }
    let h = (m / 60.0).floor();
    if h < 24.0 { format!("{h} hr") } else { format!("{} d", (h / 24.0).floor()) }
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// A found thread's row: what it is called, the line it was found by, where it is and how
/// old, and whether it is finished (`findThreads` in `App.tsx`, `threadResult` in
/// `palette-rows.ts`). A day old or more it says its date, not a count of days; the web window
/// writes that date in the system's language, and this writes it in English, as the rest of
/// the window speaks.
pub fn thread_row(item: &Item, summary: String, project: &str, now: f64) -> Row {
    let at = item.updated_at.or(item.created_at).unwrap_or(0.0);
    let age = if now - at < 86_400_000.0 {
        ago_short(at, now)
    } else {
        let day = LocalTime::at(at as i64);
        format!("{} {}", MONTHS[day.month as usize - 1], day.day)
    };
    let mut row = Row::command(&format!("thread-{}-{}", item.product, item.id), &title::shown_title(item), Does::Open((item.id.clone(), item.product.clone())));
    row.hint = Some(format!("{project} · {age}"));
    row.sub = Some(summary).filter(|s| !s.is_empty());
    row.done = item.status == "done";
    row
}

/// The threads a query adds to the bar: the best few of what the list's own search finds,
/// the one titled by the words first (`threadRows` and `titleFirst` in `Palette.tsx`). Nothing
/// typed finds nothing. The web's bar also finds repeating tasks and teammates' threads here,
/// which this window does not list.
pub fn threads_found(index: &search::Index, snap: &Snapshot, query: &str, now: f64) -> Vec<Row> {
    let q = js::trim(query);
    if q.is_empty() {
        return Vec::new();
    }
    // A project the snapshot does not name is called by its slug.
    let project = |slug: &str| snap.products.iter().find(|p| p.slug == slug).map_or_else(|| slug.to_string(), |p| p.name.clone());
    let found = search::search_items(index, &snap.items, q);
    let rows = found.into_iter().take(THREADS_SHOWN).map(|hit| thread_row(hit.item, hit.summary, &project(&hit.item.product), now)).collect();
    title_first(rows, query)
}

/* ------------------------------ the order it is drawn in ------------------------------ */

/// The rows in the order drawn, and the heading over the row each section starts at.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Listed {
    pub rows: Vec<Row>,
    pub labels: Vec<(usize, &'static str)>,
}

impl Listed {
    pub fn label_over(&self, at: usize) -> Option<&'static str> {
        self.labels.iter().find(|(row, _)| *row == at).map(|(_, label)| *label)
    }
}

/// Commands first, then the threads found, under their heading. Once something is typed, a
/// command goes ahead of the threads only when the words start words of its name, or of a
/// name it had: one the words merely touch by keyword comes after the threads, under a
/// heading of its own. And a command that carries a found thread's whole title ("Waits on
/// that thread") stays below that thread unless its own label starts with the words
/// (`paletteOrder`). The web's bar also heads people and groups, and holds one row back until
/// it is asked for by name; this window has neither.
pub fn order(matched: Vec<Row>, threads: Vec<Row>, query: &str) -> Listed {
    let named = |c: &Row| names_it(query, &c.label) || c.old_names.iter().any(|n| names_it(query, n));
    let named_titles: Vec<String> = threads.iter().filter(|t| names_it(query, &t.label)).map(|t| said(&t.label)).collect();
    let carries_a_named_title = |c: &Row| named_titles.iter().any(|t| said(&c.label).contains(t.as_str()));
    let ahead = |c: &Row| named(c) && (!carries_a_named_title(c) || said(&c.label).starts_with(&said(query)));
    let mut labels = Vec::new();
    if !threads.is_empty() && !js::trim(query).is_empty() {
        let (first, after): (Vec<Row>, Vec<Row>) = matched.into_iter().partition(ahead);
        labels.push((first.len(), "Threads"));
        if !after.is_empty() {
            labels.push((first.len() + threads.len(), "Commands"));
        }
        return Listed { rows: first.into_iter().chain(threads).chain(after).collect(), labels };
    }
    if !threads.is_empty() {
        labels.push((matched.len(), "Threads"));
    }
    Listed { rows: matched.into_iter().chain(threads).collect(), labels }
}

/// What the bar lists for what is typed: every command when nothing is, else the commands
/// the words keep, ranked, with the threads they find (`PaletteBox` in `Palette.tsx`).
pub fn listed(commands: Vec<Row>, threads: Vec<Row>, query: &str) -> Listed {
    let q = js::trim(query).to_lowercase();
    let matched = if q.is_empty() { commands } else { rank_matches(&q, commands.into_iter().filter(|c| matches_query(&q, c)).collect()) };
    order(matched, threads, query)
}

/* ---------------------------------- the row picked ---------------------------------- */

/// One key for each row, in order, unique even when two rows share an id (`commandKeys`).
pub fn row_keys(rows: &[Row]) -> Vec<String> {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    rows.iter()
        .map(|row| {
            let n = match seen.iter_mut().find(|(id, _)| *id == row.id) {
                Some((_, n)) => {
                    *n += 1;
                    *n
                }
                None => {
                    seen.push((row.id.as_str(), 0));
                    0
                }
            };
            if n > 0 { format!("{}#{n}", row.id) } else { row.id.clone() }
        })
        .collect()
}

/// Where the picked row is now: the first row when it is gone or none was picked, and nowhere
/// when there are no rows. The pick is a row, not a place, so a list that sorts again under it
/// keeps the same row lit, and Enter opens the row drawn as picked (`keptPick`).
pub fn kept_pick(key: Option<&str>, keys: &[String]) -> Option<usize> {
    if keys.is_empty() {
        return None;
    }
    Some(key.and_then(|key| keys.iter().position(|k| k == key)).unwrap_or(0))
}

/* ------------------------------- what the bar says ------------------------------- */

/// What the box says while it is empty.
pub const PLACEHOLDER: &str = "Search threads or type a command…";

const ECHO_MAX: usize = 32;

/// What stands where a row would have been when nothing matches. The query is said back,
/// because it is the one thing this line can say that the person does not already know: what
/// exactly was searched for, stray character and all. A pasted paragraph is cut at a word
/// (`emptyLine`).
pub fn empty_line(query: &str) -> String {
    let q = js::trim(query);
    let shown = if js::len(q) > ECHO_MAX {
        let cut = js::slice(q, 0, ECHO_MAX);
        let kept = match js::last_index_of(cut, " ") {
            Some(space) if space > 0 => js::slice(cut, 0, space),
            _ => cut,
        };
        format!("{}…", js::trim_end(kept))
    } else {
        q.to_string()
    };
    format!("Nothing matches \u{201c}{shown}\u{201d}.")
}

/// What a row nobody has drawn yet wears: always the same one (`PLAIN_ICON`).
pub const PLAIN_ICON: &str = "chevron";

/// The drawing each row wears by its id (`BY_ID` in `renderer/src/palette-glyphs.ts`).
const BY_ID: &[(&str, &str)] = &[
    ("new-project", "plus"),
    ("new-thread", "compose"),
    ("record-meeting", "record"),
    ("stop-recording", "stop"),
    ("filter", "filter"),
    ("clear-filter", "clear"),
    ("end-tutorial", "clear"),
    ("projects-page", "folder"),
    ("search", "search"),
    ("all-threads", "search"),
    ("view-home", "home"),
    ("view-inbox", "inbox"),
    ("view-progress", "progress"),
    ("view-later", "later"),
    ("view-done", "done"),
    ("view-groups", "hash"),
    ("view-teammates", "team"),
    ("view-connections", "link"),
    ("new-teammate", "add-person"),
    ("update", "download"),
    ("update-move", "move"),
    ("update-check", "refresh"),
    ("stale", "refresh"),
    ("resume-stopped", "play"),
    ("standing", "instructions"),
    ("usage", "gauge"),
    ("settings", "settings"),
    ("shortcuts", "keyboard"),
    ("keyhints", "keyboard"),
    ("flip-view", "board"),
    ("import-agents", "download"),
    ("tutorial", "help"),
    ("first-run", "refresh"),
    ("demo", "spark"),
    ("remote-control", "swap"),
    ("open-terminal", "terminal"),
    ("approve", "check"),
    ("run-now", "run"),
    ("done", "done"),
    ("close-task", "closed"),
    ("reopen", "reopen"),
    ("send-back", "reopen"),
    ("reply", "reply"),
    ("resume-any", "refresh"),
    ("try-again", "refresh"),
    ("resume", "refresh"),
    ("redeliver", "refresh"),
    ("read", "read"),
    ("unread", "unread"),
    ("snooze", "later"),
    ("unsnooze", "inbox"),
    ("move-refused", "move"),
    ("stop", "stop"),
    ("open", "open"),
    ("see-team", "team"),
    ("see-people", "add-person"),
    ("see-private", "lock"),
    ("pin", "pin"),
    ("rename", "rename"),
    ("summary", "text"),
    ("code", "code"),
    ("place", "cloud"),
    ("end-repeat", "stop-repeat"),
    ("add-people", "add-person"),
    ("shipped", "shipped"),
    ("team-list-shipped", "shipped"),
    ("decide", "flag"),
    ("wait-stop", "wait"),
];

/// And by the start of its id, the longest first, so `teammate-pause-` is read before
/// `teammate-` (`BY_PREFIX`).
const BY_PREFIX: &[(&str, &str)] = &[
    ("teammate-pause-", "pause"),
    ("teammate-", "person"),
    ("chat-person-", "person"),
    ("chat-", "hash"),
    ("thread-", "thread"),
    ("card-", "thread"),
    ("line-", "text"),
    ("files-", "file"),
    ("file-", "file"),
    ("comment-", "comment"),
    ("decision-", "flag"),
    ("team-list-", "flag"),
    ("priority-urgent", "priority-urgent"),
    ("priority-high", "priority-high"),
    ("priority-medium", "priority-medium"),
    ("priority-", "priority-low"),
    ("rank-", "top"),
    ("move-", "move"),
    ("give-", "give"),
    ("take-back", "reopen"),
    ("route-", "give"),
    ("model-", "spark"),
    ("group-", "hash"),
    ("cloud-", "cloud"),
    ("theme-", "theme"),
    ("wait-", "wait"),
];

/// Every name a row can wear, for the drawings to be checked against.
#[cfg(test)]
pub fn icon_names() -> Vec<&'static str> {
    BY_ID.iter().chain(BY_PREFIX).map(|(_, name)| *name).chain([PLAIN_ICON, "pause", "play"]).collect()
}

/// The line drawing at the head of a row, by the web's name for it (`iconFor`): one id for both
/// ways of a toggle, so the label says which way it goes.
pub fn icon_for(id: &str, label: &str) -> &'static str {
    let starts = |words: &str| label.get(..words.len()).is_some_and(|head| head.eq_ignore_ascii_case(words));
    if id == "pause" {
        return if starts("unpause") { "play" } else { "pause" };
    }
    if id.starts_with("teammate-pause-") && starts("resume") {
        return "play";
    }
    BY_ID.iter().find(|(of, _)| *of == id).or_else(|| BY_PREFIX.iter().find(|(start, _)| id.starts_with(start))).map_or(PLAIN_ICON, |(_, name)| name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::calendar::Day;
    use serde_json::{Value, json};

    fn item(v: Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    fn cmd(id: &str, label: &str, keywords: &'static str) -> Row {
        Row::command(id, label, Does::Home).found_by(keywords)
    }

    fn thread(id: &str, label: &str) -> Row {
        Row::command(&format!("thread-{id}"), label, Does::Open((id.to_string(), "shop".to_string())))
    }

    fn ids(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.id.as_str()).collect()
    }

    /// Rows a query keeps, ranked, as the bar does it.
    fn typed(query: &str, rows: &[Row]) -> Vec<String> {
        rank_matches(query, rows.iter().filter(|c| matches_query(query, c)).cloned().collect()).into_iter().map(|r| r.id).collect()
    }

    const NOW: f64 = 1_000_000_000.0;

    fn open_thread() -> Item {
        item(json!({ "id": "w-1", "product": "shop", "status": "open", "title": "Fix the export" }))
    }

    #[test]
    fn the_files_place_and_each_projects_page_are_found_by_the_words_for_files() {
        let rows = file_rows(&["Storefront".to_string(), "Mobile".to_string()]);
        assert_eq!(rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Go to Files", "Files in Storefront", "Files in Mobile"]);
        assert_eq!(rows[0].hint.as_deref(), Some("every file you can see"));
        assert_eq!(typed("pictures", &rows).len(), 3);
        assert_eq!(typed("files mobile", &rows), ["files-project-Mobile"]);
        assert_eq!(icon_for("files-all", "Go to Files"), "file");
        assert_eq!(icon_for("files-project-Mobile", "Files in Mobile"), "file");
        assert_eq!(rows[2].does, Does::Files(Some("Mobile".to_string())));
    }

    #[test]
    fn the_places_say_go_to_and_are_found_by_the_words_for_them() {
        let places = page_rows();
        assert_eq!(places.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Go to Home", "Go to Inbox", "Go to In progress", "Go to Later", "Go to Done"]);
        let inbox = &places[1];
        for q in ["go to", "go to inbox", "inbox", "needs you"] {
            assert!(matches_query(q, inbox), "{q}");
        }
        assert!(!matches_query("go to settings", inbox));
        // "home" finds Home first, though the Inbox answers to the word too.
        assert_eq!(typed("home", &places), ["view-home", "view-inbox"]);
        assert_eq!(typed("finished", &places), ["view-done"]);
        assert_eq!(places[3].does, Does::Tab(Tab::Later));
        // A row whose name starts with what was typed goes above one that matched inside a word.
        let mut rows = vec![cmd("wait-1", "Waits on Two logo rounds are ready to compare", "")];
        rows.extend(page_rows());
        assert_eq!(typed("go to", &rows)[..2], ["view-home", "view-inbox"]);
        assert_eq!(typed("go to", &rows).last().unwrap(), "wait-1");
        assert_eq!(typed("waits", &rows), ["wait-1"]);
    }

    #[test]
    fn the_thread_under_the_keys_gets_its_rows_in_the_webs_order() {
        let rows = thread_commands(&open_thread(), false, NOW);
        assert_eq!(ids(&rows), ["done", "reply", "snooze", "open"]);
        assert_eq!(rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Mark done", "Reply", "Later", "Open"]);
        assert_eq!(rows.iter().map(|r| r.key).collect::<Vec<_>>(), [Some("E"), Some("R"), Some("L"), Some("↵")]);
        assert_eq!(rows[0].does, Does::Done(("w-1".to_string(), "shop".to_string())));
        // The conversation on screen has no row to open.
        assert_eq!(ids(&thread_commands(&open_thread(), true, NOW)), ["done", "reply", "snooze"]);
        // A finished thread is reopened, not marked done again, and is found by the words for that.
        let finished = item(json!({ "id": "w-2", "product": "shop", "status": "done", "title": "Fix the export" }));
        let rows = thread_commands(&finished, true, NOW);
        assert_eq!(ids(&rows), ["reopen", "reply", "snooze"]);
        for q in ["reopen", "unmark done", "undo done", "not done", "bring back", "open again"] {
            assert!(matches_query(q, &rows[0]), "{q}");
        }
        assert!(!matches_query("inbox", &rows[0]));
    }

    #[test]
    fn a_thread_put_off_can_be_brought_back_and_an_agents_wait_let_go() {
        let put_off = item(json!({ "id": "w-3", "product": "shop", "status": "open", "title": "Fix the export", "snoozedUntil": NOW + 60_000.0,
            "wrote": { "snoozedUntil": { "ts": NOW - 5.0, "source": "founder" } } }));
        let rows = thread_commands(&put_off, false, NOW);
        assert_eq!(ids(&rows), ["done", "reply", "unsnooze", "snooze", "open"]);
        assert_eq!((rows[2].label.as_str(), rows[2].key), ("Back to Inbox", Some("E")));
        assert_eq!(rows[2].does, Does::Back(("w-3".to_string(), "shop".to_string())));
        // Once the moment has passed there is nothing to bring back.
        assert_eq!(ids(&thread_commands(&put_off, false, NOW + 120_000.0)), ["done", "reply", "snooze", "open"]);
        let parked = item(json!({ "id": "w-4", "product": "shop", "status": "open", "title": "Fix the export", "runAt": NOW + 60_000.0,
            "wrote": { "runAt": { "ts": NOW - 5.0, "source": "agent" } } }));
        assert_eq!(thread_commands(&parked, false, NOW)[2].label, "Let it run now");
    }

    #[test]
    fn the_whole_list_is_the_threads_rows_then_the_places() {
        let rows = commands(Some(&open_thread()), false, NOW, false);
        assert_eq!(ids(&rows), ["done", "reply", "snooze", "open", "view-home", "view-inbox", "view-progress", "view-later", "view-done"]);
        // With no thread under the keys, as on Home, only the places.
        assert_eq!(ids(&commands(None, false, NOW, false)), ["view-home", "view-inbox", "view-progress", "view-later", "view-done"]);
        // Starting a thread leads the list once this window can start one.
        let rows = commands(None, false, NOW, true);
        assert_eq!((rows[0].id.as_str(), rows[0].label.as_str(), rows[0].key), ("new-thread", "New thread", Some("N")));
        assert!(matches_query("compose", &rows[0]) && matches_query("new", &rows[0]));
    }

    /// The rows over an open thread in the web window, where two of them start with the same
    /// three letters.
    fn over_a_thread() -> Vec<Row> {
        vec![
            cmd("remote-control", "Remote control", "phone mobile continue rc"),
            cmd("open-terminal", "Open terminal", "shell command console"),
            cmd("done", "Mark done", ""),
            cmd("reply", "Reply", ""),
            cmd("read", "Mark read", ""),
            cmd("snooze", "Later", LATER_KEYWORDS),
        ]
    }

    #[test]
    fn typing_rem_puts_later_under_the_return_key() {
        let rows = over_a_thread();
        assert_eq!(typed("rem", &rows), ["snooze", "remote-control"]);
        assert_eq!(typed("REm", &rows), ["snooze", "remote-control"]);
        // One letter more still reaches the other row on its own.
        assert_eq!(typed("remo", &rows), ["remote-control"]);
        assert_eq!(typed("phone", &rows), ["remote-control"]);
        // Everything else keeps the order the list was put together in.
        assert_eq!(typed("e", &rows), ["snooze", "remote-control", "open-terminal", "done", "reply", "read"]);
        assert_eq!(typed("con", &rows), ["remote-control", "open-terminal"]);
        assert_eq!(typed("re", &rows), ["snooze", "remote-control", "reply", "read"]);
        assert!(typed("nothing here", &rows).is_empty());
        // Nothing typed is not sorted at all.
        assert_eq!(ids(&rank_matches("   ", rows.clone())), ids(&rows));
        assert_eq!(typed("later", &thread_commands(&open_thread(), false, NOW)), ["snooze"]);
    }

    #[test]
    fn a_word_that_only_touches_a_keyword_comes_after_the_threads_it_found() {
        let rename = cmd("rename", "Rename thread", "name title");
        // A command matched only by a keyword follows the threads, each under its heading.
        let listed = order(vec![rename.clone()], vec![thread("1", "A title for it")], "title");
        assert_eq!(ids(&listed.rows), ["thread-1", "rename"]);
        assert_eq!((listed.label_over(0), listed.label_over(1)), (Some("Threads"), Some("Commands")));
        // A command named by what was typed stays first, and Threads sits under it.
        let listed = order(vec![rename.clone()], vec![thread("1", "Rename the button")], "rename");
        assert_eq!(ids(&listed.rows), ["rename", "thread-1"]);
        assert_eq!((listed.label_over(0), listed.label_over(1)), (None, Some("Threads")));
        assert!(!listed.labels.iter().any(|(_, label)| *label == "Commands"));
        // With no threads found the command is still there, under no heading.
        let listed = order(vec![rename.clone()], Vec::new(), "title");
        assert_eq!(ids(&listed.rows), ["rename"]);
        assert!(listed.labels.is_empty());
        // Nothing typed is the list it always was.
        let listed = order(vec![rename, cmd("tutorial", "Take the tutorial", "")], Vec::new(), "");
        assert_eq!(ids(&listed.rows), ["rename", "tutorial"]);
        assert!(listed.labels.is_empty());
    }

    #[test]
    fn a_command_goes_ahead_only_when_the_words_start_words_of_its_name() {
        // "sum" is inside "resume" and "up" inside "interrupted": that is not its name.
        let resume = cmd("resume", "Resume interrupted agents", "");
        let task = thread("x", "Sum up this week's support mail");
        assert_eq!(ids(&order(vec![resume.clone()], vec![task.clone()], "Sum up").rows), ["thread-x", "resume"]);
        assert_eq!(order(vec![resume, cmd("snooze", "Later", "")], vec![task], "resume").rows[0].id, "resume");
        assert!(names_it("rem", "Remote control"));
        assert!(!names_it("sum up", "Resume interrupted agents"));
        assert!(!names_it("  ", "Later"));
        // A command whose own name has the words goes above threads named by them too.
        let (light, dark) = (cmd("look-light", "Theme: Light", "theme appearance"), cmd("look-dark", "Theme: Dark", "theme appearance"));
        let light_thread = thread("1", "Light theme keyboard shortcut visibility");
        assert_eq!(order(vec![light.clone()], vec![light_thread.clone(), thread("2", "Tray colour detection")], "light").rows[0].id, "look-light");
        assert_eq!(ids(&order(vec![light.clone(), dark], vec![light_thread.clone()], "theme").rows), ["look-light", "look-dark", "thread-1"]);
        // What the words only touch by keyword still goes after.
        assert_eq!(ids(&order(vec![cmd("look", "Appearance", "theme light dark")], vec![light_thread], "light").rows), ["thread-1", "look"]);
    }

    #[test]
    fn typing_a_threads_name_puts_that_thread_under_enter() {
        // A command that carries the whole title of a thread the words name stays under it.
        let waits = cmd("wait-w-9", "Waits on October investor update", "waits waiting blocked by depends needs");
        let investor = thread("w-9", "October investor update");
        let listed = order(vec![waits.clone()], vec![investor.clone()], "investor update");
        assert_eq!(ids(&listed.rows), ["thread-w-9", "wait-w-9"]);
        assert_eq!((listed.label_over(0), listed.label_over(1)), (Some("Threads"), Some("Commands")));
        assert_eq!(order(vec![waits.clone()], vec![investor.clone()], "October investor update").rows[0].id, "thread-w-9");
        assert_eq!(order(vec![waits.clone()], vec![investor.clone()], "invest").rows[0].id, "thread-w-9");
        // Typed by its own name, the command still comes first.
        assert_eq!(order(vec![waits.clone()], vec![investor.clone()], "waits on").rows[0].id, "wait-w-9");
        assert_eq!(order(vec![waits.clone()], vec![investor], "waits on october").rows[0].id, "wait-w-9");
        // And a found thread the words do not name leaves the command first.
        assert_eq!(order(vec![waits], vec![thread("3", "Board pack")], "investor update").rows[0].id, "wait-w-9");
    }

    #[test]
    fn a_command_found_by_a_name_it_had_goes_above_the_threads() {
        let later = thread_commands(&open_thread(), true, NOW).into_iter().find(|r| r.id == "snooze").unwrap();
        let remote = cmd("remote-control", "Remote control", "");
        let reminders = || vec![thread("1", "Remind me to cancel the card"), thread("2", "Reminder to write about testing")];
        let ordered = |q: &str, rows: Vec<Row>| listed(rows, reminders(), q);
        assert_eq!(ids(&ordered("remind me", vec![later.clone()]).rows), ["snooze", "thread-1", "thread-2"]);
        assert_eq!(ordered("snooze", vec![later.clone()]).rows[0].id, "snooze");
        assert_eq!(ids(&ordered("rem", vec![later.clone(), remote]).rows), ["snooze", "remote-control", "thread-1", "thread-2"]);
        let headed = ordered("remind me", vec![later.clone()]);
        assert_eq!(headed.label_over(1), Some("Threads"));
        assert!(!headed.labels.iter().any(|(_, label)| *label == "Commands"));
        // A keyword that was never its name still goes after, and so do letters from inside one.
        assert_eq!(ids(&ordered("defer", vec![later.clone()]).rows), ["thread-1", "thread-2", "snooze"]);
        assert_eq!(ids(&ordered("ind", vec![later]).rows), ["thread-1", "thread-2", "snooze"]);
    }

    #[test]
    fn the_thread_titled_by_the_words_comes_first() {
        let triage = thread("1", "Integration branch security and deployment triage");
        let passwords = thread("2", "Per-clinic staff passwords");
        let quality = thread("3", "Quality lead writes to the vet");
        assert_eq!(ids(&title_first(vec![triage.clone(), quality, passwords.clone()], "Per-clinic")), ["thread-2", "thread-1", "thread-3"]);
        // The exact title, then a title that starts with the words, then one that has them.
        let rows = vec![thread("a", "Staff passwords per-clinic follow-up"), thread("b", "Per-clinic staff passwords plan"), thread("c", "Per-clinic staff passwords")];
        assert_eq!(ids(&title_first(rows, "per-clinic staff passwords")), ["thread-c", "thread-b", "thread-a"]);
        // Rows that match alike keep the search's own order, and nothing typed moves nothing.
        assert_eq!(ids(&title_first(vec![thread("x", "Notes"), thread("y", "Notes")], "notes")), ["thread-x", "thread-y"]);
        assert_eq!(ids(&title_first(vec![triage, passwords], "  ")), ["thread-1", "thread-2"]);
    }

    #[test]
    fn the_pick_stays_on_its_row() {
        let keys = |ids: &[&str]| ids.iter().map(|k| k.to_string()).collect::<Vec<_>>();
        // When the rows sort again under it, the same row stays picked.
        assert_eq!(kept_pick(Some("c"), &keys(&["a", "b", "c"])), Some(2));
        // When its row is gone the first is picked, never a stale place.
        assert_eq!(kept_pick(Some("gone"), &keys(&["a", "b"])), Some(0));
        assert_eq!(kept_pick(None, &keys(&["a"])), Some(0));
        assert_eq!(kept_pick(Some("gone"), &[]), None);
        // Two rows that share an id still have a key each.
        assert_eq!(row_keys(&[cmd("done", "Mark done", ""), cmd("reply", "Reply", ""), cmd("done", "Done (2 selected)", "")]), ["done", "reply", "done#1"]);
    }

    fn storefront() -> Snapshot {
        let now = Day::real(2026, 10, 20).unwrap().at(12, 0);
        let min = 60_000.0;
        serde_json::from_value(json!({
            "products": [{ "slug": "shop", "name": "Storefront" }, { "slug": "app", "name": "Mobile" }],
            "items": [
                { "id": "w-1", "product": "shop", "status": "done", "title": "Checkout fails for some customers", "body": "Checkout fails for some customers",
                  "result": "Found it. Tax rounding broke on large carts.", "createdAt": now - 90.0 * min, "updatedAt": now - 4.0 * min },
                { "id": "w-2", "product": "app", "status": "open", "title": "Onboarding checklist copy", "body": "Onboarding checklist copy", "createdAt": now - 200.0 * min, "updatedAt": now - 180.0 * min },
                { "id": "w-3", "product": "gone", "status": "open", "title": "Release notes", "body": "Write them. Then check the links.", "createdAt": now - 200.0 * min,
                  "updatedAt": Day::real(2026, 10, 16).unwrap().at(9, 0) },
                { "id": "w-4", "product": "shop", "status": "open", "title": "Empty inbox drawing", "body": "Three directions.", "createdAt": now - 20_000.0 }
            ]
        }))
        .unwrap()
    }

    fn find(snap: &Snapshot, query: &str) -> Vec<Row> {
        let mut index = search::Index::default();
        index.read(&snap.items);
        threads_found(&index, snap, query, Day::real(2026, 10, 20).unwrap().at(12, 0))
    }

    #[test]
    fn a_found_thread_says_what_it_is_where_and_how_old() {
        let snap = storefront();
        let rows = find(&snap, "check");
        // Titles that have the word first, newest first; then the one that only mentions it.
        assert_eq!(ids(&rows), ["thread-shop-w-1", "thread-app-w-2", "thread-gone-w-3"]);
        let checkout = &rows[0];
        assert_eq!(checkout.label, "Checkout fails for some customers");
        assert_eq!(checkout.sub.as_deref(), Some("Checkout fails for some customers"));
        assert_eq!(checkout.hint.as_deref(), Some("Storefront · 4 min"));
        // Dimmed once finished, and only then.
        assert!(checkout.done && !rows[1].done);
        assert_eq!(checkout.does, Does::Open(("w-1".to_string(), "shop".to_string())));
        // A found thread wears the thread's drawing, whatever its project and id.
        assert_eq!(icon_for(&checkout.id, &checkout.label), "thread");
        assert_eq!(rows[1].hint.as_deref(), Some("Mobile · 3 hr"));
        // A day old or more it gives its date, and a project nobody named is its slug.
        assert_eq!(rows[2].hint.as_deref(), Some("gone · Nov 16"));
        assert_eq!(rows[2].sub.as_deref(), Some("Then check the links."));
        // Under a minute old it says now, from when it was made if it never changed.
        assert_eq!(find(&snap, "drawing")[0].hint.as_deref(), Some("Storefront · now"));
        assert_eq!(ago_short(0.0, 49.0 * 3_600_000.0), "2 d");
        // Nothing typed finds nothing: the bar then lists commands alone.
        assert!(find(&snap, "  ").is_empty());
    }

    #[test]
    fn the_best_few_threads_are_listed_and_the_one_titled_by_the_words_leads() {
        let items: Vec<Value> = (0..8).map(|n| json!({ "id": format!("w-{n}"), "product": "shop", "status": "open", "title": format!("Export run {n}"), "body": "Export it.", "updatedAt": n })).collect();
        let mut snap: Snapshot = serde_json::from_value(json!({ "products": [], "items": items })).unwrap();
        assert_eq!(ids(&find(&snap, "export")), ["thread-shop-w-7", "thread-shop-w-6", "thread-shop-w-5", "thread-shop-w-4", "thread-shop-w-3"]);
        assert_eq!(find(&snap, "export").len(), THREADS_SHOWN);
        // The thread whose title is the words goes first among the ones listed.
        snap.items[5].title = Some("Export".to_string());
        assert_eq!(ids(&find(&snap, "export"))[0], "thread-shop-w-5");
    }

    #[test]
    fn what_is_typed_decides_the_whole_list() {
        let snap = storefront();
        let over = snap.items[1].clone();
        let list = |query: &str| listed(commands(Some(&over), false, NOW, false), find(&snap, query), query);
        // Nothing typed: every command, no threads, no heading.
        let untyped = list("");
        assert_eq!(untyped.rows.len(), 9);
        assert!(untyped.labels.is_empty());
        // A word no command has: the threads alone, under their heading.
        let found = list("check");
        assert_eq!(ids(&found.rows), ["thread-shop-w-1", "thread-app-w-2", "thread-gone-w-3"]);
        assert_eq!(found.labels, [(0, "Threads")]);
        // A command's own name leads, and a thread that says the word follows it.
        let inbox = list("inbox");
        assert_eq!(ids(&inbox.rows), ["view-inbox", "thread-shop-w-4"]);
        assert_eq!(inbox.labels, [(1, "Threads")]);
        assert!(list("zebra").rows.is_empty());
    }

    #[test]
    fn nothing_found_says_what_was_searched_for() {
        assert_eq!(empty_line("  zebra "), "Nothing matches \u{201c}zebra\u{201d}.");
        // A pasted paragraph is cut at a word, not in the middle of one.
        assert_eq!(empty_line("the quick brown fox jumps over the lazy dog"), "Nothing matches \u{201c}the quick brown fox jumps over…\u{201d}.");
        assert_eq!(empty_line(&"x".repeat(40)), format!("Nothing matches \u{201c}{}…\u{201d}.", "x".repeat(32)));
    }

    #[test]
    fn every_row_this_window_draws_has_the_webs_drawing() {
        for row in commands(Some(&open_thread()), false, NOW, true).iter().chain(&page_rows()).chain(&file_rows(&["Mobile".to_string()])) {
            assert_ne!(icon_for(&row.id, &row.label), PLAIN_ICON, "{}", row.id);
        }
        assert_eq!(icon_for("new-thread", "New thread"), "compose");
        assert_eq!(icon_for("reopen", "Reopen"), "reopen");
        assert_eq!(icon_for("unsnooze", "Back to Inbox"), "inbox");
        assert_eq!(icon_for("thread-shop-w-1", "Fix the export"), "thread");
        assert_eq!(icon_for("something-else", "Something else"), PLAIN_ICON);
        // The longest start wins, and a toggle's label says which way it goes.
        assert_eq!(icon_for("teammate-pause-ada", "Pause Ada"), "pause");
        assert_eq!(icon_for("teammate-pause-ada", "Resume Ada"), "play");
        assert_eq!(icon_for("teammate-ada", "Ada"), "person");
        assert_eq!(icon_for("pause", "Unpause agents"), "play");
        assert_eq!(icon_for("priority-urgent", "Urgent"), "priority-urgent");
        assert_eq!(icon_for("priority-low", "Low"), "priority-low");
    }
}
