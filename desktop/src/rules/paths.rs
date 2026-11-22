//! A file path typed into a sentence becomes the file: a document becomes a link to it, a
//! picture becomes the picture. The web window's rule, function by function, from
//! `renderer/src/artifact-paths.ts`, with the part of `renderer/src/media-row.ts` that says
//! which of those files are drawn under the sentence that names them.
//!
//! Places in a text are counted in bytes here and in UTF-16 units there. Both only ever cut
//! the text where a match begins or ends, so the pieces are the same.

use super::js::{self, re};

const MEDIA_EXT: &str = "mp3|wav|m4a|aac|ogg|mp4|mov|m4v|webm";
// Documents too, which open in the app that reads them.
const DOC_EXT: &str = "txt|json|tsv|ya?ml|docx?|xlsx?|pptx?|rtf|pages|numbers|zip";

fn ext() -> String {
    format!("html?|pdf|png|jpe?g|gif|webp|svg|csv|md|markdown|{DOC_EXT}|{MEDIA_EXT}")
}

// `CODE_PATH`: a path a worker writes in backticks, which a code span draws as the file.
re!(CODE_PATH, r"^[\w-][\w./-]*\.(png|jpe?g|gif|webp|html?|pdf|md|csv|txt|json|mp3|wav|m4a|aac|ogg|mp4|mov|m4v|webm)$", "i");
re!(PATH, &format!(r"(?<![\w./:@~-])(?:\.\/)?[\w-][\w./-]*\.(?:{})\b", ext()), "gi");
re!(QUOTED, &format!("[\"\u{201c}]([^\"\u{201d}\\r\\n]+\\.(?:{0}))[\"\u{201d}]|['\u{2018}]([^'\u{2019}\\r\\n]+\\.(?:{0}))['\u{2019}]", ext()), "gi");
re!(MEDIA, &format!(r"\.(?:{MEDIA_EXT})$"), "i");
// The same files, written out in full from the root or from the home folder.
re!(FULL, &format!(r"(?<![\w./:@~-])(?:~|/)/?[\w./-]*\.(?:{})\b", ext()), "gi");
// A local address with no http:// in front.
re!(LOCAL, r#"(?<![\w./:@-])(?:localhost|127\.0\.0\.1):\d{2,5}(?:\/[^\s<>()[\]`'"]*)?"#, "gi");
// A folder on a line of its own is a door too.
re!(FOLDER_LINE, r"(^|\n)([ \t]*)((?:~\/|\/)[\w./-]*[\w-])(\/?)[ \t]*(?=\n|$)", "g");
re!(FILE_LIKE, r"\.[a-z0-9]{1,8}$", "i");
// A picture named in a sentence is drawn, not linked.
re!(PICTURE, r"\.(png|jpe?g|gif|webp|svg)$", "i");
re!(PATH_LIKE, r"^(?:~\/|\/|\.?\/?[\w-])[^\s:]*\.[A-Za-z0-9]{1,8}$", "");
re!(SCHEME, r"^[a-z]+:", "i");
re!(HOME, r"^\/(?:Users|home)\/[^/]+", "");
re!(QUOTED_NAME, r"^(?:~\/|\/|\.?\/?[\w-])[\w ./()-]*$", "u");
re!(DOWNLOADS_AS, r"\bDownloads(?: folder)?\s+as\s*$", "i");
re!(SENTENCE_END, r"[.,;:!?]+$", "");

/// `CODE_PATH.test`: whether a code span is nothing but a path to a file the window draws.
pub fn is_code_path(value: &str) -> bool {
    CODE_PATH.test(value)
}

/// Is this a sound or a film of the product's (`isMediaPath`).
pub fn is_media_path(path: &str) -> bool {
    if SCHEME.test(path) {
        return false;
    }
    MEDIA.test(path.split(['?', '#']).next().unwrap_or(""))
}

/// Whether an address ends as a picture does (`PICTURE.test`).
pub fn is_picture(url: &str) -> bool {
    PICTURE.test(url)
}

/// The short path a full one names, or None when it is not inside `dir` (`productPath`).
pub fn product_path(full: &str, dir: &str) -> Option<String> {
    let root = dir.trim_end_matches('/');
    if root.is_empty() {
        return None;
    }
    let abs = match full.strip_prefix("~/") {
        Some(rest) => format!("{}/{rest}", HOME.exec(root)?.get(0)?),
        None => full.to_string(),
    };
    abs.strip_prefix(root)?.strip_prefix('/').map(str::to_string)
}

/// The short path a full one names, in the project's folder or else the thread's own
/// (`shortPath`).
pub fn short_path(full: &str, dir: Option<&str>, folder: Option<&str>) -> Option<String> {
    dir.filter(|d| !d.is_empty()).and_then(|d| product_path(full, d)).or_else(|| folder.filter(|f| !f.is_empty() && full.starts_with('/')).and_then(|f| product_path(full, f)))
}

/// A person reads a file by its name; the path is the app's to follow (`nameOf`).
pub fn name_of(written: &str) -> &str {
    let name = written.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    if name.is_empty() { written } else { name }
}

/// One piece of a sentence once its files are found.
#[derive(Clone, Debug, PartialEq)]
pub enum Piece {
    Text(String),
    /// `alt` is the path as written, so a picture whose file is missing still says which.
    Picture { url: String, alt: String },
    Link { url: String, words: String },
}

struct Hit {
    at: usize,
    text: String,
    url: String,
    as_is: bool,
}

/// `encodeURIComponent`, for the one address this rule builds itself.
fn encode_uri_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) { out.push(b as char) } else { out.push_str(&format!("%{b:02X}")) }
    }
    out
}

