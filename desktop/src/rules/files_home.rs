//! Files' home (`renderer/src/files/files-home.ts`): what the team kept, folder by folder, and
//! apart from it what threads made that nobody filed; Recent, a day at a time; the path back
//! from a folder's page; what a tile says and which mark it wears. With a change drawn over the
//! list at once (`files-changes.ts`), what Z puts back (`files-undo.ts`), and the few rules of
//! `FilesPlace.tsx` and `FilesHome.tsx` that decide what is offered: the project tiles, New's
//! menu, the menus' place in the window. Ported with the web's own test cases.

use serde_json::{Value, json};

use super::calendar::Day;
use super::files_shown::{self as shown, Folder, List, Place, Placed, Sort};

/// The page of a project's files in no folder, where a folder's page would be (`NOT_IN_A_FOLDER`).
pub const NOT_IN_A_FOLDER: &str = "not-in-a-folder";
/// Two rows of three, beside Recent, before a section's own page (`SHOWN_AT_FIRST`).
pub const SHOWN_AT_FIRST: usize = 6;
/// How many files Recent lists (`RECENT_MOST`).
pub const RECENT_MOST: usize = 10;
/// How many tiles a folder's page draws at first, and how many more Show more adds (`Shelf`).
pub const SHELF_STEP: usize = 60;
/// How many matches the search's grid draws (`limit={60}`).
pub const FOUND_MOST: usize = 60;

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// Today, Yesterday, or the day: what a tile says of when its file last changed (`whenWords`).
pub fn when_words(at: f64, now: f64) -> String {
    let days = Day::of(now).0 - Day::of(at).0;
    if days <= 0 {
        return "Today".to_string();
    }
    if days == 1 {
        return "Yesterday".to_string();
    }
    let (_, month, day) = Day::of(at).ymd();
    format!("{} {day}", MONTHS[month.clamp(0, 11) as usize])
}

/// A folder called Pinned, whatever its capitals (`isPinned`).
pub fn is_pinned(name: &str) -> bool {
    shown::fold(name.trim()) == "pinned"
}

/// A pinned document that changed this week: the one thing on the page in the accent
/// (`pinChangedLately`).
pub fn pin_changed_lately(folder: &str, at: f64, now: f64) -> bool {
    is_pinned(folder) && now - at < 7.0 * 86_400_000.0
}

/// One crumb of the path back: its words and the place it opens (`Crumb`).
#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub name: String,
    pub project: Option<String>,
    pub folder: Option<String>,
}

/// The places above this one, nearest last (`crumbsFor`).
pub fn crumbs_for(folders: &[Folder], project: Option<&str>, folder: Option<&str>) -> Vec<Crumb> {
    let Some(project) = project else { return Vec::new() };
    let top = Crumb { name: "All files".to_string(), project: None, folder: None };
    let Some(folder) = folder else { return vec![top] };
    let here = folders.iter().find(|f| f.id == folder);
    let above = here.and_then(|h| h.parent.as_ref()).and_then(|p| folders.iter().find(|f| &f.id == p));
    let mut out = vec![top, Crumb { name: project.to_string(), project: Some(project.to_string()), folder: None }];
    if let Some(above) = above {
        out.push(Crumb { name: above.name.clone(), project: Some(project.to_string()), folder: Some(above.id.clone()) });
    }
    out
}

/// A folder's page: everything in it and in the folders inside it, newest first (`folderFiles`).
pub fn folder_files<'a>(files: &'a [Placed], folders: &[Folder], project: &str, folder: &str) -> Vec<&'a Placed> {
    let below = shown::files_below(files, folders, &Place { project: Some(project.to_string()), folder: Some(folder.to_string()) });
    shown::sort_files(&shown::apart_scratch(below).0, Sort::Recent)
}

/// One folder's section of a home: the folder at the top and every file under it, newest first.
#[derive(Clone, Debug, PartialEq)]
pub struct Section<'a> {
    pub folder: &'a Folder,
    pub files: Vec<&'a Placed>,
}

/// A project's home (`Home`): its folders with files in them, Pinned first then the one that
/// changed last; the top folders with nothing in them, by name; the files in no folder.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Home<'a> {
    pub kept: Vec<Section<'a>>,
    pub empty: Vec<&'a Folder>,
    pub loose: Vec<&'a Placed>,
}

fn same(a: &str, b: &str) -> bool {
    shown::fold(a) == shown::fold(b)
}

/// What a project kept, folder by folder, and what nobody filed (`homeOf`). A file in a folder
/// that is gone counts as in none; a subfolder's files are in its top folder's section; an
/// agent's check picture is no part of it.
pub fn home_of<'a>(files: &'a [Placed], folders: &'a [Folder], project: &str) -> Home<'a> {
    let here: Vec<&Folder> = folders.iter().filter(|f| same(&f.project, project)).collect();
    let known = |id: &str| here.iter().find(|f| f.id == id).copied();
    let top_of = |id: &str| -> String {
        match known(id).and_then(|f| f.parent.as_deref()) {
            Some(parent) if known(parent).is_some() => parent.to_string(),
            _ => id.to_string(),
        }
    };
    let mut by_top: Vec<(String, Vec<&Placed>)> = Vec::new();
    let mut loose = Vec::new();
    let mine: Vec<&Placed> = files.iter().filter(|f| same(f.project.as_deref().unwrap_or(""), project)).collect();
    for f in shown::apart_scratch(mine).0 {
        match f.folder.as_deref().filter(|id| known(id).is_some()) {
            Some(id) => {
                let top = top_of(id);
                match by_top.iter_mut().find(|(t, _)| *t == top) {
                    Some((_, list)) => list.push(f),
                    None => by_top.push((top, vec![f])),
                }
            }
            None => loose.push(f),
        }
    }
    let tops: Vec<&Folder> = here.iter().copied().filter(|f| f.parent.as_deref().is_none_or(|p| known(p).is_none())).collect();
    let mut kept: Vec<Section> = tops
        .iter()
        .filter_map(|folder| by_top.iter().find(|(t, _)| *t == folder.id).map(|(_, files)| Section { folder, files: shown::sort_files(files, Sort::Recent) }))
        .collect();
    kept.sort_by(|a, b| is_pinned(&b.folder.name).cmp(&is_pinned(&a.folder.name)).then(b.files[0].at.total_cmp(&a.files[0].at)));
    let mut empty: Vec<&Folder> = tops.into_iter().filter(|f| !by_top.iter().any(|(t, _)| *t == f.id)).collect();
    empty.sort_by(|a, b| super::table_file::locale_compare(&a.name, &b.name));
    Home { kept, empty, loose: shown::sort_files(&loose, Sort::Recent) }
}

