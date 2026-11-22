//! A file opened big (`ArtifactView` in `renderer/src/components/ArtifactView.tsx`): which files
//! the view holds and steps through, where a press lands among them, what the view calls the
//! file, who shared it, and how a picture in it fits and zooms.
//!
//! Ported from `shared/artifact-marks.mts` (`whereAPressLands`, `versionForPath`,
//! `stepsThrough`, `latestPicturesSaid`, `nextStep`, `familyName`, `sharedByWords`,
//! `withFilesAsked`), `shared/file-label.mts` (`labelSaidFor`),
//! `renderer/src/artifact-layout.ts` (`whereAFileOpens`), `renderer/src/picture-view.ts`
//! (`pictureFit`) and `renderer/src/artifact-pins.ts` (the zoom), with the web's test cases.
//!
//! A version line from Files (`numbers` in `threadFiles`) is not read: a thread's view numbers
//! its files' versions in the order its own lines name them, as the Files button does.

use serde_json::Value;

use super::js::{self, re};
use super::paths::decode_uri;
use super::roots;
use super::thread_files::{self, Family, Named, files_in_lines, other_files_in_lines, thread_files, view_kind};
use crate::talk::Line;

/// A place among a view's files: which file, and which of its versions.
pub type Step = (usize, usize);

/* ------------------------------ which press opens it ------------------------------ */

/// Whether a press on this file opens the big view (`fullScreenTakes`): a kind the view draws,
/// and not an address, which is a site or a local app's page. The web window opens what is to
/// be looked at (a picture, a film, a PDF, a page, a deck) full screen and what is to be read
/// (a document, a table, code) beside the conversation (`whereAFileOpens`); this window has no
/// pane beside a conversation, so both open big here. Anything else opens with the system.
pub fn opens_big(src: &str) -> bool {
    let address = src.starts_with("http://") || src.starts_with("https://");
    !address && view_kind(src).is_some()
}

/// The web window's place for a file pressed in a thread, for what a native press stands in
/// for: "full" over the window, "beside" the conversation, or neither (`whereAFileOpens`).
#[cfg(test)]
pub fn web_opens(src: &str) -> Option<&'static str> {
    if !opens_big(src) {
        return None;
    }
    let looked_at = matches!(view_kind(src), Some("image" | "film" | "pdf" | "page" | "deck"));
    Some(if looked_at { "full" } else { "beside" })
}

/* ------------------------------ the files it holds ------------------------------ */

/// A path as a press or a card hands it over: no `file://`, no query, its escapes read and no
/// leading `./` (`cleanAsked`).
fn clean_asked(raw: &str) -> String {
    let p = raw.strip_prefix("file://").unwrap_or(raw);
    let p = p.split(['?', '#']).next().unwrap_or("");
    let p = decode_uri(p);
    p.strip_prefix("./").map(str::to_string).unwrap_or(p)
}

/// Whether two paths are one file: the same, or one the end of the other (`endsWithPath`).
fn ends_with_path(whole: &str, part: &str) -> bool {
    whole == part || whole.strip_suffix(part).is_some_and(|head| head.ends_with('/'))
}

/// The files a view holds, from the lines that name them (`ArtifactView`'s `files`): every file
/// the big view reviews, and of the pages, tables and scripts, the one opened, or all of them
/// when the lines name nothing reviewed (Files opening one script or page). Then every path the
/// press handed over that the view can draw and that no version is yet (`withFilesAsked`).
pub fn files_for_view(lines: &[Line], opened: Option<&str>, asked: &[String], now: f64) -> Vec<Named> {
    let reviewed = files_in_lines(lines);
    let others = other_files_in_lines(lines);
    let mut said = reviewed.clone();
    if reviewed.is_empty() {
        said.extend(others);
    } else {
        // `otherFilesOpen`: the one opened, and one a note is on, which no note here is.
        let clean = opened.map(|p| p.strip_prefix("./").unwrap_or(p).to_string());
        said.extend(others.into_iter().filter(|f| Some(&f.path) == clean.as_ref()));
    }
    for raw in opened.into_iter().chain(asked.iter().map(String::as_str)) {
        let path = raw.strip_prefix("./").unwrap_or(raw);
        if !path.is_empty() && view_kind(path).is_some() && version_for_path(&thread_files(&said, now), path, now).is_none() {
            said.push(Named { path: path.to_string(), at: 0.0 });
        }
    }
    said
}

/// The version a path names, never another file's: the one that is that path or another way of
/// writing it, else the only one whose path ends with it or that it ends with, else the only
/// one of that name (`versionForPath`).
pub fn version_for_path(families: &[Family], raw: &str, now: f64) -> Option<Step> {
    let asked = clean_asked(raw);
    if asked.is_empty() {
        return None;
    }
    let all: Vec<(Step, &thread_files::Version)> = families.iter().enumerate().flat_map(|(fi, f)| f.versions.iter().enumerate().map(move |(vi, v)| ((fi, vi), v))).collect();
    let paths_of = |v: &thread_files::Version| std::iter::once(v.path.clone()).chain(v.also.iter().cloned()).collect::<Vec<_>>();
    if let Some((at, _)) = all.iter().find(|(_, v)| paths_of(v).contains(&asked)) {
        return Some(*at);
    }
    let tail: Vec<Step> = all.iter().filter(|(_, v)| paths_of(v).iter().any(|p| p.ends_with(&format!("/{asked}")) || asked.ends_with(&format!("/{p}")))).map(|(at, _)| *at).collect();
    if tail.len() == 1 {
        return tail.first().copied();
    }
    let name = roots::shown_name(&asked, now);
    let named: Vec<Step> = all.iter().filter(|(_, v)| v.name == name).map(|(at, _)| *at).collect();
    if named.len() == 1 { named.first().copied() } else { None }
}

