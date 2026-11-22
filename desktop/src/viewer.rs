//! A file opened big, over the whole window (`ArtifactView` in
//! `renderer/src/components/ArtifactView.tsx`, `ArtifactStage.tsx` for what each kind draws):
//! the file filling the window, the thread's name for it and its version above, Escape back.
//!
//! It opens from a conversation, on a picture or a file named in a message or in the thread's
//! list of files, and from the Files place, on a tile. The files it steps through are the ones
//! the lines it was opened on name (`rules/viewer.rs`); the version line steps one file's
//! versions, the arrows at its sides the other files.
//!
//! What each kind draws:
//! - a picture, fitted to the stage and never blown up, a tall one fitted to its width; the pill
//!   under it zooms it, and so do the keys, a pinch and ⌘ with the wheel;
//! - a document, as a page to read (`markdown::blocks_doc`), with who shared it over it and its
//!   headings beside it once there are three;
//! - a script or any text, with its line numbers and its colours (`rules/code.rs`), and a table
//!   the same way;
//! - a page, a film, a PDF or a deck, which the web window draws in a frame or a player this
//!   window does not have: what it is, and the button that opens it outside.
//!
//! Left out, as the person has never used them: comments down the right and the box to write
//! one, Approve, Share and renaming. Opened from a thread, its corner mark puts the file in the
//! pane beside the conversation (`pane.rs`).

use std::cell::Cell;
use std::rc::Rc;

use rune::{Cursor, El, Icon, ScrollPos, ScrollSpec, Span, TextCase, TextStyle, Tx, col, el, icon, rich, row, text};
use serde_json::Value;

use crate::files::{self, Files};
use crate::pictures::{Look, Want};
use crate::rules::code::{self, Kind};
use crate::rules::thread_files::{self, Family, Version};
use crate::rules::viewer::{self as rules, Step};
use crate::talk::Line;
use crate::theme::*;
use crate::{State, View, feel, icons, markdown};

/// The bar over the file, its edge included (`.av-top`).
const TOP: f32 = 44.0;
/// The line of places a file from Files was sent in, under it (`.fp-sent`).
const FOOT: f32 = 40.0;
/// The paper the view is on (`--doc-page`).
fn paper() -> rune::Color {
    rgb(40, 38, 37)
}
/// A script's sheet (`--code-ground`).
fn code_ground() -> rune::Color {
    rgb(34, 32, 31)
}
/// What a framed file stands on: a hairline and a soft shadow (`--artifact-lift`).
fn lift() -> [rune::Shadow; 2] {
    [shadow(0.0, 0.0, 0.0, 1.0, edge(), false), shadow(0.0, 18.0, 48.0, 0.0, rgba(0, 0, 0, 0.32), false)]
}

/// Where a file opened big was opened from.
#[derive(Clone, Debug, PartialEq)]
pub enum From {
    /// A conversation, as `(id, product)`.
    Thread(String, String),
    /// A file of the Files place, by its key.
    Files(String),
}

/// A place a file from Files was sent in (`FileSentIn`).
#[derive(Clone, Debug, PartialEq)]
pub struct SentIn {
    pub kind: String,
    pub product: Option<String>,
    pub id: Option<String>,
    pub title: String,
}

/// What a script or a table is drawn from: its lines, numbered and coloured once, in pieces of
/// a few hundred lines each, so a long one is laid out a piece at a time.
pub struct Code {
    /// Each piece: its numbers, one to a line, its lines, and their colours.
    pieces: Vec<(String, String, Vec<Span>)>,
    /// The widest line, in characters, which the sheet is as wide as.
    widest: usize,
    lines: usize,
}

/// What is read off the disk for the version in hand.
pub enum Words {
    Doc(Rc<str>),
    Code(Rc<Code>),
}

/// A file open big.
pub struct Open {
    pub from: From,
    lines: Vec<Line>,
    families: Vec<Family>,
    pub at: Step,
    /// The pictures the press handed over, the agent's latest pictures, which the arrows step
    /// through when the file is one of them.
    handed: Vec<String>,
    latest: Vec<String>,
    /// Where a short path is looked for, likeliest first. Files hands over whole paths.
    roots: Vec<String>,
    /// The version in hand: where it is on this Mac, and its words when it is drawn as words.
    found: Option<String>,
    words: Option<Words>,
    /// How far a picture is zoomed in, from the size it opened at.
    pub zoom: f32,
    /// From Files: the name a person gave it, and where it was sent.
    title: Option<String>,
    sent_in: Vec<SentIn>,
    /// What the thread called the version in hand, read once as it lands (`labelSaidFor`).
    said_as: Option<String>,
}

/// The view, open or not.
#[derive(Default)]
pub struct Viewer {
    pub open: Option<Open>,
}

/// Where the stage's scroller is: how far it has moved, how far it can, and what it shows.
#[derive(Clone, Copy, Default)]
struct At {
    offset: [f32; 2],
    max: [f32; 2],
    viewport: [f32; 2],
}

thread_local! {
    /// Where the stage's scroller is, as it last said: what a key and the zoom move it from.
    static STAGE: Cell<At> = Cell::new(At::default());
}
const STAGE_KEY: &str = "viewer-stage";

/// What a scroller of the stage says as it moves, kept for the keys and the zoom.
fn heard(at: ScrollPos) {
    STAGE.with(|s| s.set(At { offset: at.offset, max: at.max, viewport: at.viewport }));
}

/// The most of a file read as words, and the longest line drawn whole.
const MOST_BYTES: u64 = 4 << 20;
const LONGEST_LINE: usize = 4000;
/// Lines to a piece of a script's sheet.
const PIECE: usize = 200;

/// Where a path is on this Mac: a whole path is itself, a short one is looked for under each root.
pub(crate) fn place_of(roots: &[String], path: &str) -> Option<String> {
    let path = path.strip_prefix("file://").map(crate::rules::paths::decode_uri).unwrap_or_else(|| path.to_string());
    let rel = path.strip_prefix("./").unwrap_or(&path);
    let places: Vec<String> = if rel.starts_with('/') { vec![rel.to_string()] } else { roots.iter().map(|root| format!("{root}/{}", crate::rules::paths::decode_uri(rel))).collect() };
    places.into_iter().find(|p| std::path::Path::new(p).is_file())
}

