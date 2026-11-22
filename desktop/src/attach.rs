//! The files in a box, before it is sent: in the box a thread is started in, and in a
//! conversation's reply box (`Holder`). Dropped on the box or the card, picked with the paper
//! clip, or pasted into the words. They stand in a row between the words and the bar, as the
//! web window draws them in both (`AttachRow` in `renderer/src/components/AttachRow.tsx`, the
//! `.attach-*` rules in `styles.css`): a picture small, as tall as the row, with a × in its
//! corner, and any other file as a chip with its name, its size and a ×, which removes it. A
//! file that cannot go lights its chip, and a line under the row says why. Which files can go,
//! and how they are sent, is `rules/attach.rs`.
//!
//! The reply box keeps its files with its words: leaving a conversation keeps them for the next
//! visit, and Send saves them into the project before the reply goes, as the web window's
//! `DockComposer` does (`ReplyFiles`).
//!
//! A file on this Mac is never read to be sent: the engine is handed its path and copies it.
//! The box reads its size where it is attached, and a picture's shape off its header, which is
//! all the row needs to stand where it will from the first frame. The picture itself is read
//! and shrunk to what the row draws off the main thread, and lands in its place.
//!
//! The one file that is on its way in for a while is a TIFF or a HEIC, a photo from a phone:
//! as in the web window, the engine turns it into a PNG or a JPEG the moment it is attached
//! (`zero:stage-attachment`, `main/stage-attachment.mts`), so it goes as a picture the window,
//! the agent and the conversation can all draw. Until the engine answers it is a chip with its
//! name and Send waits for it; one the engine cannot turn goes as the file it is.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::Arc;

use rune::file_dialog::FileDialogOptions;
use rune::{ClipboardContent, El, Pixels, SurfaceFit, TextStyle, Tx, Waker, el, row, text};
use serde_json::{Value, json};

use crate::State;
use crate::rules::attach::{self as rules, Pending, Source};
use crate::source::{self, Source as Reading};
use crate::theme::*;

/// Which box a file goes in: the one a thread is started in (on Home or the New thread card),
/// or the open conversation's reply box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    Compose,
    Reply,
}

/// The files of the reply boxes: the open conversation's, those left in the boxes of the
/// conversations the person has left, by `product/id`, until they come back to them, and the
/// conversation whose files are being saved before its reply goes, while Send says so.
#[derive(Default)]
pub struct ReplyFiles {
    pub open: Vec<Attached>,
    kept: std::collections::HashMap<String, Vec<Attached>>,
    pub saving: Option<String>,
}

impl ReplyFiles {
    /// Leaving the conversation `thread`: its box's files wait for the next visit.
    pub fn put_away(&mut self, thread: &str) {
        let files = std::mem::take(&mut self.open);
        if files.is_empty() {
            self.kept.remove(thread);
        } else {
            self.kept.insert(thread.to_string(), files);
        }
    }

    /// Coming to the conversation `thread`: its box holds what it held.
    pub fn take_out(&mut self, thread: &str) {
        self.open = self.kept.remove(thread).unwrap_or_default();
    }

    /// The files of `thread`'s box, open or kept: let go, as the box empties when its reply goes.
    pub fn clear(&mut self, thread: &str, open: bool) {
        if open {
            self.open.clear();
        }
        self.kept.remove(thread);
    }

    /// Files put back in `thread`'s box, where Z or a refusal hands back a reply that carried
    /// them, unless the box holds newer ones: then those stay, as the words do.
    pub fn put_back(&mut self, thread: &str, open: bool, files: Vec<Attached>) {
        let into = if open { &mut self.open } else { self.kept.entry(thread.to_string()).or_default() };
        if into.is_empty() {
            *into = files;
        }
    }

    fn all_mut(&mut self) -> impl Iterator<Item = &mut Attached> {
        self.open.iter_mut().chain(self.kept.values_mut().flatten())
    }
}

