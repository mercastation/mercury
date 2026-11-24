//! Files' home (`FilesHome` in `renderer/src/files/FilesHome.tsx`, its look in
//! `files-home.css`): what the person opens Files on. A project's folders as sections of tiles,
//! Pinned first, and what nobody filed after them, with Recent down the left a day at a time; on
//! All files, a tile for each project. A folder's own page and the page of the files in no
//! folder, with the path back over them. The search's results. The menus on a file and on a
//! folder, and New's. Its rules are `rules/files_home.rs`; the state and the writes are the Files
//! place's (`files_page.rs`), as `FilesPlace.tsx` holds them for `FilesHome`.

use std::cell::RefCell;
use std::collections::HashMap;

use rune::{Color, Cursor, Drag, El, Phase, SurfaceFit, TextStyle, Tx, col, el, icon, row, text, text_input};

use crate::files_page::{Menu, MenuOn, Renaming, Step};
use crate::pictures::{Look, Want};
use crate::rules::files_home::{self as home, DocTop, Making, NOT_IN_A_FOLDER, Shelf};
use crate::rules::files_shown::{self as shown, Folder, List, Placed};
use crate::theme::*;
use crate::{State, chrome, feel, icons};

/* ------------------------------ what the page reads ------------------------------ */

thread_local! {
    /// A document's title and first sentence, read once from the top of its file, by the file
    /// and what says it changed: its versions and its size (`useDocTop`).
    static TOPS: RefCell<HashMap<(String, u32, u64), DocTop>> = RefCell::new(HashMap::new());
}

/// A markdown file's title and first sentence. Anything else, or a file not on this computer,
/// has neither.
pub fn doc_top(f: &Placed) -> DocTop {
    if f.kind.as_deref() != Some("markdown") {
        return DocTop::default();
    }
    let Some(path) = f.opened.clone() else { return DocTop::default() };
    let key = (path.clone(), f.versions, f.bytes.unwrap_or(0.0) as u64);
    if let Some(top) = TOPS.with(|t| t.borrow().get(&key).cloned()) {
        return top;
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    let top = match std::fs::File::open(&path).and_then(|file| file.take(32 << 10).read_to_end(&mut bytes)) {
        Ok(_) => home::doc_top(&String::from_utf8_lossy(&bytes)),
        Err(_) => DocTop::default(),
    };
    TOPS.with(|t| t.borrow_mut().insert(key, top.clone()));
    top
}

/// The name a tile shows for a file, its document's title read when it is one.
pub fn name_of(f: &Placed) -> String {
    home::tile_name(f, doc_top(f).title.as_deref())
}

/// Who changed it last, as a tile's foot says it: "you" for the person, else whose agent
/// (`changedBy`). This window does not tell one teammate from another.
fn who(s: &State, f: &Placed) -> String {
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(serde_json::Value::as_str);
    let said = shown::changed_by(f, me, |_| "A teammate".to_string());
    if said == "You" { "you".to_string() } else { said }
}

/// Words cut to `lines` lines as Chromium's `-webkit-line-clamp` cuts them: wrapped a word at a
/// time, the last line kept kept as it wrapped and "…" put after it, characters taken off its
/// end only while the ellipsis does not fit. Rune's own `max_lines` fills the last line to its
/// end before the ellipsis, which reads a word further than the web window does.
pub fn clamped(words: &str, font: TextStyle, width: f32, lines: usize) -> String {
    let wide = |s: &str| rune::text::measure(s, &[], font, None).0;
    let mut kept: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in words.split(' ') {
        let tried = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if line.is_empty() || wide(&tried) <= width {
            line = tried;
            continue;
        }
        kept.push(std::mem::replace(&mut line, word.to_string()));
        if kept.len() == lines {
            // More follows the last line kept: it ends on the ellipsis.
            let mut last = kept.pop().unwrap_or_default();
            while !last.is_empty() && wide(&format!("{}…", last.trim_end())) > width {
                last.pop();
            }
            kept.push(format!("{}…", last.trim_end()));
            return kept.join(" ");
        }
    }
    words.to_string()
}

/* ------------------------------ sizes ------------------------------ */

/// `.fh`: at most 1120 across, in the middle of the page.
const MOST: f32 = 1120.0;
/// Recent's column and the gap after it (`.fh-split`).
const RAIL: f32 = 232.0;
const RAIL_GAP: f32 = 44.0;
/// Under this window width Recent stands over the folders (`@media (max-width: 1100px)`).
const STACKS_UNDER: f32 = 1100.0;
/// A tile's height and the gap between tiles (`.fh-open`, `.fh-grid`).
const TILE_H: f32 = 164.0;
const GAP: f32 = 16.0;

/// The page's own width and where `.fh` stands in it.
fn widths(s: &State) -> (f32, f32, f32) {
    let (page_w, _) = s.page_size();
    let inside = (page_w - 44.0 - 32.0).max(1.0);
    let fh = inside.min(MOST);
    (page_w, fh, (inside - fh) / 2.0)
}

fn split(s: &State) -> bool {
    s.size.0 > STACKS_UNDER
}

/* ------------------------------ small pieces ------------------------------ */

/// A heading of the page's sections (`.fh-head-name`): 19 points, 600, 26 tall.
fn head_font() -> TextStyle {
    TextStyle::new(19.0, 600.0).lh(26.0).tracking(-0.228)
}

/// A count beside a section's name (`.fh-count`): 14 points, faint, its figures tabular.
fn count(n: usize) -> El {
    pen(text(n.to_string()).font(TextStyle::new(14.0, 400.0).lh(26.0).features("'tnum' 1")).color(muted()), 14.0)
}

/// One of the quiet buttons that go somewhere (`.fh-see-all`, `.fh-more`, `.fh-every`): 28 tall,
/// eight in, its words in the text's ink on the wash under the pointer.
fn quiet(words: &str, tone: Color, key: String) -> El {
    row()
        .key(key)
        .h(28.0)
        .px(8.0)
        .radius(7.0)
        .items_center()
        .hover_group()
        .hover(feel::wash())
        .cursor(Cursor::Pointer)
        .child(pen(text(words).font(sans(13.0, 400.0)).color(tone).group_color(ink()), 13.0))
}

/// The ⋯ of a tile or a folder (`.fh-more-btn`): shown while `shown`, the strong wash and the
/// text's ink under the pointer or while its menu is open.
fn more_button(key: String, side: f32, shown: bool, open: bool, name: &str) -> El {
    let b = row()
        .key(key)
        .size(side, side)
        .radius(8.0)
        .center()
        .fill(if open { feel::wash_strong() } else { surface() })
        .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule(), true))
        .hover_group()
        .hover(feel::wash_strong())
        .cursor(Cursor::Pointer)
        .help(format!("More for {name}"))
        .expanded(open)
        .child(icon(&icons::FH_DOTS, 14.0).color(if open { ink() } else { dim() }).group_color(ink()));
    if shown { b } else { b.opacity(0.0) }
}

/// A file's mark (`.fh-glyph`): 20 points in a box of 22, or `size` in `boxed`.
fn glyph(name: &str, boxed: f32, size: f32) -> El {
    row().size(boxed, boxed).center().child(icon(icons::files_glyph(name), size).color(dim()))
}

/// The box a name is typed into, as a tile's, a folder's or a new folder's: Return or leaving
/// it keeps the name, Escape gives it up (`NameInput`, `NameBox`).
pub(crate) fn name_box(s: &State, font: TextStyle, placeholder: &'static str) -> El {
    text_input(&s.files.name_box, font)
        .key(crate::files_page::NAME_BOX)
        .single_line()
        .grow(1.0)
        .min_w(0.0)
        .color(ink())
        .placeholder(placeholder, muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.files.name_box)
        .on_submit(|s: &mut State, tx: &mut Tx| s.name_done(tx))
        .on_input_key(|s: &mut State, e: &rune::KeyEvent, tx: &mut Tx| {
            if e.key != rune::edit::Key::Escape {
                return false;
            }
            s.name_given_up(tx);
            true
        })
}

/// `.fh-name-box`: 26 tall, seven in and seven out to the left, the surface and an inset strong
/// line, 14 points; `w` and `font` as the place it stands in sets them.
fn fh_name_box(s: &State, w: Option<f32>, h: f32, font: TextStyle, placeholder: &'static str) -> El {
    let b = row().h(h).ml(-7.0).px(7.0).radius(6.0).items_center().fill(surface()).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule(), true)).child(name_box(s, font, placeholder));
    match w {
        Some(w) => b.w(w),
        None => b.grow(1.0).min_w(0.0),
    }
}

