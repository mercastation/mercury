//! The files a new thread carries: which of them can go, what the box says of one that cannot,
//! what a picture pasted with no name of its own is called, and how the thread's body links
//! them once the engine has saved them. A port of `renderer/src/attachments.ts`, with what it
//! reads of `shared/carried-files.mts` and `shared/picture-formats.mts`. The title of a thread
//! sent with files and no words is `thread_message` in `rules/compose.rs`.

use std::sync::Arc;

use base64::Engine;
use serde_json::{Value, json};

use crate::rules::js::re;

/// The most one message sends, and the words the box says it in (`MAX_SENT_BYTES` and
/// `MAX_SENT_WORDS` in `shared/carried-files.mts`). A bigger file is never read at all.
pub const MAX_SENT_BYTES: u64 = 1 << 30;
pub const MAX_SENT_WORDS: &str = "1 GB";

/// The channel a new thread's files go to before the thread does (`api.saveAttachments`,
/// answered in `main/ipc.mts` by `main/save-attachments.mts`).
pub const SAVE_ATTACHMENTS: &str = "zero:save-attachments";

/// And the one a dropped picture the window cannot draw goes to, to be turned into one it
/// can (`api.stageAttachment`, answered by `main/stage-attachment.mts`).
pub const STAGE_ATTACHMENT: &str = "zero:stage-attachment";

const KB: f64 = 1024.0;
const MB: f64 = 1024.0 * KB;
const GB: f64 = 1024.0 * MB;

/// `x.toFixed(digits)` for a size, which is never negative: of two numbers as near, the larger.
fn to_fixed(x: f64, digits: i32) -> String {
    let scale = 10f64.powi(digits);
    format!("{:.*}", digits as usize, (x * scale).round() / scale)
}

/// A file's size as the box says it: KB under a megabyte and never 0, MB to a tenth under a
/// hundred, whole MB above, GB to a tenth and without a trailing nought (`sizeWords`).
pub fn size_words(bytes: u64) -> String {
    let b = bytes as f64;
    if b < MB {
        return format!("{} KB", (b / KB).round().max(1.0));
    }
    if b >= GB {
        let fixed = to_fixed(b / GB, 1);
        return format!("{} GB", fixed.strip_suffix(".0").unwrap_or(&fixed));
    }
    format!("{} MB", to_fixed(b / MB, if b >= 100.0 * MB { 0 } else { 1 }))
}

/// Whether a file is over what a message can send, which the box says before anything is sent
/// (`tooBigToSend`).
pub fn too_big_to_send(bytes: u64) -> bool {
    bytes > MAX_SENT_BYTES
}

/// What the chip of a file too big to go says in its title (`overTheLimit` in
/// `shared/carried-files.mts`).
pub fn over_the_limit(bytes: u64) -> String {
    format!("{} is over the {MAX_SENT_WORDS} a message can send. Send a smaller copy, or share a link.", size_words(bytes))
}

re!(PICTURE_NAME, r"\.(png|jpe?g|gif|webp)$", "i");

/// A name the window draws as a picture, and the engine links as one (`isImageName`).
pub fn is_image_name(name: &str) -> bool {
    PICTURE_NAME.test(name)
}

re!(TIFF, r"\.tiff?$", "i");
re!(HEIC, r"\.hei[cf]$", "i");
re!(EXTENSION, r"\.[^.]+$", "");

/// The name a picture the window cannot draw goes on under once the engine has turned it into
/// one it can, a TIFF into a PNG and a HEIC into a JPEG, or none for any other file
/// (`turnedName` in `shared/picture-formats.mts`).
pub fn turned_name(name: &str) -> Option<String> {
    let to = if TIFF.test(name) {
        ".png"
    } else if HEIC.test(name) {
        ".jpg"
    } else {
        return None;
    };
    Some(EXTENSION.replace(name, |_| to.to_string()))
}

/// What a picture pasted with no name of its own is called: a screenshot copied on a Mac
/// (`collectFiles`, which names any `image.png` so).
pub fn pasted_name(now_ms: f64) -> String {
    format!("pasted-{}.png", (now_ms.max(0.0) as u64) % 100_000)
}