/// A file's words, up to the most a file is read to, its bytes taken as UTF-8 where they can be.
pub(crate) fn read_text(found: &str) -> Option<String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(found).ok()?.take(MOST_BYTES).read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// A script's or a table's lines, numbered and coloured, cut into pieces.
pub(crate) fn code_of(text: &str, colour: bool) -> Code {
    let all: Vec<&str> = text.strip_suffix('\n').unwrap_or(text).split('\n').collect();
    let mut pieces = Vec::new();
    let mut widest = 0;
    for (p, chunk) in all.chunks(PIECE).enumerate() {
        let (mut numbers, mut words, mut spans) = (String::new(), String::new(), Vec::new());
        for (i, line) in chunk.iter().enumerate() {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let line: String = if line.chars().count() > LONGEST_LINE { line.chars().take(LONGEST_LINE).chain(['…']).collect() } else { line.to_string() };
            widest = widest.max(line.chars().count());
            if i > 0 {
                numbers.push('\n');
                words.push('\n');
            }
            numbers.push_str(&(p * PIECE + i + 1).to_string());
            // Past the web window's six thousand lines the colouring costs more than it gives.
            if colour && p * PIECE + i < 6000 {
                for (kind, piece) in code::tokenize(&line) {
                    let from = words.len();
                    words.push_str(&piece);
                    if let Some(c) = colour_of(kind) {
                        let span = Span::new(from, words.len()).color(c);
                        spans.push(if kind == Kind::Comment { span.italic() } else { span });
                    }
                }
            } else {
                words.push_str(&line);
            }
        }
        pieces.push((numbers, words, spans));
    }
    Code { pieces, widest, lines: all.len() }
}

/// A piece's colour on the dark ground (`--code-com` and the rest).
fn colour_of(kind: Kind) -> Option<rune::Color> {
    Some(match kind {
        Kind::Plain => return None,
        Kind::Comment => rgb(0x8b, 0x94, 0x9e),
        Kind::Text => rgb(0xa5, 0xd6, 0xff),
        Kind::Keyword => rgb(0xff, 0x7b, 0x72),
        Kind::Number | Kind::Constant => rgb(0x79, 0xc0, 0xff),
        Kind::Type | Kind::Made => rgb(0xff, 0xa6, 0x57),
        Kind::Run => rgb(0xd2, 0xa8, 0xff),
    })
}

impl Open {
    /// A view of the files `lines` name, opened on `path` (or with none, on the newest version
    /// of the last file), stepping through `handed` when the press handed pictures over.
    fn new(from: From, lines: Vec<Line>, path: Option<&str>, handed: Vec<String>, roots: Vec<String>, now: f64) -> Option<Open> {
        let named = rules::files_for_view(&lines, path, &handed, now);
        let families = thread_files::thread_files(&named, now);
        let at = rules::where_a_press_lands(&families, path, now)?;
        let latest = rules::latest_pictures_said(&lines);
        let mut open = Open { from, lines, families, at, handed, latest, roots, found: None, words: None, zoom: 1.0, title: None, sent_in: Vec::new(), said_as: None };
        open.land();
        Some(open)
    }
    fn family(&self) -> &Family {
        &self.families[self.at.0]
    }
    fn version(&self) -> &Version {
        &self.families[self.at.0].versions[self.at.1]
    }
    /// What the view draws the version in hand as (`viewKind`).
    pub fn kind(&self) -> Option<&'static str> {
        thread_files::view_kind(&self.version().path)
    }
    /// The version in hand is found and read: a document's words, a script's lines.
    fn land(&mut self) {
        self.found = place_of(&self.roots, &self.version().path);
        self.zoom = 1.0;
        let v = self.version();
        let paths: Vec<&str> = std::iter::once(v.path.as_str()).chain(v.also.iter().map(String::as_str)).collect();
        self.said_as = rules::label_said_for(&self.lines, &paths);
        let kind = self.kind();
        self.words = self.found.as_deref().filter(|_| matches!(kind, Some("markdown" | "code" | "table"))).and_then(|found| {
            let text = read_text(found)?;
            Some(match kind {
                Some("markdown") => Words::Doc(Rc::from(text.as_ref())),
                _ => Words::Code(Rc::new(code_of(&text, kind == Some("code")))),
            })
        });
        STAGE.with(|at| at.set(At { offset: [0.0; 2], ..at.get() }));
    }
    /// What the arrows step through.
    fn steps(&self, now: f64) -> Vec<Step> {
        rules::arrows_through(&self.families, &self.handed, &self.latest, Some(self.at), now)
    }
}

/* ------------------------------ opening and closing ------------------------------ */