/// A sentence cut into words, links and pictures (`pieces`). Empty when it names no file,
/// which leaves the text as it was. `any_full`: in a conversation between people a whole path
/// may be a teammate's. `folder`: the thread's own code folder, where its runs write.
pub fn pieces(text: &str, dir: &str, any_full: bool, folder: Option<&str>) -> Vec<Piece> {
    let mut hits: Vec<Hit> = Vec::new();
    let mut quoted: Vec<(usize, usize)> = Vec::new();
    let home = HOME.exec(dir).and_then(|m| m.get(0));
    for m in QUOTED.each(text) {
        let Some((start, end)) = m.span(1).or(m.span(2)) else { continue };
        let name = &text[start..end];
        // Even an outside path must be consumed whole, or its last word becomes a misleading
        // relative link. The quotes themselves remain ordinary text.
        quoted.push((start, end));
        if !QUOTED_NAME.test(name) {
            continue;
        }
        let full = name.starts_with("~/") || name.starts_with('/');
        let mut url = if full { short_path(name, Some(dir), folder) } else { Some(name.to_string()) };
        // "Downloads as <quoted filename>" names its folder outright.
        let before = &text[..m.span(0).map_or(0, |s| s.0)];
        if let (false, false, Some(home), true) = (full, name.contains('/'), home, DOWNLOADS_AS.test(before)) {
            url = Some(format!("file://{home}/Downloads/{}", encode_uri_component(name)));
        }
        if let Some(url) = url.filter(|u| !u.is_empty()) {
            hits.push(Hit { at: start, text: name.to_string(), url, as_is: false });
        }
    }
    let in_quote = |at: usize| quoted.iter().any(|(start, end)| at >= *start && at < *end);
    for (start, end) in PATH.matches(text) {
        if !in_quote(start) {
            hits.push(Hit { at: start, text: text[start..end].to_string(), url: text[start..end].to_string(), as_is: false });
        }
    }
    if !dir.is_empty() || folder.is_some_and(|f| !f.is_empty()) {
        for (start, end) in FULL.matches(text) {
            if in_quote(start) {
                continue;
            }
            let written = &text[start..end];
            let rel = short_path(written, Some(dir), folder).or_else(|| (any_full && written.starts_with('/')).then(|| written.to_string()));
            if let Some(rel) = rel.filter(|r| !r.is_empty()) {
                hits.push(Hit { at: start, text: written.to_string(), url: rel, as_is: false });
            }
        }
        for m in FOLDER_LINE.each(text) {
            let (Some((at, _)), named, slash) = (m.span(3), js::group(&m, 3), js::group(&m, 4)) else { continue };
            if FILE_LIKE.test(named) {
                continue;
            }
            let Some(rel) = product_path(named, dir).filter(|r| !r.is_empty()) else { continue };
            hits.push(Hit { at, text: format!("{named}{slash}"), url: format!("{rel}/"), as_is: true });
        }
    }
    for (start, end) in LOCAL.matches(text) {
        // The full stop or comma that ends the sentence is not part of the address.
        let address = SENTENCE_END.strip(&text[start..end]);
        hits.push(Hit { at: start, url: format!("http://{address}"), text: address, as_is: true });
    }
    // A stable sort, as JavaScript's is: of two hits at one place the first found stands.
    hits.sort_by_key(|h| h.at);

    let mut out = Vec::new();
    let mut at = 0;
    for hit in hits {
        if hit.at < at {
            continue;
        }
        if hit.at > at {
            out.push(Piece::Text(text[at..hit.at].to_string()));
        }
        at = hit.at + hit.text.len();
        if PICTURE.test(&hit.url) {
            out.push(Piece::Picture { url: hit.url, alt: hit.text });
        } else {
            let words = if hit.as_is { hit.text } else { name_of(&hit.text).to_string() };
            out.push(Piece::Link { url: hit.url, words });
        }
    }
    if out.is_empty() {
        return out;
    }
    if at < text.len() {
        out.push(Piece::Text(text[at..].to_string()));
    }
    out
}