/* ------------------------------ a tile ------------------------------ */

/// A picture's own strip on its tile (`.fh-preview`), filling it from its middle; nothing while
/// it is on its way, and None when it will not load, so the tile wears its mark instead.
fn preview(s: &State, path: &str, w: f32) -> Option<El> {
    let places = vec![path.to_string()];
    let want = Want::Cover(s.pictures.pixels(w), s.pictures.pixels(58.0));
    let inside = match s.pictures.look(places.clone(), want) {
        Look::Here(held) => el().size(w, 58.0).pixels_fit(&held.pixels, SurfaceFit::Fill).surface_anchor(rune::Align::Center, rune::Align::Center),
        Look::Missing => return None,
        Look::Arriving(_) => el().size(w, 58.0),
    };
    let strip = el()
        .size(w, 58.0)
        .radius(8.0)
        .fill(feel::wash())
        .clip_rounded(8.0)
        .child(inside)
        .child(el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(8.0).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, edge(), true)))
        .on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on));
    Some(strip)
}

/// A file's tile (`Tile`, `.fh-tile`): its mark or its picture, its name, its line and its foot,
/// 164 tall. Under the pointer, or with its menu open, its ground and edge are lit and it stands
/// a point higher, its ⋯ shown. A press opens the file; a right press, its menu at the pointer.
fn tile(s: &State, f: &Placed, folder: Option<&Folder>, w: f32) -> El {
    let key = f.key.clone();
    let top = doc_top(f);
    let name = home::tile_name(f, top.title.as_deref());
    let renaming = s.files.renaming == Some(Renaming::File(key.clone()));
    let open = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::File(key.clone()));
    let lit = s.files.hover.tile.as_deref() == Some(key.as_str()) || open;
    let inner = w - 36.0;
    let picture = if home::shows_picture(f) { f.opened.as_deref().and_then(|p| preview(s, p, inner + 12.0)) } else { None };
    let line = if f.kind.as_deref() == Some("markdown") { top.line.clone() } else { Some(home::tile_line(f)) };
    let pinned = folder.is_some_and(|x| home::pin_changed_lately(&x.name, f.at, s.now));

    let mut face = col().w(w).h(TILE_H).pt(16.0).px(18.0).pb(14.0);
    let had_picture = picture.is_some();
    face = match picture {
        Some(strip) => face.child(strip.ml(-6.0).mt(-4.0)),
        None => face.child(glyph(home::glyph_for(f, folder.map(|x| x.name.as_str()), top.title.as_deref()), 22.0, 20.0)),
    };
    let name_font = TextStyle::new(14.5, 600.0).lh(20.0).tracking(-0.058);
    let mut named = row().mt(if had_picture { 10.0 } else { 12.0 }).w(inner).gap(7.0).items_center();
    named = if renaming {
        named.child(fh_name_box(s, None, 26.0, TextStyle::new(14.0, 600.0).lh(20.0), "Name"))
    } else {
        named.child(row().min_w(0.0).child(pen(text(name.clone()).font(name_font).color(ink()).truncate(), 14.5)))
    };
    if pinned {
        named = named.child(el().size(6.0, 6.0).radius(3.0).fill(accent()).help("Changed this week"));
    }
    face = face.child(named);
    face = face.child(match line {
        Some(line) => {
            let (font, lines) = (TextStyle::new(13.0, 400.0).lh(19.0), if had_picture { 1 } else { 2 });
            pen(text(clamped(&line, font, inner, lines)).font(font).color(dim()).wrap().max_w(inner).max_lines(lines), 13.0).mt(3.0)
        }
        None => el().mt(3.0),
    });
    face = face.child(el().grow(1.0)).child(row().w(inner).pt(8.0).child(pen(text(format!("{} · {}", home::when_words(f.at, s.now), who(s, f))).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()).truncate(), 12.0)));

    let ground = el()
        .pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0))
        .radius(14.0)
        .fill(ink_at(if lit { 0.07 } else { 0.04 }))
        .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, if lit { rule() } else { edge() }, true));
    let mut t = el().key(format!("fh-tile-{key}")).size(w, TILE_H).radius(14.0).child(ground).child(face);
    if lit {
        t = t.offset(0.0, -1.0);
    }
    let (k1, k2, k3) = (key.clone(), key.clone(), key.clone());
    t = t.on_hover_change(move |s: &mut State, inside, _| {
        if inside {
            s.files.hover.tile = Some(k1.clone());
        } else if s.files.hover.tile.as_deref() == Some(k1.as_str()) {
            s.files.hover.tile = None;
        }
    });
    if renaming {
        return t;
    }
    if name != f.name {
        t = t.help(f.name.clone());
    }
    let more = more_button(format!("fh-more-{key}"), 28.0, lit, open, &name).absolute(w - 12.0 - 28.0, 12.0).on_press(move |s: &mut State| s.toggle_file_menu(&k2, None));
    t.child(more)
        .cursor(Cursor::Pointer)
        .on_press(move |s: &mut State| s.open_tile(&key))
        // The page's pointer is in whole points (`clientX`), four past it (`openMenu`).
        .on_secondary_press(move |s: &mut State, p, _| s.toggle_file_menu(&k3, Some((p.x.floor() + 4.0, p.y.floor() + 4.0))))
}

/// Rows of tiles `n` across, sixteen apart each way (`.fh-grid`).
fn grid(cells: Vec<El>, n: usize) -> El {
    let mut out = col().gap(GAP);
    let mut cells = cells.into_iter().peekable();
    while cells.peek().is_some() {
        out = out.child(row().gap(GAP).items_start().children(cells.by_ref().take(n)));
    }
    out
}

fn tile_w(main: f32, n: usize) -> f32 {
    (main - GAP * (n - 1) as f32) / n as f32
}

/* ------------------------------ the page's top ------------------------------ */

/// A control over the sky's band, as the strip's are: the sky's dark at 82% over the sky blurred
/// behind it at rest, `lit` under the pointer or while its menu is open (`.app.sky-shown
/// :is(.th-new, .fp-find):not(:hover):not([aria-expanded])`). `key` names it for the pointer.
/// An edge of `edge_w` is the strong line, drawn over the sky behind as the strip's buttons
/// draw theirs (`inbox::strip_button`).
pub(crate) fn over_the_sky(s: &State, key: &'static str, radius: f32, edge_w: f32, open: bool, lit: Color) -> El {
    let b = row().key(key).radius(radius);
    let whole = || el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0));
    let b = if s.pointed.is(key) || open {
        let b = b.fill(lit);
        if edge_w > 0.0 { b.border(edge_w, rule()) } else { b }
    } else {
        match chrome::sky_through(key, &s.sky_inbox_soft, crate::inbox::band_in_the_window(s), s.sky_inbox.size(), chrome::SOFT_STRIP, radius, edge_w) {
            Some(behind) => {
                let behind = behind.child(whole().fill(crate::inbox::under(0.82)));
                let behind = if edge_w > 0.0 { behind.child(whole().radius(radius).border(edge_w, rule())) } else { behind };
                let b = b.child(behind);
                if edge_w > 0.0 { b.border(edge_w, rgba(0, 0, 0, 0.0)) } else { b }
            }
            None => {
                let b = b.fill(crate::inbox::under(0.82));
                if edge_w > 0.0 { b.border(edge_w, rule()) } else { b }
            }
        }
    };
    feel::pointed(b.border_layout(), key)
}

pub const FIND: &str = "files-find";
pub const NEW: &str = "files-new";