impl State {
    /// A file a conversation names, pressed: its picture, its mention, its row in the list of
    /// the thread's files. A picture steps through the pictures of the part of the thread the
    /// press was in, or `turn` where the press hands its own over (a folder's pictures). A file
    /// the view cannot draw, or a press with no conversation under it, opens with the system.
    pub fn open_big(&mut self, href: &str, turn: Option<Vec<String>>) {
        let View::Thread { id, product } = self.view.clone() else {
            return crate::files::open_found(self, href);
        };
        if !rules::opens_big(href) {
            return crate::files::open_found(self, href);
        }
        let Some(item) = self.snap.items.iter().find(|i| i.id == id && i.product == product) else { return };
        let roots = Files::of(self, item).roots;
        let key = format!("{product}/{id}");
        let answered = self.threads.get(&key).or_else(|| self.fixture.get("threads").and_then(|t| t.get(&key)));
        let lines: Vec<Line> = answered.and_then(|a| a.get("zero:item-history")).and_then(|h| h.get("lines")).and_then(|l| serde_json::from_value(l.clone()).ok()).unwrap_or_default();
        let href = crate::rules::paths::decode_uri(href.strip_prefix("./").unwrap_or(href));
        let picture = thread_files::review_kind(&href) == Some("image");
        let turn = turn.unwrap_or_else(|| if picture { rules::turn_pictures(&lines, &href) } else { Vec::new() });
        match Open::new(From::Thread(id, product), lines, Some(&href), turn, roots, self.now) {
            Some(open) => self.show_big(open),
            None => crate::files::open_found(self, &href),
        }
    }
    /// A tile of the Files place pressed, once the app has said what the file is
    /// (`zero:files-place-open`): its lines, one to a version, at the paths they open from.
    pub fn open_placed(&mut self, key: &str, view: &Value) {
        let lines: Vec<Line> = view.get("lines").and_then(|l| serde_json::from_value(l.clone()).ok()).unwrap_or_default();
        let sent_in: Vec<SentIn> = view.get("sentIn").and_then(Value::as_array).map(|all| {
            all.iter()
                .map(|p| {
                    let said = |f: &str| p.get(f).and_then(Value::as_str).map(str::to_string);
                    SentIn { kind: said("kind").unwrap_or_default(), product: said("product"), id: said("id").or_else(|| said("threadId")), title: said("title").unwrap_or_default() }
                })
                .collect()
        }).unwrap_or_default();
        let Some(mut open) = Open::new(From::Files(key.to_string()), lines, None, Vec::new(), Vec::new(), self.now) else { return };
        open.title = view.get("title").and_then(Value::as_str).map(str::to_string);
        open.sent_in = sent_in;
        self.show_big(open);
    }
    fn show_big(&mut self, open: Open) {
        // It stands over the page alone, as the web window's covers its own notices.
        self.close_files();
        self.leave_the_strip();
        self.pointed.hint = None;
        self.viewer.open = Some(open);
    }
    /// Escape, Back or the close mark: back to where it was opened from.
    pub fn close_big(&mut self) {
        self.viewer.open = None;
    }
    /// An arrow at the stage's side, or ← and →: the file before or after (`stepFile`).
    pub fn step_file(&mut self, by: isize) {
        let now = self.now;
        let Some(open) = self.viewer.open.as_mut() else { return };
        if let Some(next) = rules::next_step(&open.families, &open.steps(now), open.at, by) {
            open.at = next;
            open.land();
        }
    }
    /// The version line's arrows, and Show vN: another version of the same file (`stepVersion`).
    pub fn step_version(&mut self, by: isize) {
        let Some(open) = self.viewer.open.as_mut() else { return };
        let last = open.family().versions.len() - 1;
        let vi = open.at.1.saturating_add_signed(by).min(last);
        if vi != open.at.1 {
            open.at.1 = vi;
            open.land();
        }
    }
    /// The open mark: the file in the app that owns it, or the browser for a page.
    pub fn open_outside(&mut self) {
        if let Some(found) = self.viewer.open.as_ref().and_then(|o| o.found.clone()) {
            crate::files::open_with_system(&found);
        }
    }
    /// A place in the line under a file from Files: the thread it was sent in, opened.
    fn go_to_place(&mut self, place: &SentIn) {
        let (Some(product), Some(id)) = (place.product.clone(), place.id.clone()) else { return };
        if self.snap.items.iter().any(|i| i.id == id && i.product == product) {
            self.close_big();
            self.open_thread(&id, &product);
        }
    }
}

/// What a picture's page had open big when it was photographed, done here once the window has
/// its clock: a picture of a conversation pressed (`pictureOpen`, its path and its part's
/// pictures), its versions stepped back (`olderVersions`), or a tile of the Files place
/// pressed (`fileOpen`, the file's name, opened on the view the fixture holds for it).
pub fn pictured(s: &mut State) {
    if let Some(pressed) = s.fixture.remove("pictureOpen") {
        let path = pressed.get("path").and_then(Value::as_str).unwrap_or_default().to_string();
        let turn: Vec<String> = pressed.get("turn").and_then(|t| serde_json::from_value(t.clone()).ok()).unwrap_or_default();
        s.open_big(&path, Some(turn));
    }
    if let Some(name) = s.fixture.remove("fileOpen") {
        let name = name.as_str().unwrap_or_default();
        let key = s.fixture.get("filesPlace").and_then(|l| l.get("files")).and_then(Value::as_array).and_then(|files| {
            files.iter().find(|f| f.get("name").and_then(Value::as_str) == Some(name)).and_then(|f| f.get("key")).and_then(Value::as_str).map(str::to_string)
        });
        let view = key.as_ref().and_then(|key| s.fixture.get("filesPlaceViews").and_then(|v| v.get(key)).cloned());
        if let (Some(key), Some(view)) = (key, view) {
            s.open_placed(&key, &view);
        }
    }
    if let Some(back) = s.fixture.remove("olderVersions").and_then(|n| n.as_i64()) {
        s.step_version(-(back as isize));
    }
}

/// A link pressed in a document opened big (`FileLink` in `ArtifactStage.tsx`): an address goes
/// to the browser, a file beside the document to the app that owns it, a place in the page
/// nowhere.
pub fn open_doc_link(folder: &str, href: &str) {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("mailto:") {
        return files::open_with_system(href);
    }
    if href.starts_with('#') || href.is_empty() {
        return;
    }
    if let Some(found) = place_of(&[folder.to_string()], href.split('#').next().unwrap_or(href)) {
        files::open_with_system(&found);
    }
}

/* ------------------------------ the keys ------------------------------ */

/// The keys while a file is open big: every key is the view's, and none reaches the page
/// under it (`rest` in `ArtifactView.tsx`). Escape goes back; ← and → step the files; a
/// picture zooms on + and −, fits on 0 and shows its own pixels on 1; the arrows and the page
/// keys move what is drawn. A chord is the system's.
pub fn key(s: &mut State, e: &rune::KeyEvent, tx: &mut Tx) -> Option<bool> {
    use rune::edit::Key;
    let open = s.viewer.open.as_ref()?;
    if e.mods.cmd || e.mods.ctrl {
        return Some(false);
    }
    let picture = open.kind() == Some("image");
    match &e.key {
        Key::Escape => s.close_big(),
        Key::Right => s.step_file(1),
        Key::Left => s.step_file(-1),
        Key::Down => scroll_by(tx, 80.0),
        Key::Up => scroll_by(tx, -80.0),
        Key::PageDown => scroll_by(tx, 0.9 * STAGE.with(|at| at.get().viewport[1])),
        Key::PageUp => scroll_by(tx, -0.9 * STAGE.with(|at| at.get().viewport[1])),
        Key::Text(t) if picture => match t.as_str() {
            "=" | "+" => zoom_step(s, true, tx),
            "-" => zoom_step(s, false, tx),
            "0" => zoom_to(s, 1.0, tx),
            "1" => {
                let fit = picture_fit(s).map_or(1.0, |f| f.scale);
                zoom_to(s, rules::real_zoom(fit), tx);
            }
            _ => {}
        },
        _ => {}
    }
    Some(true)
}

