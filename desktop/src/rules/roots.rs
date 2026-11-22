//! Where a file a message names is looked for, and what it is called once found: the folders
//! the message itself names first, then the thread's own designs folder, the worker's own
//! checkout, and the five places always tried (`renderer/src/message-folders.ts`). With the
//! rule that lists every file a message refers to (`shared/referenced-files.mts`), the whole
//! path a picture's source may already be (`wholePicturePath` in `renderer/src/picture.ts`)
//! and the name a file is shown by (`shownName` in `shared/artifact-marks.mts`).

use super::js::{self, re};
use super::paths::{decode_uri, product_path};

const EXT: &str = "png|jpe?g|gif|webp|svg|html?|pdf|md|markdown|csv|tsv|txt|json|ya?ml|sql|sh|css|jsx?|tsx?|mjs|cjs|mts|cts|py|rb|go|rs|toml|change";

// A path as it appears in prose or inside a link's parentheses: no leading slash, no scheme.
re!(PATH, &format!(r"(?<![\w./:@-])(?:\.\/)?[\w-][\w./-]*\.(?:{EXT})\b"), "gi");
re!(LINK, r#"!?\[[^\]\n]*\]\(\s*([^)\s]+?)\s*(?:\s+"[^"]*")?\)"#, "g");
// The whole path, from the root, as a worker that pasted one writes it.
re!(ABS_PATH, &format!(r"(?<![\w.:@/-])/[\w-][\w./-]*\.(?:{EXT})\b"), "gi");
re!(ABSOLUTE, r"^([a-z]+:|\/|#)", "i");
re!(IMAGE, r"\.(png|jpe?g|gif|webp)$", "i");
re!(HAS_KNOWN_EXT, &format!(r"\.(?:{EXT})$"), "i");

/// Every file referenced across some pieces of a message, once each, in the order a reader
/// meets them (`referencedFiles`). With `dir`, the product's own folder, a path written out in
/// full that lands inside it is read as the short path it is.
pub fn referenced_files(texts: &[&str], dir: Option<&str>) -> Vec<String> {
    let dir = dir.unwrap_or("").trim_end_matches('/');
    let mut found: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut take = |raw: &str| {
        let mut path = js::trim(raw.strip_prefix("./").unwrap_or(raw));
        if let (false, Some(rest)) = (dir.is_empty(), path.strip_prefix(dir).and_then(|p| p.strip_prefix('/'))) {
            path = rest;
        }
        if path.is_empty() || ABSOLUTE.test(path) || !HAS_KNOWN_EXT.test(path) {
            return;
        }
        let key = path.to_lowercase();
        if !seen.contains(&key) {
            seen.push(key);
            found.push(path.to_string());
        }
    };
    for text in texts {
        // Links first, so a target that also matches the bare-path scan is recorded once.
        let mut hits: Vec<(usize, &str)> = Vec::new();
        let mut links: Vec<(usize, usize)> = Vec::new();
        for m in LINK.each(text) {
            let (Some(whole), Some(target)) = (m.span(0), m.get(1)) else { continue };
            hits.push((whole.0, target));
            links.push(whole);
        }
        // A link's words are a label and its target is already taken.
        let in_link = |at: usize| links.iter().any(|(from, to)| at >= *from && at < *to);
        hits.extend(PATH.matches(text).into_iter().filter(|m| !in_link(m.0)).map(|(a, b)| (a, &text[a..b])));
        if !dir.is_empty() {
            hits.extend(ABS_PATH.matches(text).into_iter().filter(|m| !in_link(m.0)).map(|(a, b)| (a, &text[a..b])));
        }
        hits.sort_by_key(|h| h.0);
        for (_, path) in hits {
            take(path);
        }
    }
    found
}

/// Whether a referenced path is a picture (`isImagePath`).
pub fn is_image_path(path: &str) -> bool {
    IMAGE.test(path)
}

// A path written out in full, from the root or the home folder.
re!(FULL, r"(?<![\w./:@~-])(?:~\/|\/)[\w./-]+", "g");
// A short folder path, ending in its slash: `designs/w-3/`.
re!(SHORT, r#"(?<![\w./:@~-])[\w-][\w.-]*(?:\/[\w.-]+)*\/(?=[\s)`'"\]]|$)"#, "g");
re!(HAS_EXT, r"\.[a-z0-9]{1,8}$", "i");
re!(TRAILING_MARKS, r"[.,;:]+$", "");

fn parent(rel: &str) -> &str {
    rel.rfind('/').map_or("", |at| &rel[..at])
}

/// Every folder inside `dir` the message names, as paths relative to `dir`, in order
/// (`namedFolders`).
pub fn named_folders(texts: &[&str], dir: Option<&str>) -> Vec<String> {
    let Some(dir) = dir.filter(|d| !d.is_empty()) else { return Vec::new() };
    let mut out: Vec<String> = Vec::new();
    let mut add = |rel: &str| {
        let clean = rel.trim_end_matches('/');
        if !clean.is_empty() && !out.iter().any(|f| f == clean) {
            out.push(clean.to_string());
        }
    };
    for text in texts {
        for (start, end) in FULL.matches(text) {
            let token = TRAILING_MARKS.strip(&text[start..end]);
            let Some(rel) = product_path(&token, dir).filter(|r| !r.is_empty()) else { continue };
            let bare = rel.trim_end_matches('/');
            add(if HAS_EXT.test(bare) && !rel.ends_with('/') { parent(bare) } else { bare });
        }
        for (start, end) in SHORT.matches(text) {
            add(&text[start..end]);
        }
    }
    for file in referenced_files(texts, Some(dir)) {
        add(parent(&file));
    }
    out
}

/// The folder a thread's runs work in (`taskFolder`).
pub fn task_folder(repo: Option<&str>, id: Option<&str>) -> Option<String> {
    match (repo.filter(|r| !r.is_empty()), id.filter(|i| !i.is_empty())) {
        (Some(repo), Some(id)) => Some(format!("{}/.claude/worktrees/{id}", repo.trim_end_matches('/'))),
        _ => None,
    }
}

/// Every folder a picture this message names may be in, the likeliest first (`pictureRoots`).
pub fn picture_roots(dir: Option<&str>, repo: Option<&str>, id: Option<&str>, texts: &[&str]) -> Vec<String> {
    let dir = dir.filter(|d| !d.is_empty());
    let repo = repo.filter(|r| !r.is_empty());
    let mut roots: Vec<String> = Vec::new();
    let mut add = |root: Option<String>| {
        if let Some(root) = root.filter(|r| !roots.contains(r)) {
            roots.push(root);
        }
    };
    if let Some(dir) = dir {
        for folder in named_folders(texts, Some(dir)) {
            add(Some(format!("{dir}/{folder}")));
        }
    }
    add(dir.zip(id.filter(|i| !i.is_empty())).map(|(dir, id)| format!("{dir}/designs/{id}")));
    // The worker's own checkout: every run works there, so a path it writes is relative to it.
    add(task_folder(repo, id));
    add(dir.map(str::to_string));
    add(dir.map(|dir| format!("{dir}/designs")));
    add(dir.map(|dir| format!("{dir}/attachments")));
    add(repo.map(str::to_string));
    add(repo.map(|repo| format!("{repo}/designs")));
    roots
}

/// The whole path a picture's source names, bare or as a file url, or None when it is only a
/// name (`wholePicturePath`, on a Mac).
pub fn whole_picture_path(src: &str) -> Option<String> {
    if src.starts_with('/') {
        return Some(src.to_string());
    }
    src.strip_prefix("file://").map(decode_uri)
}

// `<sha12>-name`, how a shared thread names its copy, and `<time>-<random>-name`, how a saved
// attachment is named.
re!(COPY_PREFIX, r"^(?:[0-9a-f]{12}-)+", "");
re!(SAVED_PREFIX, r"^([a-z0-9]{8})-[a-z0-9]{1,6}-", "");
// A copy carries a mark in its name: a thread's attachments, or a group's file as kept here.
re!(SENT_COPY, r"(^|\/)(attachments|\.room-files)\/", "");

/// The first moment the app could have saved a file at: the first of January 2025, in milliseconds.
const FIRST_SAVED_MS: u64 = 1_735_689_600_000;
const DAY_MS: u64 = 86_400_000;

/// A file's name as a person gave it (`shownName`): without the mark a copy carries in front.
/// `now` is the clock, in milliseconds: a saved file's stamp must read as a moment that has been.
pub fn shown_name(path: &str, now: f64) -> String {
    let clean = path.split(['?', '#']).next().unwrap_or("");
    let base = clean.rsplit('/').next().unwrap_or("");
    if !SENT_COPY.test(clean) && !COPY_PREFIX.test(base) {
        return base.to_string();
    }
    let rest = COPY_PREFIX.strip(base);
    // The stamp is the clock in base 36, so "payments-auth-" fits the shape and is words.
    let stamp = SAVED_PREFIX.exec(&rest).and_then(|m| m.get(1)).and_then(|s| u64::from_str_radix(s, 36).ok());
    let saved = stamp.is_some_and(|at| at >= FIRST_SAVED_MS && at as f64 <= now + DAY_MS as f64);
    if saved { SAVED_PREFIX.strip(&rest) } else { rest }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIR: &str = "/Users/you/Work/accounts/acct/team-app";
    const REPO: &str = "/Users/you/dev/team-app";

    fn hers() -> String {
        ["- **Version 1, `quiet-1.png`:** one grey line for each action.", "- **Version 2, `quiet-2.png`:** the quietest.", "", &format!("{DIR}/designs/w-49b4e45403/")].join("\n")
    }

    // tests/a-picture-named-beside-its-folder-is-drawn.test.mts
    #[test]
    fn the_folders_a_message_names() {
        assert_eq!(named_folders(&[&hers()], Some(DIR)), ["designs/w-49b4e45403"]);
        assert_eq!(named_folders(&[&format!("{DIR}/designs/w-1")], Some(DIR)), ["designs/w-1"]);
        assert_eq!(named_folders(&["~/Work/accounts/acct/team-app/designs/w-2/"], Some(DIR)), ["designs/w-2"]);
        assert_eq!(named_folders(&["the shots are in designs/w-3/ now"], Some(DIR)), ["designs/w-3"]);
        // The folder each named file sits in, so a bare name beside it resolves.
        assert_eq!(named_folders(&[&format!("{DIR}/designs/w-a18/r2/round3.html")], Some(DIR)), ["designs/w-a18/r2"]);
        // Not a folder outside the project, and a file written out in full is not a folder.
        assert!(named_folders(&["/tmp/w-e5f-shots/", "/tmp/w-e5f-shots/on-dark.png"], Some(DIR)).is_empty());
        assert_eq!(named_folders(&[&format!("{DIR}/designs/w-9.html")], Some(DIR)), ["designs"]);
        assert!(named_folders(&[&hers()], None).is_empty());
    }

    #[test]
    fn where_a_picture_is_looked_for() {
        let roots = picture_roots(Some(DIR), Some(REPO), Some("w-49b4e45403"), &[&hers()]);
        assert_eq!(roots[0], format!("{DIR}/designs/w-49b4e45403"));
        // The thread's own designs folder, even when the message named none.
        assert!(picture_roots(Some(DIR), Some(REPO), Some("w-77"), &["`quiet-1.png`"]).contains(&format!("{DIR}/designs/w-77")));
        // Everywhere it always looked, in the same order, after those.
        let plain = picture_roots(Some(DIR), Some(REPO), Some("w-77"), &[]);
        assert_eq!(plain[plain.len() - 5..], [DIR.to_string(), format!("{DIR}/designs"), format!("{DIR}/attachments"), REPO.to_string(), format!("{REPO}/designs")]);
        // Each folder once.
        let twice = picture_roots(Some(DIR), Some(REPO), Some("w-49b4e45403"), &[&hers(), &hers()]);
        assert!(twice.iter().enumerate().all(|(n, root)| !twice[..n].contains(root)));
        // Without a project folder: the worker's checkout and the code folder only.
        assert_eq!(picture_roots(None, Some(REPO), Some("w-1"), &[&hers()]), [format!("{REPO}/.claude/worktrees/w-1"), REPO.to_string(), format!("{REPO}/designs")]);
    }

    // tests/a-picture-in-the-workers-own-checkout-is-drawn.test.mts, tests/a-file-in-the-threads-own-folder-…
    #[test]
    fn the_folder_a_threads_runs_work_in() {
        assert_eq!(task_folder(Some("/tmp/verify/repos/web-app"), Some("w-9e0f3b13cc")).as_deref(), Some("/tmp/verify/repos/web-app/.claude/worktrees/w-9e0f3b13cc"));
        assert_eq!(task_folder(None, Some("w-9e0f3b13cc")), None);
        let roots = picture_roots(Some(DIR), Some(REPO), Some("w-77"), &[]);
        assert_eq!(roots[1], format!("{REPO}/.claude/worktrees/w-77"));
    }

    // tests/referenced-files.test.mts
    #[test]
    fn every_file_a_message_refers_to_in_order() {
        let text = "See [the report](reports/week-32.md), then `designs/round-3.html` and designs/round-4.png.";
        assert_eq!(referenced_files(&[text], None), ["reports/week-32.md", "designs/round-3.html", "designs/round-4.png"]);
        // Once each, whatever the case; a link's words are a label, not a second file.
        assert_eq!(referenced_files(&["designs/A.png and designs/a.png", "[notes.md](docs/notes.md)"], None), ["designs/A.png", "docs/notes.md"]);
        // A url and an absolute path are not files of the product's, unless inside its folder.
        assert!(referenced_files(&["https://example.com/a.png and /etc/hosts.md"], None).is_empty());
        assert_eq!(referenced_files(&[&format!("It is {DIR}/designs/the-first-row.html now, not /tmp/out.html")], Some(DIR)), ["designs/the-first-row.html"]);
        assert!(is_image_path("a/b.PNG") && !is_image_path("a/b.svg") && !is_image_path("a/b.html"));
    }

    #[test]
    fn a_picture_source_that_is_already_a_whole_path() {
        assert_eq!(whole_picture_path("/tmp/shots/a.png").as_deref(), Some("/tmp/shots/a.png"));
        assert_eq!(whole_picture_path("file:///tmp/shots/a%20b.png").as_deref(), Some("/tmp/shots/a b.png"));
        assert_eq!(whole_picture_path("designs/a.png"), None);
    }

    #[test]
    fn a_files_name_as_a_person_gave_it() {
        let now = 1_795_000_000_000.0;
        assert_eq!(shown_name("designs/w-1/total-line.png?v=2", now), "total-line.png");
        // A shared thread's copy carries a hash in front, even named bare.
        assert_eq!(shown_name("0123456789ab-audit.md", now), "audit.md");
        // A saved attachment carries when it was saved and a few random letters.
        assert_eq!(shown_name("attachments/mc5k2q0a-x7-receipt.png", now), "receipt.png");
        // Words that only fit the shape of a stamp are the name's own.
        assert_eq!(shown_name("attachments/payments-auth-audit.md", now), "payments-auth-audit.md");
        assert_eq!(shown_name("notes/payments-auth-audit.md", now), "payments-auth-audit.md");
    }
}
