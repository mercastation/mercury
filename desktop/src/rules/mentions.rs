//! What a reference in a message is and the short name it goes by, for the quiet mention that
//! draws it: the icon says what kind of thing it is, the name says which
//! (`renderer/src/mentions.ts`, with `renderer/src/source-reference.ts`).

use super::js::re;
use super::paths::is_media_path;

/// `MentionKind`. A thread's mention is drawn by another rule and is not among these.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Folder,
    Page,
    Doc,
    Image,
    Code,
    Media,
    Link,
    File,
}

re!(ADDRESS, r"^(?:[a-z][a-z0-9+.-]*:)", "i");
re!(FILE_SCHEME, r"^file:", "i");
re!(AFTER_NAME, r"[?#].*$", "");
re!(PAGE, r"\.html?$", "i");
re!(DOC, r"\.(md|markdown|txt|pdf|docx?|rtf)$", "i");
re!(IMAGE, r"\.(png|jpe?g|gif|webp|svg|heic)$", "i");
re!(MAILTO, r"^mailto:", "i");
re!(SCHEME_SLASHES, r"^[a-z][a-z0-9+.-]*:\/\/", "i");
re!(FILE_SLASHES, r"^file:\/\/", "i");
re!(SOURCE, r"^((?:\/|\.\/)?[\w@.-]+(?:\/[\w@.-]+)*\.(?:[cm]?jsx?|[cm]?tsx?|py|go|rs|rb|java|kt|swift|[ch](?:pp)?|cs|css|scss|vue|svelte|sql|sh|json|ya?ml|toml))(?::[1-9]\d*(?:-[1-9]\d*)?|#L[1-9]\d*)?$", "i");
re!(HAS_SPACE, r"\s", "");
re!(THREAD_FOLDER, r"^w-[a-f0-9]{10}$", "i");

/// A single source file's name, with or without a line after it: the file (`sourceReference`).
pub fn source_reference(value: &str) -> Option<String> {
    if value.contains(['\r', '\n']) {
        return None;
    }
    let file = SOURCE.exec(super::js::trim(value))?.get(1)?;
    (!file.split('/').any(|part| part == "..")).then(|| file.to_string())
}

fn is_address(value: &str) -> bool {
    ADDRESS.test(value) && !FILE_SCHEME.test(value)
}

/// `mentionKind`.
pub fn mention_kind(target: &str) -> Kind {
    let value = super::js::trim(target);
    if value.is_empty() {
        return Kind::File;
    }
    if is_address(value) {
        return Kind::Link;
    }
    if value.ends_with('/') {
        return Kind::Folder;
    }
    let bare = AFTER_NAME.strip(value);
    if PAGE.test(&bare) {
        Kind::Page
    } else if DOC.test(&bare) {
        Kind::Doc
    } else if IMAGE.test(&bare) {
        Kind::Image
    } else if is_media_path(value) {
        Kind::Media
    } else if source_reference(value).is_some() {
        Kind::Code
    } else {
        Kind::File
    }
}

/// `mentionName`: the file's name, the last folder of a folder, an address without its scheme.
pub fn mention_name(target: &str) -> String {
    let value = super::js::trim(target);
    if MAILTO.test(value) {
        return value["mailto:".len()..].to_string();
    }
    if is_address(value) {
        return SCHEME_SLASHES.strip(value).trim_end_matches('/').to_string();
    }
    let path = FILE_SLASHES.strip(value);
    let name = path.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    if name.is_empty() { value.to_string() } else { name.to_string() }
}

/// What a mention shows in place of what the worker wrote (`mentionLabel`). A path or an
/// address written out is shortened to the name it goes by; words the worker chose come back
/// None and stay theirs.
pub fn mention_label(target: &str, text: Option<&str>) -> Option<String> {
    let name = mention_name(target);
    let words = text.map(super::js::trim).filter(|w| !w.is_empty());
    let Some(words) = words.filter(|w| *w != target) else { return Some(name) };
    let written_out = |marks: &[char]| !HAS_SPACE.test(words) && words.contains(marks);
    if mention_kind(target) == Kind::Link {
        return written_out(&['/', '.', ':']).then_some(name);
    }
    if words.ends_with(&name) {
        return Some(name);
    }
    written_out(&['/', '.']).then(|| mention_name(words))
}

