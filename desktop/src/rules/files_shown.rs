//! What the Files place shows of the list the app hands it (`zero:files-place`, built in
//! `main/files-place.mts`): which files a page holds, how a search and a kind narrow them, in
//! what order, and what a tile says of each. Ported from `renderer/src/files/files-shown.ts`
//! with the web's own test cases, and the age a tile tells (`ago` in `renderer/src/format.ts`).

use serde::Deserialize;
use serde_json::Value;

/// One place a file was sent in (`FileSentIn`).
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct SentIn {
    pub kind: String,
    pub product: Option<String>,
    pub id: Option<String>,
    pub title: String,
}

/// One file of the place, at its newest version (`PlacedFile` in `renderer/src/types.ts`).
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Placed {
    pub key: String,
    pub project: Option<String>,
    pub name: String,
    /// The name a person gave it in Files; `name` stays its name on disk.
    pub title: Option<String>,
    pub kind: Option<String>,
    pub versions: u32,
    pub at: f64,
    pub by: Option<String>,
    /// The agent that made the newest version: absent for a person, `""` for a plain agent.
    pub agent: Option<String>,
    pub open: u32,
    pub approved_by: Vec<Option<String>>,
    pub changed_since_approval: Option<Value>,
    pub sent_in: Vec<SentIn>,
    /// The newest version on this Mac, or none while it has not come down.
    pub opened: Option<String>,
    pub folder: Option<String>,
    pub folder_follows: bool,
    pub bytes: Option<f64>,
    pub on_mac: Option<f64>,
    pub older: Option<f64>,
    pub cloud: bool,
}

/// A folder a person made in a project (`FileFolder`).
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Folder {
    pub id: String,
    pub project: String,
    pub name: String,
    pub parent: Option<String>,
}

/// What `zero:files-place` answers (`FilesPlaceList`).
#[derive(Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct List {
    pub files: Vec<Placed>,
    pub projects: Vec<String>,
    pub folders: Vec<Folder>,
    pub mine: Vec<String>,
}

/// The kinds, in the order the page offers them, and their words (`KIND_ORDER`, `KIND_PLURAL`).
pub const KIND_ORDER: [&str; 8] = ["markdown", "image", "film", "deck", "pdf", "page", "table", "code"];
pub fn kind_plural(kind: &str) -> &'static str {
    match kind {
        "markdown" => "Documents",
        "image" => "Pictures",
        "film" => "Films",
        "deck" => "Decks",
        "pdf" => "PDFs",
        "page" => "Pages",
        "table" => "Tables",
        _ => "Code",
    }
}
/// One of a kind (`KIND_ONE`).
pub fn kind_one(kind: Option<&str>) -> &'static str {
    match kind {
        Some("markdown") => "Document",
        Some("image") => "Picture",
        Some("film") => "Film",
        Some("deck") => "Deck",
        Some("pdf") => "PDF",
        Some("page") => "Page",
        Some("table") => "Table",
        Some("code") => "Code",
        _ => "File",
    }
}
/// The word a list row's thumbnail wears for a kind that draws no picture there.
pub fn thumb_word(kind: Option<&str>) -> &'static str {
    match kind {
        Some("markdown") => "DOC",
        Some("pdf") => "PDF",
        Some("deck") => "DECK",
        Some("page") => "PAGE",
        Some("code") => "CODE",
        Some("table") => "TABLE",
        _ => "FILE",
    }
}

/// Lower case and without its accents, as a search compares words (`fold`, which decomposes
/// each letter and drops the marks): the Latin letters a name is written in, each to its own
/// letter, and a combining mark dropped.
fn fold(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| !('\u{300}'..='\u{36f}').contains(c)).map(bare_letter).collect()
}

/// A Latin letter without its accent, or the letter as it is.
fn bare_letter(c: char) -> char {
    const ACCENTED: [(&str, char); 19] = [
        ("àáâãäåāăą", 'a'),
        ("çćĉċč", 'c'),
        ("ďđ", 'd'),
        ("èéêëēĕėęě", 'e'),
        ("ĝğġģ", 'g'),
        ("ĥħ", 'h'),
        ("ìíîïĩīĭį", 'i'),
        ("ĵ", 'j'),
        ("ķ", 'k'),
        ("ĺļľŀł", 'l'),
        ("ñńņň", 'n'),
        ("òóôõöøōŏő", 'o'),
        ("ŕŗř", 'r'),
        ("śŝşš", 's'),
        ("ţťŧ", 't'),
        ("ùúûüũūŭůűų", 'u'),
        ("ŵ", 'w'),
        ("ýÿŷ", 'y'),
        ("źżž", 'z'),
    ];
    ACCENTED.iter().find(|(from, _)| from.contains(c)).map_or(c, |(_, to)| *to)
}

