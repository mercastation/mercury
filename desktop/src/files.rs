//! The files a message names, drawn as the web window draws them: a picture in place with its
//! name under it (`ArtifactImg` in `renderer/src/components/Focus.tsx`), several in a row
//! under their sentence (`.media-row`), and what is said when one is nowhere. Every size here
//! was read off the running renderer.
//!
//! The files are on this Mac, so they are read from it directly: a short path is looked for
//! in the folders the web window looks in (`rules/roots.rs`), under the project's own folder
//! and its code folder, both of which the snapshot names. `pictures.rs` does the reading.
//!
//! Pressing a file opens it where the web window does (`open`): big over the window, beside the
//! conversation (`pane.rs`), or with the system, through `open_with_system` at the foot of this
//! file. A right press on one opens the web window's menu for it (`link_menu`, `picture_menu`).

use rune::{ClipboardContent, ClipboardImage, Cursor, El, PointerEvent, PointerPress, PopupItem, PopupMenu, SurfaceFit, TextStyle, Tx, col, el, row, text};
use serde_json::Value;

use crate::State;
use crate::data::Item;
use crate::pictures::{Held, Listed, Look, Pictures, Want};
use crate::rules::paths::{decode_uri, normalize_uri};
use crate::rules::roots;
use crate::theme::*;

/// The widest a picture is drawn, however wide the column (`max-width: min(100%, 760px)`).
const WIDEST: f32 = 760.0;
/// What the window keeps for everything but the picture: its tallest is the window less this
/// (`max-height: calc(100vh - 320px)`).
const AROUND: f32 = 320.0;
/// One of a row is this tall, edge and all (`.media-row .inline-image`).
const ROW_TALL: f32 = 240.0;
const ROW_GAP: f32 = 14.0;

/// What a conversation's files are read against: its project's folders, the thread, where a
/// short path is looked for, and the pictures read so far.
pub struct Files<'a> {
    /// The project's own folder. Empty when the snapshot names none.
    pub dir: String,
    pub thread: String,
    /// The folder the thread's runs work in (`taskFolder`).
    pub folder: Option<String>,
    /// Every folder a short path may be in, the likeliest first (`pictureRoots`).
    pub roots: Vec<String>,
    /// Whether the conversation draws pictures of its own, which keeps a folder it names to
    /// one line (`drawsPictures`).
    pub draws_pictures: bool,
    /// The other threads, for a designs folder named after one.
    snap: &'a crate::data::Snapshot,
    pub pictures: &'a Pictures,
    /// The window's height, which caps a picture's.
    pub window_h: f32,
    pub now: f64,
}

/// What `Files` holds of its own (`Files::owned`).
#[derive(Clone, Debug, PartialEq)]
pub struct Owned {
    dir: String,
    thread: String,
    folder: Option<String>,
    roots: Vec<String>,
    draws_pictures: bool,
}

