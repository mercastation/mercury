//! The files a message names, drawn as the web window draws them: a picture in place with its
//! name under it (`ArtifactImg` in `renderer/src/components/Focus.tsx`), several in a row
//! under their sentence (`.media-row`), and what is said when one is nowhere. Every size here
//! was read off the running renderer.
//!
//! The files are on this Mac, so they are read from it directly: a short path is looked for
//! in the folders the web window looks in (`rules/roots.rs`), under the project's own folder
//! and its code folder, both of which the snapshot names. `pictures.rs` does the reading.
//!
//! Pressing a file opens it with the system, through `open_with_system` at the foot of this
//! file. The web window opens it beside the conversation, which this window does not have yet.

use rune::{El, SurfaceFit, TextStyle, col, el, row, text};
use serde_json::Value;

use crate::State;
use crate::data::Item;
use crate::pictures::{Held, Listed, Look, Pictures, Want};
use crate::rules::paths::decode_uri;
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

impl<'a> Files<'a> {
    /// For one thread: its project's folders off the snapshot, and the folders its own words
    /// name (`place` in `Focus.tsx`, which reads the row's four fields).
    pub fn of(s: &'a State, item: &Item) -> Files<'a> {
        let product = s.snap.products.iter().find(|p| p.slug == item.product);
        let named = |field: &str| product.and_then(|p| p.rest.get(field)).and_then(Value::as_str).unwrap_or("").to_string();
        let (dir, repo) = (named("dir"), named("repoPath"));
        let rest = |field: &str| item.rest.get(field).and_then(Value::as_str).unwrap_or("");
        let texts = [item.result.as_deref().unwrap_or(""), rest("note"), item.body.as_deref().unwrap_or(""), rest("answer")];
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
/// the web window and holds still here).
fn frame(here: Option<&Held>, wide: f32, tall: f32, fit: Option<SurfaceFit>) -> El {
    let frame = el().size(wide, tall).radius(10.0).border(1.0, edge());
    match here {
        Some(held) => {
            let inside = el().absolute(1.0, 1.0).size((wide - 2.0).max(0.0), (tall - 2.0).max(0.0)).radius(9.0);
            let path = held.path.clone();
            frame.child(match fit {
                Some(fit) => inside.pixels_fit(&held.pixels, fit),
                None => inside.pixels(&held.pixels),
            })
            .on_press(move |_: &mut State| open_with_system(&path))
        }
        None => frame.fill(rgb(47, 45, 44)),
    }
}

/// Tells the store when a picture comes on screen and when it leaves, which is what decides
/// whose pixels are let go first when there are more than the budget holds.
fn watched(e: El, places: Vec<String>, want: Want) -> El {
    e.on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on))
}

/// A picture on a line of its own, as large as the column and the window let it be, with its
/// name under it. `room` is the width it has and `size` the reading size. Gives the block and
/// whether it stands on a line of text (a missing one is a mark in a line; a picture is not).
pub fn picture(f: &Files, src: &str, room: f32, size: f32) -> El {
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
            let line = kept(1.6 * size);
            return el().h(line).child(missing(src).mt(baseline(size) - 15.5));
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
    let after = 8.0 + if stands { under_baseline(size) } else { 0.0 } - 2.0;
    let block = col().child(frame(held, wide, tall, None).mt(8.0)).child(name_under(src, wide, f.now).mt(after).mb(8.0));
    watched(block, places, want)
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
        watched(col().w(wide).child(frame(held, drawn, ROW_TALL, Some(SurfaceFit::Fill))).child(name_under(src, wide, f.now).mt(6.0)), places, want)
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
    let mut name = row().h(line).items_center().gap(8.0).child(rune::icon(&crate::icons::MENTION_FOLDER, 15.0).color(muted())).child(text(titled).font(TextStyle::new(size, 500.0).lh(line)).color(ink()));
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
    let name = row().h(name_line).items_start().child(name.on_press(open_folder.clone()));
    let wide = ((room - 20.0) / 3.0).max(1.0);
    let tall = wide * 10.0 / 16.0;
    let shot = |inside: El| el().size(wide, tall).radius(6.0).fill(rgb(47, 45, 44)).border(1.0, edge()).child(inside.absolute(1.0, 1.0).size(wide - 2.0, tall - 2.0).radius(5.0));
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
            let (file, base) = (file.clone(), paths_base(file).to_string());
            let framed = shot(inside).on_press(move |_: &mut State| open_with_system(&file));
            watched(col().w(wide).child(framed).child(row().mt(4.0).child(row().min_w(0.0).child(text(base).font(label).color(muted()).truncate()))), places, want)
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
        block = block.child(row().mt(6.0).mb(under).child(text(format!("{more} more in the folder")).font(TextStyle::new(small, 400.0).lh(small_line)).color(dim()).on_press(open_folder)));
    }
    block
}

fn paths_base(file: &str) -> &str {
    file.rsplit('/').next().unwrap_or(file)
}

/// Opens what a link in a message points at: an address in the browser, a file in the app
/// that reads it. A short path is looked for where the web app's finder looks
/// (`main/artifact-path.mts`): the thread's own checkout, then the project's folders.
pub fn open(s: &mut State, href: &str) {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("mailto:") {
        return open_with_system(href);
    }
    let crate::View::Thread { id, product } = &s.view else { return };
    let Some(item) = s.snap.items.iter().find(|i| &i.id == id && &i.product == product) else { return };
    let files = Files::of(s, item);
    let found = files.places(href).into_iter().find(|place| std::path::Path::new(place).exists());
    match found {
        Some(place) => open_with_system(&place),
        None => eprintln!("{} could not be found.", crate::rules::mentions::mention_name(href)),
    }
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
    // A headless run draws pictures and opens nothing.
    if rune::tooling::headless_run().is_some() {
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
