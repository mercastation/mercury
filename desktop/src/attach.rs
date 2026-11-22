//! The files in the box a thread is started in, before it is sent: dropped on the box or the
//! card, picked with the paper clip, or pasted into the words. They stand in a row between the
//! words and the bar, as the web window draws them (`AttachRow` in
//! `renderer/src/components/AttachRow.tsx`, the `.attach-*` rules in `styles.css`): a picture
//! small, as tall as the row, with a × in its corner, and any other file as a chip with its
//! name, its size and a ×, which removes it. A file that cannot go lights its chip, and a line
//! under the row says why. Which files can go, and how they are sent, is `rules/attach.rs`.
//!
//! A file on this Mac is never read to be sent: the engine is handed its path and copies it.
//! The box reads its size where it is attached, and a picture's shape off its header, which is
//! all the row needs to stand where it will from the first frame. The picture itself is read
//! and shrunk to what the row draws off the main thread, and lands in its place.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rune::file_dialog::FileDialogOptions;
use rune::{ClipboardContent, El, Pixels, SurfaceFit, TextStyle, Tx, el, row, text};

use crate::State;
use crate::rules::attach::{self as rules, Pending, Source};
use crate::theme::*;

/// A picture in the row is this tall, edge and all (`.attach-thumb img { height: 84px }`), and
/// no wider than this (`max-width: 220px`), cropped to fill what it has (`object-fit: cover`).
const THUMB_TALL: f32 = 84.0;
const THUMB_WIDEST: f32 = 220.0;
/// A chip is a line of words 12 points large, three points over and under it, and its edge.
const CHIP_TALL: f32 = 23.5;
/// Between two files, along the row and between its lines.
const GAP: f32 = 6.0;

/// A picture in the box: its own size in pixels, read off its header as it came in, and the
/// small copy the row draws, which a worker fills in.
#[derive(Clone, Debug)]
pub struct Thumb {
    pub natural: (u32, u32),
    pub pixels: Pixels,
}

/// One file in the box: what is sent, and what the row draws of it.
#[derive(Clone, Debug)]
pub struct Attached {
    pub file: Pending,
    /// A picture's, when this window can read it. One named as a picture that it cannot read is
    /// drawn as a chip, rather than as a broken picture.
    pub thumb: Option<Thumb>,
}

impl PartialEq for Attached {
    /// The same file, going the same way: its drawing is no part of what is sent.
    fn eq(&self, other: &Attached) -> bool {
        self.file == other.file
    }
}

/// A file that could not be read, in words for the box: the web window's for a file that is not
/// there (`whyWords('missing', mine)`), else the system's own, without its number.
fn why(e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::NotFound {
        return "the file was not found on this computer".to_string();
    }
    let said = e.to_string();
    said.split(" (os error").next().unwrap_or(&said).to_string()
}

impl Attached {
    /// A file on this Mac, as a drop or the picker hands it over: its size read now, which says
    /// whether it can go, and a picture's shape off its header. A folder goes as the files in
    /// it, which the engine reads, and has no size here (`filesIn` in
    /// `main/save-attachments.mts`).
    pub fn from_disk(path: &Path) -> Attached {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| "file".to_string());
        let (size, failed) = match std::fs::metadata(path) {
            Ok(meta) if meta.is_dir() => (None, None),
            Ok(meta) => (Some(meta.len()), None),
            Err(e) => (None, Some(rules::could_not_attach(&name, &why(&e)))),
        };
        let image = rules::is_image_name(&name);
        let drawn = image && failed.is_none() && size.is_some_and(|size| !rules::too_big_to_send(size));
        let thumb = drawn.then(|| crate::pictures::size_of(path)).flatten().map(|natural| Thumb { natural, pixels: Pixels::new() });
        Attached { file: Pending { name, source: Source::Disk(path.to_string_lossy().to_string()), image, size, failed }, thumb }
    }

    /// A picture pasted from the clipboard, as PNG, under the name it goes by.
    pub fn pasted(png: &[u8], natural: (u32, u32), name: String) -> Attached {
        let file = Pending { name, source: Source::Pasted(Arc::from(png)), image: true, size: Some(png.len() as u64), failed: None };
        Attached { file, thumb: Some(Thumb { natural, pixels: Pixels::new() }) }
    }
}