impl<'a> Files<'a> {
    /// For one thread: its project's folders off the snapshot, and the folders its own words
    /// name (`place` in `Focus.tsx`, which reads the row's four fields).
    pub fn of(s: &'a State, item: &Item) -> Files<'a> {
        let product = s.snap.products.iter().find(|p| p.slug == item.product);
        let named = |field: &str| product.and_then(|p| p.rest.get(field)).and_then(Value::as_str).unwrap_or("").to_string();
        let (dir, repo) = (named("dir"), named("repoPath"));
        let rest = |field: &str| item.rest.get(field).and_then(Value::as_str).unwrap_or("");
        let texts = [item.result.as_deref().unwrap_or(""), rest("note"), item.body.as_deref().unwrap_or(""), rest("answer")];
        // The page says whose pictures it is about to ask for before it asks for any: the
        // window's own turn may come after the first drawing of a conversation just opened,
        // and must not take that for another conversation's and let go of what was asked.
        s.pictures.keep(Some(&format!("{}/{}", item.product, item.id)));
        Files {
            folder: roots::task_folder(Some(&repo), Some(&item.id)),
            roots: roots::picture_roots(Some(&dir), Some(&repo), Some(&item.id), &texts),
            draws_pictures: roots::referenced_files(&texts, Some(&dir)).iter().any(|file| roots::is_image_path(file)),
            snap: &s.snap,
            dir,
            thread: item.id.clone(),
            pictures: &s.pictures,
            window_h: s.size.1,
            now: s.now,
        }
    }

    /// What it holds of its own, without what it borrows from the window: kept by a long
    /// conversation, whose rows are built one by one as they come near the screen (`thread.rs`).
    pub fn owned(&self) -> Owned {
        Owned { dir: self.dir.clone(), thread: self.thread.clone(), folder: self.folder.clone(), roots: self.roots.clone(), draws_pictures: self.draws_pictures }
    }

    /// The same files again, from what `owned` kept, against the window as it is now.
    pub fn again(s: &'a State, kept: &Owned) -> Files<'a> {
        Files {
            dir: kept.dir.clone(),
            thread: kept.thread.clone(),
            folder: kept.folder.clone(),
            roots: kept.roots.clone(),
            draws_pictures: kept.draws_pictures,
            snap: &s.snap,
            pictures: &s.pictures,
            window_h: s.size.1,
            now: s.now,
        }
    }

    /// What a document opened big reads its pictures against: its own folder, where a picture
    /// it names beside itself is (`fileBeside` in `ArtifactStage.tsx`).
    pub fn beside(s: &'a State, folder: &str) -> Files<'a> {
        Files {
            dir: folder.to_string(),
            thread: String::new(),
            folder: None,
            roots: vec![folder.to_string()],
            draws_pictures: true,
            snap: &s.snap,
            pictures: &s.pictures,
            window_h: s.size.1,
            now: s.now,
        }
    }

    /// An invented project at `dir` with nothing read yet, for the tests of what a message's
    /// words become.
    #[cfg(test)]
    pub fn invented(dir: &str, snap: &'a crate::data::Snapshot, pictures: &'a Pictures) -> Files<'a> {
        Files { dir: dir.to_string(), thread: "w-0a1b2c3d4e".to_string(), folder: None, roots: vec![dir.to_string()], draws_pictures: false, snap, pictures, window_h: 760.0, now: 0.0 }
    }

    /// The places a file may be, in the order they are tried, for an address as markdown
    /// hands it on. A whole path, bare or as a file url, is that file and nothing else; a short
    /// one is tried under every root. The web window asks for the name with its escapes still
    /// in it and leaves the decoding to the app's finder; here the file is on this disk, so the
    /// name is decoded first.
    pub fn places(&self, src: &str) -> Vec<String> {
        match roots::whole_picture_path(src) {
            Some(whole) => vec![decode_uri(&whole)],
            None => {
                let rel = decode_uri(src.strip_prefix("./").unwrap_or(src));
                self.roots.iter().map(|root| format!("{root}/{rel}")).collect()
            }
        }
    }
}

/// How far under its baseline a line of reading text ends. A picture drawn in a paragraph
/// stands on that line's baseline, so this much room follows it. The baseline is the font's
/// ascent (1.005 em, landed on a device pixel of a 2x screen) under half the line's spare
/// room, and Chromium puts it on the device pixel at or above that.
fn under_baseline(size: f32) -> f32 {
    kept(1.6 * size) - baseline(size)
}
fn baseline(size: f32) -> f32 {
    let snap = |v: f32| (v * 2.0).round() / 2.0;
    let (up, down) = (snap(1.005 * size), snap(0.295 * size));
    (((kept(1.6 * size) - up - down) / 2.0 + up) * 2.0).floor() / 2.0
}

/// The words under a picture: its name, small and quiet, cut short where the picture ends.
fn name_under(src: &str, wide: f32, now: f64) -> El {
    row().w(wide).h(16.0).child(row().min_w(0.0).child(text(roots::shown_name(src, now)).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(dim()).truncate()))
}

/// A worker's picture that is nowhere: its name in place of a broken mark (`.missing-artifact`).
fn missing(src: &str) -> El {
    let words = format!("missing image: {}", src.strip_prefix("./").unwrap_or(src));
    row().child(el().px(8.0).py(2.0).radius(6.0).border(1.0, edge()).border_dash(3.0, 2.0).border_layout().child(text(words).font(mono(12.5, 16.0)).color(muted())))
}

/// The framed box a picture is drawn in, `wide` by `tall` with its edge: the picture when it
/// is here, and the place it will take while it is on its way (`.arriving`, which shimmers in
/// the web window and holds still here). A picture's edge brightens under the pointer
/// (`.inline-image-open:hover .inline-image`), whose look `opens` gives what it is drawn in.
fn frame(here: Option<&Held>, wide: f32, tall: f32, fit: Option<SurfaceFit>) -> El {
    let frame = el().size(wide, tall).radius(10.0).border(1.0, edge());
    match here {
        Some(held) => {
            let inside = el().absolute(1.0, 1.0).size((wide - 2.0).max(0.0), (tall - 2.0).max(0.0)).radius(9.0);
            frame
                .child(match fit {
                    Some(fit) => inside.pixels_fit(&held.pixels, fit),
                    None => inside.pixels(&held.pixels),
                })
                .child(crate::feel::edge_lit(10.0, muted(), false))
        }
        None => frame.fill(raised()),
    }
}

/// A picture and its name under it, which open it big over the window, stepping through the
/// pictures of its part of the thread (`openBig` in `Focus.tsx`), or with ⌘ held with the
/// system: the magnifier for a pointer, as over the web window's
/// (`.inline-image-open { cursor: zoom-in }`), the picture's edge brighter under it, and its
/// path as written for a title. A right press opens its menu, the picture standing `drawn`
/// points down the block, from its top to its foot. A picture on its way opens nothing yet.
fn opens(block: El, here: Option<&Held>, src: &str, drawn: (f32, f32)) -> El {
    match here {
        Some(held) => {
            let (path, carried) = (held.path.clone(), roots::whole_picture_path(src).is_some());
            let menu_path = path.clone();
            let written = src.to_string();
            block
                .hover_group()
                .cursor(Cursor::ZoomIn)
                .help(src)
                .on_press_tx(move |s: &mut State, tx: &mut Tx| if tx.mods().cmd { open_with_system(&path) } else { s.open_big(&written, None) })
                .on_secondary_press_local(move |s: &mut State, at: PointerEvent, tx: &mut Tx| picture_menu(s, &menu_path, carried, at.y >= drawn.0 && at.y < drawn.1, at.window(), tx))
        }
        None => block,
    }
}

/// Tells the store when a picture comes on screen and when it leaves, which is what decides
/// whose pixels are let go first when there are more than the budget holds.
fn watched(e: El, places: Vec<String>, want: Want) -> El {
    e.on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on))
}

