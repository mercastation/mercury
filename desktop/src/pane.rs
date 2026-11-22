//! A file opened beside the conversation (`DocPane` in `renderer/src/components/DocPane.tsx`,
//! laid out by `App.tsx` with `artifact-layout.ts` and `workspace-navigation.css`): the
//! conversation narrowed on the left, and on the right a card with the file's path, its marks,
//! when it was last edited and the file itself, scrolling on its own. Its rules are
//! `rules/pane.rs`.
//!
//! What opens it, as in the web window: a mention of a document, a table or a script in a
//! message, a row of the thread's Files list of the same kinds, the big view's mark that puts a
//! file beside, and the newest answer setting a document on a line of its own, which opens it by
//! itself. Pictures, films, PDFs, pages and decks open big (`viewer.rs`), and ⌘ sends any of them
//! to the system.
//!
//! The card's five marks: the globe, which shares the file by a link where the app shares files
//! (`zero:file-link` names a site); two panes side by side, which open the file's notes, the big
//! view standing in for them here; the frame's corners, which give the file the whole window,
//! the big view; open outside; and close. The divider at the card's left edge is dragged to give
//! the file more or less of the body, and remembered; a double press puts it back where it opens.
//! A card under a thousand points wide gives the file all of it, the reply box under it.
//!
//! What is drawn is read off this Mac's disk, as a picture in a message is: a document as the
//! big view lays one out, a script or any text as its lines, a table as its grid, a picture fitted
//! into the card. It is read again when it changes on disk while it is open (`FOLLOW_FILE_MS`).
//! Nothing here is written: the web window's pane lets a person type into a document.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use rune::{Cursor, Drag, El, Phase, TextStyle, Tx, col, el, icon, row, text};
use serde_json::Value;

use crate::files::{self, Files};
use crate::pictures::{Look, Want};
use crate::rules::pane::{self as rules, Kind, Placement};
use crate::theme::*;
use crate::{State, View, icons, viewer};

/// The card's air inside the pane, all round (`.doc-pane .doc-card { margin: 12px }`).
const AIR: f32 = 12.0;
/// The scroller a document or a script is read in.
const STAGE_KEY: &str = "pane-stage";
/// What the globe says: the panel that makes a file's link is the web window's.
pub const SHARE_NOT_HERE: &str = "A link to share this file is made in Mercury's own window.";

/// What the file is drawn from, read off the disk.
enum Read {
    Doc(Rc<str>),
    Lines(Rc<viewer::Code>),
    Table(Rc<crate::table::Grid>),
}

/// The file open beside a conversation.
pub struct Doc {
    pub product: String,
    /// The thread it was opened over, whose own copy of a file comes first.
    pub thread: String,
    /// The path as it was pressed or named.
    pub src: String,
    /// Opened by the thread itself rather than by a press: Escape then leaves the thread.
    pub auto: bool,
    /// Where it is looked for, likeliest first, and the project's folders its path is told from.
    roots: Vec<String>,
    crumb_roots: Vec<String>,
    found: Option<String>,
    read: Option<Read>,
    /// The file's time on disk when it was read, in milliseconds, and when the disk was last asked.
    mtime: f64,
    looked: f64,
}

impl Doc {
    pub fn kind(&self) -> Kind {
        rules::pane_kind(&self.src)
    }
    /// Finds the file and reads what is drawn as words.
    fn land(&mut self, now: f64) {
        self.looked = now;
        self.found = viewer::place_of(&self.roots, &self.src);
        self.mtime = self.found.as_deref().map_or(0.0, modified);
        let kind = self.kind();
        self.read = self.found.as_deref().filter(|_| matches!(kind, Kind::Markdown | Kind::Text | Kind::Table)).and_then(viewer::read_text).map(|text| match kind {
            Kind::Markdown => Read::Doc(Rc::from(text.as_str())),
            Kind::Table => Read::Table(Rc::new(crate::table::Grid::read(&text, &self.src))),
            _ => Read::Lines(Rc::new(viewer::code_of(&text, true))),
        });
    }
    /// Asked of the disk again: read again when it has a newer time than the copy on screen, or
    /// when it was not there and has come (`newerOnDisk`).
    fn follow(&mut self, now: f64) -> bool {
        self.looked = now;
        let found = self.found.clone().or_else(|| viewer::place_of(&self.roots, &self.src));
        let at = found.as_deref().map(modified);
        if self.found.is_none() && found.is_some() || rules::newer_on_disk(self.mtime, at) {
            self.land(now);
            return true;
        }
        false
    }
}

/// When a file was last written, in milliseconds.
fn modified(path: &str) -> f64 {
    std::fs::metadata(path).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0.0, |d| d.as_millis() as f64)
}

/// The pane: the file open, and what is kept about it.
#[derive(Default)]
pub struct Pane {
    pub doc: Option<Doc>,
    /// The split while the divider is being dragged, until it is let go.
    pub dragging: Option<f32>,
    /// A file the thread opened by itself that the person closed, until its next answer
    /// (`closedBeside`), as `product`, `id`, the path and the answer.
    closed_beside: HashSet<String>,
    /// The file each thread left had open beside it, and the thread's newest answer then
    /// (`leavePlace`): coming back with nothing new puts it back.
    places: HashMap<String, (f64, Option<(String, bool)>)>,
    /// What last decided whether the newest answer's file opens by itself: it is asked again
    /// only when the thread, its answer or the room beside it changes, as the web window's effect.
    auto_asked: Option<String>,
    /// What the app says of sharing the open file (`zero:file-link`), by `product/path`, and
    /// its answers landing from the thread that asked.
    links: HashMap<String, Value>,
    landed: Links,
}