/// The files as they are sent.
pub fn pending(files: &[Attached]) -> Vec<Pending> {
    files.iter().map(|a| a.file.clone()).collect()
}

/// Whether Send may go for the files the box holds: none that cannot go (`filesReady`).
pub fn ready(files: &[Attached]) -> bool {
    rules::files_ready(&pending(files), false)
}

/// How wide a picture stands in the row, edge and all: as wide as its shape makes it at the
/// row's height, up to the widest.
fn thumb_wide(natural: (u32, u32)) -> f32 {
    let (w, h) = (natural.0.max(1) as f32, natural.1.max(1) as f32);
    ((THUMB_TALL - 2.0) * w / h).min(THUMB_WIDEST - 2.0) + 2.0
}

/// The size of the copy a picture is drawn from, in device pixels: enough to fill its place
/// inside the edge, cropped, and never larger than the picture itself.
fn copy_size(natural: (u32, u32), scale: f32) -> (u32, u32) {
    let (w, h) = (natural.0.max(1) as f32, natural.1.max(1) as f32);
    let fit = ((thumb_wide(natural) - 2.0) * scale / w).max((THUMB_TALL - 2.0) * scale / h);
    if fit < 0.9 { (((w * fit).round() as u32).max(1), ((h * fit).round() as u32).max(1)) } else { natural }
}

/// Reads a picture of the box and puts the copy the row draws in its place: on a worker, or on
/// the spot for a still picture, which cannot wait. `Pixels` wakes the window when it lands.
fn read_thumb(a: &Attached, scale: f32, still: bool) {
    let Some(thumb) = &a.thumb else { return };
    let (source, pixels, size) = (a.file.source.clone(), thumb.pixels.clone(), copy_size(thumb.natural, scale));
    let read = move || {
        let bytes = match &source {
            Source::Disk(path) => std::fs::read(path).ok().map(Arc::from),
            Source::Pasted(png) => Some(png.clone()),
        };
        let Some(picture) = bytes.and_then(|bytes| crate::pictures::decode(&bytes)) else { return };
        let natural = (picture.width(), picture.height());
        let copy = if size == natural { picture.into_rgba8() } else { picture.resize_exact(size.0, size.1, image::imageops::FilterType::CatmullRom).into_rgba8() };
        pixels.set(copy.width(), copy.height(), copy.into_raw());
    };
    if still { read() } else { std::thread::spawn(read); }
}

/// A picture in the row (`.attach-thumb`): drawn in its edge, opened large by a press, with the
/// × in its corner that takes it out (`.attach-remove`). Before its copy lands its place stands
/// empty in its edge, as an image not yet decoded does in the web window.
fn picture(at: usize, a: &Attached, thumb: &Thumb) -> El {
    let wide = thumb_wide(thumb.natural);
    let mut drawn = el().absolute(1.0, 1.0).size(wide - 2.0, THUMB_TALL - 2.0).radius(5.0).pixels_fit(&thumb.pixels, SurfaceFit::Fill);
    // The web window opens it large in a viewer of its own; here a file opens with the system,
    // as a picture in a conversation does. A pasted one is not a file yet, and opens nothing.
    if let Source::Disk(path) = &a.file.source {
        let path = path.clone();
        drawn = drawn.cursor(rune::Cursor::ZoomIn).on_background_press(move |_: &mut State| crate::files::open_with_system(&path));
    }
    let remove = row()
        .absolute(wide - 22.0, 4.0)
        .size(18.0, 18.0)
        .center()
        .radius(6.0)
        .fill(rgb(40, 38, 37))
        .border(1.0, rule())
        .border_layout()
        .child(text("×").font(TextStyle::new(12.0, 400.0).lh(12.0)).color(dim()))
        .cursor(rune::Cursor::Pointer)
        .on_background_press_tx(move |s: &mut State, tx| s.remove_attached(at, tx));
    el().size(wide, THUMB_TALL).radius(6.0).border(1.0, edge()).child(drawn).child(remove)
}

