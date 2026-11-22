//! The words of an answer, read: the options it offers, whether that offer still stands, and
//! its text without the markdown. A port of the parts of `renderer/src/format.ts` the list
//! rules stand on.

use super::js::{self, group, re};
use crate::data::Item;

/// One choice an answer offers (`ParsedOption`).
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub n: f64,
    pub text: String,
    pub recommended: bool,
}

re!(HEADING, r"^#{1,4}\s", "");
re!(OPTION_WORD, r"option", "i");
re!(NUMBERED, r"^\s*(\d+)[.)]\s+(.*)$", "");
re!(RECOMMENDED, r"\(recommended\)", "i");

fn is_options_heading(line: &str) -> bool {
    HEADING.test(line) && OPTION_WORD.test(line)
}

/// The worker brief asks for a "## Options" section with a numbered list, "(recommended)"
/// marking the default. Parsing is tolerant; a body without options simply reads as prose.
pub fn parse_options(body: Option<&str>) -> Vec<Choice> {
    let Some(body) = body.filter(|b| !b.is_empty()) else { return Vec::new() };
    let lines: Vec<&str> = body.split('\n').collect();
    if !lines.iter().any(|l| is_options_heading(l)) {
        return headless_options(&lines).map(|h| h.options).unwrap_or_default();
    }
    let mut in_options = false;
    let mut options = Vec::new();
    for line in lines {
        if HEADING.test(line) {
            in_options = OPTION_WORD.test(line);
            continue;
        }
        if !in_options {
            continue;
        }
        if let Some(m) = NUMBERED.exec(line) {
            options.push(Choice { n: number(group(&m, 1)), text: js::trim(group(&m, 2)).to_string(), recommended: RECOMMENDED.test(group(&m, 2)) });
        }
    }
    options
}

/// `Number(digits)`.
fn number(digits: &str) -> f64 {
    digits.parse().unwrap_or(f64::NAN)
}

// Agents do not always write the heading. Without it, a numbered list counts as an offer only
// when it closes the message, runs 1, 2, 3..., and marks one choice recommended, so a list of
// steps stays prose.
re!(ITEM, r"^ {0,3}(\d+)[.)]\s+(.*)$", "");
re!(LEADING_MARK, r"^(?:\*\*|__)?\[?recommended(?:\]|\s*[:\p{Pd}]|\*\*|__)\s*(?:\*\*|__)?\s*[:\p{Pd}]?\s*", "iu");
re!(TRAILING_MARK, r"\s*(?:\p{Pd}\s*)?(?:\*\*|__)?[([]recommended\b[^)\]]*[)\]](?:\*\*|__)?\s*$|\s*\p{Pd}\s*(?:\*\*|__)?recommended(?:\*\*|__)?\s*$", "iu");
re!(INDENTED, r"^\s{2,}\S", "");

struct Headless {
    options: Vec<Choice>,
}

fn headless_options(lines: &[&str]) -> Option<Headless> {
    let last = lines.iter().rposition(|l| ITEM.test(l))?;
    let continues = |l: &str| js::trim(l).is_empty() || INDENTED.test(l);
    let mut end = last + 1;
    while end < lines.len() && continues(lines[end]) {
        end += 1;
    }
    let after: Vec<&&str> = lines[end..].iter().filter(|l| !js::trim(l).is_empty()).collect();
    if after.len() > 2 || after.iter().any(|l| HEADING.test(l)) {
        return None;
    }
    let mut start = last;
    for i in (0..last).rev() {
        if !(ITEM.test(lines[i]) || continues(lines[i])) {
            break;
        }
        if ITEM.test(lines[i]) {
            start = i;
        }
    }
    let mut options = Vec::new();
    for line in &lines[start..end] {
        let Some(m) = ITEM.exec(line) else { continue };
        let text = js::trim(group(&m, 2));
        let bare = LEADING_MARK.strip(text);
        let marked = bare != text || TRAILING_MARK.test(text);
        let words = if marked { format!("{} (recommended)", TRAILING_MARK.strip(&bare)) } else { text.to_string() };
        options.push(Choice { n: number(group(&m, 1)), text: words, recommended: marked });
    }
    if options.len() < 2 || options.iter().enumerate().any(|(i, o)| o.n != (i + 1) as f64) || !options.iter().any(|o| o.recommended) {
        return None;
    }
    Some(Headless { options })
}