/// Where a file in the box comes from, and so how it is sent.
#[derive(Clone, PartialEq)]
pub enum Source {
    /// A file on this Mac, dropped, picked or pasted from Finder: sent as its path, and the
    /// engine copies it (`srcPath`). A folder goes as the files inside it.
    Disk(String),
    /// A picture pasted from the clipboard, as PNG: sent as its bytes (`dataBase64`).
    Pasted(Arc<[u8]>),
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Disk(path) => write!(f, "Disk({path:?})"),
            Source::Pasted(png) => write!(f, "Pasted({} bytes)", png.len()),
        }
    }
}

/// A file in the box, waiting for Send (`PendingAttachment`).
#[derive(Clone, Debug, PartialEq)]
pub struct Pending {
    pub name: String,
    pub source: Source,
    /// Drawn small in the box, where any other file is a chip.
    pub image: bool,
    /// In bytes, so the box can say a file is too big before it is sent. None for a folder.
    pub size: Option<u64>,
    /// Why it could not be attached, in words for the box. Nothing of it is sent.
    pub failed: Option<String>,
}

/// What the box says of a file it could not take in (`collectFiles`' catch).
pub fn could_not_attach(name: &str, why: &str) -> String {
    format!("{name} could not be attached: {}.", why.strip_suffix('.').unwrap_or(why))
}

/// What stops the box sending, said for the person: a file that could not be attached, or one
/// too big to go (`attachProblem`).
pub fn attach_problem(list: &[Pending]) -> Option<String> {
    if let Some(failed) = list.iter().find_map(|a| a.failed.clone()) {
        return Some(failed);
    }
    let big = list.iter().find(|a| too_big_to_send(a.size.unwrap_or(0)))?;
    Some(format!("{} is {}, over the {MAX_SENT_WORDS} a message can send.", big.name, size_words(big.size.unwrap_or(0))))
}

/// Whether Send may go: no file still on its way in, and none that cannot go (`filesReady` in
/// `ThreadComposer.tsx`).
pub fn files_ready(list: &[Pending], coming: bool) -> bool {
    !coming && attach_problem(list).is_none()
}

/// What `zero:save-attachments` is handed: the project, and each file by its path or, pasted,
/// by its bytes (`persistAttachments`). The bytes are read here, so this runs off the main
/// thread.
pub fn save_payload(product: &str, files: &[Pending]) -> Value {
    let files: Vec<Value> = files
        .iter()
        .map(|file| match &file.source {
            Source::Disk(path) => json!({ "name": file.name, "srcPath": path }),
            Source::Pasted(png) => json!({ "name": file.name, "dataBase64": base64::engine::general_purpose::STANDARD.encode(png) }),
        })
        .collect();
    json!({ "product": product, "files": files })
}

/// The saved files as the message links them, one to a line: a picture drawn, anything else a
/// link (`attachmentLinks`). `saved` is what the engine answered, `[{ rel, name, image }]`.
pub fn attachment_links(saved: &Value) -> String {
    let field = |file: &Value, name: &str| file.get(name).and_then(Value::as_str).unwrap_or_default().to_string();
    let links: Vec<String> = saved
        .as_array()
        .map(|files| {
            files
                .iter()
                .map(|file| {
                    let (name, rel) = (field(file, "name"), field(file, "rel"));
                    if file.get("image") == Some(&Value::Bool(true)) { format!("![{name}]({rel})") } else { format!("[{name}]({rel})") }
                })
                .collect()
        })
        .unwrap_or_default();
    links.join("\n")
}

/// A new thread's payload with the files the engine saved linked under its words, a blank line
/// between (`[message.body, attached].filter(Boolean).join('\n\n')` in `sendTask`). A thread
/// with neither words nor links under its title has no body at all.
pub fn linked(mut compose: Value, saved: &Value) -> Value {
    let body = compose.get("body").and_then(Value::as_str).unwrap_or_default().to_string();
    let links = attachment_links(saved);
    let joined = [body, links].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("\n\n");
    if let Some(fields) = compose.as_object_mut() {
        if joined.is_empty() {
            fields.remove("body");
        } else {
            fields.insert("body".to_string(), json!(joined));
        }
    }
    compose
}

