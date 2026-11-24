//! What the reply boxes hold, kept across a restart as the web window keeps its own in its
//! storage (`renderer/src/drafts.ts`): each thread's words as they are typed (`zero.draft.*`),
//! the files beside them (`zero.draftimg.*`), the model, level and agent picked in its menu
//! (`zero.draftpick.*`); the replies sent and not yet written (`zero.replysending.*`); the lines
//! Zs left in threads (`zero.undoMarks.v1`); and whether the waiting agent's warning was given
//! (`zero.stoppedWarned`). The web window's storage is its own, so this window keeps its own, in
//! two files beside the note about the app's door: the words and the rest, small and written as
//! they change, and the files, whose pasted pictures can be large, written only when the files
//! change. A picture drawn from a fixture, or a window pointed by hand at some engine, keeps
//! nothing on disk unless `--drafts <file>` (or `MERCURY_DRAFTS`) says where.
//!
//! The priority and the mode picked on the box's chips are not kept, as the web window keeps
//! neither: a box opened anew starts on the thread's own.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::answer::UndoMark;
use crate::attach::Attached;
use crate::rules::attach::Source;
use crate::rules::reply_pick::ReplyPick;

const FILE: &str = "native-drafts.json";

/// How much of the store one thread's pasted pictures may take, written down
/// (`COMPOSE_DRAFT_MAX_BYTES`).
pub const MAX_BYTES: usize = 3_000_000;

/// A reply sent and not yet written: the words as typed, the whole body that went, and when
/// (`InFlightReply`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InFlight {
    pub words: String,
    pub body: String,
    pub at: f64,
    /// The files the box held, which come back with the words.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<KeptFile>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Kept {
    /// By `product/id`.
    pub words: BTreeMap<String, String>,
    pub picks: BTreeMap<String, ReplyPick>,
    pub sending: BTreeMap<String, Vec<InFlight>>,
    pub marks: Vec<UndoMark>,
    pub warned: bool,
}

/// One file of a box, as it is written down: a file on this Mac by its path, a pasted picture by
/// its bytes (`DraftAttachment`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeptFile {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png: Option<String>,
    #[serde(default)]
    pub image: bool,
}

impl KeptFile {
    pub fn of(a: &Attached) -> KeptFile {
        let (path, png) = match &a.file.source {
            Source::Disk(path) => (Some(path.clone()), None),
            Source::Pasted(bytes) => (None, Some(base64::engine::general_purpose::STANDARD.encode(bytes))),
        };
        KeptFile { name: a.file.name.clone(), path, png, image: a.file.image }
    }
}

/// One thread's files within the budget: the bytes of the newest pasted pictures come off first
/// until the rest fits; a file named by its path costs nothing and always stays. Returns what is
/// kept and how many went (`saveDraftAttachments`).
pub fn within_budget(files: Vec<KeptFile>) -> (Vec<KeptFile>, usize) {
    let mut kept = files;
    let mut dropped = 0;
    loop {
        let size = serde_json::to_string(&kept).map_or(0, |s| s.len());
        if size <= MAX_BYTES {
            return (kept, dropped);
        }
        let Some(last) = kept.iter().rposition(|f| f.png.is_some()) else {
            return (kept, dropped);
        };
        kept.remove(last);
        dropped += 1;
    }
}

/// What the box says of pictures it holds that would not be kept (`tooBig`).
pub fn too_big_note(dropped: usize) -> Option<String> {
    match dropped {
        0 => None,
        1 => Some("One pasted image is too big to save. Closing the box loses it.".to_string()),
        n => Some(format!("{n} pasted images are too big to save. Closing the box loses them.")),
    }
}

/// The drafts and where they are written, and what was last written there.
#[derive(Debug, Default)]
pub struct Store {
    file: Option<PathBuf>,
    written: String,
    files_written: String,
    /// The open box's pictures that would not be kept.
    pub too_big: usize,
}

fn files_file(file: &Path) -> PathBuf {
    file.with_file_name("native-draft-files.json")
}

fn write_beside(file: &Path, text: &str) {
    let draft = file.with_extension(format!("{}.tmp", std::process::id()));
    if let Some(folder) = file.parent() {
        let _ = std::fs::create_dir_all(folder);
    }
    if std::fs::write(&draft, text).is_ok() {
        let _ = std::fs::rename(&draft, file);
    }
}

impl Store {
    /// The drafts kept at `file`, and its files. Missing or unreadable is nothing kept.
    pub fn at(file: Option<PathBuf>) -> (Store, Kept, BTreeMap<String, Vec<KeptFile>>) {
        let read = |path: &Path| std::fs::read_to_string(path).ok();
        let kept: Kept = file.as_deref().and_then(read).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        let files: BTreeMap<String, Vec<KeptFile>> = file.as_deref().map(files_file).as_deref().and_then(read).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        let store = Store {
            written: serde_json::to_string(&kept).unwrap_or_default(),
            files_written: serde_json::to_string(&files).unwrap_or_default(),
            file,
            too_big: 0,
        };
        (store, kept, files)
    }

