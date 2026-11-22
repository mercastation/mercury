//! The Files place (`FilesPlace` in `renderer/src/files/FilesPlace.tsx`, its look in
//! `files-place.css`): every file the person's threads made, one tile each however many
//! threads it went to, on All files with a card for each project, and on a project's page.
//! A tile's face is the file itself: a picture's top, a document's first lines, a script's.
//! The search box, the kinds, Recent and Name, and grid or list narrow and order them; a press
//! on a tile opens the file big (`viewer.rs`), with the places it was sent in under it.
//!
//! The list is the app's (`zero:files-place`), read when the page opens and again as the
//! store changes; a file's own lines come when its tile is pressed (`zero:files-place-open`).
//! Both are channels the web window calls.
//!
//! Left out, as nothing here writes yet: New (a document, a table, a folder, an upload), each
//! tile's menu (rename, move to a folder, a new folder, Move to the Trash), dragging a tile onto
//! a folder, choosing files for the Trash, and the total's menu. The total is its words.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rune::{Cursor, El, Latest, SurfaceFit, TextEdit, TextStyle, Tx, Waker, col, el, icon, row, text, text_input};
use serde_json::Value;

use crate::pictures::{Look, Want};
use crate::rules::files_shown::{self as rules, DocBlock, List, Placed, Place as Where, Sort};
use crate::source::{self, Source};
use crate::theme::*;
use crate::{State, View, chrome, feel, icons};

/// The page's own state: the list as last read, and how the person narrowed and ordered it.
#[derive(Default)]
pub struct Place {
    list: Option<Rc<List>>,
    reading: Latest<Result<Value, String>>,
    /// When the list was last asked for, on the window's clock in milliseconds.
    asked_at: f64,
    /// The list is owed a reading: the page opened, or the store changed.
    stale: bool,
    /// The file a pressed tile opens, while the app is asked for its lines.
    opening: Latest<(String, Result<Value, String>)>,
    pub query: TextEdit,
    pub kind: Option<&'static str>,
    /// Only the files with comments open and nobody's approval (`review`).
    pub review: bool,
    pub sort: Sort,
    pub as_list: bool,
    /// The folder of the project's page, when the person went into one.
    pub folder: Option<String>,
    /// How many tiles are drawn, which grows as the foot comes near (`TILES_AT_ONCE`).
    drawn: usize,
    /// The check pictures agents took, folded under their line until it is pressed.
    scratch_open: bool,
}

impl Place {
    pub fn set_waker(&self, wake: Waker) {
        self.reading.set_waker(wake.clone());
        self.opening.set_waker(wake);
    }
    /// The projects the place has listed, once it has been read: the command bar's Files rows.
    pub fn projects(&self) -> Vec<String> {
        self.list.as_ref().map(|l| l.projects.clone()).unwrap_or_default()
    }
}

/// The list the page draws: the fixture's for a picture, else the app's as last read.
fn list_of(s: &State) -> Option<Rc<List>> {
    if let Some(l) = &s.files.list {
        return Some(l.clone());
    }
    let fixture = s.fixture.get("filesPlace")?;
    serde_json::from_value::<List>(fixture.clone()).ok().map(Rc::new)
}

/// The window's turn: the list is asked for when the page is open and owes a reading (at most
/// every two seconds, as a burst of the store's changes reads once), what was read lands, and
/// a pressed tile opens once its lines have come. Says whether to draw again.
pub fn tend(s: &mut State, now: f64, stale: bool) -> bool {
    let on_files = matches!(s.view, View::Files { .. });
    // The command bar lists each project's page too, so it reads the list as well.
    let wanted = on_files || s.palette.open;
    s.files.stale |= stale && wanted;
    if wanted && (s.files.list.is_none() || s.files.stale) && !s.files.reading.is_pending() && now - s.files.asked_at >= 2000.0 {
        if let Source::Door(door) = &s.source
            && let Some((url, token)) = door.now()
        {
            s.files.asked_at = now;
            s.files.stale = false;
            s.files.reading.request((url, token), |(url, token)| source::call(&url, &token, "zero:files-place", &serde_json::json!({})));
        }
    }
    let mut changed = false;
    if let Some(Ok(answer)) = s.files.reading.take()
        && let Ok(list) = serde_json::from_value::<List>(answer)
    {
        s.files.list = Some(Rc::new(list));
        changed = true;
    }
    if let Some((key, Ok(view))) = s.files.opening.take()
        && on_files
    {
        s.open_placed(&key, &view);
        changed = true;
    }
    changed
}