/// A picture on a line of its own, as large as the column and the window let it be, with its
/// name under it. `room` is the width it has, `size` the reading size and `line` the height of
/// the line of words it stands on: an agent's, or a person's, which is shorter. Gives the block and
/// whether it stands on a line of text (a missing one is a mark in a line; a picture is not).
pub fn picture(f: &Files, src: &str, room: f32, size: f32, line: f32) -> El {
    let (most_w, most_h) = (WIDEST.min(room), (f.window_h - AROUND).max(2.0));
    let places = f.places(src);
    let want = Want::Within(f.pictures.pixels(most_w - 2.0), f.pictures.pixels(most_h - 2.0));
    let look = f.pictures.look(places.clone(), want);
    // The picture's own size, inside its edge, as far as the column and the window let it.
    let boxed = |natural: (u32, u32)| {
        let (w, h) = (natural.0.max(1) as f32, natural.1.max(1) as f32);
        let fit = ((most_w - 2.0) / w).min((most_h - 2.0) / h).min(1.0);
        (w * fit + 2.0, h * fit + 2.0)
    };
    let (held, wide, tall, stands) = match &look {
        Look::Missing => {
            // A mark in a line of reading text, set on that line's baseline.
            return el().h(line).child(missing(src).mt(baseline_in(size, line) - 15.5));
        }
        Look::Here(held) => {
            let (wide, tall) = boxed(held.natural);
            (Some(held), wide, tall, true)
        }
        // Read before: its exact place, so nothing moves when it lands.
        Look::Arriving(Some(natural)) => {
            let (wide, tall) = boxed(*natural);
            (None, wide, tall, true)
        }
        // Never read: the place the web window holds, four to three across the column.
        Look::Arriving(None) => (None, most_w, (most_w * 0.75).min(most_h), false),
    };
    // A picture stands on the line's baseline, and the rest of the line follows it. The
    // place held for one not yet seen is a block of its own, with no line around it.
    let after = 8.0 + if stands { line - baseline_in(size, line) } else { 0.0 } - 2.0;
    let block = col().child(frame(held, wide, tall, None).mt(8.0)).child(name_under(src, wide, f.now).mt(after).mb(8.0));
    // As wide as the picture and not the column, as the button around it is
    // (`.inline-image-open`): the magnifier and the look under the pointer stop where it does.
    row().child(watched(opens(block, held, src, (8.0, 8.0 + tall)), places, want))
}