/// A row of Recent: the file and the folder it shows in (`RecentFile`).
#[derive(Clone, Debug, PartialEq)]
pub struct RecentFile<'a> {
    pub file: &'a Placed,
    pub folder: Option<&'a Folder>,
}

/// The files that changed last on these homes, newest first, each once, with the folder it
/// shows in (`recentOf`).
pub fn recent_of<'a>(homes: &[Home<'a>]) -> Vec<RecentFile<'a>> {
    let mut rows: Vec<RecentFile> = Vec::new();
    for home in homes {
        for s in &home.kept {
            for file in &s.files {
                if !rows.iter().any(|r| r.file.key == file.key) {
                    rows.push(RecentFile { file, folder: Some(s.folder) });
                }
            }
        }
        for file in &home.loose {
            if !rows.iter().any(|r| r.file.key == file.key) {
                rows.push(RecentFile { file, folder: None });
            }
        }
    }
    rows.sort_by(|a, b| b.file.at.total_cmp(&a.file.at));
    rows.truncate(RECENT_MOST);
    rows
}

/// Recent's rows under Today, Yesterday or their day, one run per day (`recentByDay`).
pub fn recent_by_day<'a>(rows: &[RecentFile<'a>], now: f64) -> Vec<(String, Vec<RecentFile<'a>>)> {
    let mut days: Vec<(String, Vec<RecentFile>)> = Vec::new();
    for r in rows {
        let day = when_words(r.file.at, now);
        match days.last_mut() {
            Some((last, rows)) if *last == day => rows.push(r.clone()),
            _ => days.push((day, vec![r.clone()])),
        }
    }
    days
}

/// What a document's tile says under its name: the first thing written after its title
/// (`firstLine`).
pub fn first_line(text: &str) -> Option<String> {
    shown::doc_blocks(text, 6).into_iter().find_map(|b| match b {
        shown::DocBlock::Words(t) | shown::DocBlock::Item(t) => Some(t),
        _ => None,
    })
}

/// A document's title and first sentence (`DocTop`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocTop {
    pub title: Option<String>,
    pub line: Option<String>,
}

pub fn doc_top(text: &str) -> DocTop {
    let title = match shown::doc_blocks(text, 1).into_iter().next() {
        Some(shown::DocBlock::Title(t)) => Some(t),
        _ => None,
    };
    DocTop { title, line: first_line(text) }
}

/// A name a person gave it, a document's own title, or its file name written out (`tileName`).
pub fn tile_name(f: &Placed, doc_title: Option<&str>) -> String {
    if let Some(title) = f.title.as_ref().filter(|t| !t.is_empty()) {
        return title.clone();
    }
    if f.kind.as_deref() != Some("markdown") {
        return shown::shown_name(f);
    }
    if let Some(title) = doc_title {
        return title.to_string();
    }
    // The name without its ending and its version, each dash or underscore a space unless it
    // stands between two figures, as in a date.
    let stem = match f.name.rfind('.') {
        Some(at) if at > 0 => &f.name[..at],
        _ => f.name.as_str(),
    };
    let stem = VERSION_END.replace(stem, "");
    let chars: Vec<char> = stem.chars().collect();
    let mut words = String::new();
    for (i, c) in chars.iter().enumerate() {
        let between_figures = i > 0 && chars[i - 1].is_ascii_digit() && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        words.push(if (*c == '-' || *c == '_') && !between_figures { ' ' } else { *c });
    }
    let words = words.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut letters = words.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().chain(letters).collect(),
        None => f.name.clone(),
    }
}

static VERSION_END: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| regex::Regex::new(r"-v\d+$").expect("a fixed pattern"));

/// What any other file's tile says: the thread it came from, else its kind (`tileLine`).
pub fn tile_line(f: &Placed) -> String {
    f.sent_in.iter().find(|s| s.kind != "chat" && !s.title.is_empty()).map_or_else(|| shown::kind_one(f.kind.as_deref()).to_string(), |s| s.title.clone())
}

/// The marks a person picks a tile's icon from, in the menu's order (`GLYPH_NAMES`).
pub const GLYPH_NAMES: [&str; 22] = [
    "page", "table", "image", "film", "window", "slides", "code", "tag", "compass", "people", "phone", "cloud", "binoculars", "chart", "palette", "chat", "box", "book", "shield", "chip", "layers",
    "mobile",
];

fn is_glyph(s: Option<&str>) -> Option<&'static str> {
    s.and_then(|s| GLYPH_NAMES.iter().copied().find(|g| *g == s))
}

/// A word that starts a part of the name picks the drawing (`BY_WORD`).
static BY_WORD: std::sync::LazyLock<Vec<(regex::Regex, &'static str)>> = std::sync::LazyLock::new(|| {
    [
        (r"^(pric|billing|invoice|seat)", "tag"),
        (r"^(position|pitch|messag)", "compass"),
        (r"^(customer|people|user|interview|persona|feedback)", "people"),
        (r"^(call|meeting|minutes)", "phone"),
        (r"^(cloud|server|deploy)", "cloud"),
        (r"^(competit|rival)", "binoculars"),
        (r"^(launch|result|metric|install|analytic)", "chart"),
        (r"^(brand|look|logo)", "palette"),
        (r"^(repl|post|tweet|outreach|email)", "chat"),
        (r"^(release|ship)", "box"),
        (r"^(guide|playbook|checklist|script|how)", "book"),
        (r"^(security|privacy|audit)", "shield"),
        (r"^(engine|harness|model)", "chip"),
        (r"^(mobile|ios|iphone|android)$|^phone$", "mobile"),
        (r"^(team|teammate|version)$", "layers"),
    ]
    .into_iter()
    .map(|(p, g)| (regex::Regex::new(p).expect("a fixed pattern"), g))
    .collect()
});

fn by_word(name: &str) -> Option<&'static str> {
    let stem = match name.rfind('.') {
        Some(at) if at > 0 => &name[..at],
        _ => name,
    };
    let parts = shown::parts_of(stem);
    BY_WORD.iter().find(|(word, _)| parts.iter().any(|p| word.is_match(p))).map(|(_, g)| *g)
}