impl State {
    /// The rail's Files: All files, or a project's page.
    pub fn open_files_place(&mut self, project: Option<String>) {
        self.files.folder = None;
        self.files.drawn = rules::TILES_AT_ONCE;
        self.files.stale = true;
        self.go(View::Files { project });
    }
    /// A tile pressed: the app is asked for the file's lines, and the file opens big when they
    /// come. A picture's page has them in its fixture.
    fn open_tile(&mut self, key: &str) {
        if let Some(view) = self.fixture.get("filesPlaceViews").and_then(|v| v.get(key)).cloned() {
            return self.open_placed(key, &view);
        }
        let Source::Door(door) = &self.source else { return };
        let Some((url, token)) = door.now() else { return };
        let key = key.to_string();
        self.files.opening.request((url, token, key), |(url, token, key)| {
            let view = source::call(&url, &token, "zero:files-place-open", &serde_json::json!({ "key": key }));
            (key, view)
        });
    }
}

/* ------------------------------ what a tile's face reads ------------------------------ */

/// A document's or a script's face, read once from the top of its file.
enum Face {
    Doc(Vec<DocBlock>),
    Code(String),
}

thread_local! {
    /// Faces read so far, by the file and what says it changed: its versions and its size.
    static FACES: RefCell<HashMap<(String, u32, u64), Rc<Face>>> = RefCell::new(HashMap::new());
}

/// The face of a document or a script: the top of the file, read off the disk once.
fn face_of(f: &Placed) -> Option<Rc<Face>> {
    let path = f.opened.clone()?;
    let key = (path.clone(), f.versions, f.bytes.unwrap_or(0.0) as u64);
    if let Some(face) = FACES.with(|faces| faces.borrow().get(&key).cloned()) {
        return Some(face);
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(&path).ok()?.take(32 << 10).read_to_end(&mut bytes).ok()?;
    let words = String::from_utf8_lossy(&bytes);
    let face = Rc::new(match f.kind.as_deref() {
        Some("markdown") => Face::Doc(rules::doc_blocks(&words, 8)),
        _ => Face::Code(words.split('\n').take(12).collect::<Vec<_>>().join("\n")),
    });
    FACES.with(|faces| faces.borrow_mut().insert(key, face.clone()));
    Some(face)
}

/* ------------------------------ the tiles ------------------------------ */

/// A picture's face: its top filling the tile, or the whole of it in the middle when filling
/// would cut away too much of it to read (`coverFit`), read at the tile's size.
fn picture_face(s: &State, path: &str, w: f32, h: f32) -> El {
    let places = vec![path.to_string()];
    let want = Want::Cover(s.pictures.pixels(w), s.pictures.pixels(h));
    let drawn = match s.pictures.look(places.clone(), want) {
        Look::Here(held) => {
            let natural = (held.natural.0 as f32, held.natural.1 as f32);
            if rules::cover_fills(natural, (w, h)) {
                el().size(w, h).pixels_fit(&held.pixels, SurfaceFit::Fill).surface_anchor(rune::Align::Center, rune::Align::Start)
            } else {
                el().size(w, h).pixels_fit(&held.pixels, SurfaceFit::Fit)
            }
        }
        _ => el().size(w, h),
    };
    drawn.on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on))
}

/// A document's face (`.fp-doc`): its title, headings, paragraphs and items on the page's paper,
/// fading out over its last quarter.
fn doc_face(blocks: &[DocBlock], w: f32, h: f32) -> El {
    let inner = w - 40.0;
    let words = |s: &str, size: f32, weight: f32, line: f32, c| text(s).font(TextStyle::new(size, weight).lh(line)).color(c).wrap().max_w(inner);
    let mut out = col().size(w, h).pt(20.0).px(20.0).gap(9.0).fill(rgb(40, 38, 37)).clip_fade_linear(0.0, 0.0, 0.0, 0.28 * h);
    for b in blocks {
        out = out.child(match b {
            DocBlock::Title(t) => text(t.as_str()).font(TextStyle::new(15.0, 600.0).lh(19.5).tracking(-0.075)).color(ink()).wrap().max_w(inner).mb(2.0),
            DocBlock::Head(t) => words(t, 12.0, 600.0, kept(16.8), ink()).mt(4.0),
            DocBlock::Words(t) => words(t, 11.5, 400.0, kept(17.825), dim()),
            DocBlock::Item(t) => el().mt(-4.0).pl(14.0).child(el().absolute(3.0, 0.72 * 11.5 + (17.825 - 11.5 * 1.3) / 2.0).size(3.0, 3.0).fill(muted())).child(words(t, 11.5, 400.0, kept(17.825), dim()).max_w(inner - 14.0)),
        });
    }
    out.clip_rounded(9.0)
}