/// Which field a pick lives in: whatever the pane is leading with, the result, else the
/// checkpoint, else the ask. When both the result and the checkpoint offer, the newer one wins.
pub fn options_from(item: &Item) -> &'static str {
    let in_result = !parse_options(item.result.as_deref()).is_empty();
    let in_note = !parse_options(item.text("note")).is_empty();
    if in_result && in_note {
        return if item.wrote_at("note") > item.wrote_at("result") { "note" } else { "result" };
    }
    if in_result {
        "result"
    } else if in_note {
        "note"
    } else {
        "body"
    }
}

fn field<'a>(item: &'a Item, name: &str) -> Option<&'a str> {
    match name {
        "result" => item.result.as_deref(),
        "body" => item.body.as_deref(),
        _ => item.text(name),
    }
}

/// The choices a thread offers now. On a team, a reviewer who asked for changes takes the
/// recommendation off them (`changesAskedSince`); that reads a teammate's hand-back, so it
/// would go here.
pub fn item_options(item: &Item) -> Vec<Choice> {
    parse_options(field(item, options_from(item)))
}

re!(RECOMMENDED_WORDS, r"\s*\(recommended\)\s*", "i");
re!(CLOSING_STOPS, r"[.!]+$", "");
re!(LEADING_MARKS, r"^[*_\s]+", "");
// The option that IS the decision to ship: "Ship it", "Let it ship as built", "Not now, just
// ship this", "Merge it". Leading words only.
re!(SHIPS, r"^(?:(?:yes|no|not now|not yet|ok(?:ay)?)\s*[,:;\p{Pd}]?\s+)?(?:just\s+)?(?:ship\b|merge (?:it|this)\b|let (?:it|this|them|the\b[^,.;:]*?)\s+(?:ship|land)\b)", "iu");

fn bare_option(option: &str) -> String {
    CLOSING_STOPS.strip(js::trim(&RECOMMENDED_WORDS.strip(option)))
}

fn asks_to_ship(option: &str) -> bool {
    SHIPS.test(&LEADING_MARKS.strip(&bare_option(option)))
}

// What a pick does when it is not a reply (`renderer/src/format.ts`). The words alone, or with
// a reason after them: "Ship it, then close the task" is still a reply.
re!(CLOSES, r"^close (?:this|the) (?:task|thread)(?:$|\s*[,:;\p{Pd}](?!\s*(?:then|and|also)\b)|\s+(?:since|because|as|now that)\b)", "iu");
re!(KEEPS_AND_CLOSES, r"^(?:keep|leave) (?:it|this|them|that)(?: as (?:is|sent|it is))?\s*,?\s+and close (?:this|the) (?:task|thread)$", "i");
re!(RIGHT_FIRST, r"^(?:(?:that|it|this|all)\s+)?(?:looks\s+|is\s+)?(?:right|good|fine|great|perfect|correct)\s*[,.;:!]?\s+(?:and\s+|so\s+)?(?=close\b)", "i");
re!(STAYS_ON_BRANCH, r"^(?:keep|leave)\s+(?:(?:it|this|them|that|the (?:change|changes|work|commits?))\s+)?(?:on|in)\s+(?:(?:the|its|this|a)\s+)?branch(?:\s+for now)?$", "i");
re!(SAYS_NOTHING, r"^(?:change|revise|edit|adjust|tweak|amend|rework|redo|fix)\s+(?:something|anything|it|this|that|some of it|a few things|this (?:part|bit)|parts?|bits?)(?:\s+first)?$", "i");
re!(NAMES_NOBODY, r"^commit it and ship, as the name and email below$", "i");
re!(SOMETHING_ELSE, r"^(?:something|anything) else$", "i");
re!(DISCARDS, r"^(?:discard|bin|delete|drop|throw away|throw out)\s+(?:it|them|this|that|the (?:change|changes|work|edits|diff|lot))(?:\s+away)?$|^(?:discard|throw (?:it|them|the (?:change|changes|lot)) (?:all )?away)$", "i");

/// The option that closes the thread: a pick is a reply, and a reply on a finished row starts
/// a run, so this one is done the way E does it instead (`closesTheTask`).
pub fn closes_the_task(option: &str) -> bool {
    let words = RIGHT_FIRST.strip(&bare_option(option));
    CLOSES.test(&words) || KEEPS_AND_CLOSES.test(&words)
}