/// Where a press opens (`whereAPressLands` with no note): the version its path names, never
/// another file, and with no path the newest version of the last file.
pub fn where_a_press_lands(families: &[Family], path: Option<&str>, now: f64) -> Option<Step> {
    match path {
        Some(path) => version_for_path(families, path, now),
        None => families.iter().enumerate().next_back().and_then(|(fi, f)| f.versions.len().checked_sub(1).map(|vi| (fi, vi))),
    }
}

/// What the arrows step through: one step per file `paths` name, in their order, at the newest
/// version they name. Versions are the header's to step, never the arrows' (`stepsThrough`).
pub fn steps_through(families: &[Family], paths: &[String], now: f64) -> Vec<Step> {
    let mut out: Vec<Step> = Vec::new();
    for p in paths {
        let Some((fi, vi)) = version_for_path(families, p, now) else { continue };
        match out.iter_mut().find(|s| s.0 == fi) {
            Some(had) => had.1 = had.1.max(vi),
            None => out.push((fi, vi)),
        }
    }
    out
}

/// The pictures the agent drew since the person last spoke, or in its last answer that drew
/// any (`latestPicturesSaid`).
pub fn latest_pictures_said(lines: &[Line]) -> Vec<String> {
    let pictures = |turn: &[Line]| -> Vec<String> { files_in_lines(turn).into_iter().filter(|f| thread_files::review_kind(&f.path) == Some("image")).map(|f| f.path).collect() };
    let mut turn: Vec<Line> = Vec::new();
    for l in lines.iter().rev() {
        if l.source == "founder" {
            if !pictures(&turn).is_empty() {
                break;
            }
            turn.clear();
        } else if l.source == "agent" {
            turn.insert(0, l.clone());
        }
    }
    pictures(&turn)
}

/// The pictures of the part of the thread a picture was pressed in, in the order it draws them:
/// each part is one of the person's turns and the agent's answers to it (`data-exchange`, which
/// `picturesFor` in `renderer/src/components/PictureView.tsx` reads off the page). The latest
/// part that draws the picture is the one, as a press finds the last door of its name.
pub fn turn_pictures(lines: &[Line], src: &str) -> Vec<String> {
    let clean = clean_asked(src);
    let mut parts: Vec<Vec<Line>> = vec![Vec::new()];
    for l in lines {
        let opens = l.source == "founder" && parts.last().is_some_and(|p| p.iter().any(|x| x.source != "founder"));
        if opens {
            parts.push(Vec::new());
        }
        if let Some(part) = parts.last_mut() {
            part.push(l.clone());
        }
    }
    for part in parts.iter().rev() {
        let pictures: Vec<String> = files_in_lines(part).into_iter().filter(|f| thread_files::review_kind(&f.path) == Some("image")).map(|f| f.path).collect();
        if pictures.iter().any(|p| ends_with_path(p, &clean) || ends_with_path(&clean, p)) {
            return pictures;
        }
    }
    Vec::new()
}

/// Where an arrow from `at` goes, or None at an end (`nextStep`). With steps, the next file in
/// them, whichever version of this one is open; without, the next file at its newest version.
pub fn next_step(families: &[Family], steps: &[Step], at: Step, by: isize) -> Option<Step> {
    if !steps.is_empty() {
        let i = steps.iter().position(|s| s.0 == at.0)?;
        return i.checked_add_signed(by).and_then(|j| steps.get(j)).copied();
    }
    let fi = at.0.checked_add_signed(by)?;
    families.get(fi).and_then(|f| f.versions.len().checked_sub(1).map(|vi| (fi, vi)))
}

/// What the arrows of a view opened at `at` step through (`steps` in `ArtifactView`): the
/// pictures the press handed over, else the agent's latest pictures when the file is one of
/// them, else nothing, and then every file in turn.
pub fn arrows_through(families: &[Family], handed: &[String], latest: &[String], at: Option<Step>, now: f64) -> Vec<Step> {
    let handed = steps_through(families, handed, now);
    if !handed.is_empty() {
        return handed;
    }
    let latest = steps_through(families, latest, now);
    if at.is_some_and(|at| latest.iter().any(|s| s.0 == at.0)) { latest } else { Vec::new() }
}

/// Whether the view draws its arrows: more than one step, or with none, more than one file.
pub fn draws_arrows(families: &[Family], steps: &[Step]) -> bool {
    (if steps.is_empty() { families.len() } else { steps.len() }) > 1
}

/// What the bar calls a file: with versions, its first version's name without the version
/// (`familyName`).
pub fn family_name(family: &Family) -> String {
    match family.versions.as_slice() {
        [first, _, ..] => thread_files::base_name(&first.name),
        [only] => only.name.clone(),
        [] => String::new(),
    }
}