/// A script's or a table's face (`.fp-code-face`): its first twelve lines, small, on the paper.
fn code_face(words: &str, w: f32, h: f32) -> El {
    el().size(w, h).py(14.0).px(16.0).fill(rgb(40, 38, 37)).clip_rounded(9.0).child(text(words).font(mono(10.5, 15.75)).color(dim()))
}

/// A word in the middle of an empty face (`.fp-face-empty`).
fn empty_face(words: &str, w: f32, h: f32) -> El {
    row().size(w, h).center().child(text(words).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()))
}

/// What a tile's face holds, by the file's kind (`Face` in `FilesPlace.tsx`). What this window
/// cannot draw small (a page, a PDF, a deck, a film's first frame) is its kind's word, and a
/// film its play mark.
fn face(s: &State, f: &Placed, w: f32, h: f32) -> El {
    let Some(opened) = f.opened.as_deref() else { return empty_face("Not on this computer yet", w, h) };
    match f.kind.as_deref() {
        Some("image") => picture_face(s, opened, w, h),
        Some("markdown" | "code" | "table") => match face_of(f).as_deref() {
            Some(Face::Doc(blocks)) => doc_face(blocks, w, h),
            Some(Face::Code(words)) => code_face(words, w, h),
            None => empty_face("Not on this computer yet", w, h),
        },
        Some("film") => {
            let play = row().absolute(8.0, h - 8.0 - 24.0).size(24.0, 24.0).radius(12.0).center().pl(1.0).fill(rgba(0, 0, 0, 0.62)).child(icon(&icons::PLAY, 10.0).color(rgb(255, 255, 255)));
            el().size(w, h).child(play)
        }
        kind => empty_face(rules::kind_one(kind), w, h),
    }
}

/// The marks after a tile's words: open comments, Approved, Changed since approved.
fn marks(f: &Placed) -> Vec<El> {
    let mut out = Vec::new();
    if f.open > 0 {
        out.push(row().gap(4.0).items_center().child(icon(&icons::COMMENT, 12.0).color(dim())).child(text(f.open.to_string()).font(figures(12.0, 400.0)).color(dim())));
    }
    if !f.approved_by.is_empty() {
        out.push(row().gap(6.0).items_center().child(el().size(6.0, 6.0).fill(rgb(0x6b, 0xc4, 0x8a))).child(label("Approved", 12.0, 400.0, dim())));
    } else if f.changed_since_approval.is_some() {
        out.push(label("Changed since approved", 12.0, 400.0, dim()));
    }
    out
}

/// A tile (`.fp-tile`): the face, sixteen by ten, its edge brighter under the pointer; the name
/// a person reads; and the line under it. A press opens the file big.
fn tile(s: &State, f: &Placed, w: f32) -> El {
    let h = w * 10.0 / 16.0;
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let said = rules::tile_words(f, s.now, me, |_| "A teammate".to_string());
    let name = rules::shown_name(f);
    let inside = face(s, f, w - 2.0, h - 2.0);
    let face = el()
        .size(w, h)
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, edge())
        .border_layout()
        .child(inside.radius(9.0))
        .child(feel::edge_lit(10.0, rule(), true));
    let face = if f.cloud { face.opacity(0.45) } else { face };
    let meta = row().mt(3.0).gap(12.0).items_center().w(w).child(row().grow(1.0).min_w(0.0).child(text(said).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()).truncate())).children(marks(f));
    let key = f.key.clone();
    let mut tile = col().key(format!("files-tile-{key}")).w(w).hover_group().child(face).child(row().mt(10.0).w(w).child(text(name.clone()).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate())).child(meta);
    if name != f.name {
        tile = tile.help(f.name.clone());
    }
    tile.on_press(move |s: &mut State| s.open_tile(&key))
}

