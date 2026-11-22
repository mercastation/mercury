//! A person's message, split into what they typed and what the app attached to it
//! (`typedParts` in `renderer/src/typed-message.ts`). A person's words are drawn as typed,
//! never through markdown. But the app writes every picture or file attached to a message into
//! its words as markdown, one to a line (`attachmentLinks`, `rules/attach.rs`), and drawn as
//! typed a pasted screenshot reads as its own brackets. So those lines are lifted out, to be
//! drawn as the picture or the link they are: a picture standing alone on its line, quoted or
//! not; a picture from `attachments/` anywhere on a line; and a file linked from `attachments/`
//! alone on its line. The same syntax in backticks is words, and so is a link a person typed to
//! anywhere else: the app writes neither.

use crate::rules::js::{self, re};

/// One part of a person's message.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    /// Words, drawn as typed.
    Text(String),
    /// A line the app wrote, drawn as markdown.
    Embed(String),
}

re!(PICTURE, r"^!\[[^\]\n]*\]\([^)\s]+\)$", "");
re!(ATTACHED_FILE, r"^\[[^\]\n]*\]\((?:attachments/[^)\s]+|phone-files/[0-9a-f]{64}\.[a-z0-9]{1,8})\)$", "");
// A line that is only a thread's link, as the app writes one into a brief.
re!(THREAD_WEB_LINK, r"^\[[^\]\n]+\]\(https?://[^)\s]+/t/[^/)\s]+/w-[a-f0-9]{10}\)$", "");
re!(QUOTE_MARK, r"^\s*(?:>\s?)+", "");
re!(ATTACHED_PICTURE, r"(?<!`)!\[[^\]\n]*\]\((?:\./)?attachments/[^)\s]+\)(?!`)", "");

fn is_embed(line: &str) -> bool {
    let line = js::trim(line);
    PICTURE.test(line) || ATTACHED_FILE.test(line) || THREAD_WEB_LINK.test(line)
}

/// The words and the lines the app wrote, in order (`typedParts`). A message with nothing
/// attached is one part, exactly as it came.
pub fn typed_parts(text: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut words: Vec<String> = Vec::new();
    // A line drawn on its own says the blank lines that set it apart from the words already;
    // inside the words every break stays where it was typed.
    let flush = |words: &mut Vec<String>, parts: &mut Vec<Part>| {
        let said = words.join("\n");
        let said = said.trim_matches('\n');
        if !js::trim(said).is_empty() {
            parts.push(Part::Text(said.to_string()));
        }
        words.clear();
    };
    for line in text.split('\n') {
        let bare = QUOTE_MARK.strip(line);
        if is_embed(&bare) {
            flush(&mut words, &mut parts);
            parts.push(Part::Embed(js::trim(&bare).to_string()));
            continue;
        }
        let mut at = 0;
        for caps in ATTACHED_PICTURE.each(line) {
            let Some((start, end)) = caps.span(0) else { continue };
            words.push(js::trim(&line[at..start]).to_string());
            flush(&mut words, &mut parts);
            parts.push(Part::Embed(line[start..end].to_string()));
            at = end;
        }
        words.push(if at > 0 { js::trim(&line[at..]).to_string() } else { line.to_string() });
    }
    flush(&mut words, &mut parts);
    if !parts.iter().any(|part| matches!(part, Part::Embed(_))) {
        return vec![Part::Text(text.to_string())];
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests/a-picture-you-paste-into-a-message-is-drawn.test.mts
    const SHOT: &str = "![pasted-69703.png](attachments/muw39o3w-vitsyq-pasted-69703.png)";

    fn text(s: &str) -> Part {
        Part::Text(s.to_string())
    }
    fn embed(s: &str) -> Part {
        Part::Embed(s.to_string())
    }

    #[test]
    fn a_picture_you_sent_is_lifted_out_of_your_words_and_the_words_stay_as_typed() {
        let words = "I feel like it's not fixed. I'm not sure why it's not presenting fixed locally.";
        assert_eq!(typed_parts(&format!("{words}\n\n{SHOT}")), [text(words), embed(SHOT)]);
        assert_eq!(typed_parts(SHOT), [embed(SHOT)]);
        let two = "look\n\n![a.png](attachments/x1-a.png)\n![b.png](attachments/x2-b.png)";
        assert_eq!(typed_parts(two), [text("look"), embed("![a.png](attachments/x1-a.png)"), embed("![b.png](attachments/x2-b.png)")]);
        assert_eq!(typed_parts("notes attached\n\n[notes.pdf](attachments/x3-notes.pdf)"), [text("notes attached"), embed("[notes.pdf](attachments/x3-notes.pdf)")]);
        // What this window sends with a file and a picture and no words under the title.
        let sent = "[week-32.pdf](attachments/mwrdo357-mpr3io-week-32.pdf)\n![pasted-89459.png](attachments/mwrdo357-q3890i-pasted-89459.png)";
        assert_eq!(typed_parts(sent), [embed("[week-32.pdf](attachments/mwrdo357-mpr3io-week-32.pdf)"), embed("![pasted-89459.png](attachments/mwrdo357-q3890i-pasted-89459.png)")]);
    }

    #[test]
    fn what_the_app_never_writes_stays_exactly_as_typed() {
        let cmd = "grep -r foo . --include=*.md --include=*.json";
        assert_eq!(typed_parts(&format!("{cmd}\n\n{SHOT}")), [text(cmd), embed(SHOT)]);
        for typed in ["the line `![x](attachments/y.png)` showed instead of the picture", "the docs:\n[guide](https://example.com/guide)", "one\ntwo\n\nthree", "trailing breaks stay\n\n"] {
            assert_eq!(typed_parts(typed), [text(typed)], "{typed}");
        }
    }

    #[test]
    fn where_the_words_end_and_a_picture_begins() {
        assert_eq!(typed_parts(&format!("hello\n\n{SHOT}")), [text("hello"), embed(SHOT)]);
        assert_eq!(typed_parts(&format!("{SHOT}\nand this")), [embed(SHOT), text("and this")]);
        assert_eq!(typed_parts(&format!("see {SHOT}")), [text("see"), embed(SHOT)]);
        // Quoted, it is still the picture.
        assert_eq!(typed_parts(&format!("> {SHOT}")), [embed(SHOT)]);
    }
}