/* ------------------------------ who shared it, and when ------------------------------ */

/// Who put a version on the thread, as a line names it: the person (`by`), and the agent that
/// spoke for them, `Some("")` for a plain agent, or None when a person did it themselves.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SharedBy {
    pub by: Option<String>,
    pub agent: Option<String>,
    pub edited: bool,
    pub made: bool,
}

/// Who shared the version at `path`, read off the first line that names it, as `filesInLines`
/// keeps it with the path.
pub fn shared_by(lines: &[Line], path: &str) -> SharedBy {
    for l in lines {
        let Some(patch) = &l.patch else { continue };
        let by = l.by.clone();
        if let Some(edit) = patch.get("artifact").filter(|a| a.get("do").and_then(Value::as_str) == Some("edit"))
            && edit.get("path").and_then(Value::as_str) == Some(path)
        {
            return SharedBy { by, agent: None, edited: true, made: false };
        }
        if files_in_lines(std::slice::from_ref(l)).iter().any(|f| f.path == path) {
            let voice = patch.get("voice").and_then(|v| v.get("name")).and_then(Value::as_str).map(str::to_string);
            let agent = voice.or_else(|| (l.source == "agent").then(String::new));
            let made = patch.get("made").and_then(Value::as_bool) == Some(true);
            return SharedBy { by, agent, edited: false, made };
        }
    }
    SharedBy::default()
}

/// "Shared by your agent", "Edited by you" (`sharedByWords`). `me` is the person's own id, and
/// `name_of` names anyone else.
pub fn shared_by_words(v: &SharedBy, me: Option<&str>, name_of: impl Fn(&str) -> String) -> String {
    let whose = v.by.as_deref().filter(|by| Some(*by) != me).map(&name_of);
    if v.edited {
        return format!("Edited by {}", whose.as_deref().unwrap_or("you"));
    }
    if v.made {
        return format!("Made by {}", whose.as_deref().unwrap_or("you"));
    }
    match v.agent.as_deref() {
        None => format!("Shared by {}", whose.as_deref().unwrap_or("you")),
        // A thread that is no AI teammate's speaks as plain "Agent", which names nobody.
        Some(agent) if !agent.is_empty() && agent != "Agent" => format!("Shared by {agent} for {}", whose.as_deref().unwrap_or("you")),
        Some(_) => match whose {
            Some(whose) => format!("Shared by {whose}'s agent"),
            None => "Shared by your agent".to_string(),
        },
    }
}

/// When a version was shared: "today at 1:19pm", else its day, "Jan 5" (`whenShared`).
pub fn when_shared(at: f64, now: f64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let (t, today) = (rune::time::LocalTime::at(at as i64), rune::time::LocalTime::at(now as i64));
    if (t.year, t.month, t.day) == (today.year, today.month, today.day) {
        let hour = if t.hour % 12 == 0 { 12 } else { t.hour % 12 };
        return format!("today at {hour}:{:02}{}", t.minute, if t.hour < 12 { "am" } else { "pm" });
    }
    format!("{} {}", MONTHS[(t.month as usize).clamp(1, 12) - 1], t.day)
}

/* ------------------------------ what the thread called it ------------------------------ */

const LABEL_EXT: &str = "png|jpe?g|gif|webp|svg|mp4|mov|m4v|webm|pdf|md|markdown|pptx|html?|csv|tsv";
fn label_pathish() -> String {
    format!(r"(?:\.?\/)?[\w.~-]+(?:\/[\w.~-]+)*\.(?:{LABEL_EXT})")
}
// A link or picture, with its words; a quoted name; a bare path.
re!(MENTION, &format!(r"(!?)\[([^\]]*)\]\(<?([^()\s>]+)>?\)|`({})`|(?<![\w./:@~(\[-])({})(?![\w/-])", label_pathish(), label_pathish()), "i");
re!(FILE_NAME, &format!(r"^[\w.~-]+\.(?:{LABEL_EXT})$"), "i");
re!(IS_FILE, &format!(r"\.(?:{LABEL_EXT})$"), "i");
re!(LABEL_FENCE, r"```.*?(?:```|$)", "s");
re!(LEAD, r"^\s*(?:#{1,6}\s+|>\s*|[-*+]\s+|\d+[.)]\s+)*", "");
re!(MARKS, r"\*\*|__|`", "");
re!(OPEN_MARK, r"(^|\s)[*_](\S)", "");
re!(CLOSE_MARK, r"(\S)[*_](?=\s|$|[.,:;])", "");
re!(EMPTY_BRACKETS, r"\(\s*\)|\[\s*\]", "");
re!(SPACES, r"\s+", "");
re!(EDGES, r"^[\s:\u2014\u2013-]+|[\s:\u2014\u2013-]+$", "");
re!(LETTER, r"\p{L}", "u");
/// Longer than this is a sentence about the work, not a name for one picture.
const LABEL_MOST: usize = 120;

fn same_path(a: &str, b: &str) -> bool {
    let x = a.strip_prefix("./").unwrap_or(a);
    let y = b.strip_prefix("./").unwrap_or(b);
    x == y || x.ends_with(&format!("/{y}")) || y.ends_with(&format!("/{x}"))
}