/// The search box (`.fp-find.fh-find`): 320 by 34, the magnifier, "Search files" and the `/`
/// that puts the caret in it. Escape with words in it takes them back and goes no further.
fn find(s: &State) -> El {
    let search = text_input(&s.files.query, sans(13.0, 400.0))
        .key(crate::files_page::SEARCH)
        .single_line()
        .grow(1.0)
        .min_w(0.0)
        .color(ink())
        .placeholder("Search files", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| &mut s.files.query)
        .on_input_key(|s: &mut State, e: &rune::KeyEvent, _tx: &mut Tx| {
            // With nothing in it Escape does nothing at all: the page takes no key from a box,
            // and the box keeps the caret.
            if e.key != rune::edit::Key::Escape {
                return false;
            }
            if !s.files.query.text.is_empty() {
                s.files.query = rune::TextEdit::new("");
                s.files.query.focused = true;
            }
            true
        });
    let cap = row()
        .size(20.0, 20.0)
        .radius(5.0)
        .center()
        .fill(surface())
        .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule(), true))
        .child(text("/").font(mono(11.0, 11.0)).color(muted()));
    over_the_sky(s, FIND, 9.0, 0.0, false, feel::wash())
        .w(320.0)
        .h(34.0)
        .pl(12.0)
        .pr(7.0)
        .gap(8.0)
        .items_center()
        .cursor(Cursor::Text)
        .child(icon(&icons::FILES_SEARCH, 15.0).color(muted()))
        .child(search)
        .child(cap)
        .on_press(|s: &mut State| s.files.query.focused = true)
}

/// New (`.th-new` in `NewMenu`): its edge, the plus and the word; it opens New's menu.
fn new_button(s: &State) -> El {
    let open = s.files.new_menu.is_some();
    over_the_sky(s, NEW, 6.0, 1.0, open, feel::wash_strong())
        .h(32.0)
        .px(13.0)
        .gap(8.0)
        .items_center()
        .cursor(Cursor::Pointer)
        .expanded(open)
        .child(icon(&icons::FH_PLUS, 13.0).color(dim()))
        .child(pen(label("New", 13.5, 400.0, ink()), 13.5))
        .on_press(|s: &mut State| s.toggle_new_menu())
}

/// The path back (`.fh-crumbs`): each place above this one, a chevron between, a press away.
fn crumbs(s: &State, list: &List) -> Option<El> {
    let View { project, .. } = here(s);
    let crumbs = home::crumbs_for(&list.folders, project.as_deref(), s.files.folder.as_deref());
    if crumbs.is_empty() {
        return None;
    }
    let boxes = crumbs.into_iter().enumerate().map(|(i, c)| {
        let words = c.name.clone();
        let crumb = row()
            .key(format!("fh-crumb-{i}"))
            .h(24.0)
            .px(6.0)
            .radius(6.0)
            .items_center()
            .hover_group()
            .hover(feel::wash())
            .cursor(Cursor::Pointer)
            .child(pen(text(words).font(sans(13.0, 400.0)).color(muted()).group_color(ink()), 13.0))
            .on_press(move |s: &mut State| s.go_crumb(c.project.clone(), c.folder.clone()));
        let chevron = (i > 0).then(|| icon(&icons::FH_CHEVRON, 12.0).color(muted()));
        row().gap(2.0).items_center().children(chevron).child(crumb)
    });
    Some(row().min_h(20.0).mt(-4.0).mb(6.0).ml(-6.0).gap(2.0).items_center().children(boxes))
}

/// Where the page is, from the window's view and the place's own state.
struct View {
    project: Option<String>,
}

fn here(s: &State) -> View {
    match &s.view {
        crate::View::Files { project } => View { project: project.clone() },
        _ => View { project: None },
    }
}

/// The page's heading: a folder's name with its ⋯, "Not in a folder", or the project's name or
/// Files (`FilesHome`'s `heading`).
fn heading(s: &State, title: &str, folder: Option<&Folder>) -> El {
    let h1 = |words: String| pen(text(words).font(TextStyle::new(30.0, 600.0).lh(36.0).tracking(-0.6)).color(ink()).truncate(), 30.0);
    let mut t = row().key("fh-title").grow(1.0).min_w(0.0).gap(8.0).items_center();
    match folder {
        Some(f) => {
            if s.files.renaming == Some(Renaming::Folder(f.id.clone())) {
                // `.fh-title .fh-name-box`: 360 by 38, 22 points.
                return t.child(fh_name_box(s, Some(360.0), 38.0, TextStyle::new(22.0, 600.0), "Folder name"));
            }
            let open = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::Folder(f.id.clone()));
            let id = f.id.clone();
            t = t
                .child(row().min_w(0.0).child(h1(f.name.clone())))
                .child(more_button(format!("fh-fmore-{id}"), 26.0, s.files.hover.title || open, open, &f.name).on_press(move |s: &mut State| s.toggle_folder_menu(&id)));
            t.on_hover_change(|s: &mut State, inside, _| s.files.hover.title = inside)
        }
        None => t.child(row().min_w(0.0).child(h1(title.to_string()))),
    }
}

/* ------------------------------ the home's body ------------------------------ */

/// A section's heading row (`.fh-head`): 28 tall, 14 above the tiles.
fn head_row() -> El {
    row().min_h(28.0).mb(14.0).gap(10.0).items_center()
}

/// A section's name that opens its page (`.fh-link`): underlined under the pointer.
fn link(words: &str, goes: impl Fn(&mut State) + 'static) -> El {
    let words = pen(text(words).font(head_font()).color(ink()), 19.0);
    let under = el().pin(Some(0.0), Some(23.5), Some(0.0), None).h(1.5).fill(rule()).group_opacity(0.0, 1.0);
    row().h(26.0).items_center().hover_group().cursor(Cursor::Pointer).child(words).child(under).on_press(goes)
}

/// A folder's section (`Section`): its name, its count and its ⋯, See all once it holds more
/// than six, and its first six tiles.
fn section(s: &State, id: &str, head: El, files: &[&Placed], folder: Option<&Folder>, main: f32, n: usize, more: impl Fn(&mut State) + 'static) -> El {
    let mut head = head;
    if files.len() > home::SHOWN_AT_FIRST {
        head = head.child(el().grow(1.0)).child(quiet(&format!("See all {}", files.len()), dim(), format!("fh-see-all-{id}")).mr(-8.0).on_press(more));
    }
    let w = tile_w(main, n);
    let tiles = files.iter().take(home::SHOWN_AT_FIRST).map(|f| tile(s, f, folder, w)).collect();
    let id = id.to_string();
    col()
        .key(format!("fh-sec-{id}"))
        .w(main)
        .mt(38.0)
        .child(head)
        .child(grid(tiles, n))
        .on_hover_change(move |s: &mut State, inside, _| {
            if inside {
                s.files.hover.sec = Some(id.clone());
            } else if s.files.hover.sec.as_deref() == Some(id.as_str()) {
                s.files.hover.sec = None;
            }
        })
}

/// A folder's head in its section (`FolderHead`): its name opening its page, its count, its ⋯
/// shown while the pointer is on the section or its menu is open; renaming, a box of 280 by 30.
fn folder_head(s: &State, f: &Folder, n: usize) -> El {
    if s.files.renaming == Some(Renaming::Folder(f.id.clone())) {
        return head_row().child(fh_name_box(s, Some(280.0), 30.0, TextStyle::new(17.0, 600.0), "Folder name"));
    }
    let open = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::Folder(f.id.clone()));
    let shown = open || s.files.hover.sec.as_deref() == Some(f.id.as_str());
    let (to, id) = (f.id.clone(), f.id.clone());
    head_row()
        .child(link(&f.name, move |s: &mut State| s.go_folder(Some(to.clone()))))
        .child(count(n))
        .child(more_button(format!("fh-fmore-{id}"), 26.0, shown, open, &f.name).on_press(move |s: &mut State| s.toggle_folder_menu(&id)))
}

/// A sentence of the page (`.fh-empty`, `.fh-none`): 13 points, faint.
fn says(words: String) -> El {
    pen(text(words).font(TextStyle::new(13.0, 400.0).lh(20.0)).color(muted()), 13.0)
}

/// A row of Recent (`RecentRow`, `.fh-rrow`): the file's mark and its name; a press opens it.
fn recent_row(s: &State, r: &home::RecentFile) -> El {
    let f = r.file;
    let top = doc_top(f);
    let name = home::tile_name(f, top.title.as_deref());
    let key = f.key.clone();
    let _ = s;
    row()
        .key(format!("fh-rrow-{key}"))
        .w(RAIL + 20.0)
        .h(32.0)
        .px(10.0)
        .gap(10.0)
        .radius(9.0)
        .items_center()
        .hover(feel::wash())
        .cursor(Cursor::Pointer)
        .help(name.clone())
        .child(glyph(home::glyph_for(f, r.folder.map(|x| x.name.as_str()), top.title.as_deref()), 20.0, 19.0))
        .child(row().grow(1.0).min_w(0.0).child(pen(text(name).font(TextStyle::new(14.0, 500.0).lh(20.0)).color(ink()).truncate(), 14.0)))
        .on_press(move |s: &mut State| s.open_tile(&key))
}

