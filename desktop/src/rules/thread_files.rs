//! The files a thread made, for the Files button over it: each file once, at its newest
//! version, newest first (`filesMadeHere` in `renderer/src/threads/thread-files.ts`).
//!
//! What that is made of comes from `shared/artifact-marks.mts`, ported only as far as the
//! button reads it: the files a thread's lines name (`filesInLines`, and of what it leaves out
//! only the tables, `otherFilesInLines`), the versions of one file grouped as one
//! (`threadFiles`), and what kind of file each is. The button never has a file's version line
//! (`numbers` in `threadFiles`), so a family's order here is the order its files were named,
//! or the numbers their names carry.
//!
//! Places in a text are counted in bytes here and in UTF-16 units there. They only order the
//! hits of one text and cut it where a match begins, so the files found are the same.

use std::collections::{HashMap, HashSet};

use serde_json::Value;

use super::js::{self, re};
use super::roots;
use crate::talk::Line;

/// A file a line names, as `filesInLines` keeps it: its path as written, and when it was said.
#[derive(Clone, Debug, PartialEq)]
pub struct Named {
    pub path: String,
    pub at: f64,
}

/// One version of a file (`FileVersion`): the id a note on it carries, the name it goes by,
/// where it is, its number among its file's versions, when it was named, and the other paths
/// that are the same file written another way.
#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    pub id: String,
    pub name: String,
    pub path: String,
    pub n: usize,
    pub at: f64,
    pub also: Vec<String>,
}

/// A file and its versions, the oldest first (`FileFamily`).
#[derive(Clone, Debug, PartialEq)]
pub struct Family {
    pub key: String,
    pub versions: Vec<Version>,
}

/// One row of the button's list (`ThreadFile` in `thread-files.ts`): the newest version's path
/// and name, the notes on any of its versions, and when it was last named.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadFile {
    pub src: String,
    pub name: String,
    pub notes: usize,
    pub at: f64,
}

/* ----------------------------------- kinds ----------------------------------- */

// What the big view reviews (`KINDS`), and what Files lists besides (`OTHER_KINDS`).
re!(IMAGE, r"\.(png|jpe?g|gif|webp|svg)$", "i");
re!(FILM, r"\.(mp4|mov|m4v|webm)$", "i");
re!(PDF, r"\.pdf$", "i");
re!(MARKDOWN, r"\.(md|markdown)$", "i");
re!(DECK, r"\.pptx$", "i");
re!(PAGE, r"\.html?$", "i");
re!(TABLE, r"\.(csv|tsv)$", "i");
re!(CODE, r"\.(py|ipynb|js|mjs|cjs|ts|mts|cts|tsx|jsx|rb|go|rs|sh|sql|css|txt|json|ya?ml|xml)$", "i");

/// A path without what follows its name (`split(/[?#]/)[0]`).
fn without_query(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or("")
}

/// What the big view can open, by its name there, or None (`reviewKind`).
pub fn review_kind(path: &str) -> Option<&'static str> {
    let bare = without_query(path);
    [("image", &IMAGE), ("film", &FILM), ("pdf", &PDF), ("markdown", &MARKDOWN), ("deck", &DECK)].into_iter().find(|(_, re)| re.test(bare)).map(|(kind, _)| kind)
}

/// A file Files lists that the big view does not review: a page, a table, a script (`otherKind`).
pub fn other_kind(path: &str) -> Option<&'static str> {
    let bare = without_query(path);
    [("page", &PAGE), ("table", &TABLE), ("code", &CODE)].into_iter().find(|(_, re)| re.test(bare)).map(|(kind, _)| kind)
}

/// Which page the big view draws for a file (`viewKind`).
pub fn view_kind(path: &str) -> Option<&'static str> {
    review_kind(path).or_else(|| other_kind(path))
}

/* ---------------------------------- versions ---------------------------------- */

// Only what plainly means "another go at this file". A bare number does not: shot-3.png and
// shot-4.png are two shots.
re!(VERSION_TAIL, r"(?:[-_ ]v\d+|\s*\(\d+\)|-\d+-|[-_ ](?:final|latest))$", "i");
re!(NUMBERED, r"[-_ ]v(\d+)$", "i");