/// The newest answer of a thread, which a place left is kept against (`arrivedAt`).
fn news(item: &crate::data::Item) -> f64 {
    let at = item.wrote_at("result");
    if at > 0.0 { at } else { item.created_at.unwrap_or(0.0) }
}

/// The project's folders a path is told from (`docRoots` in `App.tsx`).
fn crumb_roots(s: &State, product: &str) -> Vec<String> {
    let Some(p) = s.snap.products.iter().find(|p| p.slug == product) else { return Vec::new() };
    let named = |field: &str| p.rest.get(field).and_then(Value::as_str).unwrap_or("").to_string();
    let (dir, repo) = (named("dir"), named("repoPath"));
    let mut out = Vec::new();
    if !dir.is_empty() {
        out.extend([dir.clone(), format!("{dir}/designs"), format!("{dir}/attachments")]);
    }
    if !repo.is_empty() {
        out.extend([repo.clone(), format!("{repo}/designs")]);
    }
    out
}

/// The newest version of the file a path is one version of, by the edits saved in the app
/// (`newestOfFamily`).
fn newest_of_family(path: &str, item: &crate::data::Item, now: f64) -> String {
    use crate::rules::thread_files::{Named, thread_files};
    let edits: Vec<String> = item.rest.get("artifacts").and_then(|a| a.get("edits")).and_then(Value::as_array).map(|all| all.iter().filter_map(|e| e.get("path").and_then(Value::as_str).map(str::to_string)).collect()).unwrap_or_default();
    if edits.is_empty() {
        return path.to_string();
    }
    let named: Vec<Named> = std::iter::once(path.to_string()).chain(edits).map(|p| Named { path: p, at: 0.0 }).collect();
    let bare = path.strip_prefix("./").unwrap_or(path);
    thread_files(&named, now).into_iter().find(|f| f.versions.iter().any(|v| v.path == bare)).and_then(|f| f.versions.last().map(|v| v.path.clone())).unwrap_or_else(|| path.to_string())
}

impl State {
    /// The file open beside the conversation on screen, if one is.
    pub fn beside(&self) -> Option<&Doc> {
        let View::Thread { id, product } = &self.view else { return None };
        self.pane.doc.as_ref().filter(|d| &d.thread == id && &d.product == product)
    }
    /// How the pane stands now: beside the conversation, taking the card, or not open.
    pub fn pane_placement(&self) -> Option<Placement> {
        self.beside().map(|_| rules::artifact_placement(self.page_size().0))
    }
    /// The conversation's box beside the pane: the page, less what the file takes.
    pub fn conversation_size(&self) -> (f32, f32) {
        let (w, h) = self.page_size();
        match self.pane_placement() {
            Some(Placement::Beside) => (w - pane_width(self, w), h),
            _ => (w, h),
        }
    }
    /// A file opened beside the conversation by a press, or by the thread (`auto`). Another file
    /// open there gives way to it.
    pub fn open_beside(&mut self, src: &str, auto: bool) {
        let View::Thread { id, product } = self.view.clone() else { return };
        let Some(item) = self.open_item() else { return };
        let roots = Files::of(self, item).roots;
        let crumb_roots = crumb_roots(self, &product);
        let src = crate::rules::paths::decode_uri(src.strip_prefix("file://").unwrap_or(src));
        let mut doc = Doc { product, thread: id, src, auto, roots, crumb_roots, found: None, read: None, mtime: 0.0, looked: 0.0 };
        doc.land(crate::clock());
        self.close_files();
        self.pointed.hint = None;
        self.pane.doc = Some(doc);
        self.ask_about_sharing();
    }
    /// A file pressed in a message or in the Files list, opened beside at its newest version.
    fn open_beside_newest(&mut self, src: &str) {
        let src = match self.open_item() {
            Some(item) => newest_of_family(src, item, self.now),
            None => src.to_string(),
        };
        self.open_beside(&src, false);
    }
    /// The close mark, Escape: the file is put down (`putDocDown`). One the thread opened by
    /// itself stays shut until the thread's next answer.
    pub fn close_beside(&mut self) {
        if let Some(doc) = self.pane.doc.take()
            && doc.auto
            && let Some(item) = self.snap.items.iter().find(|i| i.id == doc.thread && i.product == doc.product)
        {
            self.pane.closed_beside.insert(closed_key(item, &doc.src));
        }
        self.pane.dragging = None;
    }
    /// Leaving a thread takes its file with it, kept against its newest answer (`leavePlace`).
    pub(crate) fn leave_the_pane(&mut self) {
        let View::Thread { id, product } = self.view.clone() else { return };
        let key = format!("{product}/{id}");
        let at = self.open_item().map_or(0.0, news);
        let doc = self.pane.doc.take().filter(|d| d.thread == id && d.product == product).map(|d| (d.src, d.auto));
        self.pane.places.insert(key, (at, doc));
        self.pane.dragging = None;
        self.pane.auto_asked = None;
    }
    /// Coming back to a thread with nothing new since puts its file back (`placeToReturnTo`).
    pub(crate) fn come_back_to_the_pane(&mut self) {
        let View::Thread { id, product } = self.view.clone() else { return };
        let key = format!("{product}/{id}");
        let at = self.open_item().map_or(0.0, news);
        match self.pane.places.get(&key).cloned() {
            Some((was, doc)) if was == at => {
                if let Some((src, auto)) = doc {
                    self.open_beside(&src, auto);
                }
            }
            Some(_) => {
                self.pane.places.remove(&key);
            }
            None => {}
        }
    }
    /// The big view's mark that puts its file beside the conversation instead (`onBeside`).
    pub fn put_big_beside(&mut self, newest: &str) {
        self.close_big();
        self.open_beside(newest, false);
    }
    /// The frame's corners in the pane: the file over the whole window, in the big view, which
    /// gives it back to the pane when it closes (`docFull`, `fullAt`).
    pub fn pane_full(&mut self) {
        let Some(src) = self.beside().map(|d| d.src.clone()) else { return };
        self.open_big(&src, None);
    }
    /// The open mark: the file in the app that owns it, an address in the browser
    /// (`openArtifactExternally`).
    pub fn pane_outside(&self) {
        if let Some(doc) = self.beside() {
            match &doc.found {
                Some(found) => files::open_with_system(found),
                None if doc.kind() == Kind::Website => files::open_with_system(&doc.src),
                None => {}
            }
        }
    }
    /// Escape over a conversation with a file beside it: true when it closed the file, which a
    /// file the thread opened by itself does not stand in the way of (`escapeClosesDoc`).
    pub fn escape_beside(&mut self) -> bool {
        match self.beside() {
            Some(doc) if rules::escape_closes_doc(doc.auto, false) => {
                self.close_beside();
                true
            }
            _ => false,
        }
    }
    /// What the app says of sharing the file now open, asked off the main thread.
    fn ask_about_sharing(&mut self) {
        let Some((product, thread, src)) = self.pane.doc.as_ref().map(|d| (d.product.clone(), d.thread.clone(), d.src.clone())) else { return };
        if !rules::can_share_file(&src) {
            return;
        }
        let key = format!("{product}/{src}");
        // A picture's fixture says what the app answered.
        if let Some(answer) = self.fixture.get("fileLink").cloned() {
            self.pane.links.insert(key, answer);
            return;
        }
        let crate::source::Source::Door(door) = &self.source else { return };
        let Some((url, token)) = door.now() else { return };
        let ask = serde_json::json!({ "product": product, "thread": thread, "src": src });
        let links = self.pane.landed.clone();
        std::thread::spawn(move || {
            if let Ok(answer) = crate::source::call(&url, &token, "zero:file-link", &ask) {
                links.lock().unwrap().insert(key, answer);
            }
        });
    }
}