/// Recent, down the left (`.fh-rail`): its heading and the files under each day.
fn rail(s: &State, recent: &[home::RecentFile]) -> El {
    let mut out = col().w(RAIL).mt(38.0).child(row().h(28.0).items_center().child(pen(text("Recent").font(head_font()).color(ink()), 19.0)));
    for (i, (day, rows)) in home::recent_by_day(recent, s.now).into_iter().enumerate() {
        let label = row().h(20.0).px(10.0).items_center().child(pen(text(day).font(TextStyle::new(12.5, 400.0).lh(20.0)).color(muted()), 12.5));
        out = out.child(col().ml(-10.0).mt(if i == 0 { 14.0 } else { 10.0 }).child(label).child(col().mt(2.0).children(rows.iter().map(|r| recent_row(s, r)))));
    }
    out
}

/// Recent beside the page's folders, or the folders alone when nothing changed lately
/// (`besideRecent`). Under 1100 points of window the two stand one over the other.
fn beside_recent(s: &State, recent: &[home::RecentFile], fh: f32, main: impl Fn(f32, usize) -> El) -> El {
    if recent.is_empty() {
        return main(fh, 4);
    }
    // One over the other, the folders still three across (`.fh-split .fh-grid` holds), and the
    // last day's ten points under Recent kept (the column is a grid item, so its margin stays).
    if !split(s) {
        return col().child(rail(s, recent).pb(10.0)).child(main(fh, 3));
    }
    let wide = fh - RAIL - RAIL_GAP;
    row().items_start().gap(RAIL_GAP).child(rail(s, recent)).child(col().w(wide).child(main(wide, 3)))
}

/// A project's tile on All files: the folder mark, its name, what it kept and when it last
/// changed. A press opens its page.
fn project_tile(s: &State, p: &Shelf, w: f32) -> El {
    let key = format!("fh-tile-project-{}", p.name);
    let lit = s.files.hover.tile.as_deref() == Some(key.as_str());
    let inner = w - 36.0;
    let face = col()
        .w(w)
        .h(TILE_H)
        .pt(16.0)
        .px(18.0)
        .pb(14.0)
        .child(glyph("folder", 22.0, 20.0))
        .child(row().mt(12.0).w(inner).child(pen(text(p.name.clone()).font(TextStyle::new(14.5, 600.0).lh(20.0).tracking(-0.058)).color(ink()).truncate(), 14.5)))
        .child(pen(text(clamped(&home::shelf_line(p), TextStyle::new(13.0, 400.0).lh(19.0), inner, 2)).font(TextStyle::new(13.0, 400.0).lh(19.0)).color(dim()).wrap().max_w(inner).max_lines(2), 13.0).mt(3.0))
        .child(el().grow(1.0))
        .child(row().w(inner).h(24.0).pt(8.0).child(pen(text(p.at.map(|at| home::when_words(at, s.now)).unwrap_or_default()).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()).truncate(), 12.0)));
    let ground = el().pin(Some(0.0), Some(0.0), Some(0.0), Some(0.0)).radius(14.0).fill(ink_at(if lit { 0.07 } else { 0.04 })).box_shadow(shadow(0.0, 0.0, 0.0, 1.0, if lit { rule() } else { edge() }, true));
    let mut t = el().key(key.clone()).size(w, TILE_H).radius(14.0).cursor(Cursor::Pointer).child(ground).child(face);
    if lit {
        t = t.offset(0.0, -1.0);
    }
    let name = p.name.clone();
    t.on_hover_change(move |s: &mut State, inside, _| {
        if inside {
            s.files.hover.tile = Some(key.clone());
        } else if s.files.hover.tile.as_deref() == Some(key.as_str()) {
            s.files.hover.tile = None;
        }
    })
    .on_press(move |s: &mut State| s.open_files_place(Some(name.clone())))
}

/// Files' home, a folder's page, the files in no folder, or the search's results, as the page
/// is (`FilesHome`).
pub fn view(s: &State, list: Option<&List>) -> El {
    let (page_w, fh, left) = widths(s);
    let project = here(s).project;
    let loaded = list.is_some();
    let empty_list = List::default();
    let list = list.unwrap_or(&empty_list);
    let folder = project.as_ref().and_then(|_| s.files.folder.as_deref()).and_then(|id| list.folders.iter().find(|f| f.id == id));
    let title = project.clone().unwrap_or_else(|| "Files".to_string());
    let loose_page = project.is_some() && folder.is_none() && s.files.folder.as_deref() == Some(NOT_IN_A_FOLDER);
    let mut page = col().w(fh).ml(left).pt(16.0).pb(24.0);
    if let Some(c) = crumbs(s, list) {
        page = page.child(c);
    }
    let shown_title = if loose_page { "Not in a folder".to_string() } else { title.clone() };
    let mut top = row().w(fh).h(36.0).gap(12.0).items_center().child(heading(s, &shown_title, folder)).child(find(s));
    if new_offers(s, list).is_some() {
        top = top.child(new_button(s));
    }
    page = page.child(top);

    let home_here = project.as_deref().map(|p| home::home_of(&list.files, &list.folders, p));
    let query = s.files.query.text.trim();
    let body = if !query.is_empty() {
        let found = crate::files_page::found(s, list);
        let mut head = head_row().child(pen(text(if found.is_empty() { "No file matches" } else { "Matches" }).font(head_font()).color(ink()), 19.0));
        if !found.is_empty() {
            head = head.child(count(found.len()));
        }
        let w = tile_w(fh, 4);
        col().key("fh-sec-found").w(fh).mt(38.0).child(head).child(grid(found.iter().take(home::FOUND_MOST).map(|f| tile(s, f, None, w)).collect(), 4))
    } else if let (Some(p), Some(f)) = (project.as_deref(), folder) {
        shelf(s, &home::folder_files(&list.files, &list.folders, p, &f.id), Some(f), fh)
    } else if loose_page {
        shelf(s, home_here.as_ref().map(|h| h.loose.as_slice()).unwrap_or(&[]), None, fh)
    } else {
        match &home_here {
            Some(h) => {
                let recent = home::recent_of(std::slice::from_ref(h));
                beside_recent(s, &recent, fh, |main, n| project_home(s, h, &title, loaded, main, n))
            }
            None => {
                let homes: Vec<home::Home> = list.projects.iter().map(|p| home::home_of(&list.files, &list.folders, p)).collect();
                let recent = home::recent_of(&homes);
                let shelves = home::shelves(list);
                beside_recent(s, &recent, fh, |main, n| {
                    let mut sec = col().key("fh-sec-projects").w(main).mt(38.0);
                    if !shelves.is_empty() {
                        sec = sec.child(head_row().child(pen(text("Projects").font(head_font()).color(ink()), 19.0)));
                    }
                    if loaded && shelves.is_empty() {
                        sec = sec.child(says("Nothing in Files yet.".to_string()));
                    }
                    let w = tile_w(main, n);
                    sec.child(grid(shelves.iter().map(|p| project_tile(s, p, w)).collect(), n))
                })
            }
        }
    };
    col().w(page_w).pt(18.0).pl(44.0).pr(32.0).pb(48.0).child(page.child(body))
}

/// A project's home beside Recent: the box a new folder is named in, each folder's section, the
/// empty folders in a sentence, the files in no folder, and Every file.
fn project_home(s: &State, h: &home::Home, title: &str, loaded: bool, main: f32, n: usize) -> El {
    let mut out = col().w(main);
    if s.files.naming.is_some() {
        // `NameBox` (`.fp-name-box`) as a section's head: 24 tall, six in and six out, the wash
        // and a strong edge, 13 points.
        let b = row().w(main).h(24.0).ml(-6.0).px(6.0).radius(6.0).items_center().fill(feel::wash()).border(1.0, rule()).border_layout().child(name_box(s, sans(13.0, 400.0), "Folder name"));
        out = out.child(col().key("fh-sec-naming").w(main).mt(38.0).child(row().min_h(28.0).items_center().child(b)));
    }
    for sec in &h.kept {
        let id = sec.folder.id.clone();
        out = out.child(section(s, &sec.folder.id, folder_head(s, sec.folder, sec.files.len()), &sec.files, Some(sec.folder), main, n, move |s: &mut State| s.go_folder(Some(id.clone()))));
    }
    if !h.empty.is_empty() {
        let names: Vec<&str> = h.empty.iter().map(|f| f.name.as_str()).collect();
        out = out.child(says(home::empty_words(&names)).mt(26.0));
    }
    if !h.loose.is_empty() {
        let head = head_row().child(link("Not in a folder", |s: &mut State| s.go_folder(Some(NOT_IN_A_FOLDER.to_string())))).child(count(h.loose.len()));
        out = out.child(section(s, NOT_IN_A_FOLDER, head, &h.loose, None, main, n, |s: &mut State| s.go_folder(Some(NOT_IN_A_FOLDER.to_string()))));
    }
    let nothing = loaded && h.kept.is_empty() && h.loose.is_empty() && h.empty.is_empty();
    if nothing {
        out = out.child(says(format!("Nothing in {title} yet.")).mt(26.0));
    }
    if loaded && !nothing {
        out = out.child(row().child(quiet("Every file, with sizes and the Trash", muted(), "fh-every".to_string()).mt(30.0).ml(-8.0).on_press(|s: &mut State| s.files_everything())));
    }
    out
}

