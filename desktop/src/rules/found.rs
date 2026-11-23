//! What the command bar's words find inside threads: the lines said in them and the files they
//! made, as the app's finder answers them (`zero:find-in-threads`, `main/everything-found.mts`),
//! turned into the bar's rows (`insideGroups` in `renderer/src/found-inside.ts`, with
//! `insideForPalette` and `openInside` in `renderer/src/App.tsx`), and the looking for a found
//! line once its thread opens (`revealLine` in `renderer/src/reveal-line.ts`).
//!
//! Comments on files, lines said in groups, decisions and teammates' shared threads are left
//! out: this window draws no comments, and has no groups and no teammates
//! (`docs/parity-command-bar.md`, "Found inside things").

use serde::Deserialize;
use serde_json::{Map, Value, json};

use super::compose::is_direct;
use super::{js, kinds, search, title};
use crate::data::{Item, Snapshot, truthy};

/// A thread, as `(id, product)`.
pub type Thread = (String, String);

/// How long the words rest before the app is asked (`setTimeout(…, 90)` in `PaletteBox`).
pub const ASK_AFTER_MS: u64 = 90;

/// The channel asked, and the one that reads the threads ahead as the bar opens.
pub const FIND: &str = "zero:find-in-threads";
pub const WARM: &str = "zero:warm-find-in-threads";

/// How many of each the bar keeps (`insideFor(q, { lines: 3, files: 3, comments: 2, … })`).
pub const LINES_KEPT: usize = 3;
pub const FILES_KEPT: usize = 3;
const COMMENTS_KEPT: usize = 2;

/// What is asked for the words, or nothing for words that are blank once trimmed: the query
/// trimmed, and a few more of each than are kept, as the bar asks (`limit.lines + 3`, …).
pub fn ask(words: &str) -> Option<Value> {
    let query = js::trim(words);
    (!query.is_empty()).then(|| json!({ "query": query, "limit": { "lines": LINES_KEPT + 3, "files": FILES_KEPT + 2, "comments": COMMENTS_KEPT + 2 } }))
}

/// Where a found thing sits (`FoundWhere`): a thread of the person's own, a conversation with
/// people, or a teammate's shared thread.
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Where {
    pub kind: String,
    pub product: Option<String>,
    pub id: Option<String>,
    pub owner_id: Option<String>,
    pub thread_id: Option<String>,
}

impl Where {
    /// `${product ?? ownerId}-${id ?? threadId}`, as a template string writes a missing one.
    fn named(&self) -> String {
        let or_null = |a: &Option<String>, b: &Option<String>| a.clone().or_else(|| b.clone()).unwrap_or_else(|| "null".to_string());
        format!("{}-{}-{}", self.kind, or_null(&self.product, &self.owner_id), or_null(&self.id, &self.thread_id))
    }
}

/// A line found in a message (`FoundLine`).
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct FoundLine {
    #[serde(flatten)]
    pub at_where: Where,
    pub uid: String,
    pub by: Option<String>,
    pub source: String,
    pub at: f64,
    pub comment: bool,
    pub text: String,
}

/// A file found by its name or by what it holds (`FoundFile`).
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct FoundFile {
    #[serde(flatten)]
    pub at_where: Where,
    pub how: String,
    pub src: String,
    pub name: String,
    pub by: Option<String>,
    pub at: f64,
    pub text: String,
}

/// What the finder answers (`FoundInThreads`). Its comments are not read here.
#[derive(Deserialize, Default, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Answer {
    pub lines: Vec<FoundLine>,
    pub files: Vec<FoundFile>,
}

impl Answer {
    /// An answer as it came back; one the window cannot read finds nothing.
    pub fn read(v: &Value) -> Answer {
        serde_json::from_value(v.clone()).unwrap_or_default()
    }
}

/// Where pressing a row goes (`InsideOpen`, for a thread of the person's own).
#[derive(Clone, Debug, PartialEq)]
pub enum Open {
    /// The thread, landing on the message whose line has this uid.
    Line { thread: Thread, uid: String },
    /// The thread with this file open: beside it, or big when it is an outside agent's row or
    /// a file that does not open beside (`outside` is the former).
    File { thread: Thread, src: String, outside: bool },
}