/// A row of the list (`.fp-line`): a small thumbnail, the name, the marks and the line, washed
/// under the pointer.
fn line(s: &State, f: &Placed, w: f32) -> El {
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let said = rules::tile_words(f, s.now, me, |_| "A teammate".to_string());
    let thumb = match (f.kind.as_deref(), f.opened.as_deref()) {
        (Some("image"), Some(opened)) => picture_face(s, opened, 44.0, 30.0),
        (Some("film"), Some(_)) => el().size(44.0, 30.0),
        (kind, _) => row().size(44.0, 30.0).center().child(text(rules::thumb_word(kind)).font(TextStyle::new(9.5, 400.0).lh(12.0).tracking(0.38)).color(muted())),
    };
    let thumb = el().size(46.0, 32.0).radius(6.0).fill(white(0.04)).border(1.0, edge()).border_layout().clip_rounded(6.0).child(thumb);
    let key = f.key.clone();
    row()
        .w(w + 12.0)
        .h(44.0)
        .pl(6.0)
        .pr(44.0)
        .gap(12.0)
        .items_center()
        .radius(6.0)
        .hover(feel::wash())
        .child(thumb)
        .child(row().grow(1.0).min_w(0.0).child(text(rules::shown_name(f)).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate()))
        .child(row().gap(12.0).children(marks(f)))
        .child(text(said).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()))
        .on_press(move |s: &mut State| s.open_tile(&key))
}

/// A project's card on All files, or a folder's (`.fp-folder`): its mark, its name and how many
/// files are in it.
fn folder_card(name: &str, count: usize, w: f32, goes: impl Fn(&mut State) + 'static) -> El {
    let words = if count == 1 { "1 file".to_string() } else { format!("{count} files") };
    row()
        .w(w)
        .h(56.0)
        .px(14.0)
        .gap(11.0)
        .items_center()
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, edge())
        .border_layout()
        .hover_group()
        .child(feel::edge_lit(10.0, rule(), true))
        .child(icon(&icons::FOLDER, 16.0).color(muted()))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .child(text(name).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate())
                .child(text(words).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted())),
        )
        .on_press(goes)
}

/* ------------------------------ the page ------------------------------ */

/// A filter or a switch (`.fp-kind`): quiet words, the one on in ink on the strong wash.
fn kind_button(words: &str, on: bool, does: impl Fn(&mut State) + 'static) -> El {
    let b = row().h(26.0).px(9.0).items_center().radius(6.0).cursor(Cursor::Pointer).child(pen(label(words, 12.5, 400.0, if on { ink() } else { muted() }).group_color(if on { ink() } else { dim() }), 12.5));
    let b = if on { b.fill(feel::wash_strong()) } else { b.hover_group() };
    b.on_press(does)
}

/// Grid or list, by its mark (`.fp-view-icon`).
fn view_button(glyph: &'static rune::Icon, on: bool, help: &'static str, list: bool) -> El {
    let b = row().size(26.0, 26.0).center().radius(6.0).cursor(Cursor::Pointer).help(help).child(icon(glyph, 13.0).color(if on { ink() } else { muted() }).group_color(if on { ink() } else { dim() }));
    let b = if on { b.fill(feel::wash_strong()) } else { b.hover_group() };
    b.on_press(move |s: &mut State| s.files.as_list = list)
}

/// The columns of the grid for a page `wide` points across: as many of at least 232 points as
/// fit with 22 between them, sharing the rest (`repeat(auto-fill, minmax(232px, 1fr))`).
pub fn columns(wide: f32) -> (usize, f32) {
    let n = (((wide + 22.0) / (232.0 + 22.0)).floor() as usize).max(1);
    (n, (wide - 22.0 * (n - 1) as f32) / n as f32)
}