/// Why a reply with these files cannot go when Send is pressed, in the web window's words
/// (`send` in `DockComposer`, and `persistAttachments`): one still on its way in, else one that
/// cannot be attached or is too big. The reply box keeps Send lit for these and says so on the
/// press, where the new thread's box keeps Send off.
pub fn reply_refusal(files: &[Attached]) -> Option<String> {
    if let Some(coming) = files.iter().find(|a| a.coming) {
        return Some(format!("{} is still being attached.", coming.file.name));
    }
    rules::attach_problem(&pending(files)).map(|problem| format!("Not sent: {problem} Your draft has been kept."))
}

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
    /// Which file of the box this is, for an answer about it that lands later.
    pub id: u64,
    pub file: Pending,
    /// A picture's, when this window can read it. One named as a picture that it cannot read is
    /// drawn as a chip, rather than as a broken picture.
    pub thumb: Option<Thumb>,
    /// Still on its way in: being turned by the engine into a picture the window draws.
    pub coming: bool,
}

impl PartialEq for Attached {
    /// The same file, going the same way: its drawing is no part of what is sent.
    fn eq(&self, other: &Attached) -> bool {
        self.file == other.file && self.coming == other.coming
    }
}

fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
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
        let thumb = Self::thumb_of(path, image && failed.is_none() && size.is_some_and(|size| !rules::too_big_to_send(size)));
        Attached { id: next_id(), file: Pending { name, source: Source::Disk(path.to_string_lossy().to_string()), image, size, failed }, thumb, coming: false }
    }

    /// A picture file's place in the row, read off its header, when it is drawn.
    fn thumb_of(path: &Path, drawn: bool) -> Option<Thumb> {
        drawn.then(|| crate::pictures::size_of(path)).flatten().map(|natural| Thumb { natural, pixels: Pixels::new() })
    }

    /// A picture pasted from the clipboard, as PNG, under the name it goes by.
    pub fn pasted(png: &[u8], natural: (u32, u32), name: String) -> Attached {
        let file = Pending { name, source: Source::Pasted(Arc::from(png)), image: true, size: Some(png.len() as u64), failed: None };
        Attached { id: next_id(), file, thumb: Some(Thumb { natural, pixels: Pixels::new() }), coming: false }
    }

    /// What the engine answered a file it was asked to turn into a picture (`collectFiles`):
    /// turned, it goes on under its new name, as the new file, drawn; only copied, it goes as
    /// the copy under its own name; refused, as the file it was. It is in either way.
    fn staged(&mut self, answer: Result<Value, String>) {
        self.coming = false;
        let Ok(staged) = answer else { return };
        let Some(path) = staged.get("path").and_then(Value::as_str).map(str::to_string) else { return };
        self.file.source = Source::Disk(path.clone());
        if let Some(name) = staged.get("name").and_then(Value::as_str) {
            self.file.name = name.to_string();
            self.file.image = rules::is_image_name(name);
            self.file.size = std::fs::metadata(&path).ok().map(|meta| meta.len()).or(self.file.size);
            self.thumb = Self::thumb_of(Path::new(&path), self.file.image);
        }
    }
}

/// The files as they are sent.
pub fn pending(files: &[Attached]) -> Vec<Pending> {
    files.iter().map(|a| a.file.clone()).collect()
}

/// Whether Send may go for the files the box holds: none on its way in and none that cannot go
/// (`filesReady`).
pub fn ready(files: &[Attached]) -> bool {
    rules::files_ready(&pending(files), files.iter().any(|a| a.coming))
}

/// The engine's answers about files it is turning into pictures, as they land.
pub struct Staging {
    tx: Sender<(u64, Result<Value, String>)>,
    rx: Receiver<(u64, Result<Value, String>)>,
    waker: Option<Waker>,
}

impl Default for Staging {
    fn default() -> Staging {
        let (tx, rx) = channel();
        Staging { tx, rx, waker: None }
    }
}

