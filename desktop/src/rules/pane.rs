//! The pane a file opens in beside the conversation (`DocPane`): which presses open it, what it
//! calls the file, how much of the body it takes and where a drag leaves it, when Escape closes
//! it, which file the newest answer opens in it by itself, and the line saying when it was last
//! edited.
//!
//! Ported from `renderer/src/doc-pane.ts` (`docKind`, `isPlainText`, `opensInPane`,
//! `crumbParts`, `FOLLOW_FILE_MS`, `newerOnDisk`, the split's numbers, `clampSplit`,
//! `splitFromPointer`, `readSplit`, `escapeClosesDoc`), `renderer/src/artifact-layout.ts`
//! (`artifactPlacement`, `artifactFraction`, `fullScreenTakes`, `whereAFileOpens`,
//! `isLocalPreview`), `renderer/src/message-artifacts.ts` (`besideDocument`, `isBookkeeping`),
//! `canShareFile` (`ShareLink.tsx`), `takesNotesSidebar` (`notes-sidebar.ts`) and `agoWords`
//! (`threads/summary-rules.ts`), with the web's test cases.

use super::js::re;
use super::roots;
use super::thread_files::view_kind;

/// What the pane draws a file as (`DocKind` and `PaneKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A page on the web (`website`) or a local app's address.
    Website,
    Html,
    Markdown,
    /// A change a run made (`.change`).
    Change,
    Image,
    Pdf,
    Film,
    /// A script or any text, drawn as its lines.
    Text,
    Table,
}

fn clean(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or("")
}

fn ends_with_any(path: &str, ends: &[&str]) -> bool {
    let lower = clean(path).to_ascii_lowercase();
    ends.iter().any(|e| lower.ends_with(&format!(".{e}")))
}

/// An address on the web, not on this Mac (`isWebsiteAddress`).
pub fn is_website(src: &str) -> bool {
    let rest = src.strip_prefix("http://").or_else(|| src.strip_prefix("https://"));
    rest.is_some_and(|rest| {
        let host = rest.split(['/', '?', '#']).next().unwrap_or("");
        let host = if host.starts_with('[') { host.split(']').next().map(|h| format!("{h}]")).unwrap_or_default() } else { host.split(':').next().unwrap_or("").to_string() };
        !host.is_empty() && !matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]")
    })
}

/// A local app's address: http or https on this Mac (`isLocalPreview`).
pub fn is_local_preview(src: &str) -> bool {
    let rest = src.strip_prefix("http://").or_else(|| src.strip_prefix("https://"));
    rest.is_some_and(|rest| {
        let host = rest.split(['/', '?', '#']).next().unwrap_or("");
        let host = if host.starts_with('[') { host.split(']').next().map(|h| format!("{h}]")).unwrap_or_default() } else { host.split(':').next().unwrap_or("").to_string() };
        matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]")
    })
}

/// What the pane would do with this path, or nothing (`docKind`): the address is asked before
/// the extension, as a page on the web that ends in `.html` is still the web.
pub fn doc_kind(path: &str) -> Option<Kind> {
    if path.is_empty() {
        return None;
    }
    if is_website(path) {
        return Some(Kind::Website);
    }
    Some(if ends_with_any(path, &["html", "htm"]) {
        Kind::Html
    } else if ends_with_any(path, &["md", "markdown"]) {
        Kind::Markdown
    } else if ends_with_any(path, &["change"]) {
        Kind::Change
    } else if ends_with_any(path, &["png", "jpg", "jpeg", "gif", "webp", "svg"]) {
        Kind::Image
    } else if ends_with_any(path, &["pdf"]) {
        Kind::Pdf
    } else if ends_with_any(path, &["mp4", "mov", "m4v", "webm"]) {
        Kind::Film
    } else {
        return None;
    })
}

/// A script or a text file the pane draws as its text (`isPlainText`).
pub fn is_plain_text(path: &str) -> bool {
    ends_with_any(path, &["py", "ipynb", "js", "mjs", "cjs", "ts", "mts", "cts", "tsx", "jsx", "rb", "go", "rs", "sh", "sql", "css", "txt", "csv", "tsv", "json", "yaml", "yml", "xml"])
}

/// A sheet (`isTablePath`), as the table's own rules tell one.
pub fn is_table_path(path: &str) -> bool {
    super::table_file::is_table_path(path)
}

/// Whether the pane opens this path at all (`opensInPane`).
pub fn opens_in_pane(path: &str) -> bool {
    doc_kind(path).is_some() || is_plain_text(path)
}