/// Words as a label reads them: what leads a heading, a quote or a list item, markdown's marks,
/// empty brackets and the dashes and colons at either end taken off (`tidy`).
fn tidy(words: &str) -> String {
    let w = LEAD.replace(words, |_| String::new());
    let w = MARKS.replace_all(&w, |_| String::new());
    let w = OPEN_MARK.replace_all(&w, |m| format!("{}{}", js::group(m, 1), js::group(m, 2)));
    let w = CLOSE_MARK.replace_all(&w, |m| js::group(m, 1).to_string());
    let w = EMPTY_BRACKETS.replace_all(&w, |_| String::new());
    let w = SPACES.replace_all(&w, |_| " ".to_string());
    EDGES.replace_all(&w, |_| String::new())
}

/// One line: the files it names, and its words with those names taken out, a link's own words
/// kept unless they are a file name (`readLine`).
fn read_line(line: &str) -> (Vec<String>, String) {
    let paths = std::cell::RefCell::new(Vec::new());
    let words = MENTION.replace_all(line, |m| {
        let path = m.get(3).or(m.get(4)).or(m.get(5)).unwrap_or("");
        let is_file = IS_FILE.test(path.split(['?', '#']).next().unwrap_or(""));
        if is_file {
            paths.borrow_mut().push(path.to_string());
        }
        // A link's words stay, unless they are only a file's name; a quoted or bare name goes.
        if m.get(3).is_some() {
            let said = js::trim(m.get(2).unwrap_or(""));
            if FILE_NAME.test(said) || (said.is_empty() && is_file) { String::new() } else { m.get(2).unwrap_or("").to_string() }
        } else if is_file {
            String::new()
        } else {
            js::group(m, 0).to_string()
        }
    });
    (paths.into_inner(), tidy(&words))
}

fn is_label(words: &str) -> bool {
    js::len(words) <= LABEL_MOST && LETTER.matches(words).len() >= 2
}

/// The label one message gives the file at `paths` (`labelIn`).
fn label_in(text: &str, paths: &[&str]) -> Option<String> {
    let unfenced = LABEL_FENCE.replace_all(text, |m| "\n".repeat(js::group(m, 0).matches('\n').count()));
    let rows: Vec<(Vec<String>, String)> = unfenced.split('\n').map(read_line).collect();
    let ours = rows.iter().position(|(found, _)| found.iter().any(|p| paths.iter().any(|q| same_path(p, q))))?;
    let own = &rows[ours].1;
    if !own.is_empty() {
        return is_label(own).then(|| own.clone());
    }
    // The nearest line of words above or below a line that only names files, never past another file.
    let near = |i: usize, by: isize| -> Option<String> {
        let mut j = i.checked_add_signed(by)?;
        while rows.get(j).is_some_and(|(p, w)| w.is_empty() && p.is_empty()) {
            j = j.checked_add_signed(by)?;
        }
        let (p, w) = rows.get(j)?;
        (p.is_empty() && is_label(w)).then(|| w.clone())
    };
    // Whether this message puts each picture's words over it or under it, by what most of its
    // pictures have. Words right against a picture say so more surely than words a blank line away.
    let pictures: Vec<usize> = rows.iter().enumerate().filter(|(_, (p, w))| !p.is_empty() && w.is_empty()).map(|(i, _)| i).collect();
    let weigh = |by: isize| -> usize {
        pictures.iter().map(|&i| if near(i, by).is_some() { if i.checked_add_signed(by).and_then(|j| rows.get(j)).is_some_and(|(_, w)| !w.is_empty()) { 2 } else { 1 } } else { 0 }).sum()
    };
    let (over, under) = (weigh(-1), weigh(1));
    if under > over { near(ours, 1).or_else(|| near(ours, -1)) } else { near(ours, -1).or_else(|| near(ours, 1)) }
}

/// What the thread's messages called the file at any of `paths`, from the first agent message
/// that gives it a label, then anyone's (`labelSaidFor`).
pub fn label_said_for(lines: &[Line], paths: &[&str]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    let said: Vec<&Line> = lines.iter().filter(|l| l.patch.is_some()).collect();
    let agents = said.iter().filter(|l| l.source == "agent");
    let others = said.iter().filter(|l| l.source != "agent");
    for l in agents.chain(others) {
        let patch = l.patch.as_ref()?;
        for field in ["result", "answer", "note", "body"] {
            if let Some(found) = patch.get(field).and_then(Value::as_str).and_then(|text| label_in(text, paths)) {
                return Some(found);
            }
        }
    }
    None
}

/* ------------------------------ a picture's fit and zoom ------------------------------ */

/// Zoom is counted from the size the picture opened at; this is as far in as it goes.
pub const MAX_ZOOM: f32 = 8.0;
const STOPS: [f32; 7] = [1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0];

/// Fitted by its height, a picture narrower than this share of the box is a strip nobody can read.
const STRIP: f32 = 0.6;
// A phone screenshot, 16:9 to 19.5:9 with room for a status bar. Longer is a scrolled page.
const PHONE_SHORTEST: f32 = 1.6;
const PHONE_TALLEST: f32 = 2.4;
/// Drawn this wide, a phone screen's words are near their own size.
const PHONE_READABLE: f32 = 300.0;