impl Staging {
    /// The window's waker, so an answer is drawn when it lands.
    pub fn wake_with(&mut self, waker: Waker) {
        self.waker = Some(waker);
    }
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
fn picture(holder: Holder, at: usize, a: &Attached, thumb: &Thumb) -> El {
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
        // Under the pointer the × takes the text's own colour and its edge the faint one
        // (`.attach-remove:hover`).
        .child(text("×").font(TextStyle::new(12.0, 400.0).lh(12.0)).color(dim()).group_color(ink()))
        .hover_group()
        .child(crate::feel::edge_lit(6.0, muted(), true))
        .cursor(rune::Cursor::Pointer)
        .help("Remove")
        .on_background_press_tx(move |s: &mut State, tx| s.remove_attached(holder, at, tx));
    // Its title is its name, and the ×'s says what it does (`title` on `.attach-thumb` and
    // `.attach-remove`).
    el().size(wide, THUMB_TALL).radius(6.0).border(1.0, edge()).help(a.file.name.as_str()).child(drawn).child(remove)
}

/// Any other file (`.attach-chip`): its name, its size quieter and in figures that line up,
/// and the ×; a press anywhere on it takes it out. One that cannot go is lit, with no size
/// (`.attach-too-big`), and the line under the row says why.
fn chip(holder: Holder, at: usize, a: &Attached) -> El {
    let file = &a.file;
    let over = file.failed.is_some() || rules::too_big_to_send(file.size.unwrap_or(0));
    let words = |s: &str, font: TextStyle, c: rune::Color| pen(text(s).font(font).color(c), 12.0);
    let frame = |line: El, edge_colour: rune::Color| row().h(CHIP_TALL).items_center().px(10.0).radius(6.0).fill(rgb(47, 45, 44)).border(1.0, edge_colour).border_layout().child(line);
    // Under the pointer its words take the text's own colour and its edge the faint one, which
    // is how one that cannot go already stands; its size keeps its own (`.attach-chip:hover`).
    let lit = |frame: El| frame.hover_group().child(crate::feel::edge_lit(6.0, muted(), true));
    // On its way in it is its name alone, and nothing takes it out until it is in
    // (`.attach-coming`, whose bar runs along its foot as the pieces of a big file land; the
    // engine turning a picture says nothing of how far it got, so here it has none).
    if a.coming {
        return lit(frame(row().child(words(&format!("{} ", file.name), sans(12.0, 400.0), dim()).group_color(ink())), edge()));
    }
    let mut line = row().child(words(&file.name, sans(12.0, 400.0), if over { ink() } else { dim() }).group_color(ink()));
    if let Some(size) = file.size.filter(|size| *size > 0 && !over) {
        line = line.child(words(&format!(" {}", rules::size_words(size)), figures(12.0, 400.0), muted()));
    }
    line = line.child(words(" ×", sans(12.0, 400.0), if over { ink() } else { dim() }).group_color(ink()));
    // Its title says why one cannot go, and of any other that a press takes it out.
    let size = file.size.unwrap_or(0);
    let title = file.failed.clone().unwrap_or_else(|| if rules::too_big_to_send(size) { rules::over_the_limit(size) } else { "Remove".to_string() });
    let chip = frame(line, if over { muted() } else { edge() }).cursor(rune::Cursor::Pointer).help(title).on_background_press_tx(move |s: &mut State, tx| s.remove_attached(holder, at, tx));
    if over { chip } else { lit(chip) }
}

/// What a box holds of files, as the place it is drawn in lays it out: the row
/// (`.attach-row`), handed to `pad` for the room that place gives it, and under it the line
/// that says why one cannot go (`.compose-note.attach-problem`), handed to `note` for its room.
/// Nothing for a box with none.
pub fn rows(s: &State, holder: Holder, pad: impl Fn(El) -> El, note: impl Fn(El) -> El) -> Vec<El> {
    let files = s.files_in(holder);
    if files.is_empty() {
        return Vec::new();
    }
    let drawn = files.iter().enumerate().map(|(at, a)| match &a.thumb {
        Some(thumb) if !a.coming => picture(holder, at, a, thumb),
        _ => chip(holder, at, a),
    });
    let mut out = vec![pad(row().flex_wrap().items_start().gap(GAP).children(drawn))];
    if let Some(problem) = rules::attach_problem(&pending(files)) {
        out.push(note(el().pt(6.0).child(pen(text(problem).font(sans(12.0, 400.0)).color(dim()).wrap(), 12.0))));
    }
    out
}

/// The room the box a thread is started in gives the line under its files, on Home and on the
/// card (`.compose-note`'s own padding).
pub fn compose_note(note: El) -> El {
    note.px(18.0).pb(2.0)
}

/// How tall the row and its line stand on one line, where the box has to know before it is
/// laid out: a picture's height, else a chip's, and the line's.
pub fn rows_tall(files: &[Attached]) -> f32 {
    if files.is_empty() {
        return 0.0;
    }
    let row = if files.iter().any(|a| a.thumb.is_some() && !a.coming) { THUMB_TALL } else { CHIP_TALL };
    row + if rules::attach_problem(&pending(files)).is_some() { 23.5 } else { 0.0 }
}

/// Whether a still picture is being taken, which reads every picture on the spot.
fn still() -> bool {
    rune::tooling::headless_run().is_some_and(|run| run.frames() == 0)
}

impl State {
    /// The files in a box.
    pub fn files_in(&self, holder: Holder) -> &Vec<Attached> {
        match holder {
            Holder::Compose => &self.compose.files,
            Holder::Reply => &self.reply_files.open,
        }
    }