/// The page, under the card's strip, scrolled with the card's other pages.
pub fn view(s: &State) -> El {
    let View::Files { project } = &s.view else { return el() };
    let (page_w, _) = s.page_size();
    let wide = page_w - 44.0 - 32.0;
    let Some(list) = list_of(s) else { return col().pt(18.0).pl(44.0) };
    let place = Where { project: project.clone(), folder: s.files.folder.clone() };
    let below = rules::files_below(&list.files, &list.folders, &place);
    let here = rules::files_in_place(&list.files, &list.folders, &place);
    let query = s.files.query.text.as_str();
    let looking = !query.trim().is_empty() || s.files.kind.is_some() || s.files.review;
    let kinds = rules::kinds_in(&below);
    let reviewing = below.iter().filter(|f| rules::needs_review(f)).count();
    // Looking for something goes through every folder under here; only browsing stays in the one place.
    let found = if looking { rules::files_shown(&below, query, s.files.kind, &list.folders).into_iter().filter(|f| !s.files.review || rules::needs_review(f)).collect() } else { here };
    let (files, scratch) = if query.trim().is_empty() { rules::apart_scratch(found) } else { (found, Vec::new()) };
    let shown = rules::sort_files(&files, s.files.sort);

    // The way back, and the box that searches (`.fp-bar`).
    let crumbs = rules::crumbs_to(&list.folders, &place);
    let last = crumbs.len() - 1;
    let crumbs = row().grow(1.0).min_w(0.0).items_center().children(crumbs.into_iter().enumerate().map(|(i, (name, to))| {
        let on = i == last;
        let crumb = row().h(28.0).items_center().child(pen(text(name).font(sans(14.0, 400.0)).color(if on { ink() } else { muted() }).group_color(ink()), 14.0));
        let crumb = if on { crumb } else { crumb.hover_group().cursor(Cursor::Pointer).on_press(move |s: &mut State| s.go_files_place(&to)) };
        let gap = (i > 0).then(|| text("/").font(sans(14.0, 400.0)).color(muted()).mx(8.0));
        row().items_center().children(gap).child(crumb)
    }));
    let search = text_input(&s.files.query, sans(13.0, 400.0))
        .key("files-search")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder("Search files", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| {
            s.files.drawn = rules::TILES_AT_ONCE;
            &mut s.files.query
        })
        .on_input_key(|s: &mut State, e: &rune::KeyEvent, tx: &mut Tx| {
            // Escape takes back what is typed first, then the keys go back to the page.
            if e.key != rune::edit::Key::Escape {
                return false;
            }
            if s.files.query.text.is_empty() {
                s.files.query.focused = false;
                tx.blur();
            } else {
                s.files.query = TextEdit::new("");
                s.files.query.focused = true;
            }
            true
        });
    let find = row()
        .w(220.0)
        .h(30.0)
        .px(10.0)
        .gap(8.0)
        .items_center()
        .radius(6.0)
        .fill(rgba(27, 25, 23, 0.82))
        .border(1.0, if s.files.query.focused { rule() } else { edge() })
        .border_layout()
        .cursor(Cursor::Text)
        .child(icon(&icons::SEARCH, 13.0).color(muted()))
        .child(search)
        .on_press(|s: &mut State| s.files.query.focused = true);
    let bar = row().w(wide).h(32.0).gap(24.0).items_center().child(crumbs).child(find);

    // The kinds, and how the files are ordered and drawn (`.fp-tools`).
    let mut kind_row = row().gap(2.0).items_center();
    if kinds.len() > 1 {
        kind_row = kind_row.child(kind_button("All", s.files.kind.is_none() && !s.files.review, |s: &mut State| {
            s.files.kind = None;
            s.files.review = false;
        }));
        for k in kinds {
            let on = s.files.kind == Some(k);
            kind_row = kind_row.child(kind_button(rules::kind_plural(k), on, move |s: &mut State| s.files.kind = if s.files.kind == Some(k) { None } else { Some(k) }));
        }
    }
    if reviewing > 0 || s.files.review {
        kind_row = kind_row.child(kind_button("Needs review", s.files.review, |s: &mut State| s.files.review = !s.files.review));
    }
    let used = rules::on_mac_of(&below);
    let mut views = row().gap(12.0).items_center();
    if used > 0.0 {
        let machine = if cfg!(target_os = "macos") { "Mac" } else { "computer" };
        views = views.child(row().h(26.0).px(9.0).gap(7.0).items_center().child(icon(&icons::DISK, 13.0).color(muted())).child(pen(text(format!("{} on this {machine}", rules::size_words(used))).font(figures(12.5, 400.0)).color(muted()), 12.5)));
    }
    let sort = s.files.sort;
    let mut sorts = row().gap(2.0).child(kind_button("Recent", sort == Sort::Recent, |s: &mut State| s.files.sort = Sort::Recent)).child(kind_button("Name", sort == Sort::Name, |s: &mut State| s.files.sort = Sort::Name));
    if sort == Sort::Size {
        sorts = sorts.child(kind_button("Size", true, |s: &mut State| s.files.sort = Sort::Recent));
    }
    views = views.child(sorts).child(row().gap(2.0).child(view_button(&icons::GRID, !s.files.as_list, "Grid", false)).child(view_button(&icons::LIST, s.files.as_list, "List", true)));
    let tools = row().w(wide).h(26.0).mt(14.0).justify_between().items_center().child(kind_row).child(views);

    let mut page = col().w(page_w).pt(18.0).pr(32.0).pb(48.0).pl(44.0).child(bar).child(tools);
    let (n, column) = columns(wide);

    // A card for each project on All files, for each folder on a project's page (`.fp-folders`).
    let mut cards: Vec<El> = Vec::new();
    if !looking {
        if project.is_none() {
            for p in &list.projects {
                let count = rules::files_below(&list.files, &list.folders, &Where { project: Some(p.clone()), folder: None }).len();
                let to = p.clone();
                cards.push(folder_card(p, count, column, move |s: &mut State| s.open_files_place(Some(to.clone()))));
            }
        } else {
            for f in rules::folders_in_place(&list.folders, &place) {
                let count = rules::files_below(&list.files, &list.folders, &Where { project: project.clone(), folder: Some(f.id.clone()) }).len();
                let id = f.id.clone();
                cards.push(folder_card(&f.name, count, column, move |s: &mut State| s.files.folder = Some(id.clone())));
            }
        }
    }
    let had_cards = !cards.is_empty();
    if had_cards {
        page = page.child(grid(cards, n, 22.0, 10.0).mt(18.0));
    }
    if shown.is_empty() && scratch.is_empty() && !had_cards {
        let empty = if looking {
            "No file matches.".to_string()
        } else if s.files.folder.is_some() {
            "Nothing in this folder yet.".to_string()
        } else {
            match project {
                Some(p) => format!("No files in {p} yet."),
                None => "Nothing in Files yet.".to_string(),
            }
        };
        page = page.child(text(empty).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(muted()).mt(26.0));
    }
    // The tiles, a step at a time: a foot past the drawn ones asks for the next step as it nears.
    let drawn = s.files.drawn.max(rules::TILES_AT_ONCE);
    let total = shown.len() + if s.files.scratch_open { scratch.len() } else { 0 };
    let tiles: Vec<El> = shown.iter().take(drawn).map(|f| if s.files.as_list { line(s, f, wide) } else { tile(s, f, column) }).collect();
    if !tiles.is_empty() {
        page = page.child(if s.files.as_list { col().mt(18.0).ml(-6.0).children(tiles) } else { grid(tiles, n, 22.0, 30.0).mt(22.0) });
    }
    // The check pictures agents took wait under one line, which opens them (`.fp-scratch`).
    if !scratch.is_empty() && drawn >= shown.len() {
        let open = s.files.scratch_open;
        let fold = row()
            .mt(30.0)
            .ml(-6.0)
            .h(26.0)
            .pl(6.0)
            .pr(8.0)
            .gap(6.0)
            .radius(7.0)
            .items_center()
            .hover(feel::wash())
            .cursor(Cursor::Pointer)
            .child(text(if open { "⌃" } else { "›" }).font(TextStyle::new(12.0, 400.0).lh(12.0)).color(dim()))
            .child(label(&rules::scratch_line(scratch.len()), 13.0, 400.0, dim()))
            .on_press(|s: &mut State| s.files.scratch_open = !s.files.scratch_open);
        page = page.child(row().child(fold));
        if open {
            let more: Vec<El> = scratch.iter().take(drawn - shown.len()).map(|f| if s.files.as_list { line(s, f, wide) } else { tile(s, f, column) }).collect();
            page = page.child(if s.files.as_list { col().mt(16.0).ml(-6.0).children(more) } else { grid(more, n, 22.0, 30.0).mt(16.0) });
        }
    }
    if drawn < total {
        page = page.child(el().h(1.0).on_visible(move |s: &mut State, on, _| {
            if on {
                s.files.drawn = rules::more_tiles(drawn, total);
            }
        }));
    }
    page
}