/// A picture in a document opened big (`.doc-prose img`): at the text's width at most, and no
/// taller than 560 points or seven tenths of the window, in the middle, its corners rounded
/// inside a hairline. A web picture, or one that is nowhere, is its words instead, as the
/// web window's reader draws a picture it cannot find.
pub fn doc_picture(f: &Files, src: &str, room: f32) -> El {
    let most = (room, 560.0_f32.min(0.7 * f.window_h));
    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("data:") {
        return row().w(room).justify_center().child(text(src).font(TextStyle::new(16.0, 400.0).italic()).color(dim()));
    }
    let places = f.places(src);
    let want = Want::Within(f.pictures.pixels(most.0 - 2.0), f.pictures.pixels(most.1 - 2.0));
    let (held, natural) = match f.pictures.look(places.clone(), want) {
        Look::Missing => return row().w(room).justify_center().child(text(src).font(TextStyle::new(16.0, 400.0).italic()).color(dim())),
        Look::Here(held) => {
            let natural = held.natural;
            (Some(held), Some(natural))
        }
        Look::Arriving(natural) => (None, natural),
    };
    let (w, h) = natural.map_or((most.0, most.0 * 0.75), |(w, h)| (w.max(1) as f32, h.max(1) as f32));
    let fit = ((most.0 - 2.0) / w).min((most.1 - 2.0) / h).min(1.0);
    let (wide, tall) = (w * fit + 2.0, h * fit + 2.0);
    row().w(room).justify_center().child(watched(frame(held.as_ref(), wide, tall, None), places, want))
}

/// One thing a row holds.
pub enum Shown {
    /// A picture, by its address.
    Picture(String),
}

/// What a sentence names, together under it at one height (`.media-row`): each picture as
/// tall as the row and as wide as its shape makes it, wrapping onto more lines.
pub fn media_row(f: &Files, shown: &[Shown], room: f32) -> El {
    let want = Want::Tall(f.pictures.pixels(ROW_TALL - 2.0));
    row().w(room).flex_wrap().items_start().gap(ROW_GAP).children(shown.iter().map(|one| {
        let Shown::Picture(src) = one;
        let places = f.places(src);
        let look = f.pictures.look(places.clone(), want);
        let shaped = |natural: (u32, u32)| ((ROW_TALL - 2.0) * natural.0.max(1) as f32 / natural.1.max(1) as f32 + 2.0).min(room);
        let (held, wide) = match &look {
            Look::Missing => return missing(src),
            Look::Here(held) => (Some(held), shaped(held.natural)),
            Look::Arriving(Some(natural)) => (None, shaped(*natural)),
            // Not seen yet: the whole width of the row, which is what the web window holds.
            Look::Arriving(None) => (None, room),
        };
        // Its place in the row is as wide as the row at most; the picture itself stops at
        // the widest any picture is drawn, and is cropped to fill what it has.
        let drawn = wide.min(WIDEST);
        watched(opens(col().w(wide).child(frame(held, drawn, ROW_TALL, Some(SurfaceFit::Fill))).child(name_under(src, wide, f.now).mt(6.0)), held, src, (0.0, ROW_TALL)), places, want)
    }))
}

/// How many of a folder's pictures are drawn (`FOLDER_SHOWS`), three across.
const FOLDER_SHOWS: usize = 6;

/// A text's baseline under the top of its own line, as `baseline` is for the reading size.
fn baseline_in(size: f32, line: f32) -> f32 {
    let snap = |v: f32| (v * 2.0).round() / 2.0;
    let (up, down) = (snap(1.005 * size), snap(0.295 * size));
    (((line - up - down) / 2.0 + up) * 2.0).floor() / 2.0
}