/// The words of a name: its runs of letters and figures (`partsOf`).
fn parts_of(s: &str) -> Vec<String> {
    fold(s).split(|c: char| !c.is_alphanumeric()).filter(|p| !p.is_empty()).map(str::to_string).collect()
}

/// A pattern compiled once, the first time it is used: the page asks for them on every frame.
macro_rules! once {
    ($name:ident, $source:expr) => {
        static $name: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| regex::Regex::new($source).expect("a fixed pattern"));
    };
}

once!(MACHINE_NAME, r"(?i)^page-\d{4}-\d{2}-\d{2}T[\d-]+Z?(?:[-_ ]v\d+)?\.\w+$|^screen ?shot[ -]\d{4}-\d{2}-\d{2}\b.*\.\w+$|^(?:[0-9a-f]{12,}|\d{10,})\.\w+$");
once!(SCRATCH_NAME, r"(?i)^page-\d{4}-\d{2}-\d{2}T[\d-]+Z?\.\w+$|(?:^|[-_ ])checks?\.\w+$|(?:^|[-_ ])contact[-_ ]?sheet\.\w+$");
once!(LINK, r"!?\[([^\]]*)\]\([^)]*\)");
once!(MARKS, r"(\*\*|__|`|~~)");
once!(ENDS, r"^[*_]|[*_]$");
once!(RULE, r"^[-*_]{3,}$");
once!(HEAD, r"^(#{1,6})\s+(.*)$");
once!(ITEM, r"^(?:[-*+]|\d+[.)])\s+(.*)$");
once!(QUOTE, r"^>\s*");

/// A page shot (`page-2026-10-31T17-42-40-037Z.png`), a Mac screenshot's name, a hash: a name
/// no person chose (`MACHINE_NAME`).
fn machine_name(name: &str) -> bool {
    MACHINE_NAME.is_match(name)
}

/// What a tile calls a file: the name a person gave it, else its own name, unless no person
/// chose that, when its thread's title is (`shownName`).
pub fn shown_name(f: &Placed) -> String {
    if let Some(title) = f.title.as_ref().filter(|t| !t.is_empty()) {
        return title.clone();
    }
    if !machine_name(&f.name) {
        return f.name.clone();
    }
    f.sent_in.iter().find(|s| s.kind != "chat" && !s.title.is_empty()).map_or_else(|| f.name.clone(), |s| s.title.clone())
}

/// What a search leaves (`filesShown`): each word starts a part of the name, the project or the
/// folder; the files named for a word come before, and instead of, the ones that only sit in a
/// project or folder of that word.
pub fn files_shown<'a>(files: &[&'a Placed], query: &str, kind: Option<&str>, folders: &[Folder]) -> Vec<&'a Placed> {
    let words = parts_of(query);
    let (mut named, mut placed) = (Vec::new(), Vec::new());
    for f in files {
        if kind.is_some_and(|k| f.kind.as_deref() != Some(k)) {
            continue;
        }
        let own = parts_of(&format!("{} {}", f.name, shown_name(f)));
        let folder = folders.iter().find(|x| Some(&x.id) == f.folder.as_ref()).map_or("", |x| x.name.as_str());
        let mut all = own.clone();
        all.extend(parts_of(&format!("{} {folder}", f.project.as_deref().unwrap_or(""))));
        let starts = |parts: &[String], w: &str| parts.iter().any(|p| p.starts_with(w));
        if !words.iter().all(|w| starts(&all, w)) {
            continue;
        }
        if words.is_empty() || words.iter().any(|w| starts(&own, w)) { named.push(*f) } else { placed.push(*f) }
    }
    if named.is_empty() { placed } else { named }
}