fn by_kind(kind: Option<&str>) -> &'static str {
    match kind {
        Some("table") => "table",
        Some("image") => "image",
        Some("film") => "film",
        Some("page") => "window",
        Some("deck") => "slides",
        Some("code") => "code",
        _ => "page",
    }
}

/// The icon a person picked, then a word in its name or its document's title, then its folder's
/// name (never Pinned's), then its kind (`glyphFor`).
pub fn glyph_for(f: &Placed, folder: Option<&str>, doc_title: Option<&str>) -> &'static str {
    if let Some(g) = is_glyph(f.icon.as_deref()) {
        return g;
    }
    by_word(&shown::shown_name(f))
        .or_else(|| doc_title.and_then(by_word))
        .or_else(|| folder.filter(|n| !is_pinned(n)).and_then(by_word))
        .unwrap_or_else(|| by_kind(f.kind.as_deref()))
}

/// A picture on this computer shows itself, unless a person picked it an icon (`markOf`).
pub fn shows_picture(f: &Placed) -> bool {
    f.kind.as_deref() == Some("image") && f.opened.is_some() && is_glyph(f.icon.as_deref()).is_none()
}

/// The mark Automatic would give the file, drawn beside the word in the icon menu: a picture's
/// is the picture mark, anything else the mark it would have with no icon picked.
pub fn automatic_glyph(f: &Placed) -> &'static str {
    let bare = Placed { icon: None, ..f.clone() };
    if shows_picture(&bare) { "image" } else { glyph_for(&bare, None, None) }
}

/// "A, B and C" (`andJoined`).
pub fn and_joined(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The sentence under a home's sections that names its empty folders.
pub fn empty_words(names: &[&str]) -> String {
    format!("{} {} empty.", and_joined(names), if names.len() == 1 { "is" } else { "are" })
}

/// A project's tile on All files (`ProjectShelf`): how many files are in a folder, how many in
/// none, and when the newest of them changed.
#[derive(Clone, Debug, PartialEq)]
pub struct Shelf {
    pub name: String,
    pub kept: usize,
    pub loose: usize,
    pub at: Option<f64>,
}

pub fn shelves(list: &List) -> Vec<Shelf> {
    list.projects
        .iter()
        .map(|p| {
            let h = home_of(&list.files, &list.folders, p);
            let kept = h.kept.iter().map(|s| s.files.len()).sum();
            let newest = h.kept.iter().map(|s| s.files[0].at).chain(h.loose.first().map(|f| f.at)).fold(None, |m: Option<f64>, a| Some(m.map_or(a, |m| m.max(a))));
            Shelf { name: p.clone(), kept, loose: h.loose.len(), at: newest }
        })
        .collect()
}

/// What a project's tile says under its name.
pub fn shelf_line(s: &Shelf) -> String {
    let kept = match s.kept {
        0 => "Nothing kept yet".to_string(),
        1 => "1 file kept".to_string(),
        n => format!("{n} files kept"),
    };
    if s.loose > 0 { format!("{kept}, {} not in a folder", s.loose) } else { kept }
}

/* ------------------------------ what New offers ------------------------------ */

/// What New makes (`Making`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Making {
    Document,
    Table,
    Folder,
    Upload,
}

impl Making {
    /// The head of the list of projects it asks for (`MAKING_IN`).
    pub fn in_words(self) -> &'static str {
        match self {
            Making::Document => "New document in",
            Making::Table => "New table in",
            Making::Folder => "New folder in",
            Making::Upload => "Add files to",
        }
    }
    pub fn word(self) -> &'static str {
        match self {
            Making::Document => "Document",
            Making::Table => "Table",
            Making::Folder => "Folder",
            Making::Upload => "Upload from computer",
        }
    }
}

/// What New's menu offers, or None where New is not drawn (`NewMenu`): a document and a table
/// in a project of one's own (or from All files), a folder where one may be made, files from
/// the computer where a document may be made.
pub fn new_offers(project: Option<&str>, mine: &[String], can_folder: bool) -> Option<Vec<Making>> {
    let can_document = project.is_none_or(|p| mine.iter().any(|m| m.to_lowercase() == p.to_lowercase()));
    if (project.is_none() && mine.is_empty()) || (!can_document && !can_folder) {
        return None;
    }
    let mut out = Vec::new();
    if can_document {
        out.extend([Making::Document, Making::Table]);
    }
    if can_folder {
        out.push(Making::Folder);
    }
    if can_document {
        out.push(Making::Upload);
    }
    Some(out)
}

/// What a choice on New does (`choose`): the project it goes in at once, or None when the menu
/// asks which.
pub fn new_goes_in(project: Option<&str>, mine: &[String]) -> Option<String> {
    match (project, mine) {
        (Some(p), _) => Some(p.to_string()),
        (None, [one]) => Some(one.clone()),
        _ => None,
    }
}

/* ------------------------------ a menu's place ------------------------------ */

/// The width of a menu (`MENU_WIDTH`, `.fh-menu`), and of the icon menu (`.is-wide`).
pub const MENU_WIDTH: f32 = 210.0;
pub const MENU_WIDE: f32 = 252.0;

/// Where a menu opened from a button stands: its right edge on the button's, six under it, or for
/// a folder's, from the button's left (`atButton`).
pub fn at_button(rect: [f32; 4], from_left: bool) -> (f32, f32) {
    let [x, y, w, h] = rect;
    (if from_left { x + MENU_WIDTH } else { x + w }, y + h + 6.0)
}

/// The menu's top left, kept eight inside the window and turned upward where it would run off
/// the foot (`Menu`'s layout effect): `at` is where it was asked to end on the right.
pub fn menu_place(at: (f32, f32), size: (f32, f32), window: (f32, f32)) -> (f32, f32) {
    let (w, h) = size;
    let x = (at.0 - w).min(window.0 - w - 8.0).max(8.0);
    let y = (if at.1 + h > window.1 - 8.0 { at.1 - h - 12.0 } else { at.1 }).max(8.0);
    (x, y)
}

/* ------------------------------ what is written ------------------------------ */

/// One change to a folder or a file, as `zero:files-place-folder` takes it (`FileFolderAct`).
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    Make { project: String, name: String, parent: Option<String> },
    Rename { id: String, name: String },
    Move { key: String, folder: Option<String> },
    Name { key: String, name: String },
    Icon { key: String, icon: String },
    Remove { id: String },
    Restore { ids: Vec<String> },
}