/// One found thing as the bar draws it (`InsideRow`).
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub id: String,
    /// The matching line, or a file's name.
    pub text: String,
    /// Only for a file found by what it holds: its name, drawn before the line.
    pub file: Option<String>,
    pub who: String,
    pub at: f64,
    /// The thread's name as the list draws it.
    pub place: String,
    pub open: Open,
}

/// The found things of one kind, under their heading (`InsideGroup`).
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub label: &'static str,
    pub rows: Vec<Found>,
}

fn part<'a>(snap: &'a Snapshot, key: &str) -> Option<&'a Map<String, Value>> {
    snap.rest.get(key).and_then(Value::as_object)
}

fn people(snap: &Snapshot) -> &[Value] {
    part(snap, "team").and_then(|t| t.get("people")).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

/// The person, by their id on a team the window shows (`team?.me` from `teamView`: signed in,
/// with Multiplayer not switched off).
fn me(snap: &Snapshot) -> Option<&str> {
    let team = part(snap, "team")?;
    let multiplayer = part(snap, "alpha").and_then(|a| a.get("multiplayer")) != Some(&Value::Bool(false));
    if !multiplayer || !truthy(team.get("signedIn")) {
        return None;
    }
    team.get("me").and_then(|m| m.get("id")).and_then(Value::as_str).filter(|id| !id.is_empty())
}

/// A teammate's first name, "Someone" when the window does not know them (`firstName` in
/// `renderer/src/team/company.ts`, of `team.byId`, which is empty without a team view).
fn first_name(snap: &Snapshot, id: &str) -> String {
    let person = me(snap).and_then(|_| people(snap).iter().find(|p| p.get("id").and_then(Value::as_str) == Some(id)));
    let said = |field: &str| person.and_then(|p| p.get(field)).and_then(Value::as_str).filter(|s| !s.is_empty());
    said("name").or_else(|| said("email")).unwrap_or("Someone").split_whitespace().next().unwrap_or("Someone").to_string()
}

/// Who wrote it (`who` in `insideGroups`): an agent's line is "Agent", whoever's account its
/// record names (a run on a signed-in account is filed under that person's id); of the rest
/// the person is "You", a teammate their first name, and one with no one named the person's.
fn who(snap: &Snapshot, by: Option<&str>, source: &str) -> String {
    if source == "agent" {
        return "Agent".to_string();
    }
    match by {
        Some(by) if me(snap) == Some(by) => "You".to_string(),
        Some(by) => first_name(snap, by),
        None => "You".to_string(),
    }
}

/// A thread's name as the list draws it (`threadSearchDetails(…).title` in
/// `renderer/src/threads/search.ts`): a conversation with people that is not a task is called
/// by their names, else "Conversation"; any other thread by its shown title. The people are
/// named in the order the conversation's record lists them.
pub fn title_of(snap: &Snapshot, item: &Item) -> String {
    let product = snap.products.iter().find(|p| p.slug == item.product);
    if !product.is_some_and(is_direct) || kinds::is_task_row(item) {
        return title::shown_title(item);
    }
    let mine = me(snap);
    let team = product.and_then(|p| p.rest.get("team"));
    let ids = |v: Option<&Value>| v.and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>()).unwrap_or_default();
    let mut everyone = ids(team.and_then(|t| t.get("people")));
    everyone.extend(team.and_then(|t| t.get("sharedBy")).and_then(Value::as_str).map(str::to_string));
    everyone.extend(ids(item.rest.get("people")));
    let mut them: Vec<String> = Vec::new();
    for id in everyone {
        if !id.is_empty() && Some(id.as_str()) != mine && !them.contains(&id) {
            them.push(id);
        }
    }
    if them.is_empty()
        && let Some(by) = item.rest.get("createdBy").and_then(Value::as_str).filter(|by| !by.is_empty() && Some(*by) != mine)
    {
        them.push(by.to_string());
    }
    let names: Vec<&str> = them.iter().filter_map(|id| people(snap).iter().find(|p| p.get("id").and_then(Value::as_str) == Some(id.as_str())).and_then(|p| p.get("name")).and_then(Value::as_str)).filter(|n| !n.is_empty()).collect();
    if names.is_empty() { "Conversation".to_string() } else { names.join(", ") }
}