/// The words of a link somebody wrote: one that spells out its own path reads as the file's
/// name, and words that are not the path stay (`linkArtifactPaths`, on a `link` node).
pub fn link_words<'a>(url: &str, said: &'a str) -> &'a str {
    if said == url && PATH_LIKE.test(said) { name_of(said) } else { said }
}

// `media-row.ts`: what a sentence names that draws or plays under it.
re!(DRAWN, r"\.(?:png|jpe?g|gif|webp|svg|mp4|mov|m4v|webm)(?:[?#]|$)", "i");
re!(ELSEWHERE, r"^(?:https?|data|mailto):", "i");
// A voice note is a message of its own and draws its own player where it is said.
re!(VOICE, r"voice-note|\bvoice-", "i");
re!(FILM, r"\.(?:mp4|mov|m4v|webm)(?:[?#]|$)", "i");
re!(ONLY_MARKS, r"[^\s.,;:!?&+-]", "");

/// Whether an address is a file of the product's that draws or plays (`isFile`).
pub fn is_drawn_file(at: &str) -> bool {
    !at.is_empty() && !ELSEWHERE.test(at) && DRAWN.test(at) && !VOICE.test(at)
}

/// Whether what is drawn is a film, which a row plays and this window only names.
pub fn is_film(at: &str) -> bool {
    FILM.test(at)
}

/// Whether a piece of text says something, and is not only the commas between two files
/// (`isWords`, for a text node).
pub fn says_something(text: &str) -> bool {
    ONLY_MARKS.test(text)
}

/// A file's name off its address (`baseName` in `media-row.ts`).
pub fn base_name(at: &str) -> &str {
    at.split(['?', '#']).next().unwrap_or(at).rsplit('/').next().unwrap_or(at)
}