/// How a picture opens in a box (`pictureFit`): the scale it opens at, whether it is a tall one
/// fitted to the width that scrolls, and the scale that shows all of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fit {
    pub scale: f32,
    pub tall: bool,
    pub whole: f32,
}

/// A tall picture fits the width and scrolls, never past its real pixels, unless it is a phone
/// screen that reads whole (`pictureFit`).
pub fn picture_fit(natural: (f32, f32), room: (f32, f32)) -> Fit {
    if !(natural.0 > 0.0 && natural.1 > 0.0 && room.0 > 0.0 && room.1 > 0.0) {
        return Fit { scale: 1.0, tall: false, whole: 1.0 };
    }
    let by_width = (room.0 / natural.0).min(1.0);
    let whole = by_width.min(room.1 / natural.1);
    let drawn = natural.0 * whole;
    let aspect = natural.1 / natural.0;
    let phone = (PHONE_SHORTEST..=PHONE_TALLEST).contains(&aspect) && drawn >= natural.0.min(PHONE_READABLE);
    let tall = natural.1 * by_width > room.1 && drawn < STRIP * room.0 && !phone;
    Fit { scale: if tall { by_width } else { whole }, tall, whole }
}

/// The zoom, counted from the opening scale, that shows the whole picture (`lowestZoom`).
pub fn lowest_zoom(fit: Fit) -> f32 {
    if fit.scale > 0.0 { (fit.whole / fit.scale).min(1.0) } else { 1.0 }
}

/// The next stop in or out from `zoom` (`nextZoom`). `floor` is the zoom that shows the whole
/// picture, below 1 when it opened fitted to its width.
pub fn next_zoom(zoom: f32, inward: bool, floor: f32) -> f32 {
    let mut stops: Vec<f32> = STOPS.to_vec();
    if floor < 1.0 {
        stops.insert(0, floor);
    }
    if inward {
        return stops.iter().copied().find(|z| *z > zoom + 0.01).unwrap_or(MAX_ZOOM);
    }
    stops.iter().rev().copied().find(|z| *z < zoom - 0.01).unwrap_or(floor.min(1.0))
}

/// A pinch's or a ⌘-wheel's zoom (`wheelZoom`).
pub fn wheel_zoom(zoom: f32, dy: f32, floor: f32) -> f32 {
    (zoom * (-dy * 0.01).exp()).clamp(floor.min(1.0), MAX_ZOOM)
}

/// The zoom that shows the picture at its own pixels (`realZoom`).
pub fn real_zoom(fit: f32) -> f32 {
    if fit > 0.0 && fit < 1.0 { (1.0 / fit).min(MAX_ZOOM) } else { 1.0 }
}

/// What the zoom's pill says: the share of the picture's own size it is drawn at (`zoomWords`).
pub fn zoom_words(fit: f32, zoom: f32) -> String {
    format!("{}%", js_round(fit * zoom * 100.0))
}