/// Whether the typed words name anything to look for (`queryOf(query).terms.length`): quotes
/// alone do not.
fn has_terms(query: &str) -> bool {
    js::words(&search::fold(query).replace('"', " ")).iter().any(|w| !w.is_empty())
}

/// The rows the bar draws from what the finder answered for `query`, each kept only when the
/// window lists its thread, at most three lines newest first and three files in the finder's
/// order, Lines then Files, a group with none left out (`insideGroups`).
pub fn groups(query: &str, found: &Answer, snap: &Snapshot) -> Vec<Group> {
    if !has_terms(query) {
        return Vec::new();
    }
    // Where a found thing sits, or nothing when this window cannot open it.
    let place = |w: &Where| -> Option<&Item> {
        if w.kind == "shared" {
            return None;
        }
        let (product, id) = (w.product.as_deref()?, w.id.as_deref()?);
        snap.items.iter().find(|i| i.product == product && i.id == id)
    };
    let thread = |item: &Item| (item.id.clone(), item.product.clone());
    let mut lines: Vec<Found> = found
        .lines
        .iter()
        .filter_map(|l| {
            let item = place(&l.at_where)?;
            let by = who(snap, l.by.as_deref(), &l.source);
            Some(Found {
                id: format!("line-{}-{}", l.at_where.named(), l.uid),
                text: l.text.clone(),
                file: None,
                who: if l.comment { format!("{by} commented") } else { by },
                at: l.at,
                place: title_of(snap, item),
                open: Open::Line { thread: thread(item), uid: l.uid.clone() },
            })
        })
        .collect();
    // Newest first, the finder's order kept between lines said at the same moment.
    lines.sort_by(|a, b| b.at.total_cmp(&a.at));
    lines.truncate(LINES_KEPT);
    let files: Vec<Found> = found
        .files
        .iter()
        .filter_map(|f| {
            let item = place(&f.at_where)?;
            let by_name = f.how == "name";
            Some(Found {
                id: format!("file-{}-{}", f.at_where.named(), f.src),
                text: if by_name { f.name.clone() } else { f.text.clone() },
                file: (f.how == "contents").then(|| f.name.clone()),
                who: f.by.as_deref().map_or_else(String::new, |by| who(snap, Some(by), "founder")),
                at: f.at,
                place: title_of(snap, item),
                open: Open::File { thread: thread(item), src: f.src.clone(), outside: item.truthy("agent") },
            })
        })
        .take(FILES_KEPT)
        .collect();
    [("Lines", lines), ("Files", files)].into_iter().filter(|(_, rows)| !rows.is_empty()).map(|(label, rows)| Group { label, rows }).collect()
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// How long ago, tersely (`ago` in `renderer/src/format.ts`): seconds, minutes, hours and days
/// under a week, then the month and day. The web window writes that date in the system's
/// language, and this writes it in English, as the rest of the window speaks.
pub fn ago(at: f64, now: f64) -> String {
    let s = ((now - at) / 1000.0).floor().max(0.0);
    if s < 60.0 {
        return format!("{s}s");
    }
    let m = (s / 60.0).floor();
    if m < 60.0 {
        return format!("{m}m");
    }
    let h = (m / 60.0).floor();
    if h < 24.0 {
        return format!("{h}h");
    }
    let d = (h / 24.0).floor();
    if d < 7.0 {
        return format!("{d}d");
    }
    let day = rune::time::LocalTime::at(at as i64);
    format!("{} {}", MONTHS[(day.month as usize).clamp(1, 12) - 1], day.day)
}

/// The line under a found row: who, where and when, any of them left out when empty, and where
/// left out when it is who (`palette-sub` in `PaletteBox`).
pub fn under(row: &Found, now: f64) -> String {
    let place = if row.place == row.who { "" } else { row.place.as_str() };
    let when = if row.at != 0.0 { ago(row.at, now) } else { String::new() };
    [row.who.as_str(), place, when.as_str()].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · ")
}

/* ---------------------------------- landing on a line ---------------------------------- */

/// How often the window looks for the message, how many times after the first, after how many
/// it asks for the whole conversation, and how long the message stays marked (`EVERY_MS`,
/// `TRIES`, `ASK_AFTER`, `MARKED_MS`).
pub const EVERY_MS: f64 = 50.0;
pub const TRIES: u32 = 60;
pub const ASK_AFTER: u32 = 6;
pub const MARKED_MS: f64 = 2600.0;
/// How long the mark takes to come and to go (`transition: background-color .5s ease`).
pub const MARK_FADE_MS: f64 = 500.0;

/// What one look found to do (`look` in `revealLine`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Look {
    /// It is drawn: bring it to the middle and mark it.
    Found,
    /// Not yet; look again in 50 ms.
    Again,
    /// Not yet, and the conversation is asked, once, to draw itself whole; look again in 50 ms.
    AskWhole,
    /// Never drawn: stop, saying nothing.
    GiveUp,
}