/// Any other file (`.attach-chip`): its name, its size quieter and in figures that line up,
/// and the ×; a press anywhere on it takes it out. One that cannot go is lit, with no size
/// (`.attach-too-big`), and the line under the row says why.
fn chip(at: usize, a: &Attached) -> El {
    let file = &a.file;
    let over = file.failed.is_some() || rules::too_big_to_send(file.size.unwrap_or(0));
    let words = |s: &str, font: TextStyle, c: rune::Color| pen(text(s).font(font).color(c), 12.0);
    let mut line = row().child(words(&file.name, sans(12.0, 400.0), if over { ink() } else { dim() }));
    if let Some(size) = file.size.filter(|size| *size > 0 && !over) {
        line = line.child(words(&format!(" {}", rules::size_words(size)), figures(12.0, 400.0), muted()));
    }
    line = line.child(words(" ×", sans(12.0, 400.0), if over { ink() } else { dim() }));
    row()
        .h(CHIP_TALL)
        .items_center()
        .px(10.0)
        .radius(6.0)
        .fill(rgb(47, 45, 44))
        .border(1.0, if over { muted() } else { edge() })
        .border_layout()
        .child(line)
        .cursor(rune::Cursor::Pointer)
        .on_background_press_tx(move |s: &mut State, tx| s.remove_attached(at, tx))
}

/// What the box holds of files, as the place it is drawn in lays it out: the row
/// (`.attach-row`), handed to `pad` for the room that place gives it, and under it the line
/// that says why one cannot go (`.compose-note.attach-problem`). Nothing for a box with none.
pub fn rows(s: &State, pad: impl Fn(El) -> El) -> Vec<El> {
    let files = &s.compose.files;
    if files.is_empty() {
        return Vec::new();
    }
    let drawn = files.iter().enumerate().map(|(at, a)| match &a.thumb {
        Some(thumb) => picture(at, a, thumb),
        None => chip(at, a),
    });
    let mut out = vec![pad(row().flex_wrap().items_start().gap(GAP).children(drawn))];
    if let Some(problem) = rules::attach_problem(&pending(files)) {
        out.push(el().pt(6.0).px(18.0).pb(2.0).child(pen(text(problem).font(sans(12.0, 400.0)).color(dim()).wrap(), 12.0)));
    }
    out
}

/// How tall the row and its line stand on one line, where the box has to know before it is
/// laid out: a picture's height, else a chip's, and the line's.
pub fn rows_tall(files: &[Attached]) -> f32 {
    if files.is_empty() {
        return 0.0;
    }
    let row = if files.iter().any(|a| a.thumb.is_some()) { THUMB_TALL } else { CHIP_TALL };
    row + if rules::attach_problem(&pending(files)).is_some() { 23.5 } else { 0.0 }
}

/// Whether a still picture is being taken, which reads every picture on the spot.
fn still() -> bool {
    rune::tooling::headless_run().is_some_and(|run| run.frames() == 0)
}

impl State {
    /// Files dropped on the box or the card, picked with the paper clip, or pasted from Finder,
    /// in the order they came (`addFiles` in `ThreadComposer.tsx`).
    pub fn attach_paths(&mut self, paths: &[PathBuf]) {
        for path in paths {
            let attached = Attached::from_disk(path);
            read_thumb(&attached, self.pictures.scale(), still());
            self.compose.files.push(attached);
        }
    }

    /// A picture pasted with no file behind it: a screenshot, as PNG.
    pub fn attach_pasted(&mut self, png: &[u8], natural: (u32, u32), name: String) {
        let attached = Attached::pasted(png, natural, name);
        read_thumb(&attached, self.pictures.scale(), still());
        self.compose.files.push(attached);
    }

    /// ⌘V in the words. Files copied in Finder attach as themselves; a picture attaches,
    /// named for the moment it came (`fromPaste`); anything else is the words' own to paste.
    /// A Finder copy also carries the file's icon as a picture, which is not what was meant.
    pub fn paste_into_the_box(&mut self, content: &ClipboardContent, now_ms: f64) -> bool {
        if !content.files().is_empty() {
            self.attach_paths(content.files());
            return true;
        }
        if let Some(picture) = content.image() {
            self.attach_pasted(picture.png(), picture.size(), rules::pasted_name(now_ms));
            return true;
        }
        false
    }

    /// The paper clip: the system's picker, over the window, and the file picked attaches
    /// (the box's hidden `<input type="file">`). Cancelling it changes nothing.
    pub fn pick_files(&mut self, tx: &mut Tx) {
        self.compose.menu = None;
        tx.open_file(FileDialogOptions::default(), |s: &mut State, picked, tx| match picked {
            Ok(Some(path)) => s.attach_paths(&[path]),
            Ok(None) => {}
            Err(e) => s.tell(&format!("The file picker did not open: {e}"), tx),
        });
    }