/// The kinds the files here are, in the page's order (`kindsIn`).
pub fn kinds_in(files: &[&Placed]) -> Vec<&'static str> {
    KIND_ORDER.into_iter().filter(|k| files.iter().any(|f| f.kind.as_deref() == Some(*k))).collect()
}

/// Who made the newest version (`changedBy`): a person, an agent by its name, or whose agent.
pub fn changed_by(f: &Placed, me: Option<&str>, name_of: impl Fn(&str) -> String) -> String {
    let mine = f.by.is_none() || f.by.as_deref() == me;
    match f.agent.as_deref() {
        None => if mine { "You".to_string() } else { name_of(f.by.as_deref().unwrap_or("")) },
        Some(agent) if !agent.is_empty() && agent != "Agent" => agent.to_string(),
        Some(_) => if mine { "Your agent".to_string() } else { format!("{}'s agent", name_of(f.by.as_deref().unwrap_or(""))) },
    }
}

/// One block of a document's top as a tile draws it (`docBlocks`).
#[derive(Clone, Debug, PartialEq)]
pub enum DocBlock {
    Title(String),
    Head(String),
    Words(String),
    Item(String),
}

/// The top of a markdown file as a tile draws it: its title, headings, paragraphs and list
/// items, tables and code left out, at most `most` blocks (`docBlocks`).
pub fn doc_blocks(text: &str, most: usize) -> Vec<DocBlock> {
    let (rule, head, item, quote) = (&*RULE, &*HEAD, &*ITEM, &*QUOTE);
    let plain = |s: &str| {
        let s = LINK.replace_all(s, "$1");
        let s = MARKS.replace_all(&s, "");
        ENDS.replace_all(&s, "").trim().to_string()
    };
    let mut out: Vec<DocBlock> = Vec::new();
    let (mut fenced, mut gap) = (false, true);
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if line.is_empty() {
            gap = true;
            continue;
        }
        if fenced || line.starts_with('|') || rule.is_match(line) {
            continue;
        }
        if let Some(m) = head.captures(line) {
            let words = plain(&m[2]);
            out.push(if m[1].len() == 1 && out.is_empty() { DocBlock::Title(words) } else { DocBlock::Head(words) });
        } else if let Some(m) = item.captures(line) {
            out.push(DocBlock::Item(plain(&m[1])));
        } else if let (Some(DocBlock::Words(last)), false) = (out.last_mut(), gap) {
            *last = format!("{last} {}", plain(&quote.replace(line, "")));
        } else {
            out.push(DocBlock::Words(plain(&quote.replace(line, ""))));
        }
        gap = false;
        if out.len() >= most {
            break;
        }
    }
    out.retain(|b| !matches!(b, DocBlock::Title(w) | DocBlock::Head(w) | DocBlock::Words(w) | DocBlock::Item(w) if w.is_empty()));
    out
}

/// `v3`, for a file with versions (`versionWord`).
pub fn version_word(f: &Placed) -> Option<String> {
    (f.versions > 1).then(|| format!("v{}", f.versions))
}

/// A picture an agent took to check its own work, that nobody is reviewing (`isScratch`).
pub fn is_scratch(f: &Placed) -> bool {
    f.kind.as_deref() == Some("image") && f.agent.is_some() && SCRATCH_NAME.is_match(&f.name) && f.versions <= 1 && f.open == 0 && f.approved_by.is_empty() && (f.folder.is_none() || f.folder_follows)
}

/// The files that wait under one line, and the rest (`apartScratch`).
pub fn apart_scratch(files: Vec<&Placed>) -> (Vec<&Placed>, Vec<&Placed>) {
    files.into_iter().partition(|f| !is_scratch(f))
}

pub fn scratch_line(n: usize) -> String {
    if n == 1 { "1 check picture an agent took".to_string() } else { format!("{n} check pictures agents took") }
}

/// An open comment on a file nobody approved (`needsReview`).
pub fn needs_review(f: &Placed) -> bool {
    f.open > 0 && f.approved_by.is_empty()
}

/// The order the page lists files in (`FileSort`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Sort {
    #[default]
    Recent,
    Name,
    Size,
}