/// A name without its extension, as `lastIndexOf('.')` cuts it: a name that starts with its
/// only dot has none.
fn split_ext(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if dot > 0 => name.split_at(dot),
        _ => (name, ""),
    }
}

/// The stem with every version tail taken off, as long as something is left (`baseStem`).
fn base_stem(name: &str) -> String {
    let mut stem = split_ext(name).0.to_string();
    while VERSION_TAIL.test(&stem) {
        let shorter = VERSION_TAIL.strip(&stem);
        if shorter.is_empty() {
            break;
        }
        stem = shorter;
    }
    stem
}

/// 3 for `rows-v3.csv`, 1 for a name with no tail, None for any other tail: final, (2)
/// (`numberInName`).
pub fn number_in_name(name: &str) -> Option<f64> {
    let stem = split_ext(name).0;
    if let Some(n) = NUMBERED.exec(stem).and_then(|m| m.get(1)) {
        return n.parse().ok();
    }
    if VERSION_TAIL.test(stem) { None } else { Some(1.0) }
}

/// `group-seat-picker-v3.png` as `group-seat-picker.png` (`baseName`).
pub fn base_name(name: &str) -> String {
    format!("{}{}", base_stem(name), split_ext(name).1)
}

/// The family a file joins: a kind the big view reviews by its stem and that kind, so every
/// version of a picture is one picture; a page or a script by its stem and extension, as it
/// was keyed by its whole name before its versions folded (`keyOf`).
pub fn key_of(name: &str, path: &str) -> String {
    let stem = base_stem(name).to_lowercase();
    match review_kind(path) {
        Some(kind) => format!("{stem}\0{kind}"),
        None => format!("{stem}\0{}", base_name(name)),
    }
}

/// Whether two paths are one file: the same, or one a bare name or a relative path inside the
/// other (`endsWithPath`). Two folders are two files.
fn ends_with_path(whole: &str, part: &str) -> bool {
    whole == part || whole.strip_suffix(part).is_some_and(|head| head.ends_with('/'))
}

/// The files of a thread, grouped into families of versions, each family's oldest version
/// first in the order they were first said (`threadFiles`, with no version line). The same
/// name in two folders is two versions; a bare name or a relative path inside a whole one is
/// the same version written again, except a copy sent to a thread, which is new each time.
/// `now` is the clock that says whether a copy's stamp is a moment that has been.
pub fn thread_files(files: &[Named], now: f64) -> Vec<Family> {
    let mut families: Vec<Family> = Vec::new();
    let mut keyed: HashMap<String, usize> = HashMap::new();
    let mut seen_paths: HashSet<&str> = HashSet::new();
    for f in files {
        let path = f.path.strip_prefix("./").unwrap_or(&f.path);
        if path.is_empty() || !seen_paths.insert(path) {
            continue;
        }
        let name = roots::shown_name(path, now);
        let key = key_of(&name, path);
        let at = *keyed.entry(key.clone()).or_insert_with(|| {
            families.push(Family { key, versions: Vec::new() });
            families.len() - 1
        });
        let family = &mut families[at];
        let same: Vec<usize> = (0..family.versions.len()).filter(|&i| family.versions[i].name == name).collect();
        let written = same.iter().rev().copied().find(|&i| {
            let v = &family.versions[i];
            std::iter::once(&v.path).chain(&v.also).any(|p| ends_with_path(p, path) || ends_with_path(path, p))
        });
        if let Some(i) = written.filter(|_| !roots::is_sent_copy(path)) {
            family.versions[i].also.push(path.to_string());
            continue;
        }
        let id = if same.is_empty() { name.clone() } else { format!("{name}#{}", same.len() + 1) };
        let n = family.versions.len() + 1;
        family.versions.push(Version { id, name, path: path.to_string(), n, at: f.at, also: Vec::new() });
    }
    for family in &mut families {
        // A handoff names the newest first; names that number themselves, each once, say the order.
        let numbers: Vec<Option<f64>> = family.versions.iter().map(|v| number_in_name(&v.name)).collect();
        let each_once = numbers.iter().enumerate().all(|(i, n)| n.is_some() && !numbers[..i].contains(n));
        if family.versions.len() > 1 && each_once {
            let mut ordered: Vec<(f64, Version)> = numbers.into_iter().flatten().zip(family.versions.drain(..)).collect();
            ordered.sort_by(|a, b| a.0.total_cmp(&b.0));
            family.versions = ordered.into_iter().enumerate().map(|(i, (_, v))| Version { n: i + 1, ..v }).collect();
        }
    }
    families
}