/// A folder a worker names on a line of its own (`FolderPreview` in `Focus.tsx`): a folder
/// mark and the name the folder goes by, which opens it, then how many pictures it holds, its
/// six newest three across, and a line for the rest. In a conversation that draws pictures of
/// its own the folder stays its name, so the same pictures are not drawn twice.
pub fn folder(f: &Files, href: &str, room: f32, size: f32) -> El {
    let line = kept(1.6 * size);
    let short = decode_uri(href.trim_end_matches('/'));
    let places = f.places(&short);
    let listed = if f.draws_pictures { None } else { Some(f.pictures.folder(places.clone())) };
    let titled = crate::rules::mentions::folder_title(href, Some(&f.thread), |id| f.snap.items.iter().find(|i| i.id == id).map(crate::rules::title::row_title));
    let pictures = match &listed {
        Some(Listed::Here(listing)) if !listing.pictures.is_empty() => Some(listing),
        _ => None,
    };
    // Its name: the mark, what the folder goes by, and how many pictures once that is known.
    // Under the pointer the mark is the text's own colour and a faint line comes in under the
    // name, four points under its baseline (`.folder-preview-name:hover`).
    let under = (((line - 1.3 * size) / 2.0 + 1.005 * size + 4.0) * 2.0).round() / 2.0;
    let titled = el().child(text(titled).font(TextStyle::new(size, 500.0).lh(line)).color(ink())).child(el().pin(Some(0.0), Some(under), Some(0.0), None).h(1.0).fill(muted()).group_opacity(0.0, 1.0));
    let mut name = row().h(line).items_center().gap(8.0).hover_group().cursor(Cursor::Pointer).child(rune::icon(&crate::icons::MENTION_FOLDER, 15.0).color(muted()).group_color(ink())).child(titled);
    if let Some(listing) = pictures {
        let count = format!("{} picture{}", listing.total, if listing.total == 1 { "" } else { "s" });
        name = name.child(text(count).font(TextStyle::new(0.9 * size, 400.0).lh(kept(1.6 * 0.9 * size))).color(muted()).ml(2.0));
    }
    let open_folder = {
        let places = places.clone();
        move |_: &mut State| {
            if let Some(found) = places.iter().find(|p| std::path::Path::new(p).is_dir()) {
                open_with_system(found);
            }
        }
    };
    // The name stands in a line of reading text with its mark's foot on the baseline, so
    // the line reaches a little under the name.
    let name_line = (line - 15.0) / 2.0 + 15.0 + under_baseline(size);
    // Its path waits in its title (`title={`${short}/`}`).
    let name = row().h(name_line).items_start().child(name.help(format!("{short}/")).on_press(open_folder.clone()));
    let wide = ((room - 20.0) / 3.0).max(1.0);
    let tall = wide * 10.0 / 16.0;
    // A picture's edge brightens under the pointer, and it opens larger (`.folder-preview-shot`).
    let shot = |inside: El| el().size(wide, tall).radius(6.0).fill(raised()).border(1.0, edge()).child(inside.absolute(1.0, 1.0).size(wide - 2.0, tall - 2.0).radius(5.0)).child(crate::feel::edge_lit(6.0, muted(), false));
    let Some(listing) = pictures else {
        // Still being read: the places its first three will take. Otherwise only its name.
        if !matches!(listed, Some(Listed::Asking)) {
            return name;
        }
        return col().child(name).child(row().mt(8.0).gap(10.0).children((0..3).map(|_| shot(el()))));
    };
    let want = Want::Cover(f.pictures.pixels(wide - 2.0), f.pictures.pixels(tall - 2.0));
    let label = TextStyle::new(0.82 * size, 400.0).lh(kept(1.6 * 0.82 * size));
    let shots: Vec<El> = listing
        .pictures
        .iter()
        .take(FOLDER_SHOWS)
        .map(|file| {
            let places = vec![file.clone()];
            let inside = match f.pictures.look(places.clone(), want) {
                // A cover keeps the picture's top, as a screenshot's top is what tells it apart.
                Look::Here(held) => el().pixels_fit(&held.pixels, SurfaceFit::Fill).surface_anchor(rune::Align::Center, rune::Align::Start),
                _ => el(),
            };
            let (file, base, folder) = (file.clone(), paths_base(file).to_string(), listing.pictures.clone());
            // The picture and its name are one button, which opens it, its title the name
            // (`.folder-preview-shot`).
            let named = col().w(wide).child(shot(inside)).child(row().mt(4.0).child(row().min_w(0.0).child(text(base.as_str()).font(label).color(muted()).truncate())));
            // A folder's pictures are listed by their whole paths, which its button carries for
            // a right press (`data-copy-file={file}`).
            let menu_file = file.clone();
            let named = named.on_secondary_press_local(move |s: &mut State, at: PointerEvent, tx: &mut Tx| picture_menu(s, &menu_file, true, at.y < tall, at.window(), tx));
            // Opened big, it steps through the folder's pictures, as the turn a press is in.
            let opened = move |s: &mut State, tx: &mut Tx| if tx.mods().cmd { open_with_system(&file) } else { s.open_big(&file, Some(folder.clone())) };
            watched(named.hover_group().cursor(Cursor::ZoomIn).help(base).on_press_tx(opened), places, want)
        })
        .collect();
    // Three across, row under row, ten points apart both ways.
    let mut grid = col().mt(8.0).gap(10.0);
    let mut shots = shots.into_iter().peekable();
    while shots.peek().is_some() {
        grid = grid.child(row().gap(10.0).children(shots.by_ref().take(3)));
    }
    let mut block = col().child(name).child(grid);
    let more = listing.total.saturating_sub(listing.pictures.len().min(FOLDER_SHOWS));
    if more > 0 {
        // A smaller line of words in a line of reading text: the reading line's foot reaches
        // under it by what its own does not.
        let small = 0.86 * size;
        let small_line = kept(1.6 * small);
        let under = (under_baseline(size) - (small_line - baseline_in(small, small_line))).max(0.0);
        let more = text(format!("{more} more in the folder")).font(TextStyle::new(small, 400.0).lh(small_line)).color(dim()).group_color(ink());
        block = block.child(row().mt(6.0).mb(under).child(row().hover_group().cursor(Cursor::Pointer).child(more).on_press(open_folder)));
    }
    block
}