/// Newest first, by the name a person reads whatever its capitals, or the biggest first
/// (`sortFiles`), leaving the list it was given alone.
pub fn sort_files<'a>(files: &[&'a Placed], by: Sort) -> Vec<&'a Placed> {
    let mut out = files.to_vec();
    match by {
        Sort::Name => out.sort_by(|a, b| compare_names(&shown_name(a), &shown_name(b))),
        Sort::Size => out.sort_by(|a, b| b.on_mac.unwrap_or(0.0).total_cmp(&a.on_mac.unwrap_or(0.0)).then(b.at.total_cmp(&a.at))),
        Sort::Recent => out.sort_by(|a, b| b.at.total_cmp(&a.at)),
    }
    out
}

/// `localeCompare` with `sensitivity: 'base'` and `numeric`: capitals and accents aside, and the
/// figures in a name read as the numbers they are.
fn compare_names(a: &str, b: &str) -> std::cmp::Ordering {
    let (a, b) = (fold(a), fold(b));
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let number = |it: &mut std::iter::Peekable<std::str::Chars>| {
                    let mut n = String::new();
                    while let Some(c) = it.peek().copied().filter(char::is_ascii_digit) {
                        n.push(c);
                        it.next();
                    }
                    n.trim_start_matches('0').to_string()
                };
                let (m, n) = (number(&mut x), number(&mut y));
                let by = m.len().cmp(&n.len()).then(m.cmp(&n));
                if by != std::cmp::Ordering::Equal {
                    return by;
                }
            }
            (Some(c), Some(d)) => {
                let by = c.cmp(&d);
                if by != std::cmp::Ordering::Equal {
                    return by;
                }
                x.next();
                y.next();
            }
        }
    }
}

/// A size as a person reads it: 625 MB, 2.1 MB, 4 KB (`sizeWords`).
pub fn size_words(bytes: f64) -> String {
    let units = ["KB", "MB", "GB", "TB"];
    let (mut n, mut u) = (bytes / 1024.0, 0);
    while n >= 1000.0 && u < units.len() - 1 {
        n /= 1024.0;
        u += 1;
    }
    if n < 1.0 {
        return "1 KB".to_string();
    }
    let shown = if n >= 100.0 || u == 0 { js_round(n).to_string() } else { trimmed(js_round(n * 10.0) as f64 / 10.0) };
    format!("{shown} {}", units[u])
}

fn js_round(v: f64) -> i64 {
    (v + 0.5).floor() as i64
}
fn trimmed(v: f64) -> String {
    if v.fract() == 0.0 { format!("{}", v as i64) } else { format!("{v}") }
}

/// What the files here take on this computer, all versions together (`onMacOf`).
pub fn on_mac_of(files: &[&Placed]) -> f64 {
    files.iter().map(|f| f.on_mac.unwrap_or(0.0)).sum()
}

/// Whether the page says that total at all: only once it reaches a megabyte (`showsTotal`).
pub fn shows_total(bytes: f64) -> bool {
    bytes >= 1024.0 * 1024.0
}

/// Where the page is: a project, or all of them, and a folder in it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Place {
    pub project: Option<String>,
    pub folder: Option<String>,
}

fn same_project(a: &str, b: &str) -> bool {
    fold(a) == fold(b)
}

/// The folders a place shows; All files shows projects instead (`foldersInPlace`).
pub fn folders_in_place<'a>(folders: &'a [Folder], place: &Place) -> Vec<&'a Folder> {
    let Some(project) = &place.project else { return Vec::new() };
    folders.iter().filter(|f| same_project(&f.project, project) && f.parent == place.folder).collect()
}

/// A project's top holds the files in no folder it can read; All files holds every file
/// (`filesInPlace`).
pub fn files_in_place<'a>(files: &'a [Placed], folders: &[Folder], place: &Place) -> Vec<&'a Placed> {
    let Some(project) = &place.project else { return files.iter().collect() };
    files
        .iter()
        .filter(|f| {
            if !same_project(f.project.as_deref().unwrap_or(""), project) {
                return false;
            }
            let at = f.folder.as_ref().filter(|id| folders.iter().any(|x| &x.id == *id));
            at == place.folder.as_ref()
        })
        .collect()
}