/* -------------------------------- what is named -------------------------------- */

const EXT: &str = "png|jpe?g|gif|webp|svg|mp4|mov|m4v|webm|pdf|md|markdown|pptx";

fn pathish() -> String {
    format!(r"(?:\.?\/)?[\w.-]+(?:\/[\w.-]+)*\.(?:{EXT})")
}

// A link's target, a path in a code span, or a path in a sentence: what the big view reviews.
re!(NAMED, &format!(r"\]\(<?((?:\.?\/|[A-Za-z]:\/)?[^()\s>:]+\.(?:{EXT}))>?\)|`({})`|(?<![\w./:@~(\[-])({})(?![\w/-])", pathish(), pathish()), "i");
// Every alternative of NAMED needs one of these, and NAMED is slow to fail on long text.
re!(ANY_EXT, &format!(r"\.(?:{EXT})"), "i");
// A fenced block of code: what is in it is code being shown, not files named. `[\s\S]` in the
// web's, which is any character: `s` and `.` say the same here.
re!(FENCE, r"```.*?(?:```|$)", "s");

// A page or a table an agent names by its bare path counts when there is a folder in it; a
// script named bare is a sentence about code. Read back from each ending.
const BARE_END_SOURCE: &str = r"\/[\w.-]+\.(?:html?|csv|tsv)(?![\w/-])";
re!(BARE_END, BARE_END_SOURCE, "i");
re!(BARE_OTHER, r"(?<![\w./:@~(\[-])((?:\.?\/)?[\w.-]+(?:\/[\w.-]+)+\.(?:html?|csv|tsv))$", "i");
// Alone on its line, a name is a claim even with no folder or for a script.
const ALONE_SOURCE: &str = r"^[ \t]*((?:\.?\/)?[\w-][\w.-]*(?:\/[\w.-]+)*\.[A-Za-z0-9]+)[ \t]*$";
re!(ALONE, ALONE_SOURCE, "m");
re!(ALONE_ANY_CASE, ALONE_SOURCE, "im");
re!(OTHER_NAMED, r"\]\(<?((?:\.?\/)?[^()\s>:]+\.[A-Za-z0-9]+)>?\)|`((?:\.?\/)?[\w.-]+(?:\/[\w.-]+)*\.[A-Za-z0-9]+)`", "");

/// "-390.png" is the end of a name said before it, not a file (`shorthand`).
fn shorthand(path: &str) -> bool {
    path.rsplit('/').next().unwrap_or("").starts_with(['-', '_'])
}

/// A field of a line's change, when it is words.
fn said<'a>(line: &'a Line, field: &str) -> Option<&'a str> {
    line.patch.as_ref()?.get(field).and_then(Value::as_str)
}

/// The reviewable paths one line's words name, repeats included (`pathsSaidOn`). A person's own
/// words only count a file they attached, a link: the notes sent to an agent name the file it
/// should make, which does not exist yet.
fn paths_said_on(line: &Line) -> Vec<String> {
    let mut out = Vec::new();
    for field in ["body", "answer", "result", "note"] {
        let Some(words) = said(line, field).filter(|w| ANY_EXT.test(w)) else { continue };
        let links_only = line.source == "founder" && (field == "answer" || field == "body");
        let words = FENCE.strip_all(words);
        for m in NAMED.each(&words) {
            if links_only && m.get(1).is_none() {
                continue;
            }
            let path = m.get(1).or(m.get(2)).or(m.get(3)).unwrap_or("");
            let path = path.strip_prefix("./").unwrap_or(path);
            if !shorthand(path) {
                out.push(path.to_string());
            }
        }
    }
    out
}

/// Every reviewable file a thread's lines name, in the order they name them, once each
/// (`filesInLines`). An edit made to a document is the version it saved.
pub fn files_in_lines(lines: &[Line]) -> Vec<Named> {
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for line in lines {
        let Some(patch) = &line.patch else { continue };
        let edit = patch.get("artifact").filter(|a| a.get("do").and_then(Value::as_str) == Some("edit"));
        if let Some(path) = edit.and_then(|a| a.get("path")).and_then(Value::as_str).filter(|p| !p.is_empty())
            && seen.insert(path.to_string())
        {
            out.push(Named { path: path.to_string(), at: line.ts });
        }
        for path in paths_said_on(line) {
            if seen.insert(path.clone()) {
                out.push(Named { path, at: line.ts });
            }
        }
    }
    out
}