/// The key a file the thread opened by itself is kept shut under (`besideKey`).
fn closed_key(item: &crate::data::Item, src: &str) -> String {
    format!("{}\n{}\n{}\n{}", item.product, item.id, src, item.result.as_deref().unwrap_or(""))
}

/// How wide the pane is in a body `body` wide: the kept split, held so both stay readable.
fn pane_width(s: &State, body: f32) -> f32 {
    let kept = s.pane.dragging.unwrap_or_else(|| rules::read_split(s.picks.get().doc_split));
    body * rules::artifact_fraction(kept, body)
}

/// What a press on a link in a message does (`openHref` in `Focus.tsx`): big, beside, or with
/// the system. An address the web window draws in the pane goes to the browser here, which has
/// no page inside it.
pub fn open_link(s: &mut State, href: &str, new_tab: bool) {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("mailto:") {
        return files::open_with_system(href);
    }
    match rules::where_a_link_goes(href, new_tab) {
        rules::LinkGoes::Full => s.open_big(href, None),
        rules::LinkGoes::Beside => s.open_beside_newest(href),
        rules::LinkGoes::Outside => files::open_found(s, href),
    }
}

/// A row of the thread's Files list pressed: what the big view looks at opens big, anything else
/// beside (`opensBig(f.src) ? openBig(f.src) : onOpenDoc(f.src)`).
pub fn open_listed(s: &mut State, src: &str) {
    match rules::where_a_file_opens(src) {
        Some(rules::Opens::Full) => s.open_big(src, None),
        _ => s.open_beside(src, false),
    }
}

/// What the window tends while a file is beside a conversation: the newest answer's file opening
/// by itself, the file read again when it changed on disk, and what the app said about sharing.
pub fn tend(s: &mut State, now: f64, tx: &mut Tx) -> bool {
    let mut changed = false;
    // What the app answered about sharing.
    let landed: Vec<(String, Value)> = s.pane.landed.lock().unwrap().drain().collect();
    if !landed.is_empty() {
        s.pane.links.extend(landed);
        changed = true;
    }
    changed |= open_by_itself(s);
    if s.pane.doc.is_some() {
        changed |= follow_the_file(s, now);
        tx.wake_after(std::time::Duration::from_millis(rules::FOLLOW_FILE_MS as u64));
    }
    changed
}

/// The open file asked of the disk again once `FOLLOW_FILE_MS` have gone by since it last was,
/// and read again when an agent has written it since (`askAtMostEvery(FOLLOW_FILE_MS, …)`).
fn follow_the_file(s: &mut State, now: f64) -> bool {
    match s.pane.doc.as_mut() {
        Some(doc) if now - doc.looked >= rules::FOLLOW_FILE_MS => doc.follow(now),
        _ => false,
    }
}

/// The note or page the newest answer sets on a line of its own opens beside it, never over a
/// file opened by hand, never on a card too narrow to have a side, and never again once closed
/// until the next answer (`besideDocument` and the effect under it in `App.tsx`).
fn open_by_itself(s: &mut State) -> bool {
    let Some(item) = s.open_item() else {
        s.pane.auto_asked = None;
        return false;
    };
    let dir = s.snap.products.iter().find(|p| p.slug == item.product).and_then(|p| p.rest.get("dir")).and_then(Value::as_str).map(str::to_string);
    let Some(named) = rules::beside_document(item.result.as_deref().unwrap_or(""), dir.as_deref()) else { return false };
    let src = newest_of_family(&named, item, s.now);
    let room = rules::artifact_placement(s.page_size().0) == Placement::Beside;
    let asked = format!("{}\n{}\n{}\n{}\n{room}", item.product, item.id, item.result.as_deref().unwrap_or(""), src);
    let closed = s.pane.closed_beside.contains(&closed_key(item, &src));
    // Found first: a path the agent got wrong would open half a window saying it is nowhere.
    let found = viewer::place_of(&Files::of(s, item).roots, &src).is_some();
    if s.pane.auto_asked.as_deref() == Some(asked.as_str()) {
        return false;
    }
    s.pane.auto_asked = Some(asked);
    if !room || closed || !found {
        return false;
    }
    if let Some(cur) = s.beside()
        && (!cur.auto || cur.src == src)
    {
        return false;
    }
    s.open_beside(&src, true);
    true
}