/// What the pane draws a path as (`docKind(src) ?? (isTablePath ? 'table' : isPlainText ? 'text' : 'html')`).
pub fn pane_kind(path: &str) -> Kind {
    doc_kind(path).unwrap_or(if is_table_path(path) {
        Kind::Table
    } else if is_plain_text(path) {
        Kind::Text
    } else {
        Kind::Html
    })
}

/// Whether the big view takes this file (`fullScreenTakes`): a kind it draws, and not an address.
pub fn full_screen_takes(src: &str) -> bool {
    !src.is_empty() && view_kind(src).is_some() && !is_website(src) && !is_local_preview(src)
}

/// Where a file pressed in a thread opens first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opens {
    /// In the pane beside the conversation: what is read and talked over.
    Beside,
    /// Big, over the window: what is looked at.
    Full,
}

/// Where a file pressed in a thread opens first, or nothing for its older way
/// (`whereAFileOpens`): a picture, a film, a PDF, a page and a deck big; a document, a table and
/// a script beside the conversation.
pub fn where_a_file_opens(src: &str) -> Option<Opens> {
    if !full_screen_takes(src) {
        return None;
    }
    let looked_at = matches!(view_kind(src), Some("image" | "film" | "pdf" | "page" | "deck"));
    Some(if looked_at || !opens_in_pane(src) { Opens::Full } else { Opens::Beside })
}

/// Where a press on a link in a message goes (`openHref` in `Focus.tsx`, with
/// `whereALinkGoes` in `renderer/src/link-click.ts`), for a thread of the person's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkGoes {
    Full,
    Beside,
    /// The system: the browser for an address, the app that owns any other file.
    Outside,
}

/// Where a press on a link in a message goes: ⌘ sends it out, a file the big view looks at
/// opens big, one to read opens beside, and anything else the pane draws (a page on the web,
/// a local app, a change) opens beside too; the rest goes to the system.
pub fn where_a_link_goes(href: &str, new_tab: bool) -> LinkGoes {
    if new_tab {
        return LinkGoes::Outside;
    }
    match where_a_file_opens(href) {
        Some(Opens::Full) => LinkGoes::Full,
        Some(Opens::Beside) => LinkGoes::Beside,
        None if opens_in_pane(href) || is_local_preview(href) => LinkGoes::Beside,
        None => LinkGoes::Outside,
    }
}

/// How the pane stands with the conversation: beside it, or taking the whole card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Beside,
    Focus,
}

/// A card too narrow to keep a readable conversation beside the file gives the file all of it
/// (`artifactPlacement`): under a thousand points.
pub fn artifact_placement(width: f32) -> Placement {
    if width < 1000.0 { Placement::Focus } else { Placement::Beside }
}

/// The share of the body the pane takes: what was kept, held so that the conversation keeps 400
/// points and the file 560 (`artifactFraction`).
pub fn artifact_fraction(fraction: f32, width: f32) -> f32 {
    (560.0 / width).max((1.0 - 400.0 / width).min(fraction))
}

pub const MIN_SPLIT: f32 = 0.25;
pub const MAX_SPLIT: f32 = 0.75;
pub const EVEN_SPLIT: f32 = 0.5;
/// Where the divider opens before a hand has moved it: just inside the ceiling, so the first
/// drag can go either way.
pub const OPENING_SPLIT: f32 = 0.72;

/// A fraction kept inside the two floors (`clampSplit`).
pub fn clamp_split(fraction: f32) -> f32 {
    if !fraction.is_finite() {
        return EVEN_SPLIT;
    }
    fraction.clamp(MIN_SPLIT, MAX_SPLIT)
}

/// Where a drag puts the divider, as the share of the body the file takes: everything right of
/// the pointer is the file (`splitFromPointer`).
pub fn split_from_pointer(x: f32, width: f32) -> f32 {
    if width == 0.0 {
        return EVEN_SPLIT;
    }
    clamp_split((width - x) / width)
}

/// The kept split, or where it opens when none was kept or it cannot be read (`readSplit`).
pub fn read_split(kept: Option<f32>) -> f32 {
    match kept {
        Some(f) if f.is_finite() => clamp_split(f),
        _ => OPENING_SPLIT,
    }
}

/// How often an open file is read again while it is open (`FOLLOW_FILE_MS`).
pub const FOLLOW_FILE_MS: f64 = 3000.0;