/// Rows of `n` cells, `gap` across and `rows_gap` down, as the page's grids lay them out.
fn grid(cells: Vec<El>, n: usize, gap: f32, rows_gap: f32) -> El {
    let mut out = col().gap(rows_gap);
    let mut cells = cells.into_iter().peekable();
    while cells.peek().is_some() {
        out = out.child(row().gap(gap).items_start().children(cells.by_ref().take(n)));
    }
    out
}

impl State {
    /// A crumb pressed: All files, a project's page, or a folder in it.
    fn go_files_place(&mut self, to: &Where) {
        let View::Files { project } = &self.view else { return };
        if *project != to.project {
            self.open_files_place(to.project.clone());
        }
        self.files.folder = to.folder.clone();
    }
}

/* ------------------------------ the strip and the keys ------------------------------ */

/// The page's name in the card's strip, over the sky as the Inbox's is, in a soft patch of the
/// sky's own dark (`.workspace-title`).
pub fn strip(s: &State) -> El {
    let (card_w, _) = s.card_size();
    let under = |a: f32| rgba(0x1b, 0x19, 0x17, a);
    let words = |shadow: bool| pen(text("Files").font(sans(17.0, 400.0).tracking(-0.51)).color(if shadow { under(1.0) } else { ink() }), 17.0);
    let shadows = [8.0, 5.0, 3.0, 1.5, 0.5].map(|sigma| el().absolute(0.0, 0.0).blur(sigma).child(words(true)));
    let patch = el().pin(Some(0.0), Some(0.0), None, None).cache().layer_opacity(0.999).inert().a11y_hidden().children(shadows);
    let title = el().child(patch).child(words(false));
    row().absolute(chrome::card_left(s), chrome::CARD[1]).size(card_w, 64.0).pl(110.0).pr(28.0).pb(1.0).items_center().window_drag().child(title)
}