/// What a picture of the web window had open beside its conversation (`docOpen`), and where its
/// divider was dragged to (`docSplit`).
pub fn pictured(s: &mut State) {
    if let Some(split) = s.fixture.remove("docSplit").and_then(|v| v.as_f64()) {
        s.picks.change(|p| p.doc_split = Some(rules::clamp_split(split as f32)));
    }
    if let Some(src) = s.fixture.remove("docOpen") {
        if let Some(src) = src.as_str() {
            s.open_beside(src, false);
        }
    }
}

/* ------------------------------ the card ------------------------------ */

/// A mark in the card's corner (`.doc-mark`): its glyph faint, and under the pointer in the
/// text's own colour with nothing behind it, at once both ways (the newer look takes the
/// `--doc-wash` away).
fn mark(glyph: &'static rune::Icon, size: f32, help: &'static str) -> El {
    row().size(28.0, 28.0).radius(3.0).center().hover_group().cursor(Cursor::Pointer).help(help).child(icon(glyph, size).color(dim()).group_color(ink()))
}

/// How the card holds its file, by kind: a text keeps the glass card with its line under the
/// head; a picture and a table stand on the page's own ground.
fn glass(kind: Kind) -> bool {
    !matches!(kind, Kind::Image | Kind::Table)
}

/// The head of the card: the path, and the marks (`.doc-head`).
fn head(s: &State, doc: &Doc, w: f32) -> El {
    let crumb = rules::crumb_parts(doc.found.as_deref().unwrap_or(&doc.src), &doc.crumb_roots);
    let last = crumb.len().saturating_sub(1);
    let font = sans(11.0, 400.0).lh(14.0);
    let mut path = row().min_w(0.0).shrink().gap(6.0).items_center().help(doc.found.clone().unwrap_or_else(|| doc.src.clone()));
    for (i, part) in crumb.iter().enumerate() {
        let mut piece = row().gap(6.0).items_center().min_w(0.0);
        if i > 0 {
            piece = piece.child(text("›").font(font).color(muted()).opacity(0.55));
        }
        let (words, tone) = if i == last { (crate::rules::roots::shown_name(doc.found.as_deref().unwrap_or(&doc.src), s.now), dim()) } else { (part.clone(), muted()) };
        let words = if words.is_empty() { part.clone() } else { words };
        piece = piece.child(text(words).font(font).color(tone).truncate());
        path = path.child(if i == last { piece.shrink() } else { piece });
    }
    let kind = doc.kind();
    let mut marks = row().gap(6.0).items_center();
    // The globe, where the app shares files and this one can be.
    let key = format!("{}/{}", doc.product, doc.src);
    let shares = rules::can_share_file(&doc.src) && s.pane.links.get(&key).is_some_and(|l| l.get("ok") == Some(&Value::Bool(true)) && l.get("site").and_then(Value::as_str).is_some_and(|site| !site.is_empty()));
    // Its panel, which makes the link and copies it, is the web window's (see what still differs):
    // a press says so, rather than a mark that answers the pointer and goes nowhere.
    if shares {
        marks = marks.child(mark(&icons::PANE_GLOBE, 16.0, "Share").key("pane-globe").on_press_tx(|s: &mut State, tx| s.tell(SHARE_NOT_HERE, tx)));
    }
    // The notes on it, which open with it over the whole window.
    if rules::takes_notes_sidebar(&doc.src) {
        marks = marks.child(mark(&icons::PANE_NOTES, 16.0, "Show notes").key("pane-notes").on_press(|s: &mut State| s.pane_full()));
    }
    if crate::rules::pane::full_screen_takes(&doc.src) {
        marks = marks.child(mark(&icons::PANE_FULL, 16.0, "Give this document the full workspace").key("pane-full").on_press(|s: &mut State| s.pane_full()));
    }
    marks = marks
        .child(mark(&icons::FILE_OUTSIDE, 16.0, if kind == Kind::Change { "Open this file in your editor" } else { "Open outside Mercury" }).key("pane-outside").on_press(|s: &mut State| s.pane_outside()))
        .child(mark(&icons::FILE_CLOSE, 16.0, "Close this document (esc)").key("pane-close").on_press(|s: &mut State| s.close_beside()));
    let (pad_x, line) = if glass(kind) { (16.0, true) } else { (12.0, false) };
    let head = row().w(w).h(if line { 53.0 } else { 52.0 }).pl(pad_x).pr(pad_x).gap(8.0).items_center().child(path).child(el().grow(1.0)).child(marks);
    if line { head.border_sides([0.0, 0.0, 1.0, 0.0], edge()).border_layout() } else { head.fill(paper()) }
}

/// The page's own ground under a picture or a table (`--doc-page`).
fn paper() -> rune::Color {
    rgb(40, 38, 37)
}

/// A sentence where the file would be (`.doc-missing`).
fn missing(words: &str, w: f32) -> El {
    col().w(w).padding(40.0).child(text(words).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(muted()).wrap())
}