/// Whether a fresh read is newer than what is on screen (`newerOnDisk`).
pub fn newer_on_disk(shown: f64, read: Option<f64>) -> bool {
    read.is_some_and(|at| at > shown)
}

/// Whether the file found can be written from the pane (`paneCanSave`): one inside the
/// project's own folders, or any while there is nothing to tell it by.
pub fn pane_can_save(resolved: Option<&str>, roots: &[String]) -> bool {
    let Some(resolved) = resolved.filter(|r| !r.is_empty()) else { return true };
    if roots.is_empty() {
        return true;
    }
    roots.iter().any(|root| resolved == root || resolved.starts_with(&format!("{}/", root.trim_end_matches('/'))))
}

/// Whether Escape closes the file or leaves the thread (`escapeClosesDoc`): a file the person
/// opened closes; one the thread opened by itself does not stand in the way.
pub fn escape_closes_doc(auto: bool, full_screen: bool) -> bool {
    !auto || full_screen
}

/// The path as a person would say it (`crumbParts`): from the deepest of the project's own
/// folders that holds it, its thread folders left out; a file a message carried by its own
/// path; and the last three parts when no folder holds it.
pub fn crumb_parts(full: &str, roots: &[String]) -> Vec<String> {
    let clean = full.strip_prefix("file://").unwrap_or(full).trim_end_matches('/');
    if clean.is_empty() {
        return Vec::new();
    }
    let thread_folder = |p: &str| p.len() == 12 && p.starts_with("w-") && p[2..].bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
    let without_ids = |parts: Vec<String>| -> Vec<String> {
        let last = parts.len().saturating_sub(1);
        parts.into_iter().enumerate().filter(|(i, p)| *i == last || !thread_folder(p)).map(|(_, p)| p).collect()
    };
    // A file that came with a message: its own path, not where this computer shelved it.
    if let Some(at) = clean.find("/attachments/carried/") {
        let rest = &clean[at + "/attachments/carried/".len()..];
        if let Some((hash, path)) = rest.split_once('/')
            && hash.len() == 16
            && hash.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            && !path.is_empty()
        {
            return path.split('/').map(str::to_string).collect();
        }
    }
    let mut best = "";
    for root in roots {
        let r = root.trim_end_matches('/');
        if r.is_empty() {
            continue;
        }
        if (clean == r || clean.starts_with(&format!("{r}/"))) && r.len() > best.len() {
            best = r;
        }
    }
    if !best.is_empty() {
        let head = best.split('/').filter(|p| !p.is_empty()).next_back().map(str::to_string);
        let rest = clean.get(best.len() + 1..).unwrap_or("").split('/').filter(|p| !p.is_empty()).map(str::to_string);
        return without_ids(head.into_iter().chain(rest).collect());
    }
    let parts = without_ids(clean.split('/').filter(|p| !p.is_empty()).map(str::to_string).collect());
    parts[parts.len().saturating_sub(3)..].to_vec()
}

/// The kinds a link can draw, which only the globe offers to share (`canShareFile`).
pub fn can_share_file(src: &str) -> bool {
    ends_with_any(src, &["md", "markdown", "csv", "tsv", "html", "htm", "png", "jpg", "jpeg", "gif", "webp", "svg", "pdf", "mp4", "m4v", "mov", "webm"])
}

/// A thread's file the pane draws as words, a table or a page, whose notes the side-by-side
/// mark counts (`takesNotesSidebar`).
pub fn takes_notes_sidebar(src: &str) -> bool {
    match doc_kind(src) {
        Some(Kind::Markdown | Kind::Html) => true,
        None => is_plain_text(src),
        _ => false,
    }
}

/// The files kept for the agents rather than made for the person (`isBookkeeping`).
fn is_bookkeeping(path: &str) -> bool {
    let name = clean(path).rsplit('/').next().unwrap_or("").to_lowercase();
    matches!(name.as_str(), "state.md" | "decisions.md" | "notes.md" | "instructions.md" | "memory.md" | "claude.md" | "readme.md")
}

re!(LINK_LINE, r"^\[[^\]]*\]\(<?([^)>\s]+)>?\)$", "");
re!(PATH_LINE, r"^[`*_<]*([^\s`*_<>]+)[`*_>]*$", "");