fn paths_base(file: &str) -> &str {
    file.rsplit('/').next().unwrap_or(file)
}

/// Opens what a link in a message points at (`openHref` in `Focus.tsx`): an address in the
/// browser; a picture, a film, a PDF, a page or a deck big over the window (`viewer.rs`); a
/// document, a table or a script in the pane beside the conversation (`pane.rs`); with ⌘ held
/// (`outside`), as in a browser, any of them out; anything else in the app that reads it.
pub fn open(s: &mut State, href: &str, outside: bool) {
    crate::pane::open_link(s, href, outside)
}

/// Opens a file a message names in the app that reads it. A short path is looked for where the
/// web app's finder looks (`main/artifact-path.mts`): the thread's own checkout, then the
/// project's folders.
pub fn open_found(s: &mut State, href: &str) {
    let crate::View::Thread { id, product } = &s.view else { return };
    let Some(item) = s.snap.items.iter().find(|i| &i.id == id && &i.product == product) else { return };
    let files = Files::of(s, item);
    let found = files.places(href).into_iter().find(|place| std::path::Path::new(place).exists());
    match found {
        Some(place) => open_with_system(&place),
        None => eprintln!("{} could not be found.", crate::rules::mentions::mention_name(href)),
    }
}

/// The kinds of file the web window serves from the folders it owns (`PICTURES` and `MEDIA` in
/// `main/img-scheme.mts`), and so the only kinds its right press copies as the file.
const SERVED: [&str; 20] = [
    "png", "jpg", "jpeg", "gif", "webp", "avif", "bmp", "ico", "svg", "pdf", "pptx", "mp3", "wav", "m4a", "aac", "ogg", "mp4", "mov", "m4v", "webm",
];

/// The folders the web window serves files from (`imageRoots` in `main/main.mts`): the
/// account's, which holds every project's own folder, and every project's code folder. This
/// window is not told the account's folder, so each project's own folder stands in for it.
fn owned(s: &State) -> Vec<String> {
    let named = |p: &crate::data::Product, field: &str| p.rest.get(field).and_then(Value::as_str).unwrap_or("").to_string();
    s.snap.products.iter().flat_map(|p| [named(p, "dir"), named(p, "repoPath")]).filter(|root| !root.is_empty()).collect()
}

/// Whether a right press may copy this as the file (`servable` in `main/img-scheme.mts`): a
/// whole path to a picture or a film inside a folder the app owns. A path that steps up out of
/// its folder is not inside it, as `inside` resolves both before it compares them.
fn servable(file: &str, roots: &[String]) -> bool {
    let path = std::path::Path::new(file);
    let kind = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
    path.is_absolute()
        && kind.is_some_and(|k| SERVED.contains(&k.as_str()))
        && !path.components().any(|c| c == std::path::Component::ParentDir)
        && roots.iter().any(|root| path.starts_with(root))
}

/// The file itself on the clipboard, as a file and nothing else, so Finder or Mail pastes the
/// file rather than its path (`copyFile`). Something that is no longer a file copies nothing.
fn copy_file(file: &str, tx: &mut Tx) {
    if std::path::Path::new(file).is_file() {
        tx.copy_rich(ClipboardContent::default().with_files([std::path::PathBuf::from(file)]));
    }
}