/// The file itself, in a box `w` by `h` under the head.
fn body(s: &State, doc: &Doc, w: f32, h: f32) -> El {
    let kind = doc.kind();
    let name = crate::rules::roots::shown_name(&doc.src, s.now);
    if doc.found.is_none() {
        return missing("That file could not be found.", w);
    }
    match (kind, &doc.read) {
        (Kind::Markdown, Some(Read::Doc(words))) => {
            let folder = doc.found.as_deref().and_then(|f| f.rsplit_once('/')).map_or(String::new(), |(dir, _)| dir.to_string());
            let byline = if doc.mtime > 0.0 { rules::edited_words(doc.mtime, s.now) } else { String::new() };
            let held = s.pictures.held();
            let version = (doc.found.clone(), doc.mtime.to_bits(), w.to_bits(), h.to_bits(), held, (s.now / 60_000.0).floor() as i64, s.viewer.outline_open);
            rune::memo("pane-doc", version, || viewer::doc_page(s, words, &folder, byline.clone(), (w, h), STAGE_KEY, None, true))
        }
        (Kind::Text, Some(Read::Lines(code))) => {
            let version = (doc.found.clone(), doc.mtime.to_bits(), w.to_bits(), h.to_bits());
            rune::memo("pane-lines", version, || {
                let wide = (viewer::code_widest(code) as f32 * viewer::CODE_ADVANCE + 56.0 + 32.0).max(w);
                let lines = col().w(wide).pt(18.0).pb(40.0).children(viewer::code_lines(code));
                el().size(w, h).child(el().key(STAGE_KEY).size(w, h).scroll_xy().child(lines))
            })
        }
        (Kind::Table, Some(Read::Table(grid))) => crate::table::view(s, grid, &doc.src, (w, h)),
        (Kind::Image, _) => picture(s, doc, w, h),
        (Kind::Markdown | Kind::Text | Kind::Table, None) => missing(&format!("{name} could not be read."), w),
        // A page, a PDF, a film, a change or a site: the web window draws each in a frame or a
        // player this window has none of, so the card says what the file is and opens it outside.
        _ => outside(doc, &name, w, h),
    }
}

/// A picture in the card, the whole of it, never larger than its own pixels, in the middle of
/// the room 16 points in from either side and over the card's foot (`.doc-picture`).
fn picture(s: &State, doc: &Doc, w: f32, h: f32) -> El {
    let room = ((w - 32.0).max(1.0), (h - 16.0).max(1.0));
    let places = vec![doc.found.clone().unwrap_or_default()];
    let look = s.pictures.look(places, Want::Within(s.pictures.pixels(room.0), s.pictures.pixels(room.1)));
    let held = match look {
        Look::Here(held) => held,
        Look::Missing => return missing("That picture could not be drawn.", w),
        Look::Arriving(_) => return el().size(w, h),
    };
    let (nw, nh) = (held.natural.0.max(1) as f32, held.natural.1.max(1) as f32);
    let scale = (room.0 / nw).min(room.1 / nh).min(1.0);
    let (pw, ph) = (nw * scale, nh * scale);
    el().size(w, h).child(el().absolute((w - pw) / 2.0, (h - 16.0 - ph) / 2.0).size(pw, ph).radius(10.0).clip_rounded(10.0).pixels(&held.pixels))
}

/// What the card holds for a file this window cannot draw inside it: what it is, and Open
/// outside, as the big view says it.
fn outside(doc: &Doc, name: &str, w: f32, h: f32) -> El {
    let what = match doc.kind() {
        Kind::Html => "a page, which opens in your browser",
        Kind::Film => "a film, which plays in the app that opens it",
        Kind::Pdf => "a PDF, which opens in the app that reads it",
        Kind::Website => "a page on the web, which opens in your browser",
        Kind::Change => "a change, which opens in your editor",
        _ => "a file this window cannot draw",
    };
    let button = row()
        .h(28.0)
        .pl(10.0)
        .pr(12.0)
        .gap(6.0)
        .items_center()
        .radius(6.0)
        .border(1.0, rule())
        .border_layout()
        .hover(crate::feel::wash())
        .cursor(Cursor::Pointer)
        .child(icon(&icons::FILE_OUTSIDE, 14.0).color(ink()))
        .child(label("Open outside", 13.0, 400.0, ink()))
        .on_press(|s: &mut State| s.pane_outside());
    let card = col().items_center().gap(14.0).px(24.0).child(text(format!("{name} is {what}.")).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(dim()).wrap()).child(button);
    row().size(w, h).center().child(card)
}

/// The divider on the card's left edge (`.doc-grip`): nine points to reach for over the card's
/// edge, the resize pointer, and under the pointer or while dragged a faint pill on it. Dragged,
/// the file takes what is right of the pointer; a double press puts it back where it opens.
fn grip(s: &State, body: (f32, f32), body_x: f32, card_h: f32) -> El {
    let dragging = s.pane.dragging.is_some();
    let pill = el().absolute(1.0, (card_h - 30.0) / 2.0).size(7.0, 30.0).radius(6.0).fill(rule());
    let pill = if dragging { pill } else { pill.group_opacity(0.0, 1.0) };
    let (body_w, _) = body;
    // Kept as it moves and written down once it is let go, as the web window writes its own.
    el().size(9.0, card_h)
        .key("pane-grip")
        .hover_group()
        .cursor(Cursor::ResizeColumn)
        .label("Resize the document")
        .child(pill)
        .on_drag(move |s: &mut State, d: &mut Drag, _| {
            let [x, _] = d.window_point();
            let split = rules::split_from_pointer(x - body_x - AIR, body_w);
            match d.phase {
                Phase::Down => s.pane.dragging = Some(rules::read_split(s.picks.get().doc_split)),
                Phase::Move => s.pane.dragging = Some(split),
                Phase::Up => {
                    let moved = s.pane.dragging.take().filter(|_| !d.cancelled() && (d.dx != 0.0));
                    if let Some(split) = moved {
                        s.picks.change(|p| p.doc_split = Some(split));
                    }
                }
            }
        })
        .on_double_press(|s: &mut State| s.picks.change(|p| p.doc_split = Some(rules::OPENING_SPLIT)))
}