/// A window opened straight onto the Files place (`--view files`) came there from Home; one
/// opened for a picture of a project's page or of the list says so in its fixture
/// (`filesProject`, `filesAsList`).
pub fn pictured(s: &mut State) {
    let View::Files { .. } = s.view else { return };
    let project = s.fixture.get("filesProject").and_then(Value::as_str).map(str::to_string);
    s.view = View::Files { project };
    s.history.push(View::Home);
    s.files.as_list = s.fixture.get("filesAsList").and_then(Value::as_bool).unwrap_or(false);
    s.files.drawn = rules::TILES_AT_ONCE;
    // A headless picture of a door's Files cannot wait for the list, so it asks for it here.
    if let (Source::Door(door), Some(_)) = (&s.source, rune::tooling::headless_run())
        && let Some((url, token)) = door.now()
        && let Ok(list) = source::call(&url, &token, "zero:files-place", &serde_json::json!({})).and_then(|v| serde_json::from_value::<List>(v).map_err(|e| e.to_string()))
    {
        s.files.list = Some(Rc::new(list));
    }
}

/// The keys on the Files page (`filesShown` in `renderer/src/App.tsx`'s keydown): `/` puts the
/// caret in the search box, Escape leaves the page. N still starts a thread.
pub fn key(s: &mut State, e: &rune::KeyEvent, tx: &mut Tx) -> Option<bool> {
    use rune::edit::Key;
    if !matches!(s.view, View::Files { .. }) {
        return None;
    }
    match &e.key {
        Key::Text(t) if t == "/" => {
            s.files.query.focused = true;
            tx.focus_when_shown("files-search", false);
            Some(true)
        }
        Key::Escape => {
            if s.history.is_empty() { s.go(View::Inbox) } else { s.back() }
            Some(true)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_has_as_many_columns_of_232_points_as_fit() {
        // The page at 1280 by 760 is 1100 points across: four tiles of 258.5.
        assert_eq!(columns(1100.0), (4, 258.5));
        assert_eq!(columns(700.0).0, 2);
        assert_eq!(columns(200.0), (1, 200.0));
    }
}