fn scroll_by(tx: &mut Tx, dy: f32) {
    let at = STAGE.with(Cell::get);
    tx.scroll_to_instant(STAGE_KEY, at.offset[0], (at.offset[1] + dy).clamp(0.0, at.max[1].max(0.0)));
}

/* ------------------------------ a picture ------------------------------ */

/// The stage a picture has, and the room inside its padding (`roomInside`): 52 points either
/// side, 24 over and under.
fn picture_room(s: &State) -> (f32, f32) {
    let foot = s.viewer.open.as_ref().is_some_and(|o| !o.sent_in.is_empty());
    let stage = (s.size.0, s.size.1 - TOP - if foot { FOOT } else { 0.0 });
    ((stage.0 - 104.0).max(1.0), (stage.1 - 48.0).max(1.0))
}

/// The picture in hand's size, when it has been read, and how it fits.
fn picture_fit(s: &State) -> Option<rules::Fit> {
    let open = s.viewer.open.as_ref()?;
    let natural = natural_of(s, open)?;
    Some(rules::picture_fit((natural.0 as f32, natural.1 as f32), picture_room(s)))
}

/// Where the picture is looked for: a thread's every root, as its message's picture is, so both
/// share one copy; a file from Files at its own path.
fn picture_places(open: &Open) -> Vec<String> {
    match (&open.found, &open.from) {
        (Some(found), From::Files(_)) => vec![found.clone()],
        (found, _) => {
            let path = &open.version().path;
            let rel = path.strip_prefix("./").unwrap_or(path);
            if rel.starts_with('/') {
                return vec![rel.to_string()];
            }
            let mut places: Vec<String> = open.roots.iter().map(|root| format!("{root}/{}", crate::rules::paths::decode_uri(rel))).collect();
            if let Some(found) = found.as_ref().filter(|f| !places.contains(f)) {
                places.insert(0, found.clone());
            }
            places
        }
    }
}

/// The picture's own size, as the store last read it.
fn natural_of(s: &State, open: &Open) -> Option<(u32, u32)> {
    let room = picture_room(s);
    match s.pictures.look(picture_places(open), Want::Within(s.pictures.pixels(room.0), s.pictures.pixels(room.1))) {
        Look::Here(held) => Some(held.natural),
        Look::Arriving(known) => known,
        Look::Missing => None,
    }
}

fn zoom_step(s: &mut State, inward: bool, tx: &mut Tx) {
    let Some(fit) = picture_fit(s) else { return };
    let zoom = s.viewer.open.as_ref().map_or(1.0, |o| o.zoom);
    zoom_to(s, rules::next_zoom(zoom, inward, rules::lowest_zoom(fit)), tx);
}

/// The picture zoomed to `zoom`, the point in the middle of what is in view kept in the middle
/// (`zoomAnchor` with no pin in hand and the pointer off the picture).
fn zoom_to(s: &mut State, zoom: f32, tx: &mut Tx) {
    let room = picture_room(s);
    let Some(natural) = s.viewer.open.as_ref().and_then(|o| natural_of(s, o)) else { return };
    let natural = (natural.0.max(1) as f32, natural.1.max(1) as f32);
    let fit = rules::picture_fit(natural, room);
    let Some(open) = s.viewer.open.as_mut() else { return };
    let zoom = zoom.clamp(rules::lowest_zoom(fit).min(1.0), rules::MAX_ZOOM);
    if (zoom - open.zoom).abs() < 1e-4 {
        return;
    }
    let (before, after) = (laid_picture(natural, fit, open.zoom, room), laid_picture(natural, fit, zoom, room));
    let at = STAGE.with(Cell::get);
    let mid = (at.offset[0] + at.viewport[0] / 2.0, at.offset[1] + at.viewport[1] / 2.0);
    let (fx, fy) = ((mid.0 - before.x) / before.w.max(1.0), (mid.1 - before.y) / before.h.max(1.0));
    open.zoom = zoom;
    let x = (after.x + fx * after.w - at.viewport[0] / 2.0).max(0.0);
    let y = (after.y + fy * after.h - at.viewport[1] / 2.0).max(0.0);
    tx.scroll_to_instant(STAGE_KEY, x, y);
}

/// Where the picture sits in the stage's scrolled content, and how big: in the middle of the
/// room where it is smaller than it, and from the padding's edge where it is larger.
struct Laid {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    content: (f32, f32),
}
fn laid_picture(natural: (f32, f32), fit: rules::Fit, zoom: f32, room: (f32, f32)) -> Laid {
    let (w, h) = (natural.0 * fit.scale * zoom, natural.1 * fit.scale * zoom);
    // A tall one keeps its foot clear of the zoom's pill (`.is-tall { padding-bottom: 72px }`).
    let under = if fit.tall { 72.0 } else { 24.0 };
    let content = ((w + 104.0).max(room.0 + 104.0), (h + 24.0 + under).max(room.1 + 48.0));
    let x = if w + 104.0 <= content.0 { (content.0 - w) / 2.0 } else { 52.0 };
    let y = if h + 24.0 + under < content.1 { 24.0 + ((content.1 - 24.0 - under) - h).max(0.0) / 2.0 } else { 24.0 };
    Laid { x, y, w, h, content }
}

