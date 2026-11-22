//! What a conversation's reply box says folded shut: the person's own words when there are
//! any, the files in it counted in words, and the invitation only when there is nothing under
//! it (`foldedReply` in `renderer/src/folded-reply.ts`).

/// How much of the words the folded box carries before it trails off (`FOLDED_BUDGET`).
pub const BUDGET: usize = 100;

/// What the folded box says (`FoldedReply`).
#[derive(Clone, Debug, PartialEq)]
pub struct Folded {
    /// Whether something of the person's is under the fold, which decides the box's voice.
    pub draft: bool,
    /// Its line: the words, else the invitation, or the files when there are no words.
    pub text: String,
    /// The files beside the words, counted, or nothing.
    pub files: String,
}

/// One line out of however many were written.
fn one_line(words: &str) -> String {
    words.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Cut on a word, never mid-word, saying there is more; a long unbroken word is cut where the
/// budget ends rather than left as three letters (`clip`).
fn clip(line: &str, budget: usize) -> String {
    if line.chars().count() <= budget {
        return line.to_string();
    }
    let cut: String = line.chars().take(budget).collect();
    let space = cut.rfind(' ').map(|at| cut[..at].chars().count());
    let kept = match space {
        Some(at) if at as f32 > budget as f32 * 0.6 => cut.chars().take(at).collect::<String>(),
        _ => cut,
    };
    format!("{}…", kept.trim_end())
}

/// The files in the box, in the words the person would use: images when every one is a
/// picture, else files (`filesSaid`). A file with no name does not count.
fn files_said(files: &[(&str, bool)]) -> String {
    let kept: Vec<_> = files.iter().filter(|(name, _)| !name.is_empty()).collect();
    if kept.is_empty() {
        return String::new();
    }
    let noun = if kept.iter().all(|(_, image)| *image) { "image" } else { "file" };
    format!("{} {noun}{}", kept.len(), if kept.len() == 1 { "" } else { "s" })
}

/// What the folded box says over `words` and `files` (each a name and whether it is a picture).
pub fn folded_reply(words: &str, files: &[(&str, bool)]) -> Folded {
    let line = one_line(words);
    let files = files_said(files);
    if line.is_empty() && files.is_empty() {
        return Folded { draft: false, text: "Reply…".to_string(), files: String::new() };
    }
    // Pictures with nothing typed over them: the count is the message, in the words' place.
    if line.is_empty() {
        return Folded { draft: true, text: files, files: String::new() };
    }
    Folded { draft: true, text: clip(&line, BUDGET), files }
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests/a-folded-reply-box-shows-what-is-in-it.test.mts
    #[test]
    fn a_box_never_typed_in_still_invites_a_reply() {
        assert_eq!(folded_reply("", &[]), Folded { draft: false, text: "Reply…".into(), files: String::new() });
        assert!(!folded_reply("   \n\n  ", &[]).draft);
    }

    #[test]
    fn a_box_part_way_through_says_the_words_back_on_one_line() {
        let hers = "Great. Now our code viewer has really bad UX. I just am not sure how to use it.";
        let folded = folded_reply(hers, &[]);
        assert!(folded.draft);
        assert_eq!(folded.text, hers);
        assert_eq!(folded_reply("First thought.\n\nSecond thought.", &[]).text, "First thought. Second thought.");
        // It trails off on a word, never mid-word.
        let long = vec!["sentence"; 40].join(" ");
        let text = folded_reply(&long, &[]).text;
        assert!(text.chars().count() <= BUDGET + 1 && text.ends_with('…'));
        assert!(text.trim_end_matches('…').split(' ').all(|w| w == "sentence"));
        // A sentence that fits exactly is kept as typed.
        let fits = "x".repeat(BUDGET);
        assert_eq!(folded_reply(&fits, &[]).text, fits);
        // A long unbroken word is not cut to almost nothing.
        let url = format!("See https://example.com/{}", "a".repeat(200));
        assert!(folded_reply(&url, &[]).text.chars().count() as f32 > BUDGET as f32 * 0.9);
    }

    #[test]
    fn pictures_with_no_words_are_the_message_and_files_ride_beside_words() {
        assert_eq!(folded_reply("", &[("pasted-35385.png", true)]), Folded { draft: true, text: "1 image".into(), files: String::new() });
        assert_eq!(folded_reply("", &[("one.png", true), ("two.png", true)]).text, "2 images");
        assert_eq!(folded_reply("", &[("notes.pdf", false)]).text, "1 file");
        assert_eq!(folded_reply("", &[("shot.png", true), ("notes.pdf", false)]).text, "2 files");
        let folded = folded_reply("Here is the screen.", &[("shot.png", true)]);
        assert_eq!((folded.text.as_str(), folded.files.as_str()), ("Here is the screen.", "1 image"));
        assert!(!folded_reply("", &[("", true)]).draft, "a nameless one does not count");
    }
}