/// Everything a search looks through from here: this place and what is inside it (`filesBelow`).
pub fn files_below<'a>(files: &'a [Placed], folders: &[Folder], place: &Place) -> Vec<&'a Placed> {
    match (&place.project, &place.folder) {
        (None, _) => files.iter().collect(),
        (Some(project), None) => files.iter().filter(|f| same_project(f.project.as_deref().unwrap_or(""), project)).collect(),
        (Some(_), Some(folder)) => {
            let inside: Vec<&str> = std::iter::once(folder.as_str()).chain(folders.iter().filter(|f| f.parent.as_ref() == Some(folder)).map(|f| f.id.as_str())).collect();
            files.iter().filter(|f| f.folder.as_deref().is_some_and(|at| inside.contains(&at))).collect()
        }
    }
}

/// The way back, as the page spells it over its files (`crumbsTo`): each crumb's words and place.
pub fn crumbs_to(folders: &[Folder], place: &Place) -> Vec<(String, Place)> {
    let mut crumbs = vec![("All files".to_string(), Place::default())];
    let Some(project) = &place.project else { return crumbs };
    crumbs.push((project.clone(), Place { project: Some(project.clone()), folder: None }));
    let here = folders.iter().find(|f| Some(&f.id) == place.folder.as_ref());
    if let Some(above) = here.and_then(|h| h.parent.as_ref()).and_then(|p| folders.iter().find(|f| &f.id == p)) {
        crumbs.push((above.name.clone(), Place { project: Some(project.clone()), folder: Some(above.id.clone()) }));
    }
    if let Some(here) = here {
        crumbs.push((here.name.clone(), Place { project: Some(project.clone()), folder: Some(here.id.clone()) }));
    }
    crumbs
}

/// How many tiles are drawn at once, and the next step when the foot comes near: every tile at
/// once on a store of thousands froze the web window (`TILES_AT_ONCE`, `moreTiles`).
pub const TILES_AT_ONCE: usize = 60;
pub fn more_tiles(drawn: usize, total: usize) -> usize {
    if drawn >= total { drawn } else { (drawn + TILES_AT_ONCE).min(total) }
}

/// How long ago, as a tile says it: seconds, minutes, hours, days, then the date (`ago`).
pub fn ago(at: f64, now: f64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let s = ((now - at) / 1000.0).floor().max(0.0) as i64;
    if s < 60 {
        return format!("{s}s");
    }
    let m = s / 60;
    if m < 60 {
        return format!("{m}m");
    }
    let h = m / 60;
    if h < 24 {
        return format!("{h}h");
    }
    let d = h / 24;
    if d < 7 {
        return format!("{d}d");
    }
    let t = rune::time::LocalTime::at(at as i64);
    format!("{} {}", MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day)
}

/// Whether a picture fills its tile, or sits whole in it because filling would cut away too much
/// of it to read (`coverFit`): more than four tenths.
pub fn cover_fills(natural: (f32, f32), tile: (f32, f32)) -> bool {
    if !(natural.0 > 0.0 && natural.1 > 0.0 && tile.0 > 0.0 && tile.1 > 0.0) {
        return true;
    }
    let ratio = (natural.0 / natural.1) / (tile.0 / tile.1);
    let kept = if ratio > 1.0 { 1.0 / ratio } else { ratio };
    1.0 - kept <= 0.4
}