/// The note or page the newest answer puts on a line of its own, which opens beside the thread
/// by itself (`besideDocument`): a document, a page, a table or a PDF, never a picture or a film.
pub fn beside_document(answer: &str, dir: Option<&str>) -> Option<String> {
    for raw in answer.split('\n') {
        let line = raw.trim();
        let bare = LINK_LINE.exec(line).and_then(|m| m.get(1)).or_else(|| PATH_LINE.exec(line).and_then(|m| m.get(1)));
        let Some(bare) = bare else { continue };
        let Some(path) = roots::referenced_files(&[bare], dir).into_iter().next() else { continue };
        if ends_with_any(&path, &["md", "markdown", "html", "htm", "csv", "tsv", "pdf"]) && !is_bookkeeping(&path) {
            return Some(path);
        }
    }
    None
}

/// How long ago, as the line over a document says it (`agoWords`, lower-cased as the pane
/// writes it): "just now", "3 min ago", "2 hours ago", "yesterday", "4 days ago".
pub fn edited_words(at: f64, now: f64) -> String {
    const MIN: f64 = 60_000.0;
    const HOUR: f64 = 60.0 * MIN;
    const DAY: f64 = 24.0 * HOUR;
    let gap = (now - at).max(0.0);
    let ago = if gap < MIN {
        "just now".to_string()
    } else if gap < HOUR {
        format!("{} min ago", (gap / MIN).floor())
    } else if gap < DAY {
        let h = (gap / HOUR).floor();
        format!("{h} hour{} ago", if h == 1.0 { "" } else { "s" })
    } else if gap < 2.0 * DAY {
        "yesterday".to_string()
    } else {
        format!("{} days ago", (gap / DAY).floor())
    };
    format!("Edited {ago}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests/the-side-panel-takes-three-quarters.test.mts
    #[test]
    fn the_pane_opens_on_three_quarters_and_a_drag_stays_inside_the_floors() {
        assert_eq!(OPENING_SPLIT, 0.72);
        assert!(OPENING_SPLIT > EVEN_SPLIT && OPENING_SPLIT > MIN_SPLIT && OPENING_SPLIT < MAX_SPLIT);
        assert_eq!(clamp_split(0.9), MAX_SPLIT);
        assert_eq!(clamp_split(0.1), MIN_SPLIT);
        assert!((split_from_pointer(400.0, 1600.0) - 0.75).abs() < 1e-5);
        assert!((split_from_pointer(1200.0, 1600.0) - 0.25).abs() < 1e-5);
        assert_eq!(split_from_pointer(10.0, 0.0), EVEN_SPLIT);
        // What a hand dragged outranks where it opens; nothing kept, or nothing readable, opens there.
        assert_eq!(read_split(None), OPENING_SPLIT);
        assert_eq!(read_split(Some(f32::NAN)), OPENING_SPLIT);
        assert_eq!(read_split(Some(0.41)), 0.41);
        assert_eq!(read_split(Some(0.95)), MAX_SPLIT);
    }

    // tests/artifacts-leave-room-for-the-conversation.test.mts
    #[test]
    fn a_narrow_card_gives_the_file_all_of_it_and_a_wide_one_keeps_both_readable() {
        assert_eq!(artifact_placement(999.0), Placement::Focus);
        assert_eq!(artifact_placement(1000.0), Placement::Beside);
        assert_eq!(artifact_placement(1001.0), Placement::Beside);
        assert!((artifact_fraction(0.75, 1000.0) - 0.6).abs() < 1e-6);
        assert!((artifact_fraction(0.2, 1000.0) - 0.56).abs() < 1e-6);
        // At 1280 by 760 the card is 1176 wide: the file opens on what leaves the chat 400.
        assert!((artifact_fraction(OPENING_SPLIT, 1176.0) * 1176.0 - 776.0).abs() < 1e-3);
        for url in ["http://localhost:3000/", "http://127.0.0.1:48765/?x=1"] {
            assert!(is_local_preview(url), "{url}");
        }
        for url in ["https://example.com", "http://localhost.evil.com:3000", "javascript:alert(1)", "designs/page.html"] {
            assert!(!is_local_preview(url), "{url}");
        }
    }

    // tests/a-table-opens-beside-and-any-file-can-move-to-the-side.test.mts
    #[test]
    fn what_is_read_opens_beside_and_what_is_looked_at_opens_big() {
        for f in ["home-routes.csv", "designs/w-1/leads.csv", "export.TSV", "notes/plan.md", "src/tax.ts", "notes.txt"] {
            assert_eq!(where_a_file_opens(f), Some(Opens::Beside), "{f}");
        }
        for f in ["a.png", "a.mp4", "a.pdf", "a.pptx", "designs/w-1/options.html", "routes.csv.png"] {
            assert_eq!(where_a_file_opens(f), Some(Opens::Full), "{f}");
        }
        // An address is not a file the big view takes, nor is a change or a kind nothing draws.
        for f in ["https://example.com/a.html", "http://localhost:3000/", "runs/w-1/the-change-it-made.change", "build.zip"] {
            assert_eq!(where_a_file_opens(f), None, "{f}");
        }
    }

    #[test]
    fn a_link_in_a_message_goes_big_beside_or_out() {
        assert_eq!(where_a_link_goes("notes/plan.md", false), LinkGoes::Beside);
        assert_eq!(where_a_link_goes("notes/plan.md", true), LinkGoes::Outside);
        assert_eq!(where_a_link_goes("shots/a.png", false), LinkGoes::Full);
        assert_eq!(where_a_link_goes("pages/checkout.html", false), LinkGoes::Full);
        assert_eq!(where_a_link_goes("https://example.com/notes", false), LinkGoes::Beside);
        assert_eq!(where_a_link_goes("http://localhost:3000", false), LinkGoes::Beside);
        assert_eq!(where_a_link_goes("exports/round-7.zip", false), LinkGoes::Outside);
        assert_eq!(where_a_link_goes("mailto:a@example.com", false), LinkGoes::Outside);
    }

    #[test]
    fn the_pane_knows_a_kind_by_the_address_first_then_the_ending() {
        assert_eq!(doc_kind("https://example.com/report.html"), Some(Kind::Website));
        assert_eq!(doc_kind("designs/a.HTML"), Some(Kind::Html));
        assert_eq!(doc_kind("notes/a.md?x=1"), Some(Kind::Markdown));
        assert_eq!(doc_kind("shots/a.webp"), Some(Kind::Image));
        assert_eq!(doc_kind("a.zip"), None);
        assert_eq!(pane_kind("exports/a.csv"), Kind::Table);
        assert_eq!(pane_kind("src/a.ts"), Kind::Text);
        assert_eq!(pane_kind("a.key"), Kind::Html);
        assert!(opens_in_pane("a.json") && !opens_in_pane("a.zip"));
    }

    // tests/a-document-opens-beside-the-message.test.mts, tests/a-deck-or-a-film-opens-in-the-app.test.mts
    #[test]
    fn the_crumb_says_which_file_not_where_the_disk_keeps_it() {
        let roots: Vec<String> = ["/Users/you/Zero/accounts/00000000/agentbox", "/Users/you/Zero/accounts/00000000/astral/designs", "/Users/you/Zero/accounts/00000000/agentbox/attachments", "/Users/you/Desktop/dev/zero"].map(String::from).to_vec();
        assert_eq!(crumb_parts("/Users/you/Zero/accounts/00000000/agentbox/STATE.md", &roots), ["agentbox", "STATE.md"]);
        assert_eq!(crumb_parts("/Users/you/Zero/accounts/00000000/astral/designs/w-74b0b5cd87/pane-card/the-card-you-picked.html", &roots), ["designs", "pane-card", "the-card-you-picked.html"]);
        assert_eq!(crumb_parts("/Users/you/Zero/accounts/00000000/astral/designs/w-74b0b5cd87/table.csv", &roots), ["designs", "table.csv"]);
        assert_eq!(crumb_parts("/some/where/w-74b0b5cd87/table.csv", &[]), ["some", "where", "table.csv"]);
        assert_eq!(crumb_parts("/some/where/else/entirely/deep/report.html", &[]), ["entirely", "deep", "report.html"]);
        assert_eq!(crumb_parts("file:///Users/you/Zero/accounts/00000000/agentbox/STATE.md", &roots), ["agentbox", "STATE.md"]);
        assert_eq!(crumb_parts("/a/direct-1/attachments/carried/0123456789abcdef/reports/plan.md", &["/a/direct-1".to_string()]), ["reports", "plan.md"]);
        assert_eq!(crumb_parts("/a/direct-1/attachments/k-plan.md", &["/a/direct-1".to_string()]), ["direct-1", "attachments", "k-plan.md"]);
    }

    // tests/a-note-or-page-on-its-own-line-opens-beside-the-thread.test.mts
    #[test]
    fn a_note_or_page_on_its_own_line_opens_beside_the_thread() {
        assert_eq!(beside_document("Tightened the middle.\n\napplication-draft.md\n", None).as_deref(), Some("application-draft.md"));
        assert_eq!(beside_document("Three options.\n\ndesigns/w-1/options.html", None).as_deref(), Some("designs/w-1/options.html"));
        assert_eq!(beside_document("`scripts/launch-video.md`", None).as_deref(), Some("scripts/launch-video.md"));
        assert_eq!(beside_document("**designs/w-1/round-2.html**", None).as_deref(), Some("designs/w-1/round-2.html"));
        assert_eq!(beside_document("[the draft](drafts/script.md)", None).as_deref(), Some("drafts/script.md"));
        assert_eq!(beside_document("/home/a/docs/agentbox/designs/w-1/a.html", Some("/home/a/docs/agentbox")).as_deref(), Some("designs/w-1/a.html"));
        assert_eq!(beside_document("It's saved in the draft: application-draft.md", None), None);
        assert_eq!(beside_document("Those are approved, see designs/w-1/options.html for the record.", None), None);
        assert_eq!(beside_document("One page.\n\nreports/q3-summary.pdf", None).as_deref(), Some("reports/q3-summary.pdf"));
        assert_eq!(beside_document("The deck is in reports/q3.pdf.", None), None);
        assert_eq!(beside_document("designs/w-1/the-row.png", None), None);
        assert_eq!(beside_document("renderer/src/App.tsx", None), None);
        assert_eq!(beside_document("STATE.md", None), None);
        assert_eq!(beside_document("CLAUDE.md", None), None);
        assert_eq!(beside_document("designs/w-1/a.html\ndesigns/w-1/b.html", None).as_deref(), Some("designs/w-1/a.html"));
        assert_eq!(beside_document("", None), None);
    }

    #[test]
    fn escape_closes_a_file_the_person_opened_and_leaves_one_the_thread_opened() {
        assert!(escape_closes_doc(false, false));
        assert!(!escape_closes_doc(true, false));
        assert!(escape_closes_doc(true, true));
    }

    #[test]
    fn only_what_a_link_can_draw_is_shared_and_only_words_take_notes() {
        assert!(can_share_file("notes/a.md") && can_share_file("a.CSV") && can_share_file("a.png"));
        assert!(!can_share_file("src/a.ts") && !can_share_file("a.zip"));
        assert!(takes_notes_sidebar("notes/a.md") && takes_notes_sidebar("src/a.ts") && takes_notes_sidebar("a.html"));
        assert!(!takes_notes_sidebar("a.png") && !takes_notes_sidebar("a.pdf") && !takes_notes_sidebar("https://example.com"));
    }

    #[test]
    fn the_line_over_a_document_says_when_it_was_edited() {
        let now = 1_000_000_000.0;
        assert_eq!(edited_words(now - 20_000.0, now), "Edited just now");
        assert_eq!(edited_words(now - 3.0 * 60_000.0, now), "Edited 3 min ago");
        assert_eq!(edited_words(now - 60.0 * 60_000.0, now), "Edited 1 hour ago");
        assert_eq!(edited_words(now - 5.0 * 3_600_000.0, now), "Edited 5 hours ago");
        assert_eq!(edited_words(now - 30.0 * 3_600_000.0, now), "Edited yesterday");
        assert_eq!(edited_words(now - 4.0 * 86_400_000.0, now), "Edited 4 days ago");
        assert!(newer_on_disk(10.0, Some(11.0)) && !newer_on_disk(10.0, Some(10.0)) && !newer_on_disk(10.0, None));
    }

    // tests/a-file-a-finished-cloud-thread-committed-opens-as-its-branch-has-it.test.mts
    #[test]
    fn saves_only_a_file_inside_the_project() {
        let roots = ["/docs".to_string(), "/repo/".to_string()];
        assert!(!pane_can_save(Some("/tmp/agentbox-branch-files/abc/w-1/README.md"), &roots));
        assert!(!pane_can_save(Some("/repo-old/README.md"), &roots));
        assert!(pane_can_save(Some("/repo/README.md"), &roots));
        assert!(pane_can_save(Some("/repo/.claude/worktrees/w-1/README.md"), &roots));
        assert!(pane_can_save(Some("/docs/designs/w-1/plan.md"), &roots));
        // Not yet found, or no project to compare with: as before.
        assert!(pane_can_save(None, &roots) && pane_can_save(Some(""), &roots));
        assert!(pane_can_save(Some("/anywhere/x.md"), &[]));
    }
}