fn picture_stage(s: &State, open: &Open, stage: (f32, f32)) -> El {
    let room = picture_room(s);
    let places = picture_places(open);
    let look = |want| s.pictures.look(places.clone(), want);
    let fitted = Want::Within(s.pictures.pixels(room.0), s.pictures.pixels(room.1));
    let (natural, mut held) = match look(fitted) {
        Look::Here(held) => (Some(held.natural), Some(held)),
        Look::Arriving(known) => (known, None),
        Look::Missing => return said(&format!("{} could not be found.", open.version().name)),
    };
    let Some(natural) = natural.map(|(w, h)| (w.max(1) as f32, h.max(1) as f32)) else {
        // Not read yet: the web window waits in silence too, until the picture says it is open.
        return el().size(stage.0, stage.1);
    };
    let fit = rules::picture_fit(natural, room);
    let laid = laid_picture(natural, fit, open.zoom, room);
    // Zoomed in, the copy is made again at the size it is drawn, up to the picture's own.
    if (open.zoom > 1.0 || fit.tall)
        && let Look::Here(bigger) = look(Want::Within(s.pictures.pixels(laid.w), s.pictures.pixels(laid.h)))
    {
        held = Some(bigger);
    }
    let [hairline, soft] = lift();
    let picture = el().absolute(laid.x, laid.y).size(laid.w, laid.h).radius(10.0).box_shadow(hairline).box_shadow(soft);
    let picture = match &held {
        Some(held) => picture.child(el().size(laid.w, laid.h).radius(10.0).clip_rounded(10.0).pixels(&held.pixels)),
        None => picture,
    };
    let zoomed = open.zoom > 1.0 || fit.tall;
    let scroller = el()
        .key(STAGE_KEY)
        .size(stage.0, stage.1)
        .scroll_xy()
        .on_scroll_offset(heard)
        .child(el().size(laid.content.0, laid.content.1).child(picture))
        // A pinch, or ⌘ and the wheel, zooms about what is in the middle.
        .on_magnify(|s: &mut State, m: rune::MagnifyEvent, tx: &mut Tx| {
            let zoom = s.viewer.open.as_ref().map_or(1.0, |o| o.zoom);
            zoom_to(s, zoom * (1.0 + m.delta), tx);
            true
        })
        .on_wheel(|s: &mut State, w: rune::WheelEvent, tx: &mut Tx| {
            if !w.mods.cmd {
                return false;
            }
            let (zoom, floor) = (s.viewer.open.as_ref().map_or(1.0, |o| o.zoom), picture_fit(s).map_or(1.0, rules::lowest_zoom));
            zoom_to(s, rules::wheel_zoom(zoom, -w.dy, floor), tx);
            true
        });
    let scroller = if zoomed { scroller.cursor(Cursor::Grab) } else { scroller };
    el().size(stage.0, stage.1).child(scroller).child(zoom_pill(open, fit, stage))
}

/// The zoom's pill under the picture (`.av-zoom.file-pager`): out, the size it is drawn at,
/// which fits it again, and in.
fn zoom_pill(open: &Open, fit: rules::Fit, stage: (f32, f32)) -> El {
    let floor = rules::lowest_zoom(fit);
    let step = |glyph: &'static Icon, live: bool, inward: bool, help: &str| {
        let b = row().size(28.0, 28.0).radius(7.0).center().child(icon(glyph, 14.0).color(dim()).group_color(ink()));
        if live {
            b.hover_group().hover(feel::wash_strong()).cursor(Cursor::Pointer).help(help).on_press_tx(move |s: &mut State, tx| zoom_step(s, inward, tx))
        } else {
            b.opacity(0.35)
        }
    };
    let words = rules::zoom_words(fit.scale, open.zoom);
    let fit_button = row()
        .min_w(52.0)
        .h(28.0)
        .px(8.0)
        .radius(6.0)
        .center()
        .hover(feel::wash_strong())
        .cursor(Cursor::Pointer)
        .help("Fit (0), real size (1)")
        .child(text(words).font(mono(12.0, 12.0).features("'tnum' 1")).color(ink()))
        .on_press_tx(|s: &mut State, tx| zoom_to(s, 1.0, tx));
    let pill = row()
        .h(36.0)
        .px(4.0)
        .gap(2.0)
        .items_center()
        .radius(10.0)
        .border(1.0, rule())
        .border_layout()
        .fill(rgba(54, 52, 49, 0.92))
        .backdrop_blur(8.0)
        .box_shadow(shadow(0.0, 10.0, 30.0, 0.0, rgba(0, 0, 0, 0.35), false))
        .child(step(&icons::ZOOM_OUT, open.zoom > floor + 0.001, false, "Zoom out (-)"))
        .child(fit_button)
        .child(step(&icons::ZOOM_IN, open.zoom < rules::MAX_ZOOM, true, "Zoom in (+)"));
    // In the middle of the stage, its foot sixteen points over the stage's.
    row().pin(Some(0.0), None, Some(0.0), Some(16.0)).justify_center().child(pill).w(stage.0)
}

/* ------------------------------ a document ------------------------------ */

/// The three columns a document's sheet is laid out in (`.doc-sheet`), its margin, the text and
/// the outline's room, with the gap between them, for a stage `wide` points across.
fn sheet(wide: f32) -> (f32, f32, f32, f32) {
    if wide <= 600.0 {
        return (18.0, (wide - 36.0).max(1.0), 18.0, 0.0);
    }
    let (gap, right_least) = if wide > 1060.0 { (40.0, 220.0) } else { (0.0, 48.0) };
    let free = wide - 2.0 * gap;
    let text = 820.0_f32.min(free - 48.0 - right_least).max(0.0);
    let rest = free - text;
    let right = (rest / 2.0).max(right_least);
    (rest - right, text, right, gap)
}

/// The headings a document's outline lists (`outlineOf`): those of the first three levels but
/// the first top heading, which the page already shows as its title; the highest level there
/// is stands first, the rest one step in.
pub fn outline(headings: &[(u8, String)]) -> Vec<(usize, bool, String)> {
    let first_title = headings.iter().position(|(level, _)| *level == 1);
    let kept: Vec<(usize, u8, String)> = headings
        .iter()
        .enumerate()
        .map(|(at, (level, words))| (at, *level, words.split_whitespace().collect::<Vec<_>>().join(" ")))
        .filter(|(at, level, words)| !words.is_empty() && *level <= 3 && !(*level == 1 && Some(*at) == first_title))
        .collect();
    let top = kept.iter().map(|(_, level, _)| *level).min();
    kept.into_iter().map(|(at, level, words)| (at, Some(level) != top, words)).collect()
}

/// A document's stage, built again only when the version, the stage's size or the pictures held
/// change: a long document is parsed and laid out once, not on every frame.
fn doc_stage(s: &State, open: &Open, words: &Rc<str>, stage: (f32, f32)) -> El {
    let v = open.version();
    let held = s.pictures.held();
    // The byline says "today" until midnight.
    let version = (v.id.clone(), v.path.clone(), stage.0.to_bits(), stage.1.to_bits(), held, (s.now / 86_400_000.0).floor() as i64);
    rune::memo("viewer-doc", version, || doc_stage_built(s, open, words, stage))
}