/// The pages and tables an agent names by a bare path: one alone on its line, and one with a
/// folder in it read back from where its name ends (`bareOthers`). Each with where it starts.
fn bare_others(text: &str) -> Vec<(usize, String)> {
    let mut out: Vec<(usize, String)> = ALONE.each(text).iter().filter_map(|m| m.span(1).map(|(from, to)| (from, text[from..to].to_string()))).collect();
    let word_mark = |b: u8| b.is_ascii_alphanumeric() || b"_./:@~([-".contains(&b);
    for m in BARE_END.each(text) {
        let Some((first, end)) = m.span(0) else { continue };
        // Back to where the word starts, at most 400 characters: all of them are one byte.
        let mut start = first;
        while start > 0 && first - start < 400 && word_mark(text.as_bytes()[start - 1]) {
            start -= 1;
        }
        // With the character before it, which says the path does not go on to the left.
        let before = text[..start].char_indices().next_back().map_or(0, |(at, _)| at);
        if let Some(word) = BARE_OTHER.exec(&text[before..end]).and_then(|m| m.get(1)) {
            out.push((start, word.to_string()));
        }
    }
    out
}

/// What `files_in_lines` leaves out, a page, a table, a script, in the same shape
/// (`otherFilesInLines`). A link or a code span counts, and from an agent a bare path with a
/// folder in it or a name alone on its line; a person's own words only count what they
/// attached, a link.
pub fn other_files_in_lines(lines: &[Line]) -> Vec<Named> {
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for line in lines {
        let person = line.source == "founder";
        for field in ["result", "note", "body", "answer"] {
            let Some(words) = said(line, field) else { continue };
            let named = words.contains("](") || words.contains('`');
            let bare = !person && (BARE_END.test(words) || ALONE_ANY_CASE.test(words));
            if !named && !bare {
                continue;
            }
            let text = FENCE.strip_all(words);
            let mut hits: Vec<(usize, String)> = Vec::new();
            if named {
                for m in OTHER_NAMED.each(&text) {
                    if person && m.get(1).is_none() {
                        continue;
                    }
                    hits.push((m.span(0).map_or(0, |s| s.0), m.get(1).or(m.get(2)).unwrap_or("").to_string()));
                }
            }
            if bare {
                hits.extend(bare_others(&text));
            }
            hits.sort_by_key(|hit| hit.0);
            for (_, hit) in hits {
                let path = hit.strip_prefix("./").unwrap_or(&hit);
                if other_kind(path).is_none() || review_kind(path).is_some() || seen.contains(path) || shorthand(path) {
                    continue;
                }
                seen.insert(path.to_string());
                out.push(Named { path: path.to_string(), at: line.ts });
            }
        }
    }
    out
}

/// The tables a thread named. A page or a script in backticks is usually code being talked
/// about, not something the thread made (`tablesIn`).
fn tables_in(lines: &[Line]) -> Vec<Named> {
    other_files_in_lines(lines).into_iter().filter(|f| view_kind(&f.path) == Some("table")).collect()
}

/// Each file a thread's lines name, once, at its newest version, with the notes on any of its
/// versions, newest first (`filesMadeHere`). `notes` is the version each note on the thread is
/// on (`item.artifacts.notes`, each note's `file`).
pub fn files_made_here(lines: &[Line], notes: &[&str], now: f64) -> Vec<ThreadFile> {
    let mut named = files_in_lines(lines);
    named.extend(tables_in(lines));
    let mut out: Vec<ThreadFile> = thread_files(&named, now)
        .into_iter()
        .filter_map(|family| {
            let newest = family.versions.last()?;
            let ids: HashSet<&str> = family.versions.iter().map(|v| v.id.as_str()).collect();
            Some(ThreadFile {
                src: newest.path.clone(),
                name: newest.name.clone(),
                notes: notes.iter().filter(|file| ids.contains(*file)).count(),
                at: family.versions.iter().map(|v| v.at).fold(f64::NEG_INFINITY, f64::max),
            })
        })
        .collect();
    // A stable sort, as JavaScript's is: files last named at one moment keep the order named.
    out.sort_by(|a, b| b.at.total_cmp(&a.at));
    out
}