/// "Keep it on the branch" closes too, and leaves the work off main on purpose (`leavesItOnTheBranch`).
pub fn leaves_it_on_the_branch(option: &str) -> bool {
    STAYS_ON_BRANCH.test(&bare_option(option))
}

/// The option that throws the work away, which asks first (`discardsTheChange`).
pub fn discards_the_change(option: &str) -> bool {
    DISCARDS.test(&bare_option(option))
}

/// The option that says nothing without the person's own words, "Change something first": it
/// opens the reply box with its words in it instead of sending them (`needsYourWords`).
pub fn needs_your_words(option: &str) -> bool {
    let words = bare_option(option);
    SAYS_NOTHING.test(&words) || NAMES_NOBODY.test(&words) || SOMETHING_ELSE.test(&words)
}

re!(SHIPPED_NOTE, r"^Shipped to main(?: as [0-9a-f]{7,40})?\.$", "");

/// The note the app writes on a row once its ship lands (`shared/shipped-note.mts`).
pub fn is_shipped_note(text: Option<&str>) -> bool {
    SHIPPED_NOTE.test(js::trim(text.unwrap_or("")))
}

/// Whether the offer is still live: written after the person's last word, and not spent by a
/// newer message of the agent's that offers nothing of its own. The app's own "Shipped to
/// main" note answers an offer only when the offer asked whether to ship.
pub fn offer_is_live(item: &Item) -> bool {
    let options = item_options(item);
    if options.is_empty() {
        return false;
    }
    let from = options_from(item);
    let offered = item.wrote_at(from);
    let later: &[&str] = if from == "body" { &["result"] } else { &["result", "note"] };
    let after_the_ship = is_shipped_note(item.text("note")) && !options.iter().any(|o| asks_to_ship(&o.text));
    if later.iter().any(|f| *f != from && item.wrote_at(f) > offered && !(*f == "note" && after_the_ship)) {
        return false;
    }
    if item.live_answer().is_none() {
        return true;
    }
    // A teammate's words handed back to you are not your pick.
    if item.wrote("answer").is_some_and(|w| w.teammate.is_some()) {
        return true;
    }
    offered > item.wrote_at("answer")
}

re!(HEADING_LINE, r"^#{1,4}\s.*$", "gm");
re!(HEADING_HASHES, r"^#{1,4}\s*", "");
re!(PICTURE, r"!\[[^\]]*\]\([^)]*\)", "g");
re!(LINK, r"\[([^\]]*)\]\([^)]*\)", "g");
// A cloud run's key ask reads as what it asks for. Its other asks are cards in the thread, and
// machine words anywhere else.
re!(NEED_LINE, r"^\s*`?AGENTBOX-NEEDS:\s*(sign-in|key|file|code)\s.*$", "gim");
re!(KEY_LINE, r"^\s*`?AGENTBOX-NEEDS-A-KEY:\s*([A-Z][A-Z0-9_]+)\b[\s:\u2014\u2013-]*(.*?)`?\s*$", "gm");
/// The marks of markdown, out: `text.replace(/[*`>#]|(?<![A-Za-z0-9])_+|_+(?![A-Za-z0-9])/g, '')`.
/// An underscore inside a word is part of it (STRIPE_SECRET_KEY), not emphasis: read closely,
/// that regex takes every run of underscores except a single one between two ASCII letters or
/// digits. It is written out by hand because it is run over every answer in every list, and
/// the engine that can look behind took three quarters of the time the lists took.
fn strip_marks(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut k = 0;
    while k < bytes.len() {
        match bytes[k] {
            b'*' | b'`' | b'>' | b'#' => k += 1,
            b'_' => {
                let start = k;
                while bytes.get(k) == Some(&b'_') {
                    k += 1;
                }
                let inside_a_word = start > 0 && bytes[start - 1].is_ascii_alphanumeric() && bytes.get(k).is_some_and(u8::is_ascii_alphanumeric);
                if inside_a_word && k - start == 1 {
                    out.push(b'_');
                }
            }
            other => {
                out.push(other);
                k += 1;
            }
        }
    }
    // Only whole ASCII characters were taken out, so what is left is still text.
    String::from_utf8(out).expect("removing ASCII marks keeps text whole")
}