pub const FOLDER_ACT: &str = "zero:files-place-folder";
pub const NEW_FILE: &str = "zero:files-place-new";
pub const TRASH: &str = "zero:files-place-trash";
pub const UNTRASH: &str = "zero:files-place-untrash";
/// Upload from computer once the person has picked files here: what the web window's
/// `zero:files-place-upload` does after its own file window, given the paths (a channel of the
/// desktop app's local door alone, `main/local-door.mts`).
pub const UPLOAD_HERE: &str = "zero:files-place-upload-here";
/// What the page says when the app turned down an upload, in its own words or else these.
pub const UPLOAD_REFUSED: &str = "Those files could not be added.";

/// The files an upload left out, as the page says it (`upload`'s notice): a kind Files cannot
/// show. None when nothing was left out.
pub fn upload_left_words(left: &[String]) -> Option<String> {
    match left.len() {
        0 => None,
        1 => Some(format!("{} is a kind Files cannot show, so it was not added.", left[0])),
        _ => Some(format!("{} are kinds Files cannot show, so they were not added.", left.join(", "))),
    }
}

impl Act {
    pub fn payload(&self) -> Value {
        match self {
            Act::Make { project, name, parent } => json!({ "do": "make", "project": project, "name": name, "parent": parent }),
            Act::Rename { id, name } => json!({ "do": "rename", "id": id, "name": name }),
            Act::Move { key, folder } => json!({ "do": "move", "key": key, "folder": folder }),
            Act::Name { key, name } => json!({ "do": "name", "key": key, "name": name }),
            Act::Icon { key, icon } => json!({ "do": "icon", "key": key, "icon": icon }),
            Act::Remove { id } => json!({ "do": "remove", "id": id }),
            Act::Restore { ids } => json!({ "do": "restore", "ids": ids }),
        }
    }
}

/// A change as the page draws it over the list (`Shown`): a folder made, once the app gave it
/// its id, or any change but a make and a restore.
#[derive(Clone, Debug, PartialEq)]
pub enum Shown {
    Act(Act),
    Made(Folder),
}

/// A name as the app keeps it (`cleanName`): its spaces one, its ends trimmed, 120 at most.
pub fn clean_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(120).collect()
}

/// The list with the changes made here drawn over it (`withChanges`).
pub fn with_changes(list: &List, changes: &[Shown]) -> List {
    let mut out = list.clone();
    for c in changes {
        match c {
            Shown::Act(Act::Rename { id, name }) => {
                let name = clean_name(name);
                if !name.is_empty() {
                    for f in out.folders.iter_mut().filter(|f| &f.id == id) {
                        f.name = name.clone();
                    }
                }
            }
            Shown::Made(folder) => {
                if !out.folders.iter().any(|f| f.id == folder.id) {
                    out.folders.push(folder.clone());
                }
            }
            Shown::Act(Act::Remove { id }) => {
                let mut gone: Vec<String> = vec![id.clone()];
                gone.extend(out.folders.iter().filter(|f| f.parent.as_ref() == Some(id)).map(|f| f.id.clone()));
                out.folders.retain(|f| !gone.contains(&f.id));
                for f in out.files.iter_mut().filter(|f| f.folder.as_ref().is_some_and(|x| gone.contains(x))) {
                    f.folder = None;
                }
            }
            Shown::Act(Act::Move { key, folder }) => {
                for f in out.files.iter_mut().filter(|f| &f.key == key) {
                    f.folder = folder.clone();
                    f.folder_follows = false;
                }
            }
            Shown::Act(Act::Name { key, name }) => {
                let name = clean_name(name);
                if !name.is_empty() {
                    for f in out.files.iter_mut().filter(|f| &f.key == key) {
                        f.title = Some(name.clone());
                    }
                }
            }
            Shown::Act(Act::Icon { key, icon }) => {
                for f in out.files.iter_mut().filter(|f| &f.key == key) {
                    f.icon = (!icon.is_empty()).then(|| icon.clone());
                }
            }
            Shown::Act(Act::Make { .. } | Act::Restore { .. }) => {}
        }
    }
    out
}

/// What Z puts back, and what it says (`files-undo.ts`).
#[derive(Clone, Debug, PartialEq)]
pub struct Back {
    pub label: String,
    pub undoes: String,
    pub back: Vec<Act>,
}