/// The kind a row of the list wears before the name, from its extension, four letters at most
/// (`ext` in `renderer/src/threads/ThreadFiles.tsx`). A name with no extension is a file.
pub fn kind_tag(name: &str) -> &str {
    let ext = if name.contains('.') { name.rsplit('.').next().unwrap_or("") } else { "file" };
    js::slice(ext, 0, 4)
}

/// How many notes, said after the name (`.ts-files-notes`).
pub fn notes_words(notes: usize) -> String {
    if notes == 1 { "1 note".to_string() } else { format!("{notes} notes") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: f64 = 1_795_000_000_000.0;

    fn line(ts: f64, source: &str, patch: Value) -> Line {
        serde_json::from_value(json!({ "ts": ts, "source": source, "patch": patch })).unwrap()
    }
    fn agent(ts: f64, field: &str, words: &str) -> Line {
        line(ts, "agent", json!({ field: words }))
    }
    fn person(ts: f64, field: &str, words: &str) -> Line {
        line(ts, "founder", json!({ field: words }))
    }
    fn paths(found: &[Named]) -> Vec<&str> {
        found.iter().map(|f| f.path.as_str()).collect()
    }
    fn named(paths: &[&str]) -> Vec<Named> {
        paths.iter().enumerate().map(|(i, p)| Named { path: p.to_string(), at: i as f64 }).collect()
    }
    fn srcs(files: &[ThreadFile]) -> Vec<&str> {
        files.iter().map(|f| f.src.as_str()).collect()
    }
    fn names(files: &[ThreadFile]) -> Vec<&str> {
        files.iter().map(|f| f.name.as_str()).collect()
    }

    // tests/every-file-a-thread-made-is-one-press-away.test.mts
    #[test]
    fn each_file_once_newest_first_with_the_notes_on_any_version() {
        let lines = [
            agent(1.0, "result", "Drafted it in [plan.md](plan.md)."),
            agent(2.0, "result", "Here is the cover: [cover.png](cover.png)"),
            agent(3.0, "result", "Rewrote the plan: [plan.md](plan-v2.md)"),
        ];
        assert_eq!(srcs(&files_made_here(&lines, &[], NOW)), ["plan-v2.md", "cover.png"]);
        let files = files_made_here(&lines, &["cover.png", "cover.png"], NOW);
        assert_eq!(files.iter().find(|f| f.src == "cover.png").map(|f| f.notes), Some(2));
        assert!(files_made_here(&[agent(1.0, "result", "Done, no files.")], &[], NOW).is_empty());
    }

    // tests/a-table-a-thread-made-is-in-its-files-menu.test.mts
    #[test]
    fn a_table_the_thread_made_is_listed() {
        let linked = [agent(1.0, "note", "Saved [Cloud launch tracker](plans/cloud-launch.csv)")];
        assert_eq!(names(&files_made_here(&linked, &[], NOW)), ["cloud-launch.csv"]);
        // Named again by its whole path on a line of its own, it is still one file.
        let twice = [
            agent(1.0, "note", "Saved [Cloud launch tracker](plans/cloud-launch.csv)"),
            agent(2.0, "result", "Here's the table:\n\n/Users/someone/docs/agentbox/plans/cloud-launch.csv\n\nWhat changed."),
        ];
        assert_eq!(names(&files_made_here(&twice, &[], NOW)), ["cloud-launch.csv"]);
        // A .tsv, beside a picture, newest first.
        let both = [agent(1.0, "result", "The cover: [cover.png](cover.png)"), agent(2.0, "result", "All numbers: [ceiling](designs/w-1/ceiling.tsv)")];
        assert_eq!(names(&files_made_here(&both, &[], NOW)), ["ceiling.tsv", "cover.png"]);
        // Not a script or a page named in passing, nor a table a person only mentioned.
        assert!(files_made_here(&[agent(1.0, "result", "Changed `main/files-place.mjs` and `renderer/index.html`.")], &[], NOW).is_empty());
        assert!(files_made_here(&[person(1.0, "body", "Please update plans/cloud-launch.csv with the new row.")], &[], NOW).is_empty());
    }

    // tests/a-page-or-table-an-agent-names-by-its-path-is-in-files.test.mts
    #[test]
    fn a_page_or_a_table_named_by_its_bare_path() {
        let found = |text: &str, source: &str| paths(&other_files_in_lines(&[line(1.0, source, json!({ "result": text }))])).into_iter().map(str::to_string).collect::<Vec<_>>();
        let dir = "/Users/x/Zero/accounts/a/app";
        assert_eq!(found(&format!("**Pick one.**\n\n{dir}/designs/w-1/files-page.html\n"), "agent"), [format!("{dir}/designs/w-1/files-page.html")]);
        assert_eq!(found(&format!("The list is in {dir}/leads.csv."), "agent"), [format!("{dir}/leads.csv")]);
        assert_eq!(found("The page is at designs/w-1/page.html and the rows at reports/rows.tsv.", "agent"), ["designs/w-1/page.html", "reports/rows.tsv"]);
        // A link and a code span once each.
        assert_eq!(found("[The page](designs/a.html), again `designs/a.html`, and designs/a.html", "agent"), ["designs/a.html"]);
        // No bare word without a folder.
        assert!(found("Built with node.js; index.html and data.csv are the usual names.", "agent").is_empty());
        // Any file named alone on a line of its own, with or without a folder.
        assert_eq!(found("Cleaned it up.\n\nleads.csv\n", "agent"), ["leads.csv"]);
        assert_eq!(found("Here it is:\n  pricing.html  \nand more words", "agent"), ["pricing.html"]);
        assert_eq!(found("Done.\n\ndesigns/launch-checklist.txt\n", "agent"), ["designs/launch-checklist.txt"]);
        assert!(found("leads.csv is ready", "agent").is_empty());
        assert!(found("Can you clean up\n\nleads.csv", "founder").is_empty());
        // No script named bare, nothing bare in a person's own words, no address on the web.
        assert!(found("I changed main/files-place.mts and scripts/ship.sh.", "agent").is_empty());
        assert!(found("Can you make designs/w-1/page.html?", "founder").is_empty());
        assert!(found("See https://example.com/docs/page.html and http://x.io/a/b.csv", "agent").is_empty());
    }

    // tests/a-shorthand-name-makes-no-file.test.mts
    #[test]
    fn a_name_that_starts_with_a_dash_is_no_file() {
        let said = |words: &str| [agent(1.0, "result", words)];
        assert!(files_in_lines(&said("Saved onboarding-b-1-start, -2-link-live and -3-done, each as -desktop.png and -390.png. The other five as `-tight.png` files.")).is_empty());
        assert!(files_in_lines(&said("[-390.png](designs/-390.png)")).is_empty());
        assert!(other_files_in_lines(&said("`-page.html`")).is_empty());
        assert_eq!(paths(&files_in_lines(&said("Saved onboarding-b-1-start-390.png"))), ["onboarding-b-1-start-390.png"]);
    }

    // tests/a-handed-over-v3-opens-as-v3-of-3.test.mts
    #[test]
    fn versions_named_out_of_order_go_by_the_numbers_in_their_names() {
        let order = |family: &Family| family.versions.iter().map(|v| format!("{}:{}", v.n, v.name)).collect::<Vec<_>>();
        let table = &thread_files(&named(&["designs/w-1/rows-v3.csv", "designs/w-1/rows.csv", "designs/w-1/rows-v2.csv"]), NOW)[0];
        assert_eq!(order(table), ["1:rows.csv", "2:rows-v2.csv", "3:rows-v3.csv"]);
        // In the order they were named when the names carry no numbers, or two share one.
        let deck = &thread_files(&named(&["a/deck.pdf", "b/deck.pdf"]), NOW)[0];
        assert_eq!(deck.versions.iter().map(|v| v.path.as_str()).collect::<Vec<_>>(), ["a/deck.pdf", "b/deck.pdf"]);
        let shot = &thread_files(&named(&["a/shot-v2.png", "b/shot-v2.png", "a/shot.png"]), NOW)[0];
        assert_eq!(shot.versions.iter().map(|v| v.path.as_str()).collect::<Vec<_>>(), ["a/shot-v2.png", "b/shot-v2.png", "a/shot.png"]);
        // And the newest the button lists is the highest number, wherever it was named.
        let lines = [agent(1.0, "result", "[rows](attachments/ab12-rows-v3.csv) [rows](attachments/ab12-rows.csv) [rows](attachments/ab12-rows-v2.csv)")];
        assert_eq!(srcs(&files_made_here(&lines, &[], NOW)), ["attachments/ab12-rows-v3.csv"]);
    }

    // tests/two-pictures-with-one-name-in-two-folders-each-open-themselves.test.mts
    #[test]
    fn one_name_in_two_folders_is_two_versions_and_one_file_written_two_ways_is_one() {
        const OLD: &str = "designs/w-e5f506ccb3/inbox-tabs/1-needs-you.png";
        const NEW: &str = "designs/w-e5f506ccb3/two-tabs-and-team/1-needs-you.png";
        let said = |ts: f64, words: String| agent(ts, "note", &words);
        let families = thread_files(&files_in_lines(&[said(1.0, format!("Needs you: ![1-needs-you.png]({OLD})")), said(2.0, format!("Needs you, polished: ![1-needs-you.png]({NEW})"))]), NOW);
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].versions.iter().map(|v| v.path.as_str()).collect::<Vec<_>>(), [OLD, NEW]);
        // The second of one name is told apart by its id, which a note on it carries.
        assert_eq!(families[0].versions.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(), ["1-needs-you.png", "1-needs-you.png#2"]);
        let whole = format!("/Users/someone/Zero/accounts/a/agentbox-team/{NEW}");
        assert_eq!(thread_files(&files_in_lines(&[said(1.0, format!("![a]({NEW})")), said(2.0, format!("![a]({whole})"))]), NOW)[0].versions.len(), 1);
        assert_eq!(thread_files(&files_in_lines(&[said(1.0, format!("![a]({NEW})")), said(2.0, "Here it is again: `1-needs-you.png`".to_string())]), NOW)[0].versions.len(), 1);
        let apart = [said(1.0, "![a](designs/round-1/team/1-needs-you.png)".to_string()), said(2.0, "![a](designs/round-2/team/1-needs-you.png)".to_string())];
        assert_eq!(thread_files(&files_in_lines(&apart), NOW)[0].versions.len(), 2);
    }

    // tests/an-edit-to-a-document-is-the-next-version.test.mts
    #[test]
    fn an_edit_to_a_document_is_its_next_version() {
        let first = agent(1.0, "result", "I wrote [plan.md](designs/w-1/plan.md).");
        let edit = line(2.0, "founder", json!({ "artifact": { "do": "edit", "file": "plan.md", "path": "designs/w-1/plan-v2.md" } }));
        let families = thread_files(&files_in_lines(&[first.clone(), edit]), NOW);
        assert_eq!(families[0].versions.iter().map(|v| (v.n, v.name.as_str())).collect::<Vec<_>>(), [(1, "plan.md"), (2, "plan-v2.md")]);
        // A teammate's edit still on its way has no file yet, and adds no version.
        let on_its_way = line(2.0, "founder", json!({ "artifact": { "do": "edit", "file": "plan.md", "text": "# Words" } }));
        assert_eq!(thread_files(&files_in_lines(&[first, on_its_way]), NOW).iter().map(|f| f.versions.len()).sum::<usize>(), 1);
    }

    // tests/an-agents-revision-of-a-file-is-its-next-version-not-a-new-file.test.mts and
    // tests/a-file-keeps-its-place-and-its-own-name-whoever-names-it.test.mts (`familyOfName`)
    #[test]
    fn the_family_a_name_joins() {
        let family_of = |file: &str| {
            let name = roots::shown_name(file.strip_prefix("./").unwrap_or(file), NOW);
            key_of(&name, &name)
        };
        // A page or a script keeps the key it had before its versions folded.
        for name in ["seat-picker-demo.html", "Group_Booking.PY", "README", "clinics.csv"] {
            let stem = split_ext(name).0;
            assert_eq!(family_of(name), format!("{}\0{name}", stem.to_lowercase()), "{name}");
        }
        assert_eq!(family_of("seat-picker-demo-v2.html"), family_of("seat-picker-demo.html"));
        assert_ne!(family_of("group_booking.py"), family_of("group_bookings.py"));
        assert_ne!(family_of("config.json"), family_of("config.yaml"));
        assert_eq!(family_of("dc1bdc0f055d-mvwlg12b-oxe1xc-group-bookings-spec.md"), family_of("group-bookings-spec.md"));
        // Every version of a picture is one picture, whatever its tail.
        assert_eq!(key_of("group-seat-picker-v5.png", "group-seat-picker-v5.png"), "group-seat-picker\0image");
        assert_eq!(base_name("group-seat-picker-v3.png"), "group-seat-picker.png");
        assert_eq!(base_name("plan (2) final.md"), "plan.md");
        assert_eq!((number_in_name("rows-v3.csv"), number_in_name("rows.csv"), number_in_name("rows final.csv"), number_in_name("rows (2).csv")), (Some(3.0), Some(1.0), None, None));
    }

    // tests/a-file-copied-twice-shows-its-own-name.test.mts and the copy names of
    // tests/a-file-keeps-its-place-and-its-own-name-whoever-names-it.test.mts (`shownName`)
    #[test]
    fn a_copy_is_named_by_the_files_own_name() {
        let shown = |path: &str| roots::shown_name(path, NOW);
        assert_eq!(shown("attachments/0123456789ab-a1b2c3d4e5f6-retry-plan.md"), "retry-plan.md");
        assert_eq!(shown(".team/x/0123456789ab-a1b2c3d4e5f6-retry-plan.md"), "retry-plan.md");
        assert_eq!(shown("attachments/a1b2c3d4e5f6-retry-plan.md"), "retry-plan.md");
        assert_eq!(shown("docs/retry-plan.md"), "retry-plan.md");
        assert_eq!(shown("attachments/a1b2c3d4e5f6-deadbeef0000.md"), "deadbeef0000.md");
        assert_eq!(shown("dc1bdc0f055d-mvwlg12b-oxe1xc-group-bookings-spec.md"), "group-bookings-spec.md");
        assert_eq!(shown("attachments/dc1bdc0f055d-group-bookings-spec.md"), "group-bookings-spec.md");
        assert_eq!(shown("abc123-notes.md"), "abc123-notes.md");
        assert_eq!(shown("zzzzzzzzzzzz-notes.md"), "zzzzzzzzzzzz-notes.md");
        assert_eq!(shown("/s/company/mvx6zm5q-dep7ls-team-look.md"), "mvx6zm5q-dep7ls-team-look.md");
    }

    // `ext` in renderer/src/threads/ThreadFiles.tsx
    #[test]
    fn the_kind_a_row_wears() {
        assert_eq!(kind_tag("total-line-v2.png"), "png");
        assert_eq!(kind_tag("deck.markdown"), "mark");
        assert_eq!(kind_tag("README"), "file");
        assert_eq!(kind_tag("rows.tsv"), "tsv");
        assert_eq!((notes_words(1), notes_words(2)), ("1 note".to_string(), "2 notes".to_string()));
    }

    // The scene `thread-files` draws: a picture and a note each named twice, two sheets and a report.
    #[test]
    fn a_thread_whose_files_come_in_versions() {
        let lines = [
            person(1.0, "body", "Checkout fails for some customers since this morning."),
            agent(2.0, "note", "The receipt adds the tax up before rounding it, as designs/w-1/total-line.png shows. How it should round is in [the rounding note](notes/rounding.md), and every order it broke is in exports/failed-orders.csv."),
            person(3.0, "answer", "Go with 1."),
            agent(4.0, "result", "The total sits on one line again, as in designs/w-1/total-line-v2.png, and the note says why in notes/rounding-v2.md.\n\n- The refunds, order by order: exports/refunds.csv\n- The week with the fix in it: [the week's report](reports/week-32.pdf)"),
        ];
        let files = files_made_here(&lines, &[], NOW);
        assert_eq!(names(&files), ["total-line-v2.png", "rounding-v2.md", "week-32.pdf", "refunds.csv", "failed-orders.csv"]);
        assert_eq!(srcs(&files)[..2], ["designs/w-1/total-line-v2.png", "notes/rounding-v2.md"]);
        assert_eq!(files.iter().map(|f| f.at).collect::<Vec<_>>(), [4.0, 4.0, 4.0, 4.0, 2.0]);
        // A code fence names nothing.
        assert!(files_made_here(&[agent(1.0, "result", "```\ndesigns/a.png\nrows.csv\n```")], &[], NOW).is_empty());
    }
}