/// The name a folder goes by (`folderTitle`): a thread's own designs folder is "Designs for
/// this thread", another thread's carries that thread's title, anything else its own name.
pub fn folder_title(folder: &str, thread: Option<&str>, title: impl Fn(&str) -> Option<String>) -> String {
    let parts: Vec<&str> = folder.trim_end_matches('/').split('/').collect();
    let last = parts.last().copied().unwrap_or(folder);
    if parts.len() < 2 || parts[parts.len() - 2] != "designs" || !THREAD_FOLDER.test(last) {
        return last.to_string();
    }
    if Some(last) == thread {
        return "Designs for this thread".to_string();
    }
    title(last).filter(|t| !t.is_empty()).map_or_else(|| last.to_string(), |t| format!("Designs for {t}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests/a-file-or-folder-in-a-message-reads-as-a-quiet-mention.test.mts
    #[test]
    fn what_kind_of_thing_a_reference_is() {
        assert_eq!(mention_kind("designs/w-7a670f07bb/"), Kind::Folder);
        assert_eq!(mention_kind("legal/privacy.html"), Kind::Page);
        assert_eq!(mention_kind("decisions.md"), Kind::Doc);
        assert_eq!(mention_kind("report.pdf"), Kind::Doc);
        assert_eq!(mention_kind("designs/a.png"), Kind::Image);
        assert_eq!(mention_kind("main/phone-server.mjs"), Kind::Code);
        assert_eq!(mention_kind("renderer/src/App.tsx:12"), Kind::Code);
        assert_eq!(mention_kind("renders/demo.mp4"), Kind::Media);
        assert_eq!(mention_kind("https://example.com/kb/1153/"), Kind::Link);
        // An address is a link even when it ends in a file name.
        assert_eq!(mention_kind("https://example.com/a.png"), Kind::Link);
        assert_eq!(mention_kind("http://localhost:3000/page.html"), Kind::Link);
        assert_eq!(mention_kind("mailto:someone@example.com"), Kind::Link);
        // Anything it does not know is a plain file.
        assert_eq!(mention_kind("build/app.dmg"), Kind::File);
        assert_eq!(mention_kind("notes"), Kind::File);
        assert_eq!(mention_kind(""), Kind::File);
    }

    #[test]
    fn the_name_a_mention_shows() {
        assert_eq!(mention_name("legal/privacy.html"), "privacy.html");
        assert_eq!(mention_name("/Users/you/Work/designs/w-3a/latest.html"), "latest.html");
        assert_eq!(mention_name("decisions.md"), "decisions.md");
        // A line number is the point of naming one.
        assert_eq!(mention_name("renderer/src/App.tsx:12"), "App.tsx:12");
        assert_eq!(mention_name("main/x.mjs#L40"), "x.mjs#L40");
        assert_eq!(mention_name("designs/w-7a670f07bb/"), "w-7a670f07bb");
        assert_eq!(mention_name("https://example.com/kb/1153/"), "example.com/kb/1153");
        assert_eq!(mention_name("http://localhost:3000"), "localhost:3000");
        assert_eq!(mention_name("mailto:someone@example.com"), "someone@example.com");
    }

    #[test]
    fn the_words_a_mention_shows_in_place_of_what_was_written() {
        let film = "designs/w-bddbdfe747/intro-version-1.mp4";
        assert_eq!(mention_label(film, Some(&format!("/Users/you/Work/accounts/acct/video/{film}"))).as_deref(), Some("intro-version-1.mp4"));
        assert_eq!(mention_label(film, Some(film)).as_deref(), Some("intro-version-1.mp4"));
        assert_eq!(mention_label(film, None).as_deref(), Some("intro-version-1.mp4"));
        // A file name with a space in it stays whole.
        assert_eq!(mention_label("Sample Film.mp4", Some("Sample Film.mp4")).as_deref(), Some("Sample Film.mp4"));
        assert_eq!(mention_label("renders/Final Film.mp4", Some("/Users/you/renders/Final Film.mp4")).as_deref(), Some("Final Film.mp4"));
        // An address is named by the address, even when the words were its bare form.
        assert_eq!(mention_label("http://localhost:3002", Some("localhost:3002")).as_deref(), Some("localhost:3002"));
        assert_eq!(mention_label("http://www.example.com/a/b", Some("www.example.com/a/b")).as_deref(), Some("www.example.com/a/b"));
        // Words the worker chose are left alone.
        assert_eq!(mention_label("designs/demo.html", Some("the page")), None);
        assert_eq!(mention_label("missing.pdf", Some("the missing file")), None);
        assert_eq!(mention_label("https://example.com", Some("here")), None);
    }

    #[test]
    fn the_name_a_design_folder_goes_by() {
        let names = |id: &str| (id == "w-b222222222").then(|| "Phone access setup".to_string());
        assert_eq!(folder_title("designs/w-a111111111/", Some("w-a111111111"), names), "Designs for this thread");
        assert_eq!(folder_title("designs/w-b222222222/", Some("w-a111111111"), names), "Designs for Phone access setup");
        assert_eq!(folder_title("designs/w-c333333333/", Some("w-a111111111"), names), "w-c333333333");
        assert_eq!(folder_title("designs/w-a111111111/round-2/", Some("w-a111111111"), names), "round-2");
        assert_eq!(folder_title("renders/w-a111111111/", Some("w-a111111111"), names), "w-a111111111");
    }

    #[test]
    fn a_source_file_with_or_without_a_line() {
        assert_eq!(source_reference("cart.mjs:2").as_deref(), Some("cart.mjs"));
        assert_eq!(source_reference("src/rules/paths.rs#L40").as_deref(), Some("src/rules/paths.rs"));
        assert_eq!(source_reference("roundTax()"), None);
        assert_eq!(source_reference("../secrets/keys.json"), None);
        assert_eq!(source_reference("npm run ship"), None);
    }
}