fn clean(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A folder's rename taken back (`renameUndo`).
pub fn rename_undo(folders: &[Folder], id: &str, name: &str) -> Option<Back> {
    let was = folders.iter().find(|f| f.id == id)?;
    if clean(name) == was.name {
        return None;
    }
    Some(Back { label: format!("Renamed {} back", was.name), undoes: format!("rename “{}” back to “{}”", clean(name), was.name), back: vec![Act::Rename { id: id.to_string(), name: was.name.clone() }] })
}

/// A file's rename taken back (`fileRenameUndo`).
pub fn file_rename_undo(file: &Placed, name: &str) -> Option<Back> {
    let was = shown::shown_name(file);
    if clean(name) == was {
        return None;
    }
    Some(Back { label: format!("Renamed {was} back"), undoes: format!("rename “{}” back to “{was}”", clean(name)), back: vec![Act::Name { key: file.key.clone(), name: was.clone() }] })
}

/// Files moved, taken back (`moveUndo`), read before the move while `files` still says where
/// each one was.
pub fn move_undo(files: &[Placed], keys: &[String], to: Option<&str>, folders: &[Folder]) -> Option<Back> {
    let moved: Vec<&Placed> = keys.iter().filter_map(|k| files.iter().find(|f| &f.key == k)).filter(|f| f.folder.as_deref() != to).collect();
    if moved.is_empty() {
        return None;
    }
    let back: Vec<Act> = moved.iter().map(|f| Act::Move { key: f.key.clone(), folder: f.folder.clone() }).collect();
    if moved.len() > 1 {
        return Some(Back { label: format!("Moved {} files back", moved.len()), undoes: format!("move those {} files back", moved.len()), back });
    }
    let f = moved[0];
    let name = shown::shown_name(f);
    let place = folders.iter().find(|x| Some(&x.id) == f.folder.as_ref()).map_or_else(|| format!("the top of {}", f.project.as_deref().unwrap_or("")), |x| x.name.clone());
    Some(Back { label: format!("Moved {name} back"), undoes: format!("move {name} back to {place}"), back })
}

/// What a folder's deletion says (`removeFolder`): where its files went, if it had any.
pub fn deleted_words(folder: &str, files: usize, project: &str) -> String {
    let after = match files {
        0 => String::new(),
        1 => format!(" Its file is now in {project}."),
        n => format!(" Its {n} files are now in {project}."),
    };
    format!("Deleted the folder {folder}.{after}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::files_shown::SentIn;

    const DAY: f64 = 86_400_000.0;
    const HOUR: f64 = 3_600_000.0;

    fn file(name: &str) -> Placed {
        Placed { key: name.to_string(), project: Some("Agentbox Team".to_string()), name: name.to_string(), kind: Some("markdown".to_string()), versions: 1, at: 1.0, ..Placed::default() }
    }
    fn at(name: &str, folder: Option<&str>, when: f64) -> Placed {
        Placed { folder: folder.map(str::to_string), at: when, ..file(name) }
    }
    fn folder(id: &str, name: &str, parent: Option<&str>, project: &str) -> Folder {
        Folder { id: id.to_string(), project: project.to_string(), name: name.to_string(), parent: parent.map(str::to_string), at: 1.0 }
    }
    fn names(files: &[&Placed]) -> Vec<String> {
        files.iter().map(|f| f.name.clone()).collect()
    }
    /// The instant a clock on this machine reads that day and hour (`new Date(y, m, d, h, min)`).
    fn local(year: i64, month: i64, day: i64, hour: f64, minute: f64) -> f64 {
        let midnight = Day::first_of(year, month).plus(day - 1);
        // The day's midnight on the wall clock, found from noon UTC so the offset is that day's.
        let noon = (midnight.0 as f64) * DAY + 12.0 * HOUR;
        let offset = rune::time::LocalTime::at(noon as i64).offset as f64 * 1000.0;
        midnight.0 as f64 * DAY - offset + hour * HOUR + minute * 60_000.0
    }

    // files-home-shows-what-the-team-kept-and-sets-the-rest-apart: the sections.
    fn team() -> (Vec<Folder>, Vec<Placed>) {
        let folders = vec![
            folder("cus", "Customers", None, "Agentbox Team"),
            folder("pin", "Pinned", None, "Agentbox Team"),
            folder("calls", "Calls", Some("cus"), "Agentbox Team"),
            folder("brand", "Brand kit", None, "Agentbox Team"),
            folder("other", "Launch", None, "Lantern"),
        ];
        let files = vec![
            at("northwind.md", Some("cus"), 50.0),
            at("call-notes.md", Some("calls"), 90.0),
            at("positioning.md", Some("pin"), 10.0),
            at("pricing.md", Some("pin"), 20.0),
            at("loose-draft.md", None, 70.0),
            at("lost.md", Some("deleted-folder"), 60.0),
            Placed { kind: Some("image".to_string()), agent: Some("Claude Code".to_string()), ..at("page-2026-11-21T10-00-00-000Z.png", None, 99.0) },
            Placed { project: Some("Lantern".to_string()), ..at("lantern-plan.md", Some("other"), 80.0) },
        ];
        (folders, files)
    }

    #[test]
    fn a_home_puts_pinned_first_then_the_folder_that_changed_last() {
        let (folders, files) = team();
        let home = home_of(&files, &folders, "Agentbox Team");
        assert_eq!(home.kept.iter().map(|s| s.folder.name.as_str()).collect::<Vec<_>>(), ["Pinned", "Customers"]);
    }

    #[test]
    fn a_subfolders_files_are_in_its_folders_section_newest_first() {
        let (folders, files) = team();
        let home = home_of(&files, &folders, "Agentbox Team");
        assert_eq!(names(&home.kept[1].files), ["call-notes.md", "northwind.md"]);
        assert_eq!(names(&home.kept[0].files), ["pricing.md", "positioning.md"]);
    }

    #[test]
    fn an_empty_folder_is_listed_apart_and_what_nobody_filed_is_set_apart() {
        let (folders, files) = team();
        let home = home_of(&files, &folders, "Agentbox Team");
        assert_eq!(home.empty.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Brand kit"]);
        assert_eq!(names(&home.loose), ["loose-draft.md", "lost.md"]);
        let every: Vec<String> = home.kept.iter().flat_map(|s| names(&s.files)).chain(names(&home.loose)).collect();
        assert!(!every.contains(&"page-2026-11-21T10-00-00-000Z.png".to_string()));
        assert!(!every.contains(&"lantern-plan.md".to_string()));
        let bare = [file("x.md")];
        let home = home_of(&bare, &[], "Agentbox Team");
        assert!(home.kept.is_empty());
        assert_eq!(names(&home.loose), ["x.md"]);
    }

    #[test]
    fn pinned_is_a_folder_of_that_name_and_marks_a_file_changed_this_week() {
        assert!(is_pinned("Pinned") && is_pinned("pinned"));
        assert!(!is_pinned("Pinned ideas") && !is_pinned("Unpinned"));
        let now = 100.0 * DAY;
        assert!(pin_changed_lately("Pinned", now - 2.0 * DAY, now));
        assert!(!pin_changed_lately("Pinned", now - 8.0 * DAY, now));
        assert!(!pin_changed_lately("Customers", now - DAY, now));
    }

    #[test]
    fn a_tile_says_a_documents_first_sentence_under_its_title() {
        assert_eq!(first_line("# Pricing\n\n$40 a seat a month. The first 7 days are **free**.\n\n## Rules").as_deref(), Some("$40 a seat a month. The first 7 days are free."));
        assert_eq!(first_line("# Northwind\n\n- Two founders\n- Calls every day").as_deref(), Some("Two founders"));
        assert_eq!(first_line("Notes straight away, no title.").as_deref(), Some("Notes straight away, no title."));
        assert_eq!(first_line("# Team look"), None);
        assert_eq!(first_line(""), None);
        assert_eq!(doc_top("# Pricing\n\n$40 a seat."), DocTop { title: Some("Pricing".to_string()), line: Some("$40 a seat.".to_string()) });
    }

    #[test]
    fn a_tile_is_named_by_its_title_a_name_given_or_its_file_name_written_out() {
        assert_eq!(tile_name(&file("positioning.md"), Some("Positioning")), "Positioning");
        assert_eq!(tile_name(&Placed { title: Some("Our pitch".to_string()), ..file("positioning.md") }, Some("Positioning")), "Our pitch");
        assert_eq!(tile_name(&file("first-onboarding-call-2026-11-21-v18.md"), None), "First onboarding call 2026-11-21");
        assert_eq!(tile_name(&Placed { kind: Some("image".to_string()), ..file("agentcloud-feed-1.png") }, None), "agentcloud-feed-1.png");
    }

    #[test]
    fn a_tile_says_today_yesterday_or_the_day() {
        let now = local(2026, 10, 22, 15.0, 0.0);
        assert_eq!(when_words(local(2026, 10, 22, 0.0, 5.0), now), "Today");
        assert_eq!(when_words(local(2026, 10, 21, 23.0, 55.0), now), "Yesterday");
        assert_eq!(when_words(local(2026, 10, 20, 12.0, 0.0), now), "Nov 20");
    }

    #[test]
    fn any_other_tile_says_the_thread_it_came_from() {
        let sent = SentIn { kind: "thread".to_string(), title: "Agentcloud launch posts".to_string(), ..SentIn::default() };
        assert_eq!(tile_line(&Placed { kind: Some("image".to_string()), sent_in: vec![sent], ..file("a.png") }), "Agentcloud launch posts");
        assert_eq!(tile_line(&Placed { kind: Some("film".to_string()), ..file("a.mp4") }), "Film");
    }

    #[test]
    fn a_tiles_icon_follows_a_word_that_starts_a_part_of_its_name() {
        assert_eq!(glyph_for(&file("pricing.md"), None, None), "tag");
        assert_eq!(glyph_for(&file("positioning.md"), None, None), "compass");
        assert_eq!(glyph_for(&file("Onboarding call script.md"), None, None), "phone");
        assert_eq!(glyph_for(&file("user-feedback.md"), None, None), "people");
        assert_eq!(glyph_for(&Placed { kind: Some("image".to_string()), ..file("agentcloud-feed-1.png") }, None, None), "image");
        assert_eq!(glyph_for(&file("cloud-costs.md"), None, None), "cloud");
    }

    #[test]
    fn a_tiles_icon_takes_its_folders_word_but_never_pinneds() {
        assert_eq!(glyph_for(&file("northwind.md"), Some("Customers"), None), "people");
        assert_eq!(glyph_for(&file("pricing.md"), Some("Customers"), None), "tag");
        assert_eq!(glyph_for(&file("notes.md"), Some("Pinned"), None), "page");
    }

    #[test]
    fn a_tiles_icon_knows_the_products_words_and_falls_back_to_its_kind() {
        assert_eq!(glyph_for(&file("engines-and-harnesses.md"), None, None), "chip");
        assert_eq!(glyph_for(&file("Phone app.md"), None, None), "mobile");
        assert_eq!(glyph_for(&file("team-version.md"), None, None), "layers");
        assert_eq!(glyph_for(&file("Team look.md"), None, None), "palette");
        assert_eq!(glyph_for(&file("notes.md"), None, None), "page");
        assert_eq!(glyph_for(&Placed { kind: Some("table".to_string()), ..file("leads.csv") }, None, None), "table");
        assert_eq!(glyph_for(&Placed { kind: Some("film".to_string()), ..file("demo.mp4") }, None, None), "film");
        assert_eq!(glyph_for(&Placed { kind: None, ..file("x") }, None, None), "page");
    }

    #[test]
    fn an_icon_a_person_picked_wins_and_takes_a_pictures_picture_away() {
        let picked = Placed { icon: Some("book".to_string()), ..file("pricing.md") };
        assert_eq!(glyph_for(&picked, None, None), "book");
        let picture = Placed { kind: Some("image".to_string()), opened: Some("/tmp/a.png".to_string()), ..file("a.png") };
        assert!(shows_picture(&picture));
        assert!(!shows_picture(&Placed { icon: Some("chart".to_string()), ..picture.clone() }));
        assert!(!shows_picture(&Placed { opened: None, ..picture.clone() }));
        assert_eq!(automatic_glyph(&Placed { icon: Some("chart".to_string()), ..picture }), "image");
        assert_eq!(automatic_glyph(&picked), "tag");
        // An icon nobody can pick is no icon.
        assert_eq!(glyph_for(&Placed { icon: Some("rocket".to_string()), ..file("notes.md") }, None, None), "page");
    }

    // files-recent-shows-what-changed-last-by-day.
    fn recent_files(now: f64) -> (Vec<Folder>, Vec<Placed>) {
        let folders = vec![folder("cus", "Customers", None, "Agentbox"), folder("calls", "Calls", Some("cus"), "Agentbox"), folder("pin", "Pinned", None, "Agentbox"), folder("lan", "Launch", None, "Lantern")];
        let f = |name: &str, folder: Option<&str>, hours: f64| Placed { project: Some("Agentbox".to_string()), ..at(name, folder, now - hours * HOUR) };
        let files = vec![
            f("untitled.md", None, 0.1),
            f("northwind.md", Some("calls"), 2.0),
            f("positioning.md", Some("pin"), 3.0),
            f("pricing.md", Some("pin"), 26.0),
            f("old-plan.md", None, 80.0),
            Placed { kind: Some("image".to_string()), agent: Some("Claude Code".to_string()), ..f("page-2026-11-23T10-00-00-000Z.png", None, 0.0) },
            Placed { project: Some("Lantern".to_string()), ..f("lantern-plan.md", Some("lan"), 0.5) },
        ];
        (folders, files)
    }

    #[test]
    fn recent_is_the_projects_files_newest_first_each_once_with_its_top_folder() {
        let now = local(2026, 10, 23, 15.0, 0.0);
        let (folders, files) = recent_files(now);
        let rows = recent_of(&[home_of(&files, &folders, "Agentbox")]);
        assert_eq!(rows.iter().map(|r| r.file.name.as_str()).collect::<Vec<_>>(), ["untitled.md", "northwind.md", "positioning.md", "pricing.md", "old-plan.md"]);
        assert_eq!(rows.iter().map(|r| r.folder.map(|f| f.name.as_str())).collect::<Vec<_>>(), [None, Some("Customers"), Some("Pinned"), Some("Pinned"), None]);
        let both = recent_of(&[home_of(&files, &folders, "Agentbox"), home_of(&files, &folders, "Lantern")]);
        assert_eq!(both.iter().take(2).map(|r| r.file.name.as_str()).collect::<Vec<_>>(), ["untitled.md", "lantern-plan.md"]);
    }

    #[test]
    fn recent_stops_at_ten() {
        let many: Vec<Placed> = (0..14).map(|i| Placed { project: Some("Agentbox".to_string()), ..at(&format!("f{i}.md"), None, 1000.0 - i as f64) }).collect();
        assert_eq!(recent_of(&[home_of(&many, &[], "Agentbox")]).len(), RECENT_MOST);
        assert_eq!(RECENT_MOST, 10);
        assert_eq!(SHOWN_AT_FIRST, 6);
    }

    #[test]
    fn recents_days_are_today_yesterday_or_the_date_one_run_each() {
        let now = local(2026, 10, 23, 15.0, 0.0);
        let (folders, files) = recent_files(now);
        let days = recent_by_day(&recent_of(&[home_of(&files, &folders, "Agentbox")]), now);
        assert_eq!(days.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>(), ["Today", "Yesterday", "Nov 20"]);
        let runs: Vec<Vec<&str>> = days.iter().map(|(_, rows)| rows.iter().map(|r| r.file.name.as_str()).collect()).collect();
        assert_eq!(runs, [vec!["untitled.md", "northwind.md", "positioning.md"], vec!["pricing.md"], vec!["old-plan.md"]]);
        assert!(recent_by_day(&[], now).is_empty());
    }

    // files-folders-have-their-own-pages-and-a-path-back.
    #[test]
    fn the_path_back_leads_through_the_project_and_a_subfolders_parent() {
        let folders = [folder("cus", "Customers", None, "Agentbox"), folder("calls", "Calls", Some("cus"), "Agentbox")];
        assert!(crumbs_for(&folders, None, None).is_empty());
        assert_eq!(crumbs_for(&folders, Some("Agentbox"), None), [Crumb { name: "All files".into(), project: None, folder: None }]);
        let names = |c: Vec<Crumb>| c.into_iter().map(|c| c.name).collect::<Vec<_>>();
        assert_eq!(names(crumbs_for(&folders, Some("Agentbox"), Some("cus"))), ["All files", "Agentbox"]);
        assert_eq!(
            crumbs_for(&folders, Some("Agentbox"), Some("calls")),
            [
                Crumb { name: "All files".into(), project: None, folder: None },
                Crumb { name: "Agentbox".into(), project: Some("Agentbox".into()), folder: None },
                Crumb { name: "Customers".into(), project: Some("Agentbox".into()), folder: Some("cus".into()) },
            ]
        );
        assert_eq!(names(crumbs_for(&folders, Some("Agentbox"), Some(NOT_IN_A_FOLDER))), ["All files", "Agentbox"]);
    }

    #[test]
    fn a_folders_page_holds_its_files_and_its_subfolders_newest_first() {
        let (folders, files) = team();
        assert_eq!(names(&folder_files(&files, &folders, "Agentbox Team", "cus")), ["call-notes.md", "northwind.md"]);
    }

    // a-change-made-in-files-shows-at-once.
    fn changes_list() -> List {
        let f = |key: &str, folder: Option<&str>| Placed { key: key.to_string(), folder: folder.map(str::to_string), ..file(&format!("{key}.md")) };
        List {
            files: vec![f("quotes", Some("res")), f("notes", Some("calls")), Placed { icon: Some("tag".into()), title: Some("The plan".into()), ..f("plan", None) }],
            projects: vec!["Agentbox".to_string()],
            folders: vec![folder("res", "Research", None, "Agentbox"), folder("calls", "Calls", Some("res"), "Agentbox"), folder("brand", "Brand kit", None, "Agentbox")],
            mine: Vec::new(),
        }
    }
    fn folder_names(l: &List) -> Vec<String> {
        l.folders.iter().map(|f| f.name.clone()).collect()
    }
    fn on(l: &List, key: &str) -> Placed {
        l.files.iter().find(|f| f.key == key).cloned().unwrap()
    }

    #[test]
    fn a_folder_takes_its_new_name_at_once_as_the_app_keeps_it() {
        let list = changes_list();
        let shown = with_changes(&list, &[Shown::Act(Act::Rename { id: "res".into(), name: "  User   research ".into() })]);
        assert_eq!(folder_names(&shown), ["User research", "Calls", "Brand kit"]);
        let blank = with_changes(&list, &[Shown::Act(Act::Rename { id: "res".into(), name: "   ".into() })]);
        assert_eq!(folder_names(&blank), ["Research", "Calls", "Brand kit"]);
    }

    #[test]
    fn a_folder_made_is_there_once_and_a_deleted_one_takes_its_folders_and_frees_its_files() {
        let list = changes_list();
        let made = Shown::Made(folder("new", "Pricing", None, "Agentbox"));
        assert_eq!(folder_names(&with_changes(&list, std::slice::from_ref(&made))), ["Research", "Calls", "Brand kit", "Pricing"]);
        let mut after = list.clone();
        after.folders.push(folder("new", "Pricing", None, "Agentbox"));
        assert_eq!(folder_names(&with_changes(&after, &[made])), ["Research", "Calls", "Brand kit", "Pricing"]);
        let gone = with_changes(&list, &[Shown::Act(Act::Remove { id: "res".into() })]);
        assert_eq!(folder_names(&gone), ["Brand kit"]);
        assert_eq!(on(&gone, "quotes").folder, None);
        assert_eq!(on(&gone, "notes").folder, None);
    }

    #[test]
    fn a_file_moves_takes_its_name_and_icon_and_leaves_the_others_alone() {
        let mut list = changes_list();
        list.files[0].folder_follows = true;
        let moved = on(&with_changes(&list, &[Shown::Act(Act::Move { key: "quotes".into(), folder: Some("brand".into()) })]), "quotes");
        assert_eq!(moved.folder.as_deref(), Some("brand"));
        assert!(!moved.folder_follows);
        assert_eq!(on(&with_changes(&list, &[Shown::Act(Act::Move { key: "quotes".into(), folder: None })]), "quotes").folder, None);
        let shown = with_changes(&list, &[Shown::Act(Act::Name { key: "quotes".into(), name: " Human agent teams quotes ".into() }), Shown::Act(Act::Icon { key: "plan".into(), icon: String::new() })]);
        assert_eq!(on(&shown, "quotes").title.as_deref(), Some("Human agent teams quotes"));
        assert_eq!(on(&shown, "plan").icon, None);
        assert_eq!(on(&shown, "plan").title.as_deref(), Some("The plan"));
        assert_eq!(on(&with_changes(&list, &[Shown::Act(Act::Icon { key: "quotes".into(), icon: "book".into() })]), "quotes").icon.as_deref(), Some("book"));
        assert_eq!(on(&shown, "notes"), list.files[1]);
    }

    // z-undoes-a-folder-rename-or-a-file-move-on-the-files-page.
    #[test]
    fn z_puts_a_rename_and_a_move_back() {
        let list = changes_list();
        let back = rename_undo(&list.folders, "res", "Interviews").unwrap();
        assert_eq!(back.label, "Renamed Research back");
        assert_eq!(back.back, [Act::Rename { id: "res".into(), name: "Research".into() }]);
        assert_eq!(rename_undo(&list.folders, "res", " Research "), None);
        let moved = move_undo(&list.files, &["quotes".to_string()], Some("brand"), &list.folders).unwrap();
        assert_eq!(moved.label, "Moved quotes.md back");
        assert_eq!(moved.undoes, "move quotes.md back to Research");
        assert_eq!(moved.back, [Act::Move { key: "quotes".into(), folder: Some("res".into()) }]);
        let top = move_undo(&list.files, &["plan".to_string()], Some("brand"), &list.folders).unwrap();
        assert_eq!(top.undoes, "move The plan back to the top of Agentbox Team");
        assert_eq!(move_undo(&list.files, &["quotes".to_string()], Some("res"), &list.folders), None);
        let named = file_rename_undo(&list.files[2], "Our plan").unwrap();
        assert_eq!(named.label, "Renamed The plan back");
        assert_eq!(named.back, [Act::Name { key: "plan".into(), name: "The plan".into() }]);
    }

    #[test]
    fn a_change_is_written_as_the_app_takes_it() {
        assert_eq!(Act::Make { project: "Storefront".into(), name: "Calls".into(), parent: None }.payload(), json!({ "do": "make", "project": "Storefront", "name": "Calls", "parent": null }));
        assert_eq!(Act::Move { key: "k".into(), folder: None }.payload(), json!({ "do": "move", "key": "k", "folder": null }));
        assert_eq!(Act::Icon { key: "k".into(), icon: String::new() }.payload(), json!({ "do": "icon", "key": "k", "icon": "" }));
        assert_eq!(Act::Restore { ids: vec!["a".into()] }.payload(), json!({ "do": "restore", "ids": ["a"] }));
    }

    #[test]
    fn a_project_tile_says_what_it_kept_and_what_is_in_no_folder() {
        let line = |kept, loose| shelf_line(&Shelf { name: String::new(), kept, loose, at: None });
        assert_eq!(line(0, 10), "Nothing kept yet, 10 not in a folder");
        assert_eq!(line(1, 0), "1 file kept");
        assert_eq!(line(4, 2), "4 files kept, 2 not in a folder");
        let (folders, files) = team();
        let list = List { files, projects: vec!["Agentbox Team".into(), "Lantern".into(), "Empty".into()], folders, mine: Vec::new() };
        let s = shelves(&list);
        assert_eq!((s[0].kept, s[0].loose, s[0].at), (4, 2, Some(90.0)));
        assert_eq!((s[1].kept, s[1].loose), (1, 0));
        assert_eq!((s[2].kept, s[2].loose, s[2].at), (0, 0, None));
    }

    #[test]
    fn empty_folders_are_named_in_one_sentence() {
        assert_eq!(empty_words(&["Brand kit"]), "Brand kit is empty.");
        assert_eq!(empty_words(&["A", "B"]), "A and B are empty.");
        assert_eq!(empty_words(&["A", "B", "C"]), "A, B and C are empty.");
        assert_eq!(deleted_words("Calls", 0, "Storefront"), "Deleted the folder Calls.");
        assert_eq!(deleted_words("Calls", 1, "Storefront"), "Deleted the folder Calls. Its file is now in Storefront.");
        assert_eq!(deleted_words("Calls", 3, "Storefront"), "Deleted the folder Calls. Its 3 files are now in Storefront.");
    }

    #[test]
    fn new_offers_what_can_be_made_here_and_asks_where_only_when_it_must() {
        let mine = vec!["Storefront".to_string(), "Mobile".to_string()];
        assert_eq!(new_offers(Some("Storefront"), &mine, true), Some(vec![Making::Document, Making::Table, Making::Folder, Making::Upload]));
        // A project shared with the person, not their own: only a folder.
        assert_eq!(new_offers(Some("Shared"), &mine, true), Some(vec![Making::Folder]));
        assert_eq!(new_offers(Some("Shared"), &mine, false), None);
        assert_eq!(new_offers(None, &[], true), None);
        assert_eq!(new_goes_in(Some("Storefront"), &mine).as_deref(), Some("Storefront"));
        assert_eq!(new_goes_in(None, &mine[..1]).as_deref(), Some("Storefront"));
        assert_eq!(new_goes_in(None, &mine), None);
        assert_eq!(Making::Upload.in_words(), "Add files to");
    }

    #[test]
    fn an_upload_names_what_it_left_out_as_the_page_does() {
        assert_eq!(upload_left_words(&[]), None);
        assert_eq!(upload_left_words(&["tool.exe".into()]).as_deref(), Some("tool.exe is a kind Files cannot show, so it was not added."));
        assert_eq!(upload_left_words(&["a.exe".into(), "b.dmg".into()]).as_deref(), Some("a.exe, b.dmg are kinds Files cannot show, so they were not added."));
    }

    #[test]
    fn a_menu_stands_under_its_button_inside_the_window() {
        // Under a tile's ⋯, its right edge on the button's.
        assert_eq!(menu_place(at_button([600.0, 266.0, 28.0, 28.0], false), (210.0, 140.0), (1280.0, 760.0)), (418.0, 300.0));
        // A folder's from the button's left.
        assert_eq!(menu_place(at_button([480.0, 213.0, 26.0, 26.0], true), (210.0, 80.0), (1280.0, 760.0)), (480.0, 245.0));
        // Turned upward near the foot, and never past the left edge.
        assert_eq!(menu_place((100.0, 700.0), (210.0, 140.0), (1280.0, 760.0)), (8.0, 548.0));
    }
}