    fn files_in_mut(&mut self, holder: Holder) -> &mut Vec<Attached> {
        match holder {
            Holder::Compose => &mut self.compose.files,
            Holder::Reply => &mut self.reply_files.open,
        }
    }

    /// Files dropped on a box or the card, picked with the paper clip, or pasted from Finder,
    /// in the order they came (`addFiles` in `ThreadComposer.tsx`, `setAttachments` in
    /// `DockComposer`). A TIFF or a HEIC is handed to the engine to be turned into a picture the
    /// window draws, and is on its way in until it answers (`stageAttachment`); with no engine to
    /// ask, it goes as the file it is.
    pub fn attach_paths(&mut self, holder: Holder, paths: &[PathBuf]) {
        for path in paths {
            let mut attached = Attached::from_disk(path);
            let door = match &self.source {
                Reading::Door(door) => door.now(),
                _ => None,
            };
            if let (Some((url, token)), Source::Disk(src), true) = (door, &attached.file.source, attached.file.failed.is_none() && rules::turned_name(&attached.file.name).is_some()) {
                attached.coming = true;
                let (id, ask) = (attached.id, json!({ "name": attached.file.name, "srcPath": src }));
                let (tx, waker) = (self.compose.staging.tx.clone(), self.compose.staging.waker.clone());
                std::thread::spawn(move || {
                    let answer = source::call(&url, &token, rules::STAGE_ATTACHMENT, &ask);
                    let _ = tx.send((id, answer));
                    if let Some(wake) = waker {
                        wake();
                    }
                });
            }
            read_thumb(&attached, self.pictures.scale(), still());
            self.files_in_mut(holder).push(attached);
        }
    }

    /// What the engine said of the files it was turning into pictures, put in their place in
    /// whichever box holds them, a reply box left meanwhile included: from the window's turn.
    /// One taken out meanwhile is gone, and so is the answer. True when a box changed.
    pub fn land_staged(&mut self) -> bool {
        let landed: Vec<(u64, Result<Value, String>)> = self.compose.staging.rx.try_iter().collect();
        let mut changed = false;
        let scale = self.pictures.scale();
        for (id, answer) in landed {
            let Some(attached) = self.compose.files.iter_mut().chain(self.reply_files.all_mut()).find(|a| a.id == id) else { continue };
            attached.staged(answer);
            read_thumb(attached, scale, still());
            changed = true;
        }
        changed
    }