#[cfg(test)]
mod tests {
    use super::*;

    const MB_U: u64 = 1024 * 1024;

    fn file(name: &str, size: Option<u64>, failed: Option<&str>) -> Pending {
        Pending { name: name.to_string(), source: Source::Disk(format!("/Users/you/Desktop/{name}")), image: is_image_name(name), size, failed: failed.map(str::to_string) }
    }

    // tests/a-small-file-in-the-box-says-its-size-in-kb.test.mts
    #[test]
    fn a_file_under_a_megabyte_is_said_in_kb_and_never_as_nought() {
        assert_eq!(size_words(12), "1 KB");
        assert_eq!(size_words(300 * 1024), "300 KB");
        assert_eq!(size_words(1023 * 1024), "1023 KB");
        assert_eq!(size_words(MB_U), "1.0 MB");
        assert_eq!(size_words((10.6 * MB) as u64), "10.6 MB");
        assert_eq!(size_words(250 * MB_U), "250 MB");
        // A gigabyte or more to a tenth, with no nought after the point.
        assert_eq!(size_words((1.2 * GB) as u64), "1.2 GB");
        assert_eq!(size_words(1 << 30), "1 GB");
    }

    // tests/a-film-up-to-1-gb-attaches-in-pieces-and-says-when-it-cannot.test.mts
    #[test]
    fn a_box_holding_a_file_that_cannot_go_says_why_and_does_not_send() {
        assert_eq!(attach_problem(&[file("film-9mb.mp4", Some(9 * MB_U), None)]), None);
        assert_eq!(attach_problem(&[file("master-1.2gb.mov", Some((1.2 * GB) as u64), None)]).as_deref(), Some("master-1.2gb.mov is 1.2 GB, over the 1 GB a message can send."));
        assert_eq!(attach_problem(&[file("film-60mb.mp4", Some(5 * MB_U), Some("film-60mb.mp4 could not be attached: The disk is full."))]).as_deref(), Some("film-60mb.mp4 could not be attached: The disk is full."));
        // Exactly 1 GB goes, a byte more does not, and one that failed is said first.
        assert!(!too_big_to_send(1 << 30) && too_big_to_send((1 << 30) + 1));
        // The chip's title (tests/team-a-film-up-to-1-gb-goes-to-a-teammate-and-plays.test.mts,
        // tests/team-a-file-over-10-mb-goes-in-parts-and-opens-whole.test.mts).
        assert_eq!(over_the_limit((1.5 * GB) as u64), "1.5 GB is over the 1 GB a message can send. Send a smaller copy, or share a link.");
        assert_eq!(over_the_limit(1300 * MB_U), "1.3 GB is over the 1 GB a message can send. Send a smaller copy, or share a link.");
        let both = [file("raw.mov", Some((1 << 30) + 1), None), file("cut.mp4", None, Some("cut.mp4 could not be attached: no."))];
        assert_eq!(attach_problem(&both).as_deref(), Some("cut.mp4 could not be attached: no."));
        assert!(!files_ready(&both, false));
        // A file still on its way in holds Send too.
        assert!(files_ready(&[file("notes.md", Some(40), None)], false));
        assert!(!files_ready(&[file("notes.md", Some(40), None)], true));
        // A folder has no size of its own here and is never too big.
        assert_eq!(attach_problem(&[file("designs", None, None)]), None);
    }

    #[test]
    fn a_file_that_could_not_be_attached_is_said_in_one_sentence() {
        assert_eq!(could_not_attach("film-60mb.mp4", "The disk is full."), "film-60mb.mp4 could not be attached: The disk is full.");
        assert_eq!(could_not_attach("notes.md", "the file was not found on this computer"), "notes.md could not be attached: the file was not found on this computer.");
    }