/// A picture on the clipboard as a picture, read whole from its file as `copyPicture` reads it
/// (`nativeImage.createFromPath`), not the smaller copy this window draws.
fn copy_picture(file: &str, tx: &mut Tx) {
    let Ok(bytes) = std::fs::read(file) else { return };
    let png = if bytes.starts_with(b"\x89PNG") {
        ClipboardImage::from_png(bytes)
    } else {
        let mut png = Vec::new();
        image::load_from_memory(&bytes).ok().and_then(|picture| picture.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).ok()).and_then(|_| ClipboardImage::from_png(png))
    };
    if let Some(png) = png {
        tx.copy_rich(ClipboardContent::default().with_image(png));
    }
}

/// An address as Chromium hands a link's on, parsed: what an address may not hold escaped,
/// and a site named bare ending in its slash.
fn parsed(href: &str) -> String {
    match href.split_once("://") {
        Some(("http" | "https", rest)) if !rest.contains(['/', '?', '#']) => format!("{href}/"),
        _ => href.to_string(),
    }
}

/// What a link in a message points at, as the web window's link holds it (`ArtifactLink`):
/// the file it finds, and its address. An address stays itself; a file is a file url to where
/// it was found, or, while it is found nowhere, to where it would be in the project's own
/// folder (`resolveSrc`).
fn link_target(s: &State, href: &str) -> (Option<String>, String) {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("mailto:") {
        return (None, parsed(href));
    }
    let crate::View::Thread { id, product } = &s.view else { return (None, href.to_string()) };
    let Some(item) = s.snap.items.iter().find(|i| &i.id == id && &i.product == product) else { return (None, href.to_string()) };
    let files = Files::of(s, item);
    let found = files.places(href).into_iter().find(|place| std::path::Path::new(place).exists());
    let address = match (&found, roots::whole_picture_path(href)) {
        (Some(place), _) => format!("file://{}", normalize_uri(place)),
        (None, Some(whole)) => format!("file://{}", normalize_uri(&whole)),
        (None, None) if !files.dir.is_empty() => format!("file://{}", normalize_uri(&format!("{}/{}", files.dir, href.strip_prefix("./").unwrap_or(href)))),
        (None, None) => href.to_string(),
    };
    (found, address)
}

/// A right press on a link in a message opens the web window's menu for it, which the system
/// draws as it draws that one (the `context-menu` handler in `main/main.mts`): a picture or a
/// film the link finds where the app keeps them is copied as the file, first, as that is what
/// copying it means; then the link opens, as a press opens it, or gives its address.
pub fn link_menu(s: &mut State, href: &str, at: PointerPress, tx: &mut Tx) {
    let (found, address) = link_target(s, href);
    let file = found.filter(|file| servable(file, &owned(s)));
    let mut menu = PopupMenu::new();
    if file.is_some() {
        menu = menu.item(PopupItem::new("copy-file", "Copy File")).separator();
    }
    menu = menu.item(PopupItem::new("open-link", "Open Link")).item(PopupItem::new("copy-link", "Copy Link Address"));
    let href = href.to_string();
    tx.popup_menu(menu, at, move |s: &mut State, picked: Option<&str>, tx: &mut Tx| match (picked, &file) {
        (Some("copy-file"), Some(file)) => copy_file(file, tx),
        (Some("open-link"), _) => open(s, &href, true),
        (Some("copy-link"), _) => tx.copy(address),
        _ => {}
    });
}

/// A right press on a picture opens the web window's menu for one: the file, when the picture
/// carries its whole path for the press to read (`data-copy-file`), which a picture a message
/// names by a short path does not; then, over the drawing itself and not the name under it,
/// the picture.
fn picture_menu(s: &State, file: &str, carried: bool, on_drawing: bool, at: PointerPress, tx: &mut Tx) {
    let copies_file = carried && servable(file, &owned(s));
    if !copies_file && !on_drawing {
        return;
    }
    let mut menu = PopupMenu::new();
    if copies_file {
        menu = menu.item(PopupItem::new("copy-file", "Copy File"));
    }
    if on_drawing {
        if copies_file {
            menu = menu.separator();
        }
        menu = menu.item(PopupItem::new("copy-image", "Copy Image"));
    }
    let file = file.to_string();
    tx.popup_menu(menu, at, move |_: &mut State, picked: Option<&str>, tx: &mut Tx| match picked {
        Some("copy-file") => copy_file(&file, tx),
        Some("copy-image") => copy_picture(&file, tx),
        _ => {}
    });
}