/// The card, `w` by `h`, holding the file (`.doc-card`).
/// `headed`: the head is in the card. Over the whole card it is not: the web window moves it up
/// into the strip it gave up, hides the path there, and draws its marks only over a file the
/// person sent to the whole window, which this is not (`[data-chrome]` is set only then), so
/// none is drawn and Escape is the way out, as there.
fn card(s: &State, doc: &Doc, w: f32, h: f32, headed: bool) -> El {
    let kind = doc.kind();
    let inner_w = w - 2.0;
    let head_h = if !headed { 0.0 } else if glass(kind) { 53.0 } else { 52.0 };
    let foot = if kind == Kind::Image { 16.0 } else { 0.0 };
    let below = (h - 2.0 - head_h - foot).max(1.0);
    // While the big view stands over the window it covers this card, whose file is not drawn.
    let file = if s.viewer.open.is_some() { el().size(inner_w, below) } else { body(s, doc, inner_w, below + foot) };
    let inside = col().w(inner_w).h(h - 2.0).children(headed.then(|| head(s, doc, inner_w))).child(el().w(inner_w).h(below + foot).clip_rounded(0.0).child(file));
    let card = el().size(w, h).border(1.0, if glass(kind) { rule() } else { edge() }).border_layout();
    let card = if glass(kind) {
        card.radius(12.0)
            .linear_gradient(145.0_f32.to_radians(), [(0.0, rgba(239, 235, 231, 0.05)), (1.0, rgba(40, 38, 37, 0.25))])
            .box_shadow(shadow(0.0, 1.0, 0.0, 0.0, rgba(239, 235, 231, 0.04), true))
            .clip_rounded(12.0)
    } else {
        let r = if kind == Kind::Image { 6.0 } else { 0.0 };
        card.radius(r).fill(paper()).clip_rounded(r)
    };
    card.child(inside).role("complementary").label(crate::rules::roots::shown_name(&doc.src, s.now))
}

/// The page over a conversation with a file beside it, in place of the conversation alone: the
/// conversation `conversation`, already as wide as `State::conversation_size` leaves it, and the
/// pane on its right, or, on a card too narrow, the file over the whole card and the reply box
/// `dock` under it.
pub fn page(s: &State, conversation: El, dock: impl FnOnce(f32) -> El) -> El {
    let (Some(doc), Some(placement)) = (s.beside(), s.pane_placement()) else { return conversation };
    let (w, h) = s.page_size();
    match placement {
        Placement::Beside => {
            let pane_w = pane_width(s, w);
            let conv_w = w - pane_w;
            let (card_w, card_h) = (pane_w - 2.0 * AIR, h - 2.0 * AIR);
            let body_x = crate::chrome::card_left(s);
            let pane = el()
                .size(pane_w, h)
                .child(el().absolute(AIR, AIR).child(card(s, doc, card_w, card_h, true)))
                .child(el().absolute(AIR - 4.0, AIR).child(grip(s, (w, h), body_x, card_h)));
            row().w(w).h(h).child(el().w(conv_w).h(h).child(conversation)).child(pane)
        }
        Placement::Focus => {
            // The card from the top of the whole card, the strip given up to it, and the reply
            // box under it with twelve points of air (`[data-artifact-layout="focus"]`).
            let whole = s.card_size();
            let dock = dock(whole.0);
            let dock_h = rune::memo::layout_size("pane-dock").map_or(55.0, |[_, tall]| tall);
            col().w(whole.0)
                .h(whole.1)
                .child(el().w(whole.0).h((whole.1 - dock_h).max(1.0)).child(el().absolute(AIR, AIR).child(card(s, doc, whole.0 - 2.0 * AIR, (whole.1 - dock_h - 2.0 * AIR).max(1.0), false))))
                .child(dock.key("pane-dock"))
        }
    }
}

/// Whether the file takes the whole card, the strip over the conversation given up to it.
pub fn takes_the_card(s: &State) -> bool {
    s.pane_placement() == Some(Placement::Focus) && s.viewer.open.is_none()
}

/// What is left of the strip over a conversation whose file takes the card: the Inbox, back and
/// forward, standing over the card's top edge (`.topbar` at no height, its `.back-forward`).
pub fn strip(s: &State) -> El {
    let square = |glyph: &'static rune::Icon, color| row().size(26.0, 26.0).radius(6.0).center().child(icon(glyph, 15.0).color(color));
    let x = crate::chrome::card_left(s) + 20.0;
    row()
        .absolute(x, 27.0)
        .h(26.0)
        .items_center()
        .child(square(&icons::INBOX, ink()).hover(crate::feel::film()).cursor(Cursor::Pointer).on_press(|s: &mut State| {
            s.go(View::Inbox);
            s.show_tab(crate::data::Tab::Needs);
        }))
        .child(square(&icons::BACK, ink()).ml(12.0).hover(crate::feel::film()).cursor(Cursor::Pointer).help(crate::chrome::back_title(s)).on_press(|s: &mut State| s.leave_thread()))
        .child(square(&icons::FORWARD, muted()).ml(2.0).opacity(0.45))
}

/// The keys while a file is beside the conversation: Escape puts down a file the person opened.
/// Every other key goes on to the conversation, as in the web window, whose pane takes none.
pub fn key(s: &mut State, e: &rune::KeyEvent) -> bool {
    e.key == rune::edit::Key::Escape && !e.mods.cmd && !e.mods.ctrl && !e.mods.alt && s.escape_beside()
}