/// `Math.round`: halves go up.
fn js_round(v: f32) -> i64 {
    (v + 0.5).floor() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_795_000_000_000.0;

    fn said(ts: f64, text: &str) -> Line {
        serde_json::from_value(json!({ "ts": ts, "source": "agent", "patch": { "result": text } })).unwrap()
    }
    fn you(ts: f64, text: &str) -> Line {
        serde_json::from_value(json!({ "ts": ts, "source": "founder", "patch": { "answer": text } })).unwrap()
    }
    fn families(lines: &[Line]) -> Vec<Family> {
        thread_files(&files_in_lines(lines), NOW)
    }
    fn paths(v: &[&str]) -> Vec<String> {
        v.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn what_a_press_opens_big() {
        for src in ["designs/a.png", "notes/plan.md", "reports/week.pdf", "pages/checkout.html", "exports/rows.csv", "scripts/run.ts", "media/demo.mp4", "deck.pptx"] {
            assert!(opens_big(src), "{src}");
        }
        for src in ["archive.zip", "https://example.com/a.png", "http://localhost:3000/a.html", "notes"] {
            assert!(!opens_big(src), "{src}");
        }
        // The web window opens what is looked at full screen and what is read beside the chat.
        assert_eq!(web_opens("designs/a.png"), Some("full"));
        assert_eq!(web_opens("pages/checkout.html"), Some("full"));
        assert_eq!(web_opens("notes/plan.md"), Some("beside"));
        assert_eq!(web_opens("exports/rows.csv"), Some("beside"));
        assert_eq!(web_opens("scripts/run.ts"), Some("beside"));
        assert_eq!(web_opens("archive.zip"), None);
    }

    // tests/a-big-picture-steps-through-its-own-turn-in-order.test.mts
    #[test]
    fn a_big_picture_steps_through_its_own_turn_in_order() {
        let lines = [
            said(1.0, "![](designs/r1/mock.png)\n![](designs/r1/other.png)"),
            said(2.0, "The next version of [tag-other-icon.png](designs/r2/tag-other-icon.png) is [tag-other-icon-v2.png](designs/r2/tag-other-icon-v2.png).\n\n![](designs/r2/tag-other-icon.png) ![](designs/r2/tag-other-icon-v2.png)\n\n![](designs/r2/tag-other-icon-v2.png)\n\n![](designs/r1/mock.png)"),
        ];
        let fam = families(&lines);
        let name_at = |s: Step| fam[s.0].versions[s.1].path.clone();
        let turn = paths(&["designs/r2/tag-other-icon.png", "designs/r2/tag-other-icon-v2.png", "designs/r1/mock.png"]);
        let steps = steps_through(&fam, &turn, NOW);
        assert_eq!(steps.iter().map(|s| name_at(*s)).collect::<Vec<_>>(), ["designs/r2/tag-other-icon-v2.png", "designs/r1/mock.png"]);
        let back = steps_through(&fam, &paths(&["designs/r1/mock.png", "designs/r2/tag-other-icon-v2.png"]), NOW);
        assert_eq!(back.iter().map(|s| name_at(*s)).collect::<Vec<_>>(), ["designs/r1/mock.png", "designs/r2/tag-other-icon-v2.png"]);
        let once = steps_through(&fam, &paths(&["designs/r2/tag-other-icon.png", "./designs/r2/tag-other-icon.png", "designs/nowhere.png"]), NOW);
        assert_eq!(once.iter().map(|s| name_at(*s)).collect::<Vec<_>>(), ["designs/r2/tag-other-icon.png"]);
        assert!(steps_through(&fam, &[], NOW).is_empty());
        // An arrow from the v1 goes past the v2 beside it, on to the next picture.
        let opened = where_a_press_lands(&fam, Some("designs/r2/tag-other-icon.png"), NOW).unwrap();
        assert_eq!(name_at(next_step(&fam, &steps, opened, 1).unwrap()), "designs/r1/mock.png");
    }

    // tests/the-arrows-in-a-picture-walk-the-agents-latest-pictures-never-their-versions.test.mts
    #[test]
    fn the_arrows_walk_the_agents_latest_pictures_never_their_versions() {
        let lines = [
            said(1.0, "![](designs/r1/old.png)\n![](designs/r1/tour.png)"),
            you(2.0, "Another round please"),
            said(3.0, "Working on it. ![](designs/r2/tour.png)"),
            said(4.0, "The next version of [tour.png](designs/r2/tour.png) is [tour-v2.png](designs/r2/tour-v2.png).\n\n![](designs/r2/tour.png) ![](designs/r2/tour-v2.png)\n\n![](designs/r2/board.png)\n\n![](designs/r2/board-v2.png)\n\n![](designs/r2/menu.png)"),
        ];
        let fam = families(&lines);
        let name_at = |s: Option<Step>| s.map(|s| fam[s.0].versions[s.1].path.clone());
        let turn = paths(&["designs/r2/tour.png", "designs/r2/tour-v2.png", "designs/r2/board.png", "designs/r2/board-v2.png", "designs/r2/menu.png"]);
        let steps = steps_through(&fam, &turn, NOW);
        assert_eq!(steps.iter().map(|s| name_at(Some(*s)).unwrap()).collect::<Vec<_>>(), ["designs/r2/tour-v2.png", "designs/r2/board-v2.png", "designs/r2/menu.png"]);
        let at = |p: &str| where_a_press_lands(&fam, Some(p), NOW).unwrap();
        assert_eq!(name_at(next_step(&fam, &steps, at("designs/r2/tour-v2.png"), 1)).as_deref(), Some("designs/r2/board-v2.png"));
        assert_eq!(name_at(next_step(&fam, &steps, at("designs/r2/board-v2.png"), -1)).as_deref(), Some("designs/r2/tour-v2.png"));
        assert_eq!(name_at(next_step(&fam, &steps, at("designs/r2/tour.png"), 1)).as_deref(), Some("designs/r2/board-v2.png"));
        assert_eq!(name_at(next_step(&fam, &steps, at("designs/r2/board.png"), 1)).as_deref(), Some("designs/r2/menu.png"));
        assert_eq!(next_step(&fam, &steps, at("designs/r2/tour.png"), -1), None);
        assert_eq!(next_step(&fam, &steps, at("designs/r2/menu.png"), 1), None);
        // Opened from anywhere else, a picture steps through the agent's latest answer when it is one of them.
        let latest = latest_pictures_said(&lines);
        assert_eq!(latest, ["designs/r2/tour.png", "designs/r2/tour-v2.png", "designs/r2/board.png", "designs/r2/board-v2.png", "designs/r2/menu.png"]);
        assert_eq!(arrows_through(&fam, &[], &latest, Some(at("designs/r2/menu.png")), NOW), steps);
        assert!(arrows_through(&fam, &[], &latest, Some(at("designs/r1/old.png")), NOW).is_empty());
    }

    #[test]
    fn a_picture_pressed_steps_through_the_part_of_the_thread_it_is_in() {
        let lines = [
            said(1.0, "![](designs/r1/mock.png) and designs/r1/other.png"),
            you(2.0, "Again, with the cart"),
            said(3.0, "Compare shots/cart-a.png, shots/cart-b.png and shots/cart-c.png."),
            said(4.0, "And the old one: designs/r1/mock.png"),
        ];
        assert_eq!(turn_pictures(&lines, "shots/cart-b.png"), ["shots/cart-a.png", "shots/cart-b.png", "shots/cart-c.png", "designs/r1/mock.png"]);
        // A picture both parts draw belongs to the latest; one only the first drew, to the first.
        assert_eq!(turn_pictures(&lines, "./designs/r1/mock.png")[0], "shots/cart-a.png");
        assert_eq!(turn_pictures(&lines, "/Users/you/p/designs/r1/other.png"), ["designs/r1/mock.png", "designs/r1/other.png"]);
        assert!(turn_pictures(&lines, "designs/nowhere.png").is_empty());
    }

    #[test]
    fn a_press_opens_exactly_the_version_it_names_or_nothing() {
        let lines = [said(1.0, "Drew designs/a/shot.png and designs/b/shot.png, then [the note](notes/plan.md).")];
        let fam = families(&lines);
        // Two pictures of one name in two folders are two versions, each opened by its own path.
        let a = where_a_press_lands(&fam, Some("designs/a/shot.png"), NOW).unwrap();
        let b = where_a_press_lands(&fam, Some("/Users/you/project/designs/b/shot.png"), NOW).unwrap();
        assert_eq!((fam[a.0].versions[a.1].path.as_str(), fam[b.0].versions[b.1].path.as_str()), ("designs/a/shot.png", "designs/b/shot.png"));
        // A name the thread never said opens nothing; with no path, the last file's newest.
        assert_eq!(where_a_press_lands(&fam, Some("designs/c/other.png"), NOW), None);
        assert_eq!(where_a_press_lands(&fam, None, NOW), Some((1, 0)));
    }

    #[test]
    fn a_script_files_open_alone_is_held_and_one_the_lines_never_named_is_added() {
        // Files opening a script: its lines name nothing reviewed, so the script is the view's.
        let lines = [said(1.0, "[round.ts](/p/scripts/round.ts)")];
        let held = files_for_view(&lines, None, &[], NOW);
        assert_eq!(held.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), ["/p/scripts/round.ts"]);
        // Beside a picture, a script is held only when it is the one opened.
        let lines = [said(1.0, "Drew designs/a.png with `scripts/round.ts`.")];
        assert_eq!(files_for_view(&lines, None, &[], NOW).len(), 1);
        assert_eq!(files_for_view(&lines, Some("scripts/round.ts"), &[], NOW).len(), 2);
        // A picture a turn drew that no line names is held too, at the end.
        let held = files_for_view(&lines, Some("designs/a.png"), &paths(&["designs/b.png"]), NOW);
        assert_eq!(held.iter().map(|f| f.path.as_str()).collect::<Vec<_>>(), ["designs/a.png", "designs/b.png"]);
    }

    #[test]
    fn the_bar_names_a_file_with_versions_by_its_first_version_without_the_version() {
        let fam = families(&[said(1.0, "designs/total-line.png, then designs/total-line-v2.png")]);
        assert_eq!(fam.len(), 1);
        assert_eq!(family_name(&fam[0]), "total-line.png");
        let one = families(&[said(1.0, "designs/receipt.png")]);
        assert_eq!(family_name(&one[0]), "receipt.png");
    }

    #[test]
    fn who_shared_a_version_and_when() {
        let me = Some("u-me");
        let name = |id: &str| if id == "u-sofia" { "Sofia".to_string() } else { "A teammate".to_string() };
        let agent = SharedBy { by: None, agent: Some(String::new()), ..SharedBy::default() };
        assert_eq!(shared_by_words(&agent, me, name), "Shared by your agent");
        let hers = SharedBy { by: Some("u-sofia".into()), agent: Some(String::new()), ..SharedBy::default() };
        assert_eq!(shared_by_words(&hers, me, name), "Shared by Sofia's agent");
        let named = SharedBy { by: Some("u-me".into()), agent: Some("Ada".into()), ..SharedBy::default() };
        assert_eq!(shared_by_words(&named, me, name), "Shared by Ada for you");
        assert_eq!(shared_by_words(&SharedBy { agent: Some("Agent".into()), ..SharedBy::default() }, me, name), "Shared by your agent");
        assert_eq!(shared_by_words(&SharedBy::default(), me, name), "Shared by you");
        assert_eq!(shared_by_words(&SharedBy { edited: true, ..SharedBy::default() }, me, name), "Edited by you");
        // Read off the first line that names the path, an agent's.
        let lines = [said(5.0, "The plan is in [the plan](notes/plan.md).")];
        assert_eq!(shared_by(&lines, "notes/plan.md"), SharedBy { by: None, agent: Some(String::new()), edited: false, made: false });
    }

    // tests/a-big-picture-shows-what-the-chat-called-it.test.mts
    #[test]
    fn a_big_picture_shows_what_the_chat_called_it() {
        let label = |text: &str, path: &str| label_said_for(&[said(1.0, text)], &[path]);
        let text = "Here is the change.\n\nBefore:\n/docs/designs/w-1/1-names.png\n\nAfter:\n/docs/designs/w-1/2-face-only.png";
        assert_eq!(label(text, "designs/w-1/1-names.png").as_deref(), Some("Before"));
        assert_eq!(label(text, "designs/w-1/2-face-only.png").as_deref(), Some("After"));
        let text = "**Option 1: faces only**\n\n![](designs/a/1.png)\n\n**Option 2: names beside faces**\n\n![](designs/a/2.png)";
        assert_eq!(label(text, "designs/a/1.png").as_deref(), Some("Option 1: faces only"));
        assert_eq!(label(text, "designs/a/2.png").as_deref(), Some("Option 2: names beside faces"));
        assert_eq!(label("1. Faces only: designs/a/1.png", "designs/a/1.png").as_deref(), Some("Faces only"));
        assert_eq!(label("![The before](designs/a/1.png)", "designs/a/1.png").as_deref(), Some("The before"));
        let text = "Two takes, stacked.\n\n/designs/a/1.png\nOption 1 keeps the names.\n\n/designs/a/2.png\nOption 2 drops them.";
        assert_eq!(label(text, "designs/a/1.png").as_deref(), Some("Option 1 keeps the names."));
        assert_eq!(label(text, "designs/a/2.png").as_deref(), Some("Option 2 drops them."));
        assert_eq!(label_said_for(&[said(1.0, "Before:\ndesigns/a/1.png")], &["/Users/x/docs/designs/a/1.png"]).as_deref(), Some("Before"));
        assert_eq!(label("![1.png](designs/a/1.png)", "designs/a/1.png"), None);
        let prose = "I went through every screen and moved the faces to the left of each row, so the list reads by who wrote it first and what it says second, which is what you asked for.";
        assert_eq!(label(&format!("{prose}\n\n/designs/a/1.png"), "designs/a/1.png"), None);
        assert_eq!(label("Before:\n/designs/a/1.png\n/designs/a/2.png", "designs/a/2.png"), None);
        assert_eq!(label("Before:\n/designs/a/1.png", "designs/a/9.png"), None);
        assert_eq!(label("Before:\n![2-face-only.png](designs/a/2-face-only.png)", "designs/a/2-face-only.png").as_deref(), Some("Before"));
        let lines = [said(1.0, "After:\n/designs/a/2.png"), said(2.0, "I updated designs/a/2.png as you asked.")];
        assert_eq!(label_said_for(&lines, &["designs/a/2.png"]).as_deref(), Some("After"));
        // A sentence that names it keeps its own words, the name taken out.
        assert_eq!(label("The total sits on one line again, as in designs/t.png, with the tax under it.", "designs/t.png").as_deref(), Some("The total sits on one line again, as in , with the tax under it."));
    }

    // tests/a-tall-picture-opens-fit-to-width-and-zooms-where-you-look.test.mts
    #[test]
    fn a_picture_opens_fitted_and_a_tall_one_fitted_to_its_width() {
        // A small picture is drawn at its own size, never blown up.
        assert_eq!(picture_fit((360.0, 64.0), (836.0, 668.0)), Fit { scale: 1.0, tall: false, whole: 1.0 });
        // A wide one fits the box.
        let wide = picture_fit((2400.0, 1350.0), (836.0, 668.0));
        assert!((wide.scale - 836.0 / 2400.0).abs() < 1e-6 && !wide.tall);
        // A long page fits its width and scrolls; the zoom that shows it whole is below 1.
        let page = picture_fit((1200.0, 6000.0), (836.0, 668.0));
        assert!(page.tall && (page.scale - 836.0 / 1200.0).abs() < 1e-6);
        assert!(lowest_zoom(page) < 1.0);
        // A phone screen reads whole.
        let phone = picture_fit((900.0, 1950.0), (836.0, 668.0));
        assert!(!phone.tall && (phone.scale - 668.0 / 1950.0).abs() < 1e-6);
    }

    #[test]
    fn the_zoom_steps_and_says_its_size() {
        assert_eq!(next_zoom(1.0, true, 1.0), 1.5);
        assert_eq!(next_zoom(8.0, true, 1.0), 8.0);
        assert_eq!(next_zoom(1.5, false, 1.0), 1.0);
        assert_eq!(next_zoom(1.0, false, 1.0), 1.0);
        assert_eq!(next_zoom(1.0, false, 0.4), 0.4);
        assert_eq!(real_zoom(0.25), 4.0);
        assert_eq!(real_zoom(1.0), 1.0);
        assert_eq!(zoom_words(1.0, 1.0), "100%");
        assert_eq!(zoom_words(0.8355, 1.0), "84%");
        assert!((wheel_zoom(1.0, -100.0, 1.0) - std::f32::consts::E).abs() < 1e-4);
        assert_eq!(wheel_zoom(1.0, 1000.0, 1.0), 1.0);
    }

    #[test]
    fn the_day_a_version_came() {
        let today = rune::time::LocalTime::at(NOW as i64);
        let said = when_shared(NOW, NOW);
        assert!(said.starts_with("today at "), "{said}");
        assert!(said.ends_with(if today.hour < 12 { "am" } else { "pm" }));
        assert!(!when_shared(NOW - 40.0 * 86_400_000.0, NOW).starts_with("today"));
    }

    #[test]
    fn a_path_handed_over_is_read_as_a_path() {
        assert_eq!(clean_asked("file:///Users/you/a%20b.png?x=1"), "/Users/you/a b.png");
        assert_eq!(clean_asked("./designs/a.png"), "designs/a.png");
        assert!(ends_with_path("/p/designs/a.png", "designs/a.png"));
        assert!(!ends_with_path("/p/xdesigns/a.png", "designs/a.png"));
    }
}