/// The kinds of file macOS runs or launches on a double-click (`RUNS` in `main/opens-a-program.mts`).
const RUNS: [&str; 23] = [
    "command", "tool", "terminal", "term", "app", "workflow", "action", "prefpane", "saver", "qlgenerator", "kext", "bundle", "plugin", "fileloc", "webloc", "inetloc", "scpt", "scptd", "applescript", "pkg", "mpkg",
    "jar", "osax",
];

/// Whether opening this would run it (`opensAProgram`): a kind macOS launches, a link to
/// one, or any file with an executable bit. A file a worker wrote would then run as the
/// person, outside the worker's own bounds.
fn opens_a_program(path: &std::path::Path) -> bool {
    let Ok(real) = std::fs::canonicalize(path) else { return false };
    let Ok(meta) = std::fs::metadata(&real) else { return false };
    let runs = |p: &std::path::Path| p.extension().and_then(|e| e.to_str()).is_some_and(|e| RUNS.contains(&e.to_ascii_lowercase().as_str()));
    if runs(&real) || runs(path) {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.is_file() && meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        false
    }
}

/// Hands a file, a folder or an address to the system, which opens it in the app that owns
/// it. One that would run instead is shown in its folder, as the web app shows it
/// (`openForThePerson`). This is the one place to change when files open inside this window.
pub fn open_with_system(target: &str) {
    // A headless run opens nothing, and says what a press in its script would have opened; nor
    // does a test, whose presses must never reach the person's apps.
    if cfg!(test) || rune::tooling::headless_run().is_some() {
        eprintln!("a press opens {target}");
        return;
    }
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    let mut command = std::process::Command::new(opener);
    let address = target.starts_with("http://") || target.starts_with("https://") || target.starts_with("mailto:");
    if !address && opens_a_program(std::path::Path::new(target)) {
        if !cfg!(target_os = "macos") {
            return;
        }
        command.arg("-R");
    }
    let _ = command.arg(target).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_stands_on_its_lines_baseline() {
        // At the reading size of a 1280 window, the line ends 7.09 points under the picture.
        assert!((under_baseline(16.0) - 7.09375).abs() < 0.001);
        assert!((baseline(16.0) - 18.5).abs() < 0.001);
        // And at a 1600 window's, 7.78: the baseline is 19 points down, not 19.5.
        assert!((baseline(16.74) - 19.0).abs() < 0.001);
        assert!((under_baseline(16.74) - 7.78).abs() < 0.01);
    }

    // tests/a-click-on-a-workers-link-never-runs-a-program.test.mts
    #[test]
    #[cfg(unix)]
    fn a_click_never_runs_a_program() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("mercury-desktop-launch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = |name: &str, mode: u32| {
            let path = dir.join(name);
            std::fs::write(&path, "#!/bin/sh\necho hi\n").unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            path
        };
        let folder = |name: &str| {
            let path = dir.join(name);
            std::fs::create_dir_all(path.join("Contents")).unwrap();
            path
        };
        // Any of the kinds macOS runs or launches on a double-click.
        for name in ["notes.command", "notes.tool", "notes.terminal", "x.term", "X.TERM", "x.fileloc", "x.webloc", "x.inetloc", "x.scpt", "x.applescript", "x.pkg", "x.jar", "NOTES.COMMAND"] {
            assert!(opens_a_program(&file(name, 0o644)), "{name}");
        }
        for name in ["Tool.app", "Flow.workflow", "Thing.prefPane"] {
            assert!(opens_a_program(&folder(name)), "{name}");
        }
        // Any file with an executable bit, whatever its name, and a link to a program.
        assert!(opens_a_program(&file("runme", 0o755)));
        assert!(opens_a_program(&file("report.pdf", 0o744)));
        let link = dir.join("harmless.pdf");
        std::os::unix::fs::symlink(file("real.command", 0o644), &link).unwrap();
        assert!(opens_a_program(&link));
        // Not a picture, a document or an ordinary folder, nor a program that is not there.
        for name in ["shot.png", "report-2.pdf", "notes.md", "deck.key", "leads.csv", "notes.sh"] {
            assert!(!opens_a_program(&file(name, 0o644)), "{name}");
        }
        assert!(!opens_a_program(&folder("designs")));
        assert!(!opens_a_program(&dir.join("missing.command")));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