/// The looking for one message: how many looks it has had and whether the whole conversation
/// was asked for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Looking {
    pub n: u32,
    pub asked: bool,
}

impl Looking {
    /// One look, `drawn` saying whether the message is on the page now.
    pub fn look(&mut self, drawn: bool) -> Look {
        if drawn {
            return Look::Found;
        }
        let n = self.n;
        if n >= TRIES {
            return Look::GiveUp;
        }
        self.n += 1;
        if n >= ASK_AFTER && !self.asked {
            self.asked = true;
            return Look::AskWhole;
        }
        Look::Again
    }
}

/// What a message is looked for by: its line's uid, or the moment it was said for a line with
/// none (`e.uid ?? \`at-${e.at}\`` in `Thread.tsx`, `l.uid ?? \`at-${at}\`` in the finder).
pub fn line_name(uid: Option<&str>, at: f64) -> String {
    match uid {
        Some(uid) => uid.to_string(),
        None => format!("at-{}", js_number(at)),
    }
}

/// A number as a template string writes it: a whole one with no decimal point.
fn js_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e21 { format!("{}", n as i64) } else { format!("{n}") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn snap(v: Value) -> Snapshot {
        serde_json::from_value(v).unwrap()
    }

    /// The web's test inbox (`search-rows-say-who-wrote-it-where-and-open-at-the-message.test.mts`),
    /// as this window's snapshot: a thread, and a conversation with two people on a team.
    fn inbox() -> Snapshot {
        snap(json!({
            "products": [{ "slug": "lumen", "name": "Lumen" }, { "slug": "people", "name": "People", "team": { "direct": true, "people": ["p-sofia", "p-dev"] } }],
            "items": [
                { "id": "t1", "product": "lumen", "status": "open", "title": "Launch week" },
                { "id": "c1", "product": "people", "status": "open", "title": "Conversation" },
                { "id": "a1", "product": "lumen", "status": "open", "title": "An agent of its own", "agent": { "pid": 7 } },
            ],
            "team": { "signedIn": true, "me": { "id": "p-maya", "name": "Maya Ito" }, "people": [
                { "id": "p-dev", "name": "Dev Rao" }, { "id": "p-maya", "name": "Maya Ito" }, { "id": "p-sofia", "name": "Sofia Lind" },
            ] },
            "supervisor": {},
        }))
    }

    fn line(v: Value) -> FoundLine {
        let base = json!({ "kind": "thread", "product": "lumen", "id": "t1", "ownerId": null, "threadId": null, "updatedAt": 1, "source": "founder", "comment": false });
        let mut base = base.as_object().unwrap().clone();
        base.extend(v.as_object().unwrap().clone());
        serde_json::from_value(Value::Object(base)).unwrap()
    }

    fn file(v: Value) -> FoundFile {
        let base = json!({ "kind": "thread", "product": "lumen", "id": "t1", "ownerId": null, "threadId": null, "updatedAt": 1 });
        let mut base = base.as_object().unwrap().clone();
        base.extend(v.as_object().unwrap().clone());
        serde_json::from_value(Value::Object(base)).unwrap()
    }

    fn lines_of(groups: &[Group]) -> Vec<Found> {
        groups.iter().find(|g| g.label == "Lines").map(|g| g.rows.clone()).unwrap_or_default()
    }

    fn files_of(groups: &[Group]) -> Vec<Found> {
        groups.iter().find(|g| g.label == "Files").map(|g| g.rows.clone()).unwrap_or_default()
    }

    #[test]
    fn the_app_is_asked_for_the_trimmed_words_with_a_few_to_spare_and_never_for_blank_ones() {
        assert_eq!(ask("  render  "), Some(json!({ "query": "render", "limit": { "lines": 6, "files": 5, "comments": 4 } })));
        assert_eq!(ask("   "), None);
        assert_eq!(ask(""), None);
    }

    #[test]
    fn a_line_says_who_wrote_it_where_and_when_and_opens_at_that_message() {
        let found = Answer {
            lines: vec![
                line(json!({ "uid": "u1", "by": "p-dev", "at": 300, "text": "Render account is live." })),
                line(json!({ "uid": "u2", "by": null, "source": "agent", "at": 200, "text": "Render deploy finished." })),
                line(json!({ "kind": "chat", "product": "people", "id": "c1", "uid": "u3", "by": "p-sofia", "at": 100, "text": "Render check for the amber state." })),
            ],
            files: vec![],
        };
        let rows = lines_of(&groups("render", &found, &inbox()));
        assert_eq!((rows[0].who.as_str(), rows[0].at, rows[0].place.as_str(), rows[0].text.as_str()), ("Dev", 300.0, "Launch week", "Render account is live."));
        assert_eq!(rows[0].open, Open::Line { thread: ("t1".into(), "lumen".into()), uid: "u1".into() });
        assert_eq!(rows[0].id, "line-thread-lumen-t1-u1");
        // "Agent" for the agent, and a conversation named by its people, not "Conversation".
        assert_eq!(rows[1].who, "Agent");
        assert_eq!(rows[2].place, "Sofia Lind, Dev Rao");
        assert_eq!(rows[2].id, "line-chat-people-c1-u3");
    }

    #[test]
    fn the_person_is_you_whether_signed_in_or_not() {
        let mine = Answer { lines: vec![line(json!({ "uid": "u1", "by": "p-maya", "at": 1, "text": "render" }))], files: vec![] };
        assert_eq!(lines_of(&groups("render", &mine, &inbox()))[0].who, "You");
        // Alone, with no one signed in: a line of theirs carries no id.
        let alone = snap(json!({ "products": [{ "slug": "lumen", "name": "Lumen" }], "items": [{ "id": "t1", "product": "lumen", "status": "open", "title": "Launch week" }], "supervisor": {} }));
        let theirs = Answer { lines: vec![line(json!({ "uid": "u1", "by": null, "at": 1, "text": "render" }))], files: vec![] };
        assert_eq!(lines_of(&groups("render", &theirs, &alone))[0].who, "You");
        // An agent's line is the agent's, whoever's account its record names: a run on a
        // signed-in account is filed under that person's id, their own or a teammate's.
        for by in ["p-maya", "p-dev"] {
            let filed = Answer { lines: vec![line(json!({ "uid": "u1", "by": by, "source": "agent", "at": 1, "text": "render" }))], files: vec![] };
            assert_eq!(lines_of(&groups("render", &filed, &inbox()))[0].who, "Agent", "{by}");
        }
        let noted = Answer { lines: vec![line(json!({ "uid": "u1", "by": "p-maya", "source": "agent", "comment": true, "at": 1, "text": "render" }))], files: vec![] };
        assert_eq!(lines_of(&groups("render", &noted, &inbox()))[0].who, "Agent commented");
        // A name the window cannot put to anyone, and a comment's line.
        let stranger = Answer { lines: vec![line(json!({ "uid": "u1", "by": "p-gone", "at": 1, "comment": true, "text": "render" }))], files: vec![] };
        assert_eq!(lines_of(&groups("render", &stranger, &alone))[0].who, "Someone commented");
    }

    #[test]
    fn what_sits_in_a_thread_not_listed_here_or_a_teammates_is_left_out() {
        let lost = Answer { lines: vec![line(json!({ "id": "gone", "uid": "x", "by": "p-dev", "at": 1, "text": "render" }))], files: vec![] };
        assert_eq!(groups("render", &lost, &inbox()), vec![]);
        let shared = Answer { lines: vec![line(json!({ "kind": "shared", "product": null, "id": null, "ownerId": "p-dev", "threadId": "s1", "uid": "u9", "by": "p-dev", "at": 50, "text": "Render status posted." }))], files: vec![] };
        assert_eq!(groups("render", &shared, &inbox()), vec![]);
    }

    #[test]
    fn a_file_is_shown_by_its_name_or_by_the_line_that_matched_after_its_name() {
        let found = Answer {
            lines: vec![],
            files: vec![
                file(json!({ "how": "name", "src": "render-plan.md", "name": "render-plan.md", "by": "p-dev", "at": 10, "text": "" })),
                file(json!({ "how": "contents", "src": "notes.md", "name": "notes.md", "by": null, "at": 20, "text": "The render account goes live Tuesday." })),
                file(json!({ "id": "a1", "how": "name", "src": "out/render.png", "name": "render.png", "by": null, "at": 30, "text": "" })),
            ],
        };
        let rows = files_of(&groups("render", &found, &inbox()));
        // In the finder's order, not by age.
        assert_eq!((rows[0].text.as_str(), rows[0].file.as_deref(), rows[0].who.as_str()), ("render-plan.md", None, "Dev"));
        assert_eq!(rows[0].open, Open::File { thread: ("t1".into(), "lumen".into()), src: "render-plan.md".into(), outside: false });
        assert_eq!(rows[0].id, "file-thread-lumen-t1-render-plan.md");
        assert_eq!((rows[1].text.as_str(), rows[1].file.as_deref(), rows[1].who.as_str()), ("The render account goes live Tuesday.", Some("notes.md"), ""));
        // An outside agent's row opens its file big.
        assert_eq!(rows[2].open, Open::File { thread: ("a1".into(), "lumen".into()), src: "out/render.png".into(), outside: true });
    }

    #[test]
    fn each_group_keeps_to_its_limit_lines_newest_first_and_nothing_is_listed_for_blank_words() {
        let many = Answer {
            lines: (0..9).map(|i| line(json!({ "uid": format!("u{i}"), "by": "p-dev", "at": i, "text": "Render it." }))).collect(),
            files: (0..5).map(|i| file(json!({ "how": "name", "src": format!("f{i}.md"), "name": format!("f{i}.md"), "by": null, "at": 9 - i, "text": "" }))).collect(),
        };
        let got = groups("render", &many, &inbox());
        assert_eq!(got.iter().map(|g| g.label).collect::<Vec<_>>(), ["Lines", "Files"]);
        assert_eq!(lines_of(&got).iter().map(|r| r.at).collect::<Vec<_>>(), [8.0, 7.0, 6.0]);
        assert_eq!(files_of(&got).iter().map(|r| r.text.as_str()).collect::<Vec<_>>(), ["f0.md", "f1.md", "f2.md"]);
        assert_eq!(groups("   ", &many, &inbox()), vec![]);
        assert_eq!(groups("\"\"", &many, &inbox()), vec![]);
        // With files only, the Lines heading is not drawn.
        let files = Answer { lines: vec![], files: many.files.clone() };
        assert_eq!(groups("render", &files, &inbox()).iter().map(|g| g.label).collect::<Vec<_>>(), ["Files"]);
    }

    #[test]
    fn an_answer_is_read_as_it_came_and_one_the_app_could_not_make_finds_nothing() {
        let came = json!({ "ok": true, "lines": [{ "kind": "thread", "product": "lumen", "id": "t1", "ownerId": null, "threadId": null, "updatedAt": 1, "uid": "u1", "by": null, "source": "agent", "at": 5, "comment": false, "text": "Render." }], "files": [], "comments": [], "total": { "lines": 1, "files": 0, "comments": 0 } });
        assert_eq!(Answer::read(&came).lines.len(), 1);
        let refused = json!({ "ok": false, "error": "no", "lines": [], "files": [], "comments": [], "total": { "lines": 0, "files": 0, "comments": 0 } });
        assert_eq!(groups("render", &Answer::read(&refused), &inbox()), vec![]);
        assert_eq!(Answer::read(&json!("nonsense")), Answer::default());
    }

    #[test]
    fn the_line_under_says_who_where_and_when_leaving_out_what_is_empty_or_said_twice() {
        let row = |who: &str, place: &str, at: f64| Found { id: String::new(), text: String::new(), file: None, who: who.into(), at, place: place.into(), open: Open::Line { thread: (String::new(), String::new()), uid: String::new() } };
        let now = 1_700_000_000_000.0;
        assert_eq!(under(&row("Agent", "Launch week", now - 5.0 * 60_000.0), now), "Agent · Launch week · 5m");
        assert_eq!(under(&row("", "Launch week", now - 30_000.0), now), "Launch week · 30s");
        assert_eq!(under(&row("Dev", "Dev", 0.0), now), "Dev");
        assert_eq!(under(&row("You", "Launch week", 0.0), now), "You · Launch week");
    }

    #[test]
    fn ago_is_seconds_minutes_hours_days_then_the_date() {
        let now = 1_700_000_000_000.0;
        assert_eq!(ago(now, now), "0s");
        assert_eq!(ago(now + 5_000.0, now), "0s");
        assert_eq!(ago(now - 59_999.0, now), "59s");
        assert_eq!(ago(now - 60_000.0, now), "1m");
        assert_eq!(ago(now - 3_600_000.0, now), "1h");
        assert_eq!(ago(now - 6.9 * 86_400_000.0, now), "6d");
        let long_ago = now - 30.0 * 86_400_000.0;
        let day = rune::time::LocalTime::at(long_ago as i64);
        assert_eq!(ago(long_ago, now), format!("{} {}", MONTHS[day.month as usize - 1], day.day));
    }

    /// `a-found-line-is-scrolled-to-and-marked-when-its-thread-opens.test.mts`, on the looks alone.
    #[test]
    fn a_found_line_is_looked_for_every_fifty_ms_the_whole_conversation_asked_for_once_then_given_up_quietly() {
        // Drawn at once: found on the first look.
        assert_eq!(Looking::default().look(true), Look::Found);
        // Drawn a moment later: found on the fourth look, nothing asked for on the way.
        let mut l = Looking::default();
        let looks: Vec<Look> = (0..4).map(|n| l.look(n == 3)).collect();
        assert_eq!(looks, [Look::Again, Look::Again, Look::Again, Look::Found]);
        // Not drawn after six looks (300 ms): the whole conversation is asked for, once, and the
        // looking goes on until it is found.
        let mut l = Looking::default();
        let looks: Vec<Look> = (0..13).map(|n| l.look(n == 12)).collect();
        assert_eq!(looks.iter().filter(|x| **x == Look::AskWhole).count(), 1);
        assert_eq!(looks[6], Look::AskWhole);
        assert_eq!(looks[12], Look::Found);
        // Never drawn: sixty looks after the first, about three seconds, then nothing more.
        let mut l = Looking::default();
        let mut looks = 0;
        while l.look(false) != Look::GiveUp {
            looks += 1;
        }
        assert_eq!(looks, TRIES as usize);
        assert_eq!(f64::from(TRIES) * EVERY_MS, 3000.0);
        assert!(l.asked);
    }

    #[test]
    fn a_message_is_looked_for_by_its_lines_uid_or_by_when_it_was_said() {
        assert_eq!(line_name(Some("u7"), 5.0), "u7");
        assert_eq!(line_name(None, 1_700_000_000_000.0), "at-1700000000000");
        assert_eq!(line_name(None, 1.5), "at-1.5");
    }
}