    /// A picture pasted with no file behind it: a screenshot, as PNG.
    pub fn attach_pasted(&mut self, holder: Holder, png: &[u8], natural: (u32, u32), name: String) {
        let attached = Attached::pasted(png, natural, name);
        read_thumb(&attached, self.pictures.scale(), still());
        self.files_in_mut(holder).push(attached);
    }

    /// ⌘V in the words. Files copied in Finder attach as themselves; a picture attaches,
    /// named for the moment it came (`fromPaste`); anything else is the words' own to paste.
    /// A Finder copy also carries the file's icon as a picture, which is not what was meant.
    pub fn paste_into_the_box(&mut self, holder: Holder, content: &ClipboardContent, now_ms: f64) -> bool {
        if !content.files().is_empty() {
            self.attach_paths(holder, content.files());
            return true;
        }
        if let Some(picture) = content.image() {
            self.attach_pasted(holder, picture.png(), picture.size(), rules::pasted_name(now_ms));
            return true;
        }
        false
    }

    /// The paper clip: the system's picker, over the window, and the file picked attaches
    /// (the box's hidden `<input type="file">`). Cancelling it changes nothing. The box's open
    /// menu closes first.
    pub fn pick_files(&mut self, holder: Holder, tx: &mut Tx) {
        match holder {
            Holder::Compose => self.compose.menu = None,
            Holder::Reply => self.reply_menu = None,
        }
        tx.open_file(FileDialogOptions::default(), move |s: &mut State, picked, tx| match picked {
            Ok(Some(path)) => s.attach_paths(holder, &[path]),
            Ok(None) => {}
            Err(e) => s.tell(&format!("The file picker did not open: {e}"), tx),
        });
    }

    /// The × on a file: it leaves the box, and the keyboard with it, as the web window's focus
    /// goes when the button it was on goes. The reply box stays open on its words.
    pub fn remove_attached(&mut self, holder: Holder, at: usize, tx: &mut Tx) {
        let files = self.files_in_mut(holder);
        if at < files.len() {
            files.remove(at);
        }
        match holder {
            Holder::Compose => {
                self.compose.menu = None;
                self.leave_the_box(tx);
            }
            Holder::Reply => {
                self.reply_menu = None;
                self.reply.focused = false;
                tx.blur();
            }
        }
    }

    /// Send, with files in the open conversation's reply box (`send` in `DockComposer`): a file
    /// still on its way in, or one that cannot go, is said and nothing is sent; otherwise the
    /// files are saved into the thread's project (`zero:save-attachments`), Send says Sending…
    /// meanwhile, and the box keeps its words and files until the engine has them. A send already
    /// on its way is not sent twice.
    pub(crate) fn carry(&mut self, item: &crate::data::Item, words: String, tx: &mut Tx) {
        if self.reply_files.saving.is_some() {
            return;
        }
        let files = self.reply_files.open.clone();
        if let Some(refused) = reply_refusal(&files) {
            return self.tell(&refused, tx);
        }
        let thread = format!("{}/{}", item.product, item.id);
        let pick = self.reply_pick_to_send(item);
        let save = crate::write::Step::Save { product: item.product.clone(), files: pending(&files) };
        if self.send_steps(vec![save], crate::write::Sent::Carry { thread: thread.clone(), words, pick, files }) {
            self.reply_files.saving = Some(thread);
        }
    }