/// A folder's page, or the files in no folder (`Shelf`): every file, sixty at a time.
fn shelf(s: &State, files: &[&Placed], folder: Option<&Folder>, fh: f32) -> El {
    let mut out = col().key("fh-sec-shelf").w(fh).mt(22.0);
    if files.is_empty() {
        return out.child(says("Nothing in this folder yet.".to_string()));
    }
    let limit = s.files.shelf.max(home::SHELF_STEP);
    let w = tile_w(fh, 4);
    out = out.child(grid(files.iter().take(limit).map(|f| tile(s, f, folder, w)).collect(), 4));
    if files.len() > limit {
        out = out.child(row().child(quiet("Show more", muted(), "fh-more".to_string()).mt(12.0).ml(-8.0).on_press(|s: &mut State| s.files.shelf += home::SHELF_STEP)));
    }
    out
}

/// What New offers on this page, or None where it is not drawn. Upload from computer is left
/// out: the app answers it with the system's file window, which it opens over its own window
/// and not over this one.
pub fn new_offers(s: &State, list: &List) -> Option<Vec<Making>> {
    let project = here(s).project;
    let folder = project.as_ref().and_then(|_| s.files.folder.as_deref()).and_then(|id| list.folders.iter().find(|f| f.id == id));
    // Folders go one level deep.
    let can_folder = folder.is_none_or(|f| f.parent.is_none());
    home::new_offers(project.as_deref(), &list.mine, can_folder).map(|offers| offers.into_iter().filter(|m| *m != Making::Upload).collect())
}

/* ------------------------------ the menus ------------------------------ */

/// A row of a menu (`.fh-item`): 30 tall, its mark faint, the strong wash under the pointer,
/// and the tick at its end when it is the one on.
fn item(words: &str, mark: Option<El>, on: bool, does: impl Fn(&mut State, &mut Tx) + 'static) -> El {
    let mut r = row().key(format!("fh-item-{words}")).w(MENU_IN).h(30.0).px(8.0).gap(9.0).radius(6.0).items_center().hover(feel::wash_strong()).cursor(Cursor::Pointer).role("menuitem");
    if let Some(mark) = mark {
        r = r.child(mark);
    }
    // Words too long for the row run on under its edge and are cut there, no ellipsis: the
    // button's words are a flex item, which `text-overflow` does not reach.
    r = r.child(pen(text(words).font(sans(13.0, 400.0)).color(ink()), 13.0)).child(el().grow(1.0));
    if on {
        r = r.child(icon(&icons::FH_CHECK, 13.0).color(dim()));
    }
    r.clip_rounded(6.0).on_press_tx(does)
}

/// A menu's inside width: 210 less its padding.
const MENU_IN: f32 = 200.0;

fn menu_gap() -> El {
    el().h(1.0).mx(6.0).my(5.0).fill(edge())
}

fn menu_head(words: &str, top: f32, bottom: f32) -> El {
    row().px(8.0).pt(top).pb(bottom).child(pen(text(words).font(sans(12.0, 400.0)).color(muted()), 12.0))
}

/// A folder's mark in a menu, 15 points: on the line's own box, as a mark inside a span stands
/// (`<span><GlyphIcon size={15}/></span>`), two points above the middle; a subfolder's in a box
/// of its own, in the middle and fourteen in.
fn folder_mark(sub: bool) -> El {
    if sub { row().pl(14.0).child(icon(&icons::FH_FOLDER, 15.0).color(muted())) } else { el().h(19.0).child(icon(&icons::FH_FOLDER, 15.0).color(muted())) }
}

/// The menu's rows and its height, by what it is open on and at which step.
fn menu_body(s: &State, list: &List, m: &Menu) -> Option<(El, f32, f32)> {
    match &m.on {
        MenuOn::Folder(id) => {
            let f = list.folders.iter().find(|f| &f.id == id)?;
            let (a, b) = (f.id.clone(), f.id.clone());
            let rows = col()
                .child(item("Rename", None, false, move |s: &mut State, tx| s.rename_folder_start(&a, tx)))
                .child(menu_gap())
                .child(item("Delete folder", None, false, move |s: &mut State, tx| s.remove_folder(&b, tx)));
            Some((rows, home::MENU_WIDTH, 10.0 + 30.0 + 11.0 + 30.0))
        }
        MenuOn::File(key) => {
            let f = list.files.iter().find(|f| &f.key == key)?;
            Some(file_menu(s, list, f, m.step))
        }
    }
}

fn file_menu(s: &State, list: &List, f: &Placed, step: Step) -> (El, f32, f32) {
    let key = f.key.clone();
    match step {
        Step::Icon => {
            let mut grid = col().pt(2.0).px(4.0).pb(4.0).gap(4.0);
            let cell = (home::MENU_WIDE - 10.0 - 8.0 - 20.0) / 6.0;
            for names in home::GLYPH_NAMES.chunks(6) {
                let mut r = row().gap(4.0);
                for g in names {
                    let on = f.icon.as_deref() == Some(*g);
                    let k = key.clone();
                    let mut pick = row().w(cell).h(34.0).radius(8.0).center().hover_group().hover(feel::wash_strong()).cursor(Cursor::Pointer).help(*g).child(icon(icons::files_glyph(g), 18.0).color(if on { ink() } else { dim() }).group_color(ink()));
                    if on {
                        pick = pick.box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule(), true));
                    }
                    r = r.child(pick.on_press_tx(move |s: &mut State, tx| s.pick_icon(&k, g, tx)));
                }
                grid = grid.child(r);
            }
            let rows_n = home::GLYPH_NAMES.len().div_ceil(6) as f32;
            let k = key.clone();
            let auto = item("Automatic", Some(icon(icons::files_glyph(home::automatic_glyph(f)), 15.0).color(muted())), f.icon.is_none(), move |s: &mut State, tx| s.pick_icon(&k, "", tx)).w(home::MENU_WIDE - 10.0);
            let body = col().child(menu_head("Icon", 6.0, 4.0)).child(grid).child(menu_gap()).child(auto);
            (body, home::MENU_WIDE, 10.0 + 25.5 + 6.0 + rows_n * 34.0 + (rows_n - 1.0) * 4.0 + 11.0 + 30.0)
        }
        Step::Move | Step::New => {
            let here: Vec<&Folder> = s.files_folders(list);
            let tops: Vec<&Folder> = here.iter().copied().filter(|x| x.parent.is_none()).collect();
            let mut rows = col();
            let mut n: f32 = 0.0;
            for t in &tops {
                for x in std::iter::once(*t).chain(here.iter().copied().filter(|x| x.parent.as_deref() == Some(t.id.as_str()))) {
                    let (k, to) = (key.clone(), x.id.clone());
                    rows = rows.child(item(&x.name, Some(folder_mark(x.parent.is_some())), f.folder.as_deref() == Some(x.id.as_str()), move |s: &mut State, tx| s.move_file(&k, Some(to.clone()), tx)));
                    n += 1.0;
                }
            }
            if f.folder.is_some() {
                let k = key.clone();
                rows = rows.child(item("Not in a folder", Some(icon(&icons::FH_PAGE, 15.0).color(muted())), false, move |s: &mut State, tx| s.move_file(&k, None, tx)));
                n += 1.0;
            }
            let list_h = (n * 30.0).min(260.0);
            let foot = if step == Step::New {
                // `width: 100%` of the box's 181 inside, then drawn seven to the left of it.
                row().pt(3.0).pl(13.0).pr(6.0).pb(4.0).child(fh_name_box(s, Some(MENU_IN - 19.0), 26.0, sans(14.0, 400.0), "Folder name"))
            } else {
                item("New folder…", None, false, |s: &mut State, tx| s.move_menu_new_folder(tx))
            };
            let body = col().child(menu_head("Move to", 6.0, 4.0)).child(col().h(list_h).clip_rounded(0.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(rows)).child(menu_gap()).child(foot);
            (body, home::MENU_WIDTH, 10.0 + 25.5 + list_h + 11.0 + if step == Step::New { 33.0 } else { 30.0 })
        }
        Step::Menu => {
            let (a, b, c) = (key.clone(), key.clone(), key.clone());
            let mut rows = col()
                .child(item("Open", None, false, move |s: &mut State, _| {
                    s.close_files_menus();
                    s.open_tile(&a)
                }))
                .child(item("Rename", None, false, move |s: &mut State, tx| s.rename_file_start(&b, tx)))
                .child(item("Change icon…", None, false, |s: &mut State, _| s.menu_step(Step::Icon)));
            let mut h = 10.0 + 90.0;
            if f.project.is_some() {
                rows = rows.child(item("Move to…", None, false, |s: &mut State, _| s.menu_step(Step::Move)));
                h += 30.0;
            }
            if f.trashable != Some(false) {
                rows = rows.child(menu_gap()).child(item("Move to the Trash", None, false, move |s: &mut State, tx| s.trash_file(&c, tx)));
                h += 41.0;
            }
            (rows, home::MENU_WIDTH, h)
        }
    }
}