    /// Writes down what changed since the last write.
    pub fn keep(&mut self, kept: &Kept, files: &BTreeMap<String, Vec<KeptFile>>) {
        self.keep_words(kept);
        let Some(file) = self.file.clone() else { return };
        let text = serde_json::to_string(files).unwrap_or_default();
        if text != self.files_written {
            write_beside(&files_file(&file), &text);
            self.files_written = text;
        }
    }

    /// The words and the rest alone, when no box's files changed.
    pub fn keep_words(&mut self, kept: &Kept) {
        let Some(file) = self.file.clone() else { return };
        let text = serde_json::to_string(kept).unwrap_or_default();
        if text != self.written {
            write_beside(&file, &text);
            self.written = text;
        }
    }
}

impl crate::State {
    /// Writes down what the boxes hold now, where it changed: the words of every box (the open
    /// one's as typed), the picks, the replies on their way, the Zs' lines and the warning given;
    /// and, when the files of any box changed, the files, each thread's within the budget.
    pub fn keep_drafts(&mut self) {
        let open = match &self.view {
            crate::View::Thread { id, product } => Some(format!("{product}/{id}")),
            _ => None,
        };
        let mut words: BTreeMap<String, String> = self.drafts.iter().filter(|(_, w)| !w.trim().is_empty()).map(|(k, w)| (k.clone(), w.clone())).collect();
        if let Some(open) = &open {
            words.remove(open);
            // What a caught send left in the box is no longer its draft (`clearDraft`).
            if !self.reply.text.trim().is_empty() && !crate::cloud_need::unkept(self, open, &self.reply.text) {
                words.insert(open.clone(), self.reply.text.clone());
            }
        }
        let kept = Kept {
            words,
            picks: self.reply_picks.iter().map(|(k, p)| (k.clone(), p.clone())).collect(),
            sending: self.answering.in_flight.clone(),
            marks: self.answering.marks.clone(),
            warned: self.answering.warned,
        };
        // The files are looked at by which files each box holds, and only written out anew when
        // that changed: a pasted picture is megabytes of text.
        let mut print: Vec<(String, Vec<u64>)> = self.reply_files.kept().iter().map(|(k, f)| (k.clone(), f.iter().map(|a| a.id).collect())).collect();
        if let Some(open) = &open {
            print.retain(|(k, _)| k != open);
            print.push((open.clone(), self.reply_files.open.iter().map(|a| a.id).collect()));
        }
        print.retain(|(_, ids)| !ids.is_empty());
        print.sort();
        let print = format!("{print:?}");
        let files = if print != self.answering.files_print {
            self.answering.files_print = print;
            let mut all: BTreeMap<String, Vec<KeptFile>> = BTreeMap::new();
            let mut too_big = 0;
            for (thread, list) in self.reply_files.kept() {
                all.insert(thread.clone(), within_budget(list.iter().map(KeptFile::of).collect()).0);
            }
            if let Some(open) = &open {
                all.remove(open);
                let (list, dropped) = within_budget(self.reply_files.open.iter().map(KeptFile::of).collect());
                too_big = dropped;
                if !list.is_empty() {
                    all.insert(open.clone(), list);
                }
            }
            all.retain(|_, list| !list.is_empty());
            self.kept_drafts.too_big = too_big;
            Some(all)
        } else {
            None
        };
        match files {
            Some(files) => self.kept_drafts.keep(&kept, &files),
            None => self.kept_drafts.keep_words(&kept),
        }
    }

    /// What a window opening finds kept: the words, picks and files back in their boxes, the Zs'
    /// lines, the warning given, and the replies that were on their way.
    pub(crate) fn take_up_drafts(&mut self, kept: Kept, files: BTreeMap<String, Vec<KeptFile>>) {
        for (thread, words) in kept.words {
            self.drafts.insert(thread, words);
        }
        for (thread, pick) in kept.picks {
            self.reply_picks.insert(thread, pick);
        }
        for (thread, list) in files {
            let back: Vec<Attached> = list.iter().filter_map(attached).collect();
            for a in &back {
                crate::attach::read_thumb(a, self.pictures.scale(), crate::attach::still());
            }
            self.reply_files.put_back(&thread, false, back);
        }
        self.answering.marks = kept.marks;
        self.answering.warned = kept.warned;
        self.answering.in_flight = kept.sending;
    }
}

/// Where the drafts are kept: where the command line says; else beside the note the window found
/// the app's door by; else nowhere.
pub fn file(args: &[String], note: Option<&str>) -> Option<PathBuf> {
    let said = args.iter().position(|a| a == "--drafts").and_then(|at| args.get(at + 1)).cloned().or_else(|| std::env::var("MERCURY_DRAFTS").ok());
    said.map(PathBuf::from).or_else(|| note.and_then(|note| Path::new(note).parent().map(|folder| folder.join(FILE))))
}