fn doc_stage_built(s: &State, open: &Open, words: &Rc<str>, stage: (f32, f32)) -> El {
    let folder = open.found.as_deref().and_then(|f| f.rsplit_once('/')).map_or(String::new(), |(dir, _)| dir.to_string());
    // Who shared it and when, over it (`bylineOf`), from the line that named the version.
    let v = open.version();
    let by = rules::shared_by(&open.lines, &v.path);
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let mut byline = rules::shared_by_words(&by, me, |_| "A teammate".to_string());
    // Joined by a dot between two spaces, which the page collapses to one either side.
    if v.at > 0.0 {
        byline = format!("{byline} · {}", rules::when_shared(v.at, s.now));
    }
    doc_page(s, words, &folder, byline, stage, STAGE_KEY, Some(heard), false)
}

/// A document laid out as a page to read on a stage `stage` across (`.doc-sheet`, which the big
/// view's `DocStage` and the pane's `DocText` share): the line over it, its words in their
/// column, and with three headings or more its outline beside them. `key` names its scroller,
/// `heard` hears where it is, and `editor` says it is the pane's (`markdown::blocks_doc`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn doc_page(s: &State, words: &str, folder: &str, byline: String, stage: (f32, f32), key: &'static str, heard: Option<fn(ScrollPos)>, editor: bool) -> El {
    let (left, column, right, gap) = sheet(stage.0);
    let narrow = stage.0 <= 600.0;
    let files = Files::beside(s, folder);
    let (blocks, _, _) = markdown::stack(markdown::blocks_doc(words, column, stage.0, editor, &files), true);
    let byline = readable(text(byline).font(TextStyle::new(12.5, 400.0).lh(18.0)).color(muted()));
    let (top, bottom) = if narrow { (24.0, 80.0) } else { (44.0, 120.0) };
    let page = col().pt(top).pb(bottom).pl(left + gap).w(left + gap + column).child(byline.mb(18.0)).child(blocks.w(column));
    let scroller = col().key(key).size(stage.0, stage.1).scroll(ScrollSpec { stick_bottom: false });
    let scroller = match heard {
        Some(heard) => scroller.on_scroll_offset(heard),
        None => scroller,
    };
    let mut stage_el = el().size(stage.0, stage.1).child(scroller.child(page));
    // Three headings or more have an outline beside the text.
    let entries = outline(&markdown::headings(words));
    if entries.len() >= 3 && !narrow {
        let x = left + gap + column + gap;
        stage_el = stage_el.child(doc_outline(&entries, x, right, stage.0));
    }
    stage_el
}

/// The headings beside a document (`DocOutline`): in the margin, with their words, where there
/// is room for them, and as a column of short dashes at the margin's edge where there is not.
fn doc_outline(entries: &[(usize, bool, String)], x: f32, room: f32, wide: f32) -> El {
    if wide > 1060.0 {
        let head = text("Contents").font(mono(10.5, 14.0).tracking(1.26)).text_case(TextCase::Uppercase).color(muted()).mb(10.0);
        let rows = entries.iter().map(|(at, sub, words)| {
            let dash = el().w(if *sub { 6.0 } else { 10.0 }).h(1.0).fill(rule());
            let at = *at;
            row()
                .w(room.min(220.0) - 20.0)
                .py(4.0)
                .pl(if *sub { 14.0 } else { 0.0 })
                .gap(10.0)
                .items_center()
                .hover_group()
                .cursor(Cursor::Pointer)
                .help(words.as_str())
                .child(dash)
                .child(row().min_w(0.0).child(text(words.as_str()).font(TextStyle::new(12.5, 400.0).lh(17.0)).color(muted()).group_color(dim()).truncate()))
                .on_press_tx(move |_: &mut State, tx| tx.scroll_key_to(format!("doc-heading-{at}"), 32.0))
        });
        return col().absolute(x, 44.0).child(head).children(rows);
    }
    let rows = entries.iter().map(|(_, sub, _)| row().w(18.0).h(7.0).justify_end().items_center().child(el().w(if *sub { 6.0 } else { 10.0 }).h(1.0).fill(rule())));
    col().absolute(wide - 14.0 - 18.0, 44.0).children(rows)
}

/* ------------------------------ a script, a table ------------------------------ */

/// A character of a script's lines across (`.code-file`, Geist Mono at 12.5).
pub(crate) const CODE_ADVANCE: f32 = 0.6 * 12.5;

/// A script's lines in their pieces, each its numbers in a column 56 points wide, 18 short of
/// its edge, then its words coloured (`.code-file-line`, `.code-file-n`, `.code-file-text`).
pub(crate) fn code_lines(code: &Code) -> impl Iterator<Item = El> + '_ {
    let line = kept(1.65 * 12.5);
    code.pieces.iter().map(move |(numbers, words, spans)| {
        row()
            .items_start()
            .child(row().w(56.0).pr(18.0).justify_end().child(pen(text(numbers.as_str()).font(mono(12.5, line)).color(muted()), 12.5)))
            .child(pen(readable(rich(words.as_str(), spans.clone()).font(mono(12.5, line)).color(ink())), 12.5).pr(32.0))
    })
}

/// The widest of a script's lines, in characters.
pub(crate) fn code_widest(code: &Code) -> usize {
    code.widest
}

/// A script's stage, built again only when the version or the stage's size change.
fn code_stage(open: &Open, code: &Rc<Code>, stage: (f32, f32)) -> El {
    let v = open.version();
    rune::memo("viewer-code", (v.id.clone(), v.path.clone(), stage.0.to_bits(), stage.1.to_bits()), || code_stage_built(code, stage))
}

fn code_stage_built(code: &Code, stage: (f32, f32)) -> El {
    // The sheet: at most 980 points, 52 in from either side, and as wide as its widest line.
    let words_wide = code.widest as f32 * CODE_ADVANCE + 32.0;
    let sheet_w = (980.0_f32.min(stage.0 - 104.0)).max(56.0 + words_wide + 2.0);
    let pieces = code_lines(code);
    let sheet = col()
        .w(sheet_w)
        .pt(18.0)
        .pb(40.0)
        .fill(code_ground())
        .radius(12.0)
        .border(1.0, edge())
        .border_layout()
        .children(pieces);
    let content_w = (sheet_w + 104.0).max(stage.0);
    let page = col().w(content_w).pt(28.0).pb(48.0).items_center().child(sheet);
    let _ = code.lines;
    el().size(stage.0, stage.1).child(el().key(STAGE_KEY).size(stage.0, stage.1).scroll_xy().on_scroll_offset(heard).child(page))
}

/* ------------------------------ what cannot be drawn here ------------------------------ */