/// Markdown out, one line in. One copy of this rule, because search reads the same text the
/// row does. A heading is dropped for a preview and kept for search.
fn flatten(body: &str, keep_headings: bool) -> String {
    let headed = HEADING_LINE.replace_all(body, |c| if keep_headings { HEADING_HASHES.strip(group(c, 0)) } else { String::new() });
    let linked = LINK.replace_all(&PICTURE.strip_all(&headed), |c| group(c, 1).to_string());
    let asked = KEY_LINE.replace_all(&NEED_LINE.strip_all(&linked), |c| if group(c, 2).is_empty() { group(c, 1).to_string() } else { group(c, 2).to_string() });
    strip_marks(&asked).split('\n').map(js::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
}

pub fn plain_text(body: &str) -> String {
    if body.is_empty() { String::new() } else { flatten(body, false) }
}

/// The same text, with the headings still in it. Search reads this.
pub fn search_text(body: &str) -> String {
    if body.is_empty() { String::new() } else { flatten(body, true) }
}

/// A row's preview: the plain text, to its 200-character ceiling.
pub fn preview_text(body: &str) -> String {
    js::slice(&plain_text(body), 0, 200).to_string()
}

const LABEL_CHARS: usize = 32;

/// The first line of a message that is a line rather than a heading for one. A label is short,
/// ends in a colon and has nothing after it; a message that is nothing but a label prints it.
pub fn first_real_line(text: &str) -> String {
    let lines: Vec<String> = text.split('\n').map(plain_text).filter(|l| !l.is_empty()).collect();
    let real = lines.iter().find(|l| !(js::len(l) <= LABEL_CHARS && l.ends_with(':')));
    real.or(lines.first()).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn options_under_their_heading() {
        let body = "Two ways to fix the slot race.\n\n## Options\n1. Lock the slot (recommended)\n2) Queue the second booking\n\nSay which.";
        let options = parse_options(Some(body));
        assert_eq!(options.len(), 2);
        assert_eq!(options[0], Choice { n: 1.0, text: "Lock the slot (recommended)".into(), recommended: true });
        assert!(!options[1].recommended);
        assert!(parse_options(Some("Steps taken:\n1. Read the log\n2. Fixed the test")).is_empty());
        assert!(parse_options(None).is_empty());
    }

    #[test]
    fn a_closing_list_with_a_recommended_choice_is_an_offer_without_its_heading() {
        let body = "The export is fixed.\n\n1. Ship it — recommended\n2. Change the file name first";
        let options = parse_options(Some(body));
        assert_eq!(options.iter().map(|o| o.text.as_str()).collect::<Vec<_>>(), ["Ship it (recommended)", "Change the file name first"]);
        // A list that does not start at one, or recommends nothing, stays prose.
        assert!(parse_options(Some("2. Ship it (recommended)\n3. Wait")).is_empty());
        assert!(parse_options(Some("1. Ship it\n2. Wait")).is_empty());
        // And so does one with more than two lines of prose after it.
        assert!(parse_options(Some("1. Ship it (recommended)\n2. Wait\n\nOne.\n\nTwo.\n\nThree.")).is_empty());
    }

    #[test]
    fn an_offer_is_live_until_it_is_answered_or_spoken_past() {
        let offer = "Done.\n\n## Options\n1. Ship it (recommended)\n2. Change the copy first";
        let fresh = item(json!({ "result": offer, "wrote": { "result": { "ts": 20, "source": "agent" } } }));
        assert!(offer_is_live(&fresh));
        let answered = item(json!({ "result": offer, "answer": "Ship it", "wrote": { "result": { "ts": 20, "source": "agent" }, "answer": { "ts": 30, "source": "founder" } } }));
        assert!(!offer_is_live(&answered));
        let offered_again = item(json!({ "result": offer, "answer": "Ship it", "wrote": { "result": { "ts": 40, "source": "agent" }, "answer": { "ts": 30, "source": "founder" } } }));
        assert!(offer_is_live(&offered_again));
        let withdrawn = item(json!({ "result": offer, "answer": "(withdrawn)", "wrote": { "result": { "ts": 20, "source": "agent" }, "answer": { "ts": 30, "source": "founder" } } }));
        assert!(offer_is_live(&withdrawn));
        // A later checkpoint that offers nothing means the offer was acted on.
        let spent = item(json!({ "result": offer, "note": "Copy changed, tests pass.", "wrote": { "result": { "ts": 20, "source": "agent" }, "note": { "ts": 25, "source": "agent" } } }));
        assert!(!offer_is_live(&spent));
        // The app's own ship note answers only an offer that asked whether to ship.
        let shipped = item(json!({ "result": "Done.\n\n## Options\n1. Close this task (recommended)\n2. Change the copy first", "note": "Shipped to main as 0a1b2c3.", "wrote": { "result": { "ts": 20, "source": "agent" }, "note": { "ts": 25, "source": "system" } } }));
        assert!(offer_is_live(&shipped));
        let asked_to_ship = item(json!({ "result": offer, "note": "Shipped to main as 0a1b2c3.", "wrote": { "result": { "ts": 20, "source": "agent" }, "note": { "ts": 25, "source": "system" } } }));
        assert!(!offer_is_live(&asked_to_ship));
    }

    #[test]
    fn an_option_that_closes_the_thread_is_a_close_and_one_that_asks_for_work_is_a_reply() {
        assert!(closes_the_task("Close this task"));
        assert!(closes_the_task("Close the thread (recommended)"));
        assert!(closes_the_task("Close this task, since it shipped"));
        assert!(closes_the_task("Looks right, close this task"));
        assert!(closes_the_task("Keep it as sent and close this task."));
        // Work first, or work after: still a reply.
        assert!(!closes_the_task("Ship it, then close the task"));
        assert!(!closes_the_task("Close this task, then file the follow-up"));
        assert!(!closes_the_task("Keep the first layout and close this task"));
    }

    #[test]
    fn keeping_it_on_the_branch_is_a_close_that_leaves_the_work_off_main() {
        assert!(leaves_it_on_the_branch("Keep it on the branch"));
        assert!(leaves_it_on_the_branch("Leave the changes on the branch for now."));
        assert!(!leaves_it_on_the_branch("Keep the branch and open a pull request"));
    }

    #[test]
    fn throwing_the_work_away_is_the_whole_option_or_none_of_it() {
        assert!(discards_the_change("Discard it"));
        assert!(discards_the_change("Throw the changes away"));
        assert!(!discards_the_change("Discard the first layout and ship the second"));
    }

    #[test]
    fn an_option_that_names_nothing_to_change_needs_the_persons_words() {
        assert!(needs_your_words("Change something first"));
        assert!(needs_your_words("Something else"));
        assert!(needs_your_words("Commit it and ship, as the name and email below"));
        assert!(!needs_your_words("Change the empty state copy first"));
    }

    #[test]
    fn markdown_out_one_line_in() {
        assert_eq!(plain_text("## What I found\n\nThe **slot race** is in `book()`.\n> see [the log](https://example.com/log) ![shot](a.png)"), "The slot race is in book(). see the log");
        assert_eq!(search_text("## What I found\nThe fix"), "What I found The fix");
        assert_eq!(plain_text("Set STRIPE_SECRET_KEY, _then_ retry__"), "Set STRIPE_SECRET_KEY, then retry");
        assert_eq!(plain_text("AGENTBOX-NEEDS: sign-in vercel | to deploy\nDeploy waits."), "Deploy waits.");
        assert_eq!(plain_text("`AGENTBOX-NEEDS-A-KEY: SENTRY_AUTH_TOKEN — a Sentry auth token`"), "a Sentry auth token");
    }

    #[test]
    fn the_marks_come_out_exactly_as_the_web_apps_regex_takes_them() {
        let marks = crate::rules::js::Re::new(r"[*`>#]|(?<![A-Za-z0-9])_+|_+(?![A-Za-z0-9])", "g");
        for text in [
            "", "_", "__", "a_b", "a__b", "a___b_c", "_a_", "__init__", "snake_case_name", "STRIPE_SECRET_KEY", "a_", "_a", "a_1", "1_a", "a _ b", "x_ y _z",
            "*bold* `code` > quote # heading", "é_é", "a_é", "é_a", "a__é", "_ação_ e __isso__", "a_\n_b", "🚀_x_🚀", "a_b_c__d___e", "__a__b__", "#_*_`_>",
        ] {
            assert_eq!(strip_marks(text), marks.strip_all(text), "{text:?}");
        }
        assert_eq!(strip_marks("a__b and a_b"), "ab and a_b");
    }

    #[test]
    fn a_label_is_not_what_was_said() {
        assert_eq!(first_real_line("Additional:\nThe second clinic double books too."), "The second clinic double books too.");
        assert_eq!(first_real_line("Additional:"), "Additional:");
        assert_eq!(first_real_line("Here is what I found in the three files I read this morning:\nNothing."), "Here is what I found in the three files I read this morning:");
        assert_eq!(first_real_line(""), "");
    }
}