/// A kept file back in a box: from the disk as a drop takes it, or as the picture it was pasted
/// as. One whose bytes do not read is let go.
pub fn attached(file: &KeptFile) -> Option<Attached> {
    if let Some(path) = &file.path {
        let mut a = Attached::from_disk(Path::new(path));
        a.file.name = file.name.clone();
        return Some(a);
    }
    let bytes = base64::engine::general_purpose::STANDARD.decode(file.png.as_deref()?).ok()?;
    let natural = image::ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format().ok()?.into_dimensions().ok()?;
    Some(Attached::pasted(&bytes, natural, file.name.clone()))
}

/// Whether a reply written down as sent reached its thread: an answer in its ledger carries its
/// words or its body, compared with their spaces folded (`landedIn`).
pub fn landed_in(reply: &InFlight, answers: &[String]) -> bool {
    let flat = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let wanted: Vec<String> = [reply.body.as_str(), reply.words.as_str()].iter().map(|s| flat(s)).filter(|s| !s.is_empty()).collect();
    if wanted.is_empty() {
        return true;
    }
    answers.iter().any(|a| wanted.iter().any(|w| flat(a).contains(w.as_str())))
}

/// The waits before each look for a reply a restart may have lost, after the first: once at
/// once, then thirty seconds on, then ninety (`UNSENT_CHECKS_MS`).
pub const UNSENT_CHECKS_MS: [f64; 3] = [0.0, 30_000.0, 90_000.0];

/// What is said when replies came back to their boxes after a restart.
pub fn back_in_the_box(n: usize) -> String {
    if n == 1 { "Your reply didn’t send. It’s back in the box.".to_string() } else { format!("{n} replies didn’t send. They’re back in the box.") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("mercury-drafts-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        folder.join(FILE)
    }

    #[test]
    fn drafts_are_written_as_they_change_and_read_by_the_next_window() {
        let file = scratch("kept");
        let (mut store, kept, files) = Store::at(Some(file.clone()));
        assert_eq!(kept, Kept::default());
        assert!(files.is_empty());
        store.keep(&kept, &files);
        assert!(!file.exists(), "nothing changed, nothing written");
        let mut kept = Kept::default();
        kept.words.insert("shop/w-1".to_string(), "Go on, but".to_string());
        kept.picks.insert("shop/w-1".to_string(), ReplyPick { model: Some("sonnet".to_string()), effort: Some("high".to_string()), engine: None });
        kept.warned = true;
        let mut files = BTreeMap::new();
        files.insert("shop/w-1".to_string(), vec![KeptFile { name: "week.pdf".to_string(), path: Some("/tmp/week.pdf".to_string()), png: None, image: false }]);
        store.keep(&kept, &files);
        let (_, again, files_again) = Store::at(Some(file.clone()));
        assert_eq!(again, kept);
        assert_eq!(files_again, files);
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn the_newest_pasted_pictures_go_first_past_the_budget_and_a_path_always_stays() {
        let big = |name: &str| KeptFile { name: name.to_string(), path: None, png: Some("x".repeat(1_600_000)), image: true };
        let path = KeptFile { name: "a.pdf".to_string(), path: Some("/tmp/a.pdf".to_string()), png: None, image: false };
        let (kept, dropped) = within_budget(vec![big("one.png"), path.clone(), big("two.png")]);
        assert_eq!(dropped, 1);
        assert_eq!(kept.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["one.png", "a.pdf"]);
        assert_eq!(too_big_note(1).as_deref(), Some("One pasted image is too big to save. Closing the box loses it."));
        assert_eq!(too_big_note(2).as_deref(), Some("2 pasted images are too big to save. Closing the box loses them."));
        assert_eq!(too_big_note(0), None);
    }

    #[test]
    fn a_reply_reached_its_thread_when_an_answer_there_carries_its_words() {
        let reply = InFlight { words: "Ship it.".to_string(), body: "Ship it.\n\n[a.pdf](attachments/a.pdf)".to_string(), at: 1.0, files: Vec::new() };
        assert!(landed_in(&reply, &["Ship  it.".to_string()]));
        assert!(!landed_in(&reply, &["Hold it.".to_string()]));
        assert_eq!(back_in_the_box(1), "Your reply didn’t send. It’s back in the box.");
        assert_eq!(back_in_the_box(3), "3 replies didn’t send. They’re back in the box.");
    }

    #[test]
    fn drafts_sit_beside_the_doors_note_unless_the_command_line_says_where() {
        let none: [String; 0] = [];
        if std::env::var("MERCURY_DRAFTS").is_err() {
            assert_eq!(file(&none, None), None);
            assert_eq!(file(&none, Some("/somewhere/Mercury/local-door.json")), Some(PathBuf::from("/somewhere/Mercury/native-drafts.json")));
        }
        let said = ["--drafts".to_string(), "/tmp/d.json".to_string()];
        assert_eq!(file(&said, None), Some(PathBuf::from("/tmp/d.json")));
    }
}