/// A sentence in the middle of the stage (`.doc-missing`).
fn said(words: &str) -> El {
    row().w_full().h_full().center().px(80.0).child(text(words).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(dim()).wrap())
}

/// A page, a film, a PDF or a deck: what the web window draws it in is a frame or a player this
/// window does not have, so it says what the file is and opens it outside, the browser for a
/// page and the app that owns it for anything else.
fn outside_stage(open: &Open, stage: (f32, f32)) -> El {
    let v = open.version();
    let (what, glyph) = match open.kind() {
        Some("page") => ("a page, which opens in your browser", &icons::MENTION_PAGE),
        Some("film") => ("a film, which plays in the app that opens it", &icons::MENTION_MEDIA),
        Some("pdf") => ("a PDF, which opens in the app that reads it", &icons::MENTION_DOC),
        Some("deck") => ("a deck, which opens in the app that shows it", &icons::MENTION_DOC),
        _ => ("a file this window cannot draw", &icons::MENTION_FILE),
    };
    if open.found.is_none() {
        return el().size(stage.0, stage.1).child(said(&format!("{} could not be found.", v.name)));
    }
    let button = row()
        .h(28.0)
        .pl(10.0)
        .pr(12.0)
        .gap(6.0)
        .items_center()
        .radius(6.0)
        .border(1.0, rule())
        .border_layout()
        .hover(feel::wash())
        .cursor(Cursor::Pointer)
        .child(icon(&icons::FILE_OUTSIDE, 14.0).color(ink()))
        .child(label("Open outside", 13.0, 400.0, ink()))
        .on_press(|s: &mut State| s.open_outside());
    let card = col()
        .items_center()
        .gap(14.0)
        .child(icon(glyph, 28.0).color(muted()))
        .child(text(format!("{} is {what}.", v.name)).font(TextStyle::new(14.0, 400.0).lh(20.0)).color(dim()))
        .child(button);
    el().size(stage.0, stage.1).child(row().size(stage.0, stage.1).center().child(card))
}

/* ------------------------------ the bar, the stage, the line under ------------------------------ */

/// A corner mark of the bar (`.av-top .doc-mark`): its glyph, brighter under the pointer.
fn mark(glyph: &'static Icon, help: &'static str) -> El {
    row().size(28.0, 28.0).radius(6.0).center().hover_group().cursor(Cursor::Pointer).help(help).child(icon(glyph, 16.0).color(dim()).group_color(ink()))
}

/// The bar over the file (`.av-top`): back, the file's name and what the thread called it, its
/// version line, and the marks that open it outside and close the view. It moves the window, as
/// the web window's bar is its only band to move it by.
fn bar(s: &State, open: &Open) -> El {
    let family = open.family();
    let v = open.version();
    let name = open.title.clone().unwrap_or_else(|| rules::family_name(family));
    let said_as = open.said_as.clone().filter(|l| *l != name);
    let mut left = row()
        .gap(14.0)
        .items_center()
        .min_w(0.0)
        .child(mark(&icons::FILE_BACK, "Back to the thread (esc)").ml(-6.0).mr(-4.0).on_press(|s: &mut State| s.close_big()))
        .child(row().shrink().child(text(name).font(sans(13.0, 400.0)).color(ink()).truncate()));
    if let Some(said_as) = said_as {
        left = left.child(row().shrink().max_w(468.0).child(text(said_as.clone()).font(sans(13.0, 400.0)).color(dim()).truncate().help(said_as)));
    }
    let count = family.versions.len();
    if count > 1 {
        let at = open.at.1;
        let chevron = |glyph: &str, live: bool, by: isize, help: &'static str| {
            let c = row().size(20.0, 24.0).center().child(text(glyph).font(TextStyle::new(15.0, 400.0).lh(15.0)).color(if live { dim() } else { muted() }).group_color(ink()));
            if live { c.hover_group().help(help).on_press(move |s: &mut State| s.step_version(by)) } else { c.opacity(0.5).no_window_drag() }
        };
        let mut versions = row()
            .gap(4.0)
            .items_center()
            .child(chevron("‹", at > 0, -1, "Older version"))
            .child(text(format!("v{} of {count}", v.n)).font(mono(11.5, 15.0).features("'tnum' 1")).color(dim()).px(2.0))
            .child(chevron("›", at + 1 < count, 1, "Newer version"));
        if at + 1 < count {
            let newest = family.versions[count - 1].n;
            let by = (count - 1 - at) as isize;
            versions = versions.child(
                row().h(24.0).px(6.0).ml(4.0).radius(6.0).items_center().hover(feel::wash()).child(label(&format!("Show v{newest}"), 13.0, 400.0, ink())).on_press(move |s: &mut State| s.step_version(by)),
            );
        }
        left = left.child(versions);
    }
    // Opened from a thread, the mark that puts the file's newest version in the pane beside the
    // conversation instead (`onBeside`, a `FocusControl` drawn as focused), before the close mark.
    let beside = matches!(open.from, From::Thread(..)).then(|| {
        let newest = family.versions[family.versions.len() - 1].path.clone();
        mark(&icons::PANE_BESIDE, "Move it to the side · Esc closes it").on_press(move |s: &mut State| s.put_big_beside(&newest))
    });
    let mut marks = row().gap(6.0);
    if open.found.is_some() {
        marks = marks.child(mark(&icons::FILE_OUTSIDE, "Open outside").on_press(|s: &mut State| s.open_outside()));
    }
    let marks = marks.children(beside).child(mark(&icons::FILE_CLOSE, "Close (esc)").on_press(|s: &mut State| s.close_big()));
    row()
        .absolute(0.0, 0.0)
        .size(s.size.0, TOP)
        .pl(84.0)
        .pr(10.0)
        .gap(14.0)
        .items_center()
        .border_sides([0.0, 0.0, 1.0, 0.0], edge())
        .border_layout()
        .window_drag()
        .child(left)
        .child(el().grow(1.0))
        .child(marks)
}