    /// The × on a file: it leaves the box, and the keyboard with it, as the web window's focus
    /// goes when the button it was on goes.
    pub fn remove_attached(&mut self, at: usize, tx: &mut Tx) {
        if at < self.compose.files.len() {
            self.compose.files.remove(at);
        }
        self.compose.menu = None;
        self.leave_the_box(tx);
    }

    /// What a picture taken with files in the box says of them: each file by its path
    /// (`attached`), attached as a drop attaches it, and each picture pasted (`pasted`) by the
    /// PNG it was and the name the page gave it.
    pub(crate) fn picture_attached(&mut self) {
        let list = |field: &str| self.fixture.get(field).and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        let text = |file: &serde_json::Value, field: &str| file.get(field).and_then(serde_json::Value::as_str).map(str::to_string);
        let (attached, pasted) = (list("attached"), list("pasted"));
        let paths: Vec<PathBuf> = attached.iter().filter_map(|file| text(file, "path")).map(PathBuf::from).collect();
        self.attach_paths(&paths);
        for file in pasted {
            let (Some(png), Some(name)) = (text(&file, "png"), text(&file, "name")) else { continue };
            let Ok(bytes) = std::fs::read(&png) else { continue };
            if let Some(natural) = crate::pictures::size_of(Path::new(&png)) {
                self.attach_pasted(&bytes, natural, name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_stands_as_wide_as_its_shape_at_the_rows_height_up_to_the_widest() {
        // The screenshot the scenes paste, and the small receipt: as the web window measured them.
        assert!((thumb_wide((2400, 1350)) - 147.777).abs() < 0.01);
        assert!((thumb_wide((400, 300)) - 111.333).abs() < 0.01);
        // A strip wider than the widest is cropped to it, and a tall one is narrow.
        assert_eq!(thumb_wide((3600, 64)), 220.0);
        assert!((thumb_wide((900, 1950)) - 39.846).abs() < 0.01);
    }

    #[test]
    fn a_pictures_copy_fills_its_place_in_device_pixels_and_is_never_larger_than_it() {
        assert_eq!(copy_size((2400, 1350), 2.0), (292, 164));
        // A strip cropped at the widest is copied tall enough to fill the row's height.
        assert_eq!(copy_size((3600, 64), 2.0), (3600, 64));
        assert_eq!(copy_size((3600, 400), 2.0), (1476, 164));
        // Smaller than its place: kept whole, and drawn larger.
        assert_eq!(copy_size((120, 80), 2.0), (120, 80));
    }

    #[test]
    fn a_file_is_attached_with_its_size_and_a_picture_with_its_shape() {
        let folder = std::env::temp_dir().join(format!("mercury-desktop-attach-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(folder.join("designs")).unwrap();
        let notes = folder.join("rounding.md");
        std::fs::write(&notes, "# Rounding\n").unwrap();
        let shot = folder.join("total-line.png");
        image::RgbImage::from_pixel(360, 64, image::Rgb([176, 108, 52])).save(&shot).unwrap();
        let note = Attached::from_disk(&notes);
        assert_eq!((note.file.name.as_str(), note.file.size, note.file.image, note.thumb.is_some()), ("rounding.md", Some(11), false, false));
        let picture = Attached::from_disk(&shot);
        assert_eq!((picture.file.image, picture.thumb.as_ref().map(|t| t.natural)), (true, Some((360, 64))));
        // A folder goes as the files in it and has no size of its own.
        assert_eq!(Attached::from_disk(&folder.join("designs")).file.size, None);
        // One that is gone by the time it is read says so, and holds Send.
        let gone = Attached::from_disk(&folder.join("gone.pdf"));
        assert_eq!(gone.file.failed.as_deref(), Some("gone.pdf could not be attached: the file was not found on this computer."));
        assert!(!ready(&[note.clone(), gone]));
        assert!(ready(&[note, picture]));
        // A picture that does not read is a chip, not a broken picture, and still goes.
        let broken = folder.join("broken.png");
        std::fs::write(&broken, "not a picture").unwrap();
        let broken = Attached::from_disk(&broken);
        assert!(broken.file.image && broken.thumb.is_none() && broken.file.failed.is_none());
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