/// The answers the app gives about sharing, landing from the thread that asked.
pub type Links = std::sync::Arc<std::sync::Mutex<HashMap<String, Value>>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feel::tests::{first, key, press, rect_of, run, with_a_picture};
    use rune::{Key, Runtime};

    /// A conversation whose answer is `result`, in an invented project holding `notes/plan.md`
    /// and `shots/shot.png`. Gives the window and the project's folder.
    fn with_a_plan(name: &str, result: &str) -> (Runtime<State>, std::path::PathBuf) {
        let result = result.to_string();
        // The note is written before the window opens on the thread, as an agent's is before
        // its answer names it.
        let (mut rt, shot) = with_a_picture(name, move |shot| {
            let dir = std::path::Path::new(shot).parent().and_then(std::path::Path::parent).expect("its folder");
            std::fs::create_dir_all(dir.join("notes")).expect("a folder for the note");
            std::fs::write(dir.join("notes/plan.md"), "# The plan\n\nRound each line.\n").expect("an invented note");
            result.clone()
        });
        let dir = shot.parent().and_then(std::path::Path::parent).expect("its folder").to_path_buf();
        rt.frame(1.0 / 60.0);
        (rt, dir)
    }

    fn middle(rt: &Runtime<State>, key: &str) -> (f32, f32) {
        let [x, y, w, h] = rect_of(rt, key).unwrap_or_else(|| panic!("{key} is drawn"));
        (x + w / 2.0, y + h / 2.0)
    }

    #[test]
    fn a_document_opens_beside_and_the_conversation_keeps_what_the_file_leaves_it() {
        let (mut rt, dir) = with_a_plan("pane-open", "Read [the plan](notes/plan.md) first.");
        let at = first(&mut rt, rune::Cursor::Pointer);
        press(&mut rt, at.0, at.1);
        let doc = rt.state.beside().expect("the plan opens beside");
        assert_eq!((doc.kind(), doc.auto), (Kind::Markdown, false));
        assert!(matches!(doc.read, Some(Read::Doc(_))), "its words are read off the disk");
        // At 1280 by 760 the body is 1176 wide: the file opens on what leaves the chat 400.
        assert_eq!(rt.state.conversation_size().0, 400.0);
        assert_eq!(rt.state.pane_placement(), Some(Placement::Beside));
        let card = rect_of(&rt, "pane-grip").expect("the divider is drawn");
        assert_eq!((card[0], card[1]), (82.0 + 400.0 + 12.0 - 4.0, 104.0 + 12.0));
        // Escape puts it down, and the next Escape leaves the thread.
        key(&mut rt, Key::Escape);
        assert!(rt.state.beside().is_none());
        assert!(matches!(rt.state.view, View::Thread { .. }));
        key(&mut rt, Key::Escape);
        assert!(!matches!(rt.state.view, View::Thread { .. }));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_picture_still_opens_big_and_the_files_list_opens_by_kind() {
        let (mut rt, dir) = with_a_plan("pane-kinds", "Done.");
        rt.update(|s, _| open_link(s, "shots/shot.png", false));
        rt.frame(1.0 / 60.0);
        assert!(rt.state.viewer.open.is_some() && rt.state.beside().is_none(), "a picture is looked at, big");
        key(&mut rt, Key::Escape);
        rt.update(|s, _| open_listed(s, "notes/plan.md"));
        rt.frame(1.0 / 60.0);
        assert!(rt.state.viewer.open.is_none());
        assert_eq!(rt.state.beside().map(|d| d.src.clone()).as_deref(), Some("notes/plan.md"));
        rt.update(|s, _| open_listed(s, "shots/shot.png"));
        rt.frame(1.0 / 60.0);
        assert!(rt.state.viewer.open.is_some(), "a picture of the list opens big over the pane");
        assert!(rt.state.beside().is_some(), "and the pane stays under it");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_corner_marks_open_it_big_put_it_back_beside_and_close_it() {
        let (mut rt, dir) = with_a_plan("pane-marks", "Done.");
        rt.update(|s, _| s.open_beside("notes/plan.md", false));
        rt.frame(1.0 / 60.0);
        // The frame's corners: the file over the whole window, the pane kept under it.
        let (x, y) = middle(&rt, "pane-full");
        press(&mut rt, x, y);
        assert_eq!(rt.state.viewer.open.as_ref().and_then(|o| o.kind()), Some("markdown"));
        key(&mut rt, Key::Escape);
        assert!(rt.state.viewer.open.is_none() && rt.state.beside().is_some(), "Escape comes back to the pane");
        // The notes' mark opens it big too, where the web window draws its notes.
        let (x, y) = middle(&rt, "pane-notes");
        press(&mut rt, x, y);
        assert!(rt.state.viewer.open.is_some());
        // The big view's own mark puts it back beside the conversation.
        let (x, y) = middle(&rt, "viewer-beside");
        press(&mut rt, x, y);
        assert!(rt.state.viewer.open.is_none() && rt.state.beside().is_some());
        let (x, y) = middle(&rt, "pane-close");
        press(&mut rt, x, y);
        assert!(rt.state.beside().is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_divider_is_dragged_kept_and_put_back_with_a_double_press() {
        let (mut rt, dir) = with_a_plan("pane-drag", "Done.");
        rt.update(|s, _| s.open_beside("notes/plan.md", false));
        rt.frame(1.0 / 60.0);
        let (x, y) = middle(&rt, "pane-grip");
        rt.pointer_move(x, y);
        rt.pointer_down(x, y);
        for step in 1..=6 {
            rt.pointer_move(x + 60.0 * step as f32, y);
            rt.frame(1.0 / 60.0);
        }
        rt.pointer_up(x + 360.0, y);
        rt.frame(1.0 / 60.0);
        // The pointer 360 points further right: the file takes what is right of it.
        let kept = rt.state.picks.get().doc_split.expect("the split is kept");
        let body = 1176.0;
        assert!((kept - rules::split_from_pointer(x + 360.0 - 82.0 - AIR, body)).abs() < 1e-3, "{kept}");
        assert!((rt.state.conversation_size().0 - body * (1.0 - rules::artifact_fraction(kept, body))).abs() < 0.01);
        // A double press puts it back where it opens.
        let (x, y) = middle(&rt, "pane-grip");
        press(&mut rt, x, y);
        press(&mut rt, x, y);
        assert_eq!(rt.state.picks.get().doc_split, Some(rules::OPENING_SPLIT));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn leaving_a_thread_takes_its_file_and_coming_back_puts_it_back_until_a_new_answer() {
        let (mut rt, dir) = with_a_plan("pane-place", "Done.");
        rt.update(|s, _| s.open_beside("notes/plan.md", false));
        rt.frame(1.0 / 60.0);
        let thread = match &rt.state.view {
            View::Thread { id, product } => (id.clone(), product.clone()),
            _ => unreachable!(),
        };
        rt.update(|s, _| s.leave_thread());
        rt.frame(1.0 / 60.0);
        assert!(rt.state.pane.doc.is_none(), "the pane belongs to the thread");
        let (id, product) = thread.clone();
        rt.update(move |s, _| s.open_thread(&id, &product));
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.beside().map(|d| d.src.clone()).as_deref(), Some("notes/plan.md"));
        // A newer answer since is what the person came back for: the thread opens as new.
        rt.update(|s, _| s.leave_thread());
        let (id, product) = thread;
        rt.update(move |s, _| {
            if let Some(item) = s.snap.items.iter_mut().find(|i| i.id == id && i.product == product) {
                item.rest.insert("wrote".into(), serde_json::json!({ "result": { "ts": 1_790_000_100_000.0_f64, "source": "agent" } }));
            }
            s.open_thread(&id, &product);
        });
        rt.frame(1.0 / 60.0);
        assert!(rt.state.beside().is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_newest_answers_file_on_a_line_of_its_own_opens_by_itself_and_stays_shut_once_closed() {
        let (mut rt, dir) = with_a_plan("pane-auto", "The plan is written down:\n\nnotes/plan.md");
        run(&mut rt, 0.1);
        let doc = rt.state.beside().expect("the thread opens it by itself");
        assert!(doc.auto);
        // Escape does not stand in the way of a file nobody asked for: it leaves the thread.
        key(&mut rt, Key::Escape);
        assert!(!matches!(rt.state.view, View::Thread { .. }));
        rt.update(|s, _| s.back());
        run(&mut rt, 0.1);
        assert!(rt.state.beside().is_some_and(|d| d.auto), "and is there again on coming back");
        // Closed by its mark, it stays shut until the thread's next answer.
        let (x, y) = middle(&rt, "pane-close");
        press(&mut rt, x, y);
        rt.update(|s, _| {
            s.leave_thread();
            s.back();
        });
        run(&mut rt, 0.1);
        assert!(rt.state.beside().is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_card_under_a_thousand_points_gives_the_file_all_of_it() {
        let (mut rt, dir) = with_a_plan("pane-narrow", "Done.");
        rt.set_viewport((900.0, 760.0), 2.0);
        rt.frame(1.0 / 60.0);
        rt.update(|s, _| s.open_beside("notes/plan.md", false));
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.pane_placement(), Some(Placement::Focus));
        assert!(takes_the_card(&rt.state));
        // No head and no marks over it, as in the web window; Escape is the way out.
        assert!(rect_of(&rt, "pane-close").is_none());
        assert!(rect_of(&rt, "pane-dock").is_some(), "the reply box stays under it");
        key(&mut rt, Key::Escape);
        assert!(rt.state.beside().is_none());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_file_is_read_again_when_an_agent_writes_it_while_it_is_open() {
        let (mut rt, dir) = with_a_plan("pane-follow", "Done.");
        rt.update(|s, _| s.open_beside("notes/plan.md", false));
        let file = dir.join("notes/plan.md");
        std::fs::write(&file, "# The plan\n\nRound each line, then add them up.\n").expect("rewritten");
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
        std::fs::File::options().write(true).open(&file).and_then(|f| f.set_modified(later)).expect("a newer time");
        let read = |s: &State| match s.beside().and_then(|d| d.read.as_ref()) {
            Some(Read::Doc(words)) => words.to_string(),
            _ => String::new(),
        };
        // Not before its time has come round.
        let now = crate::clock();
        rt.update(move |s, _| assert!(!follow_the_file(s, now)));
        assert!(!read(&rt.state).contains("add them up"));
        rt.update(move |s, _| assert!(follow_the_file(s, now + rules::FOLLOW_FILE_MS + 1.0)));
        assert!(read(&rt.state).contains("add them up"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_newest_answer_is_kept_by_its_result_or_its_making() {
        let item: crate::data::Item = serde_json::from_value(serde_json::json!({ "id": "w-1", "product": "p", "status": "done", "createdAt": 5.0, "wrote": { "result": { "ts": 9.0, "source": "agent" } } })).unwrap();
        assert_eq!(news(&item), 9.0);
        let item: crate::data::Item = serde_json::from_value(serde_json::json!({ "id": "w-1", "product": "p", "status": "open", "createdAt": 5.0 })).unwrap();
        assert_eq!(news(&item), 5.0);
    }
}