/// The arrows at the stage's sides (`.av-step`), to the file before and after.
fn arrows(s: &State, open: &Open, stage: (f32, f32)) -> Vec<El> {
    let steps = open.steps(s.now);
    if !rules::draws_arrows(&open.families, &steps) {
        return Vec::new();
    }
    let side = |glyph: &str, by: isize, x: f32| {
        let live = rules::next_step(&open.families, &steps, open.at, by).is_some();
        let b = row()
            .absolute(x, TOP + stage.1 / 2.0 - 22.0)
            .size(28.0, 44.0)
            .radius(6.0)
            .center()
            .fill(rgba(40, 38, 37, 0.7))
            .border(1.0, edge())
            .border_layout()
            .child(text(glyph).font(TextStyle::new(18.0, 400.0).lh(18.0)).color(dim()).group_color(ink()));
        if live {
            b.hover_group().help(if by < 0 { "Previous file" } else { "Next file" }).on_press(move |s: &mut State| s.step_file(by))
        } else {
            b.opacity(0.0)
        }
    };
    vec![side("‹", -1, 12.0), side("›", 1, stage.0 - 12.0 - 28.0)]
}

/// The line under a file from Files: where it was sent (`.fp-sent`), each place opening there.
fn sent_line(open: &Open, w: f32, y: f32) -> El {
    let places = open.sent_in.iter().enumerate().map(|(at, p)| {
        let place = p.clone();
        let glyph = if p.kind == "chat" { &icons::SENT_CHAT } else { &icons::SENT_THREAD };
        row()
            .key(format!("sent-in-{at}"))
            .gap(7.0)
            .items_center()
            .hover_group()
            .child(icon(glyph, 12.0).color(muted()))
            .child(text(p.title.as_str()).font(TextStyle::new(12.5, 400.0).lh(16.0)).color(dim()).group_color(ink()))
            .on_press(move |s: &mut State| s.go_to_place(&place))
    });
    row()
        .absolute(0.0, y)
        .size(w, FOOT)
        .px(16.0)
        .gap(18.0)
        .items_center()
        .border_sides([1.0, 0.0, 0.0, 0.0], edge())
        .border_layout()
        .clip_rounded(0.0)
        .child(text("Sent in").font(mono(11.0, 14.0).tracking(0.88)).text_case(TextCase::Uppercase).color(muted()))
        .children(places)
}

/// The view over the whole window, while a file is open big. It takes the pointer and every
/// press, so nothing under it answers.
pub fn layer(s: &State) -> Option<El> {
    let open = s.viewer.open.as_ref()?;
    let (w, h) = s.size;
    let foot = if open.sent_in.is_empty() { 0.0 } else { FOOT };
    let stage = (w, (h - TOP - foot).max(1.0));
    let drawn = match (open.kind(), &open.words) {
        (_, _) if open.found.is_none() => said(&format!("{} could not be found.", open.version().name)),
        (Some("image"), _) => picture_stage(s, open, stage),
        (Some("markdown"), Some(Words::Doc(words))) => doc_stage(s, open, words, stage),
        (Some("code" | "table"), Some(Words::Code(code))) => code_stage(open, code, stage),
        (Some("markdown" | "code" | "table"), _) => said(&format!("{} could not be opened.", open.version().name)),
        _ => outside_stage(open, stage),
    };
    let mut view = el()
        .absolute(0.0, 0.0)
        .size(w, h)
        .fill(paper())
        .on_hover(|_: &mut State| {})
        .on_press(|_: &mut State| {})
        .child(el().absolute(0.0, TOP).size(stage.0, stage.1).clip_rounded(0.0).child(drawn))
        .children(arrows(s, open, stage))
        .child(bar(s, open));
    if foot > 0.0 {
        view = view.child(sent_line(open, w, h - foot));
    }
    Some(view.role("dialog").label(open.version().name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_documents_sheet_gives_the_text_its_820_points_first() {
        // Beside the web window's notes, the stage is 940 wide: the margins hold only dashes.
        assert_eq!(sheet(940.0), (60.0, 820.0, 60.0, 0.0));
        // The whole window's width: the outline takes its 220 points and the rest is the margin.
        assert_eq!(sheet(1280.0), (160.0, 820.0, 220.0, 40.0));
        assert_eq!(sheet(1600.0), (350.0, 820.0, 350.0, 40.0));
        // A narrow stage keeps 18 points either side.
        assert_eq!(sheet(500.0), (18.0, 464.0, 18.0, 0.0));
    }

    #[test]
    fn the_outline_leaves_out_the_title_and_steps_in_the_lower_levels() {
        let heads = [(1, "The receipt".to_string()), (2, "What is on it".to_string()), (3, "Taxes".to_string()), (2, "How it adds up".to_string()), (4, "Deep".to_string()), (1, "Another top".to_string())];
        let o = outline(&heads);
        assert_eq!(o.iter().map(|(_, sub, w)| (*sub, w.as_str())).collect::<Vec<_>>(), [(true, "What is on it"), (true, "Taxes"), (true, "How it adds up"), (false, "Another top")]);
        assert_eq!(o[0].0, 1);
    }

    #[test]
    fn a_scripts_lines_are_numbered_coloured_and_cut_into_pieces() {
        let text = (1..=450).map(|n| format!("const n{n} = {n};")).collect::<Vec<_>>().join("\n");
        let code = code_of(&text, true);
        assert_eq!(code.pieces.len(), 3);
        assert_eq!(code.lines, 450);
        assert!(code.pieces[0].0.starts_with("1\n2\n3"));
        assert!(code.pieces[2].0.ends_with("450"));
        assert!(!code.pieces[0].2.is_empty());
        // A table is its lines, uncoloured; a line past four thousand characters is cut there.
        let long = "a".repeat(5000);
        let table = code_of(&format!("x,y\n{long}\n"), false);
        assert!(table.pieces[0].2.is_empty());
        assert_eq!(table.widest, LONGEST_LINE + 1);
        assert_eq!(table.lines, 2);
    }

    #[test]
    fn a_picture_sits_in_the_middle_until_it_is_larger_than_its_room() {
        let room = (836.0, 668.0);
        let small = laid_picture((360.0, 64.0), rules::picture_fit((360.0, 64.0), room), 1.0, room);
        assert_eq!((small.x, small.y, small.w, small.h), (290.0, 326.0, 360.0, 64.0));
        // Zoomed past its room it starts at the padding's edge and the content grows around it.
        let wide = (2400.0, 1350.0);
        let fit = rules::picture_fit(wide, room);
        let zoomed = laid_picture(wide, fit, 2.0, room);
        assert_eq!((zoomed.x, zoomed.y), (52.0, 24.0));
        assert!(zoomed.content.0 > room.0 + 104.0);
    }
}