    #[test]
    fn a_picture_is_one_the_window_draws() {
        for name in ["shot.png", "photo.JPG", "photo.jpeg", "loop.gif", "a.webp"] {
            assert!(is_image_name(name), "{name}");
        }
        for name in ["notes.pdf", "scan.tiff", "IMG_0042.HEIC", "logo.svg", "png", "shot.png.zip"] {
            assert!(!is_image_name(name), "{name}");
        }
    }

    // tests/a-tiff-you-paste-is-sent-as-a-picture.test.mts
    #[test]
    fn a_tiff_or_a_heic_goes_on_as_a_picture_the_window_draws() {
        assert_eq!(turned_name("file-3674d0dea94a72e6b8c93fc8e9cf731c.tiff").as_deref(), Some("file-3674d0dea94a72e6b8c93fc8e9cf731c.png"));
        assert_eq!(turned_name("scan.tif").as_deref(), Some("scan.png"));
        assert_eq!(turned_name("SCAN.TIFF").as_deref(), Some("SCAN.png"));
        assert_eq!(turned_name("IMG_0042.HEIC").as_deref(), Some("IMG_0042.jpg"));
        assert_eq!(turned_name("burst.heif").as_deref(), Some("burst.jpg"));
        for name in ["shot.png", "photo.jpg", "photo.jpeg", "loop.gif", "a.webp", "tiff-notes.txt", "scan.tiff.zip", "tiff", "notes.pdf", ""] {
            assert_eq!(turned_name(name), None, "{name}");
        }
    }

    #[test]
    fn a_pasted_screenshot_is_named_by_the_clock() {
        assert_eq!(pasted_name(1_784_000_044_661.0), "pasted-44661.png");
        assert_eq!(pasted_name(1_784_000_000_007.0), "pasted-7.png");
    }

    #[test]
    fn a_file_on_disk_goes_as_its_path_and_a_pasted_picture_as_its_bytes() {
        let files = [file("week-32.pdf", Some(1024), None), Pending { name: "pasted-1.png".to_string(), source: Source::Pasted(Arc::from(&b"png"[..])), image: true, size: Some(3), failed: None }];
        assert_eq!(
            save_payload("storefront", &files),
            json!({ "product": "storefront", "files": [{ "name": "week-32.pdf", "srcPath": "/Users/you/Desktop/week-32.pdf" }, { "name": "pasted-1.png", "dataBase64": "cG5n" }] })
        );
    }

    // tests/a-picture-you-paste-to-an-ai-teammate-reaches-it.test.mts
    #[test]
    fn the_saved_files_are_linked_one_to_a_line_under_the_words() {
        let saved = json!([{ "rel": "attachments/x-pasted-1.png", "name": "pasted-1.png", "image": true }, { "rel": "attachments/y-notes.pdf", "name": "notes.pdf", "image": false }]);
        assert_eq!(attachment_links(&saved), "![pasted-1.png](attachments/x-pasted-1.png)\n[notes.pdf](attachments/y-notes.pdf)");
        assert_eq!(attachment_links(&json!([])), "");
        let compose = json!({ "product": "storefront", "title": "What is wrong here?", "kind": "directive", "priority": 0.0, "model": "opus" });
        // No words under the title: the body is the links alone.
        assert_eq!(linked(compose.clone(), &saved)["body"], json!("![pasted-1.png](attachments/x-pasted-1.png)\n[notes.pdf](attachments/y-notes.pdf)"));
        // Words under it keep their place, and the links follow after a blank line.
        let mut worded = compose.clone();
        worded["body"] = json!("Look at this.");
        assert_eq!(linked(worded.clone(), &saved)["body"], json!("Look at this.\n\n![pasted-1.png](attachments/x-pasted-1.png)\n[notes.pdf](attachments/y-notes.pdf)"));
        // Nothing saved leaves the payload as it was.
        assert_eq!(linked(worded.clone(), &json!([])), worded);
        assert_eq!(linked(compose.clone(), &json!([])), compose);
    }
}