/// New's menu (`NewMenu`, `.fp-menu`): 180 wide, 40 under New's top, its right edge on New's;
/// what can be made, or which project it goes in.
fn new_menu(s: &State, list: &List, asking: Option<Making>) -> Option<El> {
    let offers = new_offers(s, list)?;
    // Under the home's New 40 points down (`.fh-top .fp-new-box .fp-menu`); under the strip's,
    // on the page of every file, 38 (`.fp-new-box .fp-menu`).
    let (anchor, down) = if s.files.everything { (crate::files_page::STRIP_NEW, 38.0) } else { (NEW, 40.0) };
    let [x, y, w, _] = rune::memo::window_layout(anchor)?;
    let row_of = |words: &str, mark: Option<&'static rune::Icon>| {
        let mut r = row().key(format!("fp-menu-item-{words}")).w(168.0).h(28.0).px(8.0).gap(8.0).radius(6.0).items_center().hover(feel::wash_strong()).cursor(Cursor::Pointer).role("menuitem");
        if let Some(mark) = mark {
            r = r.child(icon(mark, 16.0).color(muted()));
        }
        r.child(pen(text(words).font(sans(13.0, 400.0)).color(ink()), 13.0)).clip_rounded(6.0)
    };
    let mut body = col().key("fp-menu").w(180.0).padding(5.0).radius(10.0).fill(surface()).border(1.0, rule()).border_layout().box_shadow(shadow(0.0, 1.0, 2.0, 0.0, rgba(0, 0, 0, 0.3), false)).role("menu");
    match asking {
        Some(what) => {
            body = body.child(row().px(8.0).pt(5.0).pb(4.0).child(pen(text(what.in_words()).font(sans(12.0, 400.0)).color(muted()), 12.0)));
            for p in &list.mine {
                let p2 = p.clone();
                body = body.child(row_of(p, None).on_press_tx(move |s: &mut State, tx| s.make_new(what, &p2, tx)));
            }
        }
        None => {
            for what in offers {
                let mark = match what {
                    Making::Document => &icons::FH_DOC,
                    Making::Table => &icons::FH_SHEET,
                    Making::Folder => &icons::FOLDER,
                    Making::Upload => &icons::FH_UPLOAD,
                };
                body = body.child(row_of(what.word(), Some(mark)).on_press_tx(move |s: &mut State, tx| s.choose_new(what, tx)));
            }
        }
    }
    Some(body.absolute(x + w - 180.0, y + down))
}

/// Whatever menu of Files stands open, over the page: a file's, a folder's, or New's. A press
/// anywhere else closes it (`Menu`'s and `useClosesOutside`'s `mousedown`).
pub fn layer(s: &State) -> Option<El> {
    if !matches!(s.view, crate::View::Files { .. }) || (s.files.everything && s.files.new_menu.is_none()) {
        return None;
    }
    let list = crate::files_page::list_of(s)?;
    let open = if let Some(asking) = s.files.new_menu {
        new_menu(s, &list, asking)?
    } else {
        let m = s.files.menu.as_ref()?;
        let (body, w, h) = menu_body(s, &list, m)?;
        let at = match (m.at, &m.on) {
            (Some(at), _) => at,
            (None, MenuOn::File(key)) => home::at_button(rune::memo::window_layout(&format!("fh-more-{key}"))?, false),
            (None, MenuOn::Folder(id)) => home::at_button(rune::memo::window_layout(&format!("fh-fmore-{id}"))?, true),
        };
        let (x, y) = home::menu_place(at, (w, h), s.size);
        col()
            .absolute(x, y)
            .w(w)
            .padding(5.0)
            .radius(10.0)
            .fill(surface())
            .box_shadow(shadow(0.0, 0.0, 0.0, 1.0, rule(), false))
            .box_shadow(shadow(0.0, 12.0, 32.0, 0.0, rgba(17, 25, 48, 0.16), false))
            .role("menu")
            .key("fh-menu")
            .child(body)
    };
    let (ww, wh) = s.size;
    let away = el().absolute(0.0, 0.0).size(ww, wh).on_drag(|s: &mut State, d: &mut Drag, _| {
        if d.phase == Phase::Down {
            s.close_files_menus();
        }
    });
    Some(el().absolute(0.0, 0.0).size(ww, wh).child(away).child(open))
}

#[cfg(test)]
mod tests {
    use rune::Runtime;
    use rune::edit::Key;
    use serde_json::{Value, json};

    use crate::feel::tests::{key, press, window_dressed};
    use crate::files_page::{MenuOn, Step, Wrote};
    use crate::rules::files_home::{FOLDER_ACT, NEW_FILE, NOT_IN_A_FOLDER, TRASH, UNTRASH};
    use crate::write::{Landed, Sent};
    use crate::{State, View};

    const NOW: f64 = 1_790_000_000_000.0;
    const MIN: f64 = 60_000.0;

    /// An invented list: Storefront with Pinned, Receipts (and Calls inside it) and an empty
    /// Brand kit, files in them and in none, and a Mobile project with one file in none.
    fn list() -> Value {
        let file = |name: &str, project: &str, folder: Option<&str>, ago: f64, kind: &str| {
            json!({ "key": format!("{}/{name}", project.to_lowercase()), "project": project, "name": name, "kind": kind, "versions": 1, "at": NOW - ago * MIN,
                "by": null, "agent": "", "open": 0, "approvedBy": [], "sentIn": [{ "kind": "thread", "product": "storefront", "id": "w-feel0", "title": "Invented thread number 0" }],
                "opened": null, "folder": folder, "trashable": true })
        };
        json!({
            "files": [
                file("rounding.md", "Storefront", Some("pin"), 9.0, "markdown"),
                file("receipt-plan.md", "Storefront", Some("rec"), 14.0, "markdown"),
                file("round-tax.ts", "Storefront", Some("calls"), 21.0, "code"),
                file("failed-orders.csv", "Storefront", None, 33.0, "table"),
                file("week-32.pdf", "Storefront", None, 60.0 * 30.0, "pdf"),
                file("cart.png", "Mobile", None, 340.0, "image"),
            ],
            "projects": ["Storefront", "Mobile"],
            "folders": [
                { "id": "pin", "project": "Storefront", "name": "Pinned", "parent": null, "at": 1.0 },
                { "id": "rec", "project": "Storefront", "name": "Receipts", "parent": null, "at": 1.0 },
                { "id": "calls", "project": "Storefront", "name": "Calls", "parent": "rec", "at": 1.0 },
                { "id": "brand", "project": "Storefront", "name": "Brand kit", "parent": null, "at": 1.0 },
            ],
            "mine": ["Storefront", "Mobile"],
        })
    }