/// The address markdown hands on for a link or a picture: what an address may not hold is
/// percent-encoded, and an escape already there is kept (`normalizeUri` in micromark, which
/// `mdast-util-to-hast` runs every `url` through).
pub fn normalize_uri(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut at = 0;
    for (i, c) in value.char_indices() {
        if i < at {
            continue;
        }
        at = i + c.len_utf8();
        let kept = match c {
            '%' => bytes.get(i + 1).is_some_and(u8::is_ascii_alphanumeric) && bytes.get(i + 2).is_some_and(u8::is_ascii_alphanumeric),
            '!' | '#' | '$' | '&'..=';' | '=' | '?'..='Z' | '_' | 'a'..='z' | '~' => true,
            _ => false,
        };
        if kept {
            out.push(c);
        } else {
            let mut units = [0u8; 4];
            for b in c.encode_utf8(&mut units).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

/// `decodeURIComponent`, leaving what is not an escape as it was written.
pub fn decode_uri(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        match (bytes[i], bytes.get(i + 1).copied().and_then(hex), bytes.get(i + 2).copied().and_then(hex)) {
            (b'%', Some(hi), Some(lo)) => {
                out.push((hi * 16 + lo) as u8);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIR: &str = "/Users/you/Work/accounts/acct/storefront";

    fn run(text: &str) -> Vec<Piece> {
        pieces(text, "", false, None)
    }
    fn in_dir(text: &str) -> Vec<Piece> {
        pieces(text, DIR, false, None)
    }
    fn links(found: &[Piece]) -> Vec<(&str, &str)> {
        found.iter().filter_map(|p| if let Piece::Link { url, words } = p { Some((url.as_str(), words.as_str())) } else { None }).collect()
    }
    fn pictures(found: &[Piece]) -> Vec<&str> {
        found.iter().filter_map(|p| if let Piece::Picture { url, .. } = p { Some(url.as_str()) } else { None }).collect()
    }
    fn text(s: &str) -> Piece {
        Piece::Text(s.to_string())
    }

    // tests/a-picture-a-worker-names-is-drawn.test.mts
    #[test]
    fn a_picture_named_in_a_sentence_becomes_the_picture() {
        let found = run("attachments/w-0a1b2c3d4e-corner-after.png");
        assert_eq!(found, [Piece::Picture { url: "attachments/w-0a1b2c3d4e-corner-after.png".into(), alt: "attachments/w-0a1b2c3d4e-corner-after.png".into() }]);
        // Its alt is the path, so a missing file still says which one.
        assert_eq!(run("designs/w-805/the-shot.png"), [Piece::Picture { url: "designs/w-805/the-shot.png".into(), alt: "designs/w-805/the-shot.png".into() }]);
        // Two on one line, a space apart.
        assert_eq!(pictures(&run("attachments/a-corner-after.png attachments/b-model-drawer.png")), ["attachments/a-corner-after.png", "attachments/b-model-drawer.png"]);
        // Mid-sentence, without eating the sentence.
        let found = run("The corner is at designs/corner.png, have a look.");
        assert_eq!(found, [text("The corner is at "), Piece::Picture { url: "designs/corner.png".into(), alt: "designs/corner.png".into() }, text(", have a look.")]);
        for name in ["a.png", "a.PNG", "a.jpg", "a.jpeg", "a.gif", "a.webp", "a.svg"] {
            assert_eq!(pictures(&run(&format!("shots/{name}"))).len(), 1, "{name}");
        }
    }

    #[test]
    fn what_is_not_a_picture_stays_a_link_or_text() {
        assert_eq!(links(&run("designs/landing-shots/round19.html")), [("designs/landing-shots/round19.html", "round19.html")]);
        for name in ["reports/audit.pdf", "data/rows.csv"] {
            assert_eq!(links(&run(name)).len(), 1, "{name}");
        }
        // A url ending in .png is not a file of this product.
        assert!(run("It is at https://example.com/shot.png today.").is_empty());
    }

    // tests/a-link-in-a-message-is-a-link.test.mts
    #[test]
    fn a_sound_or_a_film_named_in_a_sentence_becomes_a_link() {
        assert_eq!(links(&run("designs/w-a9/voices/first.mp3")), [("designs/w-a9/voices/first.mp3", "first.mp3")]);
        let names = ["one", "two", "three", "four", "five-more"];
        let line = names.map(|n| format!("designs/w-a9da9d725e/voices/{n}.mp3")).join(" ");
        assert_eq!(links(&run(&line)).len(), 5);
        for name in ["a.mp3", "a.MP3", "a.wav", "a.m4a", "a.aac", "a.ogg", "a.mp4", "a.mov", "a.m4v", "a.webm"] {
            let path = format!("renders/{name}");
            assert_eq!(links(&run(&path)), [(path.as_str(), name)], "{name}");
            assert!(is_media_path(&path), "{name}");
        }
        assert!(!is_media_path("designs/a.png") && !is_media_path("designs/a.html") && !is_media_path("https://example.com/a.mp3"));
        assert!(is_media_path("designs/a.mp4?t=3"));
        assert_eq!(links(&run("The script is in designs/w-1d/demo-script.md, have a read.")), [("designs/w-1d/demo-script.md", "demo-script.md")]);
    }

    #[test]
    fn a_full_path_into_the_product_folder() {
        let found = in_dir(&format!("It is at {DIR}/designs/w-3a/latest.html now."));
        assert_eq!(found, [text("It is at "), Piece::Link { url: "designs/w-3a/latest.html".into(), words: "latest.html".into() }, text(" now.")]);
        // Written from the home folder with a tilde, too.
        assert_eq!(links(&in_dir("~/Work/accounts/acct/storefront/designs/cut.mp4")), [("designs/cut.mp4", "cut.mp4")]);
        // A picture written in full is drawn, like a short one.
        assert_eq!(in_dir(&format!("{DIR}/designs/w-7b/planet.png")), [Piece::Picture { url: "designs/w-7b/planet.png".into(), alt: format!("{DIR}/designs/w-7b/planet.png") }]);
        // Outside the product it is scratch, and stays text.
        assert!(links(&in_dir("It wrote /tmp/scratch/out.html and threw it away.")).is_empty());
        assert!(links(&in_dir("/Users/you/Desktop/dev/app/renderer/index.html")).is_empty());
        // A sibling product whose name starts the same way.
        assert!(links(&in_dir("/Users/you/Work/accounts/acct/storefront-video/designs/a.html")).is_empty());
        // With no product folder known, and a script even inside it.
        assert!(links(&run(&format!("{DIR}/designs/a.html"))).is_empty());
        assert!(links(&in_dir(&format!("{DIR}/designs/w-96/land-it.sh"))).is_empty());
    }

    #[test]
    fn a_local_address_with_no_http_in_front() {
        let found = run("The app is running at localhost:3000, open it.");
        assert_eq!(found, [text("The app is running at "), Piece::Link { url: "http://localhost:3000".into(), words: "localhost:3000".into() }, text(", open it.")]);
        assert_eq!(links(&run("Draft is on localhost:3011/dev/draft.")), [("http://localhost:3011/dev/draft", "localhost:3011/dev/draft")]);
        assert_eq!(links(&run("Try 127.0.0.1:5173.")), [("http://127.0.0.1:5173", "127.0.0.1:5173")]);
        assert!(run("It binds localhost on port 3000.").is_empty());
        assert!(run("Set mylocalhost:3000 in the config.").is_empty());
    }

    #[test]
    fn source_files_and_scripts_stay_text() {
        assert!(run("The fold is in renderer/src/list-rules.ts, main/store.mts and scripts/land.sh.").is_empty());
    }

    #[test]
    fn a_code_span_that_is_nothing_but_a_path() {
        assert!(is_code_path("voices/first.mp3") && is_code_path("designs/cut.mp4") && !is_code_path("npm run ship"));
        assert_eq!(short_path(&format!("{DIR}/form-TO-SIGN.pdf"), Some(DIR), None).as_deref(), Some("form-TO-SIGN.pdf"));
        assert_eq!(product_path(&format!("{DIR}-video/a.pdf"), DIR), None);
        assert_eq!(product_path("designs/a.pdf", DIR), None);
    }

    // tests/a-file-named-in-a-thread-reads-as-its-name-not-a-disk-path.test.mts
    #[test]
    fn a_path_in_a_sentence_reads_as_the_files_name() {
        assert_eq!(links(&run("Notes on designs/w-e6b4931021/launch-posts-v4.md: tighten it")), [("designs/w-e6b4931021/launch-posts-v4.md", "launch-posts-v4.md")]);
        assert_eq!(links(&in_dir(&format!("Saved at {DIR}/designs/w-3a/changes.md now."))), [("designs/w-3a/changes.md", "changes.md")]);
        assert_eq!(links(&run("See changes.md for the list.")), [("changes.md", "changes.md")]);
        let found = run("Here: designs/w-f9/seat-picker-v4.png");
        assert_eq!(pictures(&found), ["designs/w-f9/seat-picker-v4.png"]);
        assert!(links(&found).is_empty());
        assert_eq!(links(&run("Open localhost:3000/a.html")), [("http://localhost:3000/a.html", "localhost:3000/a.html")]);
    }

    #[test]
    fn a_link_that_spells_out_its_own_path() {
        assert_eq!(link_words("designs/w-1/plan.md", "designs/w-1/plan.md"), "plan.md");
        assert_eq!(link_words("designs/w-1/plan.md", "the plan"), "the plan");
        assert_eq!(link_words("https://example.com/a/b.md", "https://example.com/a/b.md"), "https://example.com/a/b.md");
    }

    // tests/a-picture-named-beside-its-folder-is-drawn.test.mts
    #[test]
    fn a_folder_on_its_own_line_becomes_a_door() {
        assert_eq!(in_dir(&format!("{DIR}/designs/w-49b4e45403/")), [Piece::Link { url: "designs/w-49b4e45403/".into(), words: format!("{DIR}/designs/w-49b4e45403/") }]);
        // The line under "in:", which markdown keeps in the same paragraph.
        let found = in_dir(&format!("are the files starting with r3- in:\n{DIR}/designs/w-a3482b8c2c/"));
        assert_eq!(found[0], text("are the files starting with r3- in:\n"));
        assert_eq!(links(&found)[0].0, "designs/w-a3482b8c2c/");
        // With no slash it counts too.
        assert_eq!(links(&in_dir(&format!("{DIR}/designs/w-1")))[0].0, "designs/w-1/");
        // Not a folder outside the project, one inside a sentence, or the project itself.
        assert!(in_dir("/tmp/w-e5f31083ff-shots/").is_empty());
        assert!(links(&in_dir(&format!("I saved them under {DIR}/designs/w-1/ earlier"))).is_empty());
        assert!(in_dir(&format!("{DIR}/")).is_empty());
    }

    // tests/a-file-in-the-threads-own-folder-named-in-full-is-a-file-you-can-open.test.mts
    #[test]
    fn a_full_path_into_the_threads_own_folder() {
        let folder = "/tmp/verify/repos/web-app/.claude/worktrees/w-9e0f3b13cc";
        let there = |text: &str| pieces(text, "/home/you/Work/accounts/acct/web-app", false, Some(folder));
        assert_eq!(links(&there(&format!("The page is {folder}/checkout-prototype.html, committed on this task's branch."))), [("checkout-prototype.html", "checkout-prototype.html")]);
        assert_eq!(links(&there(&format!("{folder}/designs/checkout-a.html"))), [("designs/checkout-a.html", "checkout-a.html")]);
        assert_eq!(links(&there(&format!("Saved as \"{folder}/checkout-prototype.html\"."))), [("checkout-prototype.html", "checkout-prototype.html")]);
        // Not another thread's folder, the rest of the repository, or one that starts the same.
        assert!(links(&there("See /tmp/verify/repos/web-app/.claude/worktrees/w-0000aaaa11/checkout-prototype.html.")).is_empty());
        assert!(links(&there("Built from /tmp/verify/repos/web-app/src/index.html and /tmp/scratch/out.html.")).is_empty());
        assert!(links(&there(&format!("{folder}-old/checkout-prototype.html"))).is_empty());
        assert!(links(&pieces(&format!("The page is {folder}/checkout-prototype.html."), "/home/you/Work/accounts/acct/web-app", false, None)).is_empty());
    }

    #[test]
    fn a_quoted_name_is_taken_whole() {
        // A name with a space in it is one file, and the quotes stay words.
        assert_eq!(run("The old one, \"receipt old.png\", is gone."), [text("The old one, \""), Piece::Picture { url: "receipt old.png".into(), alt: "receipt old.png".into() }, text("\", is gone.")]);
        assert_eq!(links(&run("Saved as \u{201c}Week 32 report.pdf\u{201d} today.")), [("Week 32 report.pdf", "Week 32 report.pdf")]);
        // A quoted path outside the project is consumed whole and linked nowhere.
        assert!(in_dir("It wrote \"/tmp/scratch/out.html\" and left.").is_empty());
        // "Downloads as" names its folder.
        assert_eq!(links(&in_dir("It is in Downloads as \"Week 32.pdf\" now.")), [("file:///Users/you/Downloads/Week%2032.pdf", "Week 32.pdf")]);
    }

    // tests/media-sit-in-a-row-under-the-whole-sentence.test.mts, the part that is this module's.
    #[test]
    fn what_draws_or_plays_under_its_sentence() {
        assert!(is_drawn_file("chart.png") && is_drawn_file("clip.mp4") && is_drawn_file("b.webp"));
        assert!(!is_drawn_file("report.pdf") && !is_drawn_file("https://x.test/a.png") && !is_drawn_file("attachments/voice-note-1200.webm") && !is_drawn_file(""));
        assert!(is_film("a.mp4") && !is_film("a.png"));
        assert!(says_something("and") && !says_something(", ") && !says_something(" & "));
        assert_eq!(base_name("designs/a/1-needs-you.png?v=2"), "1-needs-you.png");
    }

    #[test]
    fn an_address_as_markdown_hands_it_on() {
        assert_eq!(normalize_uri("receipt old.png"), "receipt%20old.png");
        assert_eq!(normalize_uri("designs/w-1/a%20b.png"), "designs/w-1/a%20b.png");
        assert_eq!(normalize_uri("http://localhost:3000/a?b=c#d"), "http://localhost:3000/a?b=c#d");
        assert_eq!(normalize_uri("caf\u{e9}.png"), "caf%C3%A9.png");
        assert_eq!(decode_uri("receipt%20old.png"), "receipt old.png");
        assert_eq!(decode_uri("100%.png"), "100%.png");
    }
}