    /// What the engine said to a reply's files. Saved, the reply is held as any other is, its
    /// words with a link to each file under them; refused, the box keeps everything and a line
    /// says why, in the web window's words.
    pub(crate) fn carried(&mut self, landed: crate::write::Landed, tx: &mut Tx) {
        let crate::write::Sent::Carry { thread, words, pick, files } = landed.sent.clone() else {
            return;
        };
        if self.reply_files.saving.as_deref() == Some(thread.as_str()) {
            self.reply_files.saving = None;
        }
        if let Some(why) = landed.refusal() {
            return self.tell(&format!("Not sent: {why} Your draft has been kept."), tx);
        }
        let links = rules::attachment_links(landed.answer.as_ref().unwrap_or(&Value::Null));
        let Some(item) = thread.split_once('/').and_then(|(product, id)| self.snap.items.iter().find(|i| i.id == id && i.product == product)).cloned() else {
            return;
        };
        self.hold_reply(&item, words, &links, pick, files, tx);
    }

    /// What a picture taken with files in a box says of them: each file by its path, attached
    /// as a drop attaches it, and each picture pasted by the PNG it was and the name the page
    /// gave it. The box a thread is started in reads `attached`, then `pasted`; the reply box,
    /// `replyFiles`, one list in the order they came, a pasted picture's entry the one with a PNG.
    pub(crate) fn picture_attached(&mut self, holder: Holder) {
        let list = |field: &str| self.fixture.get(field).and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
        let files = match holder {
            Holder::Compose => [list("attached"), list("pasted")].concat(),
            Holder::Reply => list("replyFiles"),
        };
        let text = |file: &serde_json::Value, field: &str| file.get(field).and_then(serde_json::Value::as_str).map(str::to_string);
        for file in files {
            if let (Some(png), Some(name)) = (text(&file, "png"), text(&file, "name")) {
                let Ok(bytes) = std::fs::read(&png) else { continue };
                if let Some(natural) = crate::pictures::size_of(Path::new(&png)) {
                    self.attach_pasted(holder, &bytes, natural, name);
                }
            } else if let Some(path) = text(&file, "path") {
                self.attach_paths(holder, &[PathBuf::from(path)]);
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

    // tests/a-tiff-you-paste-is-sent-as-a-picture.test.mts
    #[test]
    fn a_photo_the_engine_turned_goes_as_the_picture_and_one_it_could_not_as_itself() {
        let folder = std::env::temp_dir().join(format!("mercury-desktop-staged-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let turned = folder.join("mt1-ab12cd-IMG_0042.jpg");
        image::RgbImage::from_pixel(400, 300, image::Rgb([120, 90, 60])).save(&turned).unwrap();
        let photo = || Attached {
            id: 7,
            file: Pending { name: "IMG_0042.HEIC".to_string(), source: Source::Disk("/Users/you/Desktop/IMG_0042.HEIC".to_string()), image: false, size: Some(2_000_000), failed: None },
            thumb: None,
            coming: true,
        };
        // On its way in, it holds Send.
        assert!(!ready(&[photo()]));
        let mut done = photo();
        done.staged(Ok(json!({ "path": turned.to_string_lossy(), "name": "IMG_0042.jpg" })));
        assert_eq!((done.file.name.as_str(), done.file.image, done.coming), ("IMG_0042.jpg", true, false));
        assert_eq!(done.file.source, Source::Disk(turned.to_string_lossy().to_string()));
        assert_eq!((done.file.size, done.thumb.map(|t| t.natural)), (std::fs::metadata(&turned).ok().map(|m| m.len()), Some((400, 300))));
        // Copied and not turned (no sips): the copy goes, under its own name, as a file.
        let mut copied = photo();
        copied.staged(Ok(json!({ "path": "/Users/you/Mercury/accounts/a/.staging/mt1-ab12cd-IMG_0042.HEIC" })));
        assert_eq!((copied.file.name.as_str(), copied.file.image, copied.thumb.is_none()), ("IMG_0042.HEIC", false, true));
        assert_eq!(copied.file.source, Source::Disk("/Users/you/Mercury/accounts/a/.staging/mt1-ab12cd-IMG_0042.HEIC".to_string()));
        // Refused: the file as it was, and in.
        let mut refused = photo();
        refused.staged(Err("sips is not here".to_string()));
        assert_eq!((&refused.file, refused.coming), (&photo().file, false));
        assert!(ready(&[refused]));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /* ------------------------------ the reply box ------------------------------ */

    use crate::feel::tests::{key, window};
    use rune::{ClipboardImage, Key, KeyEvent, Mods, Runtime};

    /// The first thread of the invented inbox, its reply box opened with R and the caret in it.
    fn replying(name: &str) -> Runtime<State> {
        let mut rt = window(name, &["--thread"]);
        key(&mut rt, Key::Text("r".into()));
        assert!(rt.state.replying && rt.state.reply.focused, "R opens the box with the caret in it");
        rt
    }

    /// A screenshot on the clipboard, as PNG.
    fn screenshot() -> ClipboardImage {
        let mut png = Vec::new();
        image::RgbImage::from_pixel(240, 150, image::Rgb([52, 120, 196])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
        ClipboardImage::from_png(png).unwrap()
    }

    fn reply_names(rt: &Runtime<State>) -> Vec<String> {
        rt.state.reply_files.open.iter().map(|a| a.file.name.clone()).collect()
    }

    #[test]
    fn a_picture_pasted_into_the_reply_box_attaches_and_words_on_the_clipboard_are_typed() {
        let mut rt = replying("reply-paste");
        rt.paste_content(ClipboardContent::new("").with_image(screenshot()));
        rt.frame(1.0 / 60.0);
        let names = reply_names(&rt);
        assert_eq!(names.len(), 1, "the picture is in the box");
        assert!(names[0].starts_with("pasted-") && names[0].ends_with(".png") && rt.state.reply_files.open[0].file.image, "{names:?}");
        assert!(rt.state.reply.text.is_empty(), "and nothing was typed");
        // Words on the clipboard are the box's own to paste.
        rt.paste("Look at the total line.");
        rt.frame(1.0 / 60.0);
        assert_eq!((rt.state.reply.text.as_str(), rt.state.reply_files.open.len()), ("Look at the total line.", 1));
        // Nothing went in the box a thread is started in.
        assert!(rt.state.compose.files.is_empty());
    }

    #[test]
    fn files_in_a_reply_box_wait_in_their_thread_while_another_is_read() {
        let folder = std::env::temp_dir().join(format!("mercury-desktop-reply-drop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        let report = folder.join("week-32.pdf");
        std::fs::write(&report, "%PDF-1.4 invented").unwrap();
        let mut rt = replying("reply-drop");
        let first = rt.state.open_item().map(|i| i.id.clone()).unwrap();
        // Dropped on the box.
        let at = rt.find("reply").expect("the box's words");
        rt.drop_files(vec![report.clone()], Some(at));
        rt.frame(1.0 / 60.0);
        assert_eq!(reply_names(&rt), vec!["week-32.pdf".to_string()]);
        // Escape lets go of a box that holds something, and it stays open on it.
        key(&mut rt, Key::Escape);
        assert!(rt.state.replying && !rt.state.reply.focused, "held open by its file");
        // Off to the next thread and back: the file waited.
        key(&mut rt, Key::Text("j".into()));
        assert_ne!(rt.state.open_item().map(|i| i.id.clone()), Some(first.clone()));
        assert!(rt.state.reply_files.open.is_empty(), "the next thread's box is its own");
        key(&mut rt, Key::Text("k".into()));
        assert_eq!(rt.state.open_item().map(|i| i.id.clone()), Some(first));
        assert_eq!(reply_names(&rt), vec!["week-32.pdf".to_string()]);
        // Its × takes it out, and an empty box folds on Escape.
        rt.update(|s, tx| s.remove_attached(Holder::Reply, 0, tx));
        rt.frame(1.0 / 60.0);
        key(&mut rt, Key::Text("r".into()));
        key(&mut rt, Key::Escape);
        assert!(!rt.state.replying, "an empty box folds");
        std::fs::remove_dir_all(&folder).unwrap();
    }

    #[test]
    fn a_file_that_cannot_go_holds_the_reply_and_says_why_in_the_web_windows_words() {
        let mut rt = replying("reply-cannot");
        let gone = std::env::temp_dir().join(format!("mercury-desktop-reply-gone-{}.mov", std::process::id()));
        rt.update(|s, _| s.attach_paths(Holder::Reply, std::slice::from_ref(&gone)));
        rt.update(|s, _| s.reply.insert("Here it is"));
        rt.key(KeyEvent { key: Key::Enter, mods: Mods { cmd: true, ..Mods::default() } });
        rt.frame(1.0 / 60.0);
        let name = gone.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.clone()), Some(format!("Not sent: {name} could not be attached: the file was not found on this computer. Your draft has been kept.")));
        assert!(rt.state.held.is_none() && rt.state.replying, "nothing is held and the box keeps it all");
        assert_eq!(rt.state.reply.text, "Here it is");
        // One still on its way in is said so (`attaching`).
        rt.state.reply_files.open[0].coming = true;
        rt.update(|s, tx| s.send_reply(tx));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.clone()), Some(format!("{name} is still being attached.")));
    }

    #[test]
    fn a_reply_carrying_files_goes_with_their_links_and_z_hands_back_its_words_and_files() {
        let mut rt = replying("reply-carry");
        rt.paste_content(ClipboardContent::new("").with_image(screenshot()));
        rt.frame(1.0 / 60.0);
        rt.update(|s, _| s.reply.insert("Use this layout."));
        let item = rt.state.open_item().cloned().unwrap();
        let thread = format!("{}/{}", item.product, item.id);
        let files = rt.state.reply_files.open.clone();
        let name = files[0].file.name.clone();
        // Send with no engine behind the window (a fixture's): nothing goes, and the box keeps it all.
        rt.update(|s, tx| s.send_reply(tx));
        assert_eq!((rt.state.reply_files.open.len(), rt.state.reply_files.saving.clone(), rt.state.reply.text.as_str()), (1, None, "Use this layout."));
        // The engine saved the picture: the reply is held with the link under its words, and the
        // box empties and folds.
        let saved = json!([{ "rel": format!("attachments/mx1-ab-{name}"), "name": name, "image": true }]);
        let landed = crate::write::Landed { sent: crate::write::Sent::Carry { thread: thread.clone(), words: "Use this layout.".to_string(), pick: None, files: files.clone() }, answer: Ok(saved), snapshot: None };
        rt.update(|s, tx| s.carried(landed, tx));
        let held = rt.state.held.as_ref().expect("the reply waits its three seconds");
        assert_eq!(held.payload["answer"], json!(format!("Use this layout.\n\n![{name}](attachments/mx1-ab-{name})")));
        assert!(!rt.state.replying && rt.state.reply_files.open.is_empty() && rt.state.reply.text.is_empty());
        // Z gives back the words and the picture, not the link.
        rt.update(|s, tx| s.undo(tx));
        assert_eq!((rt.state.reply.text.as_str(), reply_names(&rt)), ("Use this layout.", vec![name.clone()]));
        assert!(rt.state.replying && rt.state.held.is_none());
        // Refused by the engine instead, the box keeps everything and says why.
        let refused = crate::write::Landed { sent: crate::write::Sent::Carry { thread, words: "Use this layout.".to_string(), pick: None, files }, answer: Err("connection refused".to_string()), snapshot: None };
        rt.update(|s, tx| s.carried(refused, tx));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.clone()), Some("Not sent: connection refused Your draft has been kept.".to_string()));
        assert!(rt.state.held.is_none() && rt.state.reply_files.open.len() == 1);
    }
}