/// What the line under a tile says (`Tile`'s `words`): the version, who made it, how long ago
/// and its size here, each between dots.
pub fn tile_words(f: &Placed, now: f64, me: Option<&str>, name_of: impl Fn(&str) -> String) -> String {
    let mut words = Vec::new();
    words.extend(version_word(f));
    words.push(changed_by(f, me, name_of));
    if f.at > 0.0 {
        words.push(ago(f.at, now));
    }
    if let (false, Some(bytes)) = (f.cloud, f.bytes) {
        words.push(size_words(bytes));
    }
    words.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str) -> Placed {
        Placed { key: name.to_string(), project: Some("Lumen app".to_string()), name: name.to_string(), kind: Some("image".to_string()), versions: 1, at: 1.0, ..Placed::default() }
    }
    fn in_thread(mut f: Placed, title: &str) -> Placed {
        f.sent_in = vec![SentIn { kind: "thread".to_string(), title: title.to_string(), ..SentIn::default() }];
        f
    }

    // tests/the-files-page-sorts-filters-and-names-a-file-for-a-person.test.mts
    #[test]
    fn a_tiles_name_is_one_a_person_would_give() {
        assert_eq!(shown_name(&in_thread(file("page-2026-10-31T17-42-40-037Z-v5.png"), "Onboarding page")), "Onboarding page");
        assert_eq!(shown_name(&in_thread(file("page-2026-10-31T17-42-40-037Z.png"), "Onboarding page")), "Onboarding page");
        assert_eq!(shown_name(&in_thread(file("Screenshot 2026-10-31 at 17.42.40.png"), "Onboarding page")), "Onboarding page");
        assert_eq!(shown_name(&in_thread(file("onboarding-b-v2-built-v2.png"), "Onboarding page")), "onboarding-b-v2-built-v2.png");
        assert_eq!(shown_name(&in_thread(file("page-two.png"), "Onboarding page")), "page-two.png");
        assert_eq!(shown_name(&in_thread(file("plan-2026-10-31.md"), "Onboarding page")), "plan-2026-10-31.md");
        assert_eq!(shown_name(&file("page-2026-10-31T17-42-40-037Z.png")), "page-2026-10-31T17-42-40-037Z.png");
        let mut titled = file("a.png");
        titled.title = Some("The cover".to_string());
        assert_eq!(shown_name(&titled), "The cover");
    }

    #[test]
    fn the_total_is_said_only_once_it_reaches_a_megabyte() {
        assert!(!shows_total(0.0));
        assert!(!shows_total(53.0 * 1024.0));
        assert!(!shows_total(1024.0 * 1024.0 - 1.0));
        assert!(shows_total(1024.0 * 1024.0));
        assert!(shows_total(4.8 * 1024.0 * 1024.0 * 1024.0));
    }

    #[test]
    fn needs_review_is_an_open_comment_nobody_approved() {
        let mut f = file("a.png");
        f.open = 2;
        assert!(needs_review(&f));
        f.approved_by = vec![Some("m".to_string())];
        assert!(!needs_review(&f));
        assert!(!needs_review(&file("a.png")));
    }

    #[test]
    fn sorting_by_the_newest_by_name_and_by_size() {
        let mut b = file("b.png");
        b.at = 3.0;
        let mut c = file("C.png");
        c.at = 1.0;
        let mut a = file("a.png");
        a.at = 2.0;
        let list = [&b, &c, &a];
        let names = |v: Vec<&Placed>| v.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(sort_files(&list, Sort::Recent)), ["b.png", "a.png", "C.png"]);
        assert_eq!(names(sort_files(&list, Sort::Name)), ["a.png", "b.png", "C.png"]);
        assert_eq!(list[0].name, "b.png");
        // Figures in a name read as numbers.
        let (two, ten) = (file("shot-2.png"), file("shot-10.png"));
        assert_eq!(names(sort_files(&[&ten, &two], Sort::Name)), ["shot-2.png", "shot-10.png"]);
    }

    #[test]
    fn the_place_on_the_page() {
        let folders = vec![
            Folder { id: "f1".into(), project: "Lumen app".into(), name: "Launch".into(), parent: None },
            Folder { id: "f2".into(), project: "Lumen app".into(), name: "Assets".into(), parent: Some("f1".into()) },
            Folder { id: "f3".into(), project: "Plans".into(), name: "Launch".into(), parent: None },
        ];
        let at = |name: &str, folder: Option<&str>| Placed { folder: folder.map(str::to_string), ..file(name) };
        let files = vec![at("top.png", None), at("in.png", Some("f1")), at("deep.png", Some("f2")), at("gone.png", Some("not-here"))];
        let top = Place { project: Some("Lumen app".into()), folder: None };
        assert_eq!(folders_in_place(&folders, &top).iter().map(|f| f.id.as_str()).collect::<Vec<_>>(), ["f1"]);
        assert_eq!(files_in_place(&files, &folders, &top).iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["top.png", "gone.png"]);
        let inside = Place { project: Some("Lumen app".into()), folder: Some("f1".into()) };
        assert_eq!(folders_in_place(&folders, &inside).iter().map(|f| f.id.as_str()).collect::<Vec<_>>(), ["f2"]);
        assert_eq!(files_in_place(&files, &folders, &inside).iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["in.png"]);
        assert_eq!(files_in_place(&files, &folders, &Place::default()).len(), 4);
        assert_eq!(files_below(&files, &folders, &inside).iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["in.png", "deep.png"]);
        let deep = Place { project: Some("Lumen app".into()), folder: Some("f2".into()) };
        assert_eq!(crumbs_to(&folders, &deep).iter().map(|c| c.0.as_str()).collect::<Vec<_>>(), ["All files", "Lumen app", "Launch", "Assets"]);
        assert_eq!(crumbs_to(&folders, &Place::default()).len(), 1);
    }

    #[test]
    fn a_search_finds_names_before_places() {
        let mut plan = file("receipt-plan.md");
        plan.kind = Some("markdown".into());
        let mut shot = file("total-line.png");
        shot.project = Some("Receipts".into());
        let list = [&plan, &shot];
        let names = |v: Vec<&Placed>| v.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
        // "rec" starts a word of the plan's name, and only the project of the shot.
        assert_eq!(names(files_shown(&list, "rec", None, &[])), ["receipt-plan.md"]);
        assert_eq!(names(files_shown(&list, "receipts total", None, &[])), ["total-line.png"]);
        assert_eq!(names(files_shown(&list, "", Some("markdown"), &[])), ["receipt-plan.md"]);
        assert!(files_shown(&list, "nothing", None, &[]).is_empty());
        assert_eq!(kinds_in(&list), ["markdown", "image"]);
    }

    #[test]
    fn what_a_tile_says_of_a_file() {
        let mut f = file("total-line-v2.png");
        f.versions = 2;
        f.agent = Some(String::new());
        f.bytes = Some(1100.0);
        f.at = 1_000_000.0 - 6.0 * 60_000.0;
        assert_eq!(tile_words(&f, 1_000_000.0, None, |_| "Sofia".into()), "v2 · Your agent · 6m · 1 KB");
        f.by = Some("u-sofia".into());
        assert_eq!(changed_by(&f, Some("u-me"), |_| "Sofia".into()), "Sofia's agent");
        f.agent = None;
        assert_eq!(changed_by(&f, Some("u-me"), |_| "Sofia".into()), "Sofia");
        f.agent = Some("Ada".into());
        assert_eq!(changed_by(&f, Some("u-me"), |_| "Sofia".into()), "Ada");
        assert_eq!(size_words(53.0 * 1024.0), "53 KB");
        assert_eq!(size_words(2.1 * 1024.0 * 1024.0), "2.1 MB");
        assert_eq!(size_words(625.0 * 1024.0 * 1024.0), "625 MB");
        assert_eq!(size_words(10.0), "1 KB");
        assert_eq!(ago(0.0, 59_000.0), "59s");
        assert_eq!(ago(0.0, 3.0 * 3_600_000.0), "3h");
        assert_eq!(ago(0.0, 2.0 * 86_400_000.0), "2d");
    }

    #[test]
    fn a_documents_top_as_a_tile_draws_it() {
        let text = "# The receipt\n\nWhat the customer sees,\nline by line.\n\n## On it\n\n- The items\n- The **tax**\n\n```\ncode\n```\n\n| a | b |\n\nSee [the plan](plan.md).";
        assert_eq!(
            doc_blocks(text, 8),
            [
                DocBlock::Title("The receipt".into()),
                DocBlock::Words("What the customer sees, line by line.".into()),
                DocBlock::Head("On it".into()),
                DocBlock::Item("The items".into()),
                DocBlock::Item("The tax".into()),
                DocBlock::Words("See the plan.".into()),
            ]
        );
        assert_eq!(doc_blocks(text, 2).len(), 2);
    }

    #[test]
    fn a_picture_fills_its_tile_unless_too_much_would_be_cut() {
        assert!(cover_fills((1600.0, 1000.0), (256.5, 159.56)));
        assert!(!cover_fills((360.0, 64.0), (256.5, 159.56)));
        assert!(!cover_fills((900.0, 1950.0), (256.5, 159.56)));
        assert_eq!(more_tiles(60, 168), 120);
        assert_eq!(more_tiles(120, 168), 168);
        assert!(is_scratch(&Placed { agent: Some(String::new()), ..file("page-2026-10-31T17-42-40-037Z.png") }));
        assert!(!is_scratch(&file("page-2026-10-31T17-42-40-037Z.png")));
    }
}