    /// Files opened on `project`'s home (All files for None), with `more` in its fixture; what
    /// is written is taken as sent.
    fn files(name: &str, project: Option<&str>, more: Value) -> Runtime<State> {
        let project = project.map(str::to_string);
        let mut rt = window_dressed(name, &["--view", "files"], "/fixtures/storefront", "Done.", move |fixture| {
            fixture["filesPlace"] = list();
            if let Some(p) = project {
                fixture["filesProject"] = json!(p);
            }
            for (k, v) in more.as_object().cloned().unwrap_or_default() {
                fixture[k] = v;
            }
        });
        rt.state.pretend_sent = true;
        rt.frame(1.0 / 60.0);
        rt
    }

    fn press_on(rt: &mut Runtime<State>, key: &str) {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("{key} is not on the page"));
        press(rt, x, y);
    }

    /// The pointer onto `over`, which shows the ⋯ in it, then a press on the ⋯ `more`.
    fn press_shown(rt: &mut Runtime<State>, over: &str, more: &str) {
        let (x, y) = rt.find(over).unwrap_or_else(|| panic!("{over} is not on the page"));
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        press_on(rt, more);
    }

    /// A file's menu, as a hand opens it: onto its tile, then its ⋯.
    fn menu_of(rt: &mut Runtime<State>, file: &str) {
        press_shown(rt, &format!("fh-tile-{file}"), &format!("fh-more-{file}"));
    }

    fn says(rt: &Runtime<State>, words: &str) -> bool {
        rt.elements().iter().any(|e| e.text.as_deref() == Some(words))
    }

    fn lands(rt: &mut Runtime<State>, answer: Value) {
        let sent = rt.state.last_sent.clone().expect("something was sent");
        rt.update(|s, tx| s.landed(Landed { sent, answer: Ok(answer), snapshot: None }, tx));
        rt.frame(1.0 / 60.0);
    }

    fn toast(rt: &Runtime<State>) -> Option<String> {
        rt.state.toast.as_ref().map(|t| t.line.clone())
    }

    fn folder_of(rt: &Runtime<State>, key: &str) -> Option<String> {
        crate::files_page::list_of(&rt.state).and_then(|l| l.files.iter().find(|f| f.key == key).and_then(|f| f.folder.clone()))
    }

    #[test]
    fn all_files_opens_on_recent_and_a_tile_per_project_saying_what_it_kept() {
        let mut rt = files("fh-all", None, json!({}));
        for words in ["Files", "Recent", "Today", "Yesterday", "Projects", "Storefront", "Mobile", "3 files kept, 2 not in a folder", "Nothing kept yet, 1 not in a folder"] {
            assert!(says(&rt, words), "{words:?} is not on the page");
        }
        // Recent lists every project's files, the newest first.
        assert!(rt.find("fh-rrow-storefront/rounding.md").unwrap().1 < rt.find("fh-rrow-mobile/cart.png").unwrap().1);
        // No path back on All files, and no Every file.
        assert!(rt.find("fh-crumb-0").is_none() && rt.find("fh-every").is_none());
        press_on(&mut rt, "fh-tile-project-Storefront");
        assert_eq!(rt.state.view, View::Files { project: Some("Storefront".into()) });
    }

    #[test]
    fn a_projects_home_has_its_folders_in_sections_and_each_name_opens_its_page() {
        let mut rt = files("fh-project", Some("Storefront"), json!({}));
        for words in ["Storefront", "All files", "Pinned", "Receipts", "Brand kit is empty.", "Not in a folder", "Every file, with sizes and the Trash"] {
            assert!(says(&rt, words), "{words:?} is not on the page");
        }
        // Pinned before Receipts, whatever changed last.
        assert!(rt.find("fh-sec-pin").unwrap().1 < rt.find("fh-sec-rec").unwrap().1);
        // A subfolder's file is in its folder's section.
        assert!(rt.find("fh-tile-storefront/round-tax.ts").is_some());
        // A section's name opens its page, with the path back over it.
        let (x, y) = rt.find("fh-sec-rec").unwrap();
        let [sx, sy, _, _] = crate::feel::tests::rect_of(&rt, "fh-sec-rec").unwrap();
        let _ = (x, y);
        press(&mut rt, sx + 20.0, sy + 13.0);
        assert_eq!(rt.state.files.folder.as_deref(), Some("rec"));
        assert!(says(&rt, "Receipts") && rt.find("fh-crumb-1").is_some());
        // The project's crumb goes back to its home, All files' to All files.
        press_on(&mut rt, "fh-crumb-1");
        assert_eq!(rt.state.files.folder, None);
        press_on(&mut rt, "fh-crumb-0");
        assert_eq!(rt.state.view, View::Files { project: None });
    }

    #[test]
    fn the_files_in_no_folder_have_a_page_of_their_own_and_every_file_its_own() {
        let mut rt = files("fh-loose", Some("Storefront"), json!({ "filesFolder": NOT_IN_A_FOLDER }));
        assert!(says(&rt, "Not in a folder"));
        assert!(rt.find("fh-tile-storefront/week-32.pdf").is_some() && rt.find("fh-tile-storefront/rounding.md").is_none());
        press_on(&mut rt, "fh-crumb-1");
        assert_eq!(rt.state.files.folder, None);
        // Every file, at the foot of a home short enough to show it.
        let mut rt = files("fh-every", Some("Mobile"), json!({}));
        press_on(&mut rt, "fh-every");
        assert!(rt.state.files.everything, "Every file opens the page of every file");
        assert!(says(&rt, "All files") && says(&rt, "Recent") && says(&rt, "Name"), "its bar");
        // Its crumb of the project goes back to the home.
        press_on(&mut rt, "fp-crumb-1");
        assert!(!rt.state.files.everything);
        assert_eq!(rt.state.view, View::Files { project: Some("Mobile".into()) });
    }

    #[test]
    fn slash_puts_the_caret_in_the_search_and_escape_takes_its_words_back_then_leaves() {
        let mut rt = files("fh-search", None, json!({}));
        key(&mut rt, Key::Text("/".into()));
        assert!(rt.state.files.query.focused);
        key(&mut rt, Key::Text("receipt".into()));
        assert_eq!(rt.state.files.query.text, "receipt");
        assert!(says(&rt, "Matches") && rt.find("fh-tile-storefront/receipt-plan.md").is_some());
        assert!(rt.find("fh-tile-storefront/rounding.md").is_none());
        key(&mut rt, Key::Text("zz".into()));
        assert!(says(&rt, "No file matches"));
        key(&mut rt, Key::Escape);
        assert_eq!(rt.state.files.query.text, "");
        assert!(matches!(rt.state.view, View::Files { .. }), "the first Escape only takes the words back");
        // With nothing in it, Escape does nothing: Files stays, the caret in the box.
        key(&mut rt, Key::Escape);
        assert!(matches!(rt.state.view, View::Files { .. }) && rt.state.files.query.focused);
    }

    #[test]
    fn a_tiles_menu_moves_it_drawing_the_move_at_once_and_z_puts_it_back() {
        let mut rt = files("fh-move", Some("Storefront"), json!({}));
        menu_of(&mut rt, "storefront/receipt-plan.md");
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.on.clone()), Some(MenuOn::File("storefront/receipt-plan.md".into())));
        for words in ["Open", "Rename", "Change icon…", "Move to…", "Move to the Trash"] {
            assert!(says(&rt, words), "{words:?} is not on the menu");
        }
        press_on(&mut rt, "fh-item-Move to…");
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.step), Some(Step::Move));
        assert!(says(&rt, "Calls") && says(&rt, "Not in a folder"));
        press_on(&mut rt, "fh-item-Brand kit");
        assert!(rt.state.files.menu.is_none());
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/receipt-plan.md", "folder": "brand" }))));
        // Drawn there at once, before the app has said anything.
        assert_eq!(folder_of(&rt, "storefront/receipt-plan.md").as_deref(), Some("brand"));
        lands(&mut rt, json!({ "ok": true, "folder": null }));
        assert_eq!(rt.state.undos.last().map(|u| u.said.as_str()), Some("Moved receipt-plan.md back"));
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/receipt-plan.md", "folder": "rec" }))));
        lands(&mut rt, json!({ "ok": true, "folder": null }));
        assert_eq!(toast(&rt).as_deref(), Some("Moved receipt-plan.md back"));
        assert_eq!(folder_of(&rt, "storefront/receipt-plan.md").as_deref(), Some("rec"));
    }

    #[test]
    fn a_change_the_app_refuses_is_taken_back_off_and_said() {
        let mut rt = files("fh-refused", Some("Storefront"), json!({}));
        menu_of(&mut rt, "storefront/rounding.md");
        press_on(&mut rt, "fh-item-Move to…");
        press_on(&mut rt, "fh-item-Receipts");
        assert_eq!(folder_of(&rt, "storefront/rounding.md").as_deref(), Some("rec"));
        lands(&mut rt, json!({ "ok": false, "error": "That folder is gone." }));
        assert_eq!(folder_of(&rt, "storefront/rounding.md").as_deref(), Some("pin"));
        assert_eq!(toast(&rt).as_deref(), Some("That folder is gone."));
    }

    #[test]
    fn rename_makes_the_name_a_box_and_return_writes_a_new_name() {
        let mut rt = files("fh-rename", Some("Storefront"), json!({ "filesFolder": NOT_IN_A_FOLDER }));
        menu_of(&mut rt, "storefront/failed-orders.csv");
        press_on(&mut rt, "fh-item-Rename");
        assert!(rt.state.files.renaming.is_some());
        assert_eq!(rt.state.files.name_box.selected(), "failed-orders.csv", "all of the name is chosen");
        key(&mut rt, Key::Text("Orders that failed".into()));
        key(&mut rt, Key::Enter);
        assert!(rt.state.files.renaming.is_none());
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "name", "key": "storefront/failed-orders.csv", "name": "Orders that failed" }))));
        assert!(says(&rt, "Orders that failed"), "the new name is drawn at once");
        // Escape in the box changes nothing.
        let sent = rt.state.tried.len();
        menu_of(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fh-item-Rename");
        key(&mut rt, Key::Text("Nope".into()));
        key(&mut rt, Key::Escape);
        assert!(rt.state.files.renaming.is_none());
        assert_eq!(rt.state.tried.len(), sent);
        // Leaving the box keeps what was typed (`onBlur`).
        menu_of(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fh-item-Rename");
        key(&mut rt, Key::Text("Week 32".into()));
        press(&mut rt, 1200.0, 700.0);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.files.renaming.is_none());
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "name", "key": "storefront/week-32.pdf", "name": "Week 32" }))));
    }

    #[test]
    fn change_icon_picks_a_mark_or_automatic_and_escape_closes_the_menu_only() {
        let mut rt = files("fh-icon", Some("Storefront"), json!({}));
        menu_of(&mut rt, "storefront/rounding.md");
        press_on(&mut rt, "fh-item-Change icon…");
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.step), Some(Step::Icon));
        assert!(says(&rt, "Icon") && says(&rt, "Automatic"));
        key(&mut rt, Key::Escape);
        assert!(rt.state.files.menu.is_none());
        assert!(matches!(rt.state.view, View::Files { .. }), "Escape closed the menu and went no further");
        rt.update(|s, tx| s.pick_icon("storefront/rounding.md", "book", tx));
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "icon", "key": "storefront/rounding.md", "icon": "book" }))));
    }

    #[test]
    fn a_right_press_opens_a_tiles_menu_at_the_pointer() {
        let mut rt = files("fh-context", Some("Storefront"), json!({}));
        let (x, y) = rt.find("fh-tile-storefront/rounding.md").unwrap();
        rt.pointer_move(x, y);
        rt.secondary_down(x, y);
        rt.secondary_up(x, y);
        rt.frame(1.0 / 60.0);
        let m = rt.state.files.menu.clone().expect("the menu is open");
        assert_eq!(m.on, MenuOn::File("storefront/rounding.md".into()));
        assert_eq!(m.at, Some((x.floor() + 4.0, y.floor() + 4.0)));
        // A press anywhere else closes it.
        press(&mut rt, 1200.0, 700.0);
        assert!(rt.state.files.menu.is_none());
    }

    #[test]
    fn delete_folder_frees_its_files_says_where_they_went_and_z_puts_it_back() {
        let mut rt = files("fh-delete", Some("Storefront"), json!({ "filesFolder": "rec" }));
        press_shown(&mut rt, "fh-title", "fh-fmore-rec");
        press_on(&mut rt, "fh-item-Delete folder");
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "remove", "id": "rec" }))));
        // The page it was on goes back to the home, and its files are in no folder at once.
        assert_eq!(rt.state.files.folder, None);
        assert_eq!(folder_of(&rt, "storefront/receipt-plan.md"), None);
        assert_eq!(folder_of(&rt, "storefront/round-tax.ts"), None, "its folder inside goes with it");
        lands(&mut rt, json!({ "ok": true, "folder": null, "ids": ["rec", "calls"] }));
        assert_eq!(toast(&rt).as_deref(), Some("Deleted the folder Receipts. Its 2 files are now in Storefront."));
        assert!(rt.state.toast.as_ref().is_some_and(|t| t.undo));
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "restore", "ids": ["rec", "calls"] }))));
        lands(&mut rt, json!({ "ok": true, "folder": null }));
        assert_eq!(toast(&rt).as_deref(), Some("Folder back: Receipts"));
    }

    #[test]
    fn move_to_the_trash_says_what_it_gives_back_and_z_brings_it_back() {
        let mut rt = files("fh-trash", Some("Storefront"), json!({ "filesFolder": NOT_IN_A_FOLDER }));
        menu_of(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fh-item-Move to the Trash");
        assert_eq!(rt.state.tried.last(), Some(&(TRASH, json!({ "keys": ["storefront/week-32.pdf"], "older": false }))));
        assert_eq!(toast(&rt).as_deref(), Some("Moving week-32.pdf to the Trash…"));
        lands(&mut rt, json!({ "ok": true, "moved": 1, "bytes": 2048, "undo": { "token": "t-1" } }));
        assert_eq!(toast(&rt).as_deref(), Some("Moved week-32.pdf to the Trash. Empty the Trash to get the 2 KB back."));
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried.last(), Some(&(UNTRASH, json!({ "token": "t-1" }))));
        assert!(matches!(rt.state.last_sent, Some(Sent::Files(Wrote::Undone { trash: true, .. }))));
        lands(&mut rt, json!({ "ok": false, "error": "" }));
        assert_eq!(toast(&rt).as_deref(), Some("Those files are in the Trash now. Put them back from there."));
    }

    #[test]
    fn new_makes_a_document_here_or_asks_which_project_and_a_folder_is_named_in_place() {
        let mut rt = files("fh-new", Some("Storefront"), json!({}));
        press_on(&mut rt, crate::files_home::NEW);
        for words in ["Document", "Table", "Folder"] {
            assert!(says(&rt, words), "{words:?} is not on New's menu");
        }
        // Upload from computer would open the system's file window over the app's own window.
        assert!(!says(&rt, "Upload from computer"));
        press_on(&mut rt, "fp-menu-item-Document");
        assert_eq!(rt.state.tried.last(), Some(&(NEW_FILE, json!({ "project": "Storefront", "name": "", "kind": "document", "folder": null }))));
        // A folder: a box as a section, and Return makes it in the project.
        press_on(&mut rt, crate::files_home::NEW);
        press_on(&mut rt, "fp-menu-item-Folder");
        assert_eq!(rt.state.files.naming.as_deref(), Some("Storefront"));
        key(&mut rt, Key::Text("Launch".into()));
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "make", "project": "Storefront", "name": "Launch", "parent": null }))));
        lands(&mut rt, json!({ "ok": true, "folder": "launch-1" }));
        assert!(crate::files_page::list_of(&rt.state).unwrap().folders.iter().any(|f| f.id == "launch-1" && f.name == "Launch"), "the folder is drawn once the app gives its id");
        // On All files, with two projects of one's own, New asks which.
        let mut rt = files("fh-new-all", None, json!({}));
        press_on(&mut rt, crate::files_home::NEW);
        press_on(&mut rt, "fp-menu-item-Table");
        assert!(says(&rt, "New table in"));
        press_on(&mut rt, "fp-menu-item-Mobile");
        assert_eq!(rt.state.view, View::Files { project: Some("Mobile".into()) });
        assert_eq!(rt.state.tried.last(), Some(&(NEW_FILE, json!({ "project": "Mobile", "name": "", "kind": "table", "folder": null }))));
    }

    #[test]
    fn escape_with_no_menu_leaves_files() {
        let mut rt = files("fh-escape", Some("Storefront"), json!({}));
        key(&mut rt, Key::Escape);
        assert_eq!(rt.state.view, View::Home);
    }
}
