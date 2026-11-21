//! What a thread is called: a few words naming the thing being worked on, never the sentence
//! that opened it. A port of `shared/thread-title.mts`, table for table, and of `rowTitle` in
//! `renderer/src/list-rules.ts`.

use std::collections::HashSet;
use std::sync::LazyLock;

use super::format::search_text;
use super::js::{self, group, re};
use crate::data::Item;

const TITLE_WORDS: usize = 6;
// A first clause this short is the name, whole.
const CLAUSE_WORDS: usize = 7;
// A cut never keeps more than this.
const MOST_WORDS: usize = 8;

// Opening words that are manners or a request, not the work.
re!(GREETING, r"^(?:(?:hey|hi|hello|ok|okay|so|right)\b[^,.!?]{0,24}[,!]\s*)+", "i");
// Spoken openers a voice note leaves with no comma after them. "So" goes only before words a
// sentence starts on, so "So many errors" keeps it.
re!(FILLER, r"^(?:(?:ok(?:ay)?|alright|um+|uh+|basically|so(?=[,!]?\s+(?:basically|the|a|an|i|we|my|our|this|that|it|there|when|if)\b))[,!]?\s+)+", "i");
re!(ASK, r"^(?:(?:can|could|would|will) (?:you|we|someone)(?: please)?|please|pls|i (?:need|want|would like) (?:you )?to|we (?:need|have) to|i'd like (?:you )?to|let's|lets|go ahead and|help me)\s+", "i");
re!(CANNOT, r"^(?:i|we|they|it)(?: still)? (?:can't|cannot|can no longer|couldn't|am unable to|are unable to|is unable to)\s+", "i");
re!(LOOK, r"^(?:look (?:at|into)|find out|figure out|check|see)\s+(?=(?:why|how|what|whether|if)\b)", "i");
// "say they got X" reports X: the reporting words go and X stays.
re!(REPORTED, r"\b(?:say|says|said|report|reports|reported|tell us|told us|mention|mentions|mentioned)\s+(?:that\s+)?(?:they|he|she|we)\s+", "i");

fn set(list: &'static str) -> HashSet<&'static str> {
    list.split_whitespace().collect()
}

// Words a title never ends on: a name stops before them, not on them.
static TAIL: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set("a an the of to for at on in into with by from and or but nor so as than that which who
  is are was were be been being am do does did has have had
  i we you he she they it its my our your their his her this these those there here when while because since if")
});
// Where a whole clause ends: the name ends there. Otherwise it keeps its six words.
static BREAK: LazyLock<HashSet<&str>> = LazyLock::new(|| set("and but because since which that when while if so"));
// A phrase starts at these, so a cut just before one leaves the phrase before it whole.
static CUT_BEFORE: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set("at on in into onto for with without by from about after before until during over under
  off across between per via as to and or but so because since which that when while if where whether unless though although")
});
// Only for a title the six words cut short: a clause these open cannot be finished, and a last
// word of these only leads to the words that were cut.
static OPENS: LazyLock<HashSet<&str>> = LazyLock::new(|| set("whether if that because before after until unless when while where why how what which who"));
static LEADS: LazyLock<HashSet<&str>> = LazyLock::new(|| {
    set("whether before after until unless about off out up over going getting got get gets say says said
  think thinks ask asks asked tell tells told want wants need needs make makes made going trying try tries")
});
static SUBJECT: LazyLock<HashSet<&str>> = LazyLock::new(|| set("i we you he she they it"));

/// A word as the tables know it: lower case, and only its ASCII letters and apostrophes. An
/// accented letter drops out, as it does in the web app, so "são" is looked up as "so".
fn bare(w: &str) -> String {
    w.to_lowercase().chars().filter(|c| c.is_ascii_lowercase() || *c == '\'').collect()
}

re!(QUOTE_MARK, r"^\s*(?:>\s?)+", "");
re!(LIST_MARK, r"^\s*(?:[-*•]|\d{1,2}[.)])\s+", "");
re!(HEADING_MARK, r"^#+\s+", "");
re!(EMPHASIS, r"\*\*([^*]+)\*\*|__([^_]+)__|`([^`]+)`", "g");
re!(LINK, r"\[([^\]]+)\]\([^)]*\)", "g");
re!(OUTER_QUOTES, r#"^["'“”‘’]+|["'“”‘’]+$"#, "g");
re!(SPACES, r"\s+", "g");

fn plain(text: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|l| js::trim(&HEADING_MARK.strip(&LIST_MARK.strip(&QUOTE_MARK.strip(l)))).to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let joined = lines.join(" ");
    let unmarked = EMPHASIS.replace_all(&joined, |c| c.get(1).or(c.get(2)).or(c.get(3)).unwrap_or("").to_string());
    let unlinked = LINK.replace_all(&unmarked, |c| group(c, 1).to_string());
    let unquoted = OUTER_QUOTES.strip_all(&unlinked);
    js::trim(&SPACES.replace_all(&unquoted, |_| " ".into())).to_string()
}

// "Dr. Ames" and "e.g. Lisbon" do not end a sentence.
re!(ABBREVIATION, r"(?:^|\s)(?:dr|mr|mrs|ms|st|jr|sr|vs|etc|inc|ltd|no|e\.g|i\.e|[a-z])$", "i");
re!(SENTENCE_END, r#"[.!?]+(?=\s+["“'(\[]?[A-Z0-9]|\s*$)"#, "g");

/// The first sentence: up to a . ! or ? that a space and a capital, or the end, follow.
fn first_sentence_of(t: &str) -> &str {
    for (start, end) in SENTENCE_END.matches(t) {
        if &t[start..end] == "." && ABBREVIATION.test(&t[..start]) {
            continue;
        }
        return &t[..end];
    }
    t
}

// What the app writes into a message for a file. In backticks it is someone's words.
const PICTURE_LINK_SOURCE: &str = r"(?<!`)!\[([^\]\n]*)\]\([^)\n]*\)(?!`)";
re!(PICTURE_LINK, PICTURE_LINK_SOURCE, "");
re!(FILE_LINK, r"(?<![!`])\[([^\]\n]*)\]\((?:\.\/)?(?:attachments|phone-files)\/[^)\n]*\)(?!`)", "g");

/// A message's words for a line that cannot draw: no pictures, and a file down to its name.
pub fn without_files(said: &str) -> String {
    let out = FILE_LINK.replace_all(&PICTURE_LINK.strip_all(said), |c| group(c, 1).to_string());
    if out == said { out } else { js::trim(&out).to_string() }
}

re!(OPENS_ON_A_QUOTE, r"^>", "");
re!(ANY_PUNCTUATION, r"[,;:.!?…]", "");

/// A short line someone typed, kept exactly as they wrote it: six words, or a name of up to
/// eight with no punctuation at all ("Make tasks from findings 1 to 3"), which cutting would
/// break mid-phrase.
fn already_short(raw: &str) -> bool {
    let t = js::trim(raw);
    if t.is_empty() || t.contains('\n') || OPENS_ON_A_QUOTE.test(t) {
        return false;
    }
    let n = js::words(t).len();
    (n <= TITLE_WORDS && first_sentence_of(t) == t) || (n <= 8 && !ANY_PUNCTUATION.test(t))
}

/// Where a clause too long to keep whole is cut: the longest that fits six words, else the
/// shortest up to eight.
fn natural_cut(w: &[&str]) -> Option<usize> {
    let at: Vec<usize> = (0..w.len()).filter(|k| *k >= 3 && *k <= MOST_WORDS && CUT_BEFORE.contains(bare(w[*k]).as_str())).collect();
    at.iter().rev().find(|k| **k <= TITLE_WORDS).or(at.first()).copied()
}

re!(CLOSING_STOP, r"(?<!\.)\.$", "");
re!(LEADS_INTO_MORE, r"\s*[:;,]+$", "");
re!(TAG, r"^([^\s:]{1,24}):\s+(\S.*)$", "");
re!(CLAUSE_END, r"\s*(?:[,;:(]|\s[-–—]\s|—)\s*", "");
re!(TRAILING, r#"[\s.,;:!?…'"“”‘’)\]-]+$"#, "");

/// A title of a few words from whatever a thread was opened with. A short line comes back as
/// typed, less a closing full stop; a sentence comes back as its first clause if that is short,
/// or else cut where a phrase ends, with no greeting, no "can you", no quote mark and no
/// trailing punctuation.
pub fn short_title(opening: &str) -> String {
    let without = without_files(opening);
    let typed: &str = match js::trim(&without) {
        "" => PICTURE_LINK.exec(opening).map(|c| group(&c, 1)).filter(|alt| !alt.is_empty()).unwrap_or(opening),
        words => words,
    };
    let said = js::trim(typed);
    // Only before a sentence: "OK button" is a name.
    let unspoken = FILLER.strip(said);
    let text = if unspoken != said && js::words(&unspoken).len() >= 3 { js::upper_first(&unspoken) } else { typed.to_string() };
    // A line that leads into more ("Three fixes:") ends on its words, not the colon.
    if already_short(&text) {
        return LEADS_INTO_MORE.strip(&CLOSING_STOP.strip(js::trim(&text)));
    }
    let mut t = plain(&text);
    if t.is_empty() {
        return String::new();
    }
    t = CANNOT.replace(&LOOK.strip(&ASK.strip(&GREETING.strip(&t))), |_| "Cannot ".into());
    // One word before a colon ("M19-B1:", "Bug:") tags the thread; the name is what follows.
    if let Some(tag) = TAG.exec(&t) {
        let rest = short_title(group(&tag, 2));
        if !rest.is_empty() {
            return format!("{}: {rest}", group(&tag, 1));
        }
    }
    // The first sentence, then its first clause.
    let sentence = first_sentence_of(&t);
    let clause = match CLAUSE_END.split(sentence)[0] {
        "" => sentence,
        first => first,
    };
    let reported = REPORTED.strip(clause);
    let all = js::words(&reported);
    let clause = all.iter().enumerate().position(|(k, x)| k >= 3 && BREAK.contains(bare(x).as_str())).unwrap_or(all.len());
    let keep = if clause <= CLAUSE_WORDS { Some(clause) } else { natural_cut(&all[..clause]) };
    let mut w = &all[..keep.filter(|k| *k > 0).unwrap_or(TITLE_WORDS.min(clause))];
    if w.len() < clause {
        if let Some(opens) = w.iter().enumerate().position(|(k, x)| k >= 2 && OPENS.contains(bare(x).as_str())) {
            w = &w[..opens];
        }
        loop {
            let last = w.last().map(|x| bare(x)).unwrap_or_default();
            if w.len() > 2 && SUBJECT.contains(bare(w[w.len() - 2]).as_str()) {
                w = &w[..w.len() - 2];
            } else if w.len() > 2 && (LEADS.contains(last.as_str()) || TAIL.contains(last.as_str())) {
                w = &w[..w.len() - 1];
            } else {
                break;
            }
        }
    }
    while w.len() > 1 && TAIL.contains(bare(w[w.len() - 1]).as_str()) {
        w = &w[..w.len() - 1];
    }
    js::upper_first(&TRAILING.strip(&w.join(" ")))
}

/// What a thread is called on every screen: the namer's label, unless the person renamed it
/// after, in which case theirs; and either one held to the rule.
pub fn shown_title(i: &Item) -> String {
    let title = i.title.as_deref().unwrap_or("");
    let label = js::trim(i.label.as_deref().unwrap_or(""));
    if label.is_empty() {
        return short_title(title);
    }
    let (theirs, ours) = (i.wrote("title"), i.wrote("label"));
    // A time the ledger never kept compares as false, as it does in JavaScript.
    let renamed = matches!((theirs, ours), (Some(t), Some(o)) if t.source == "founder" && matches!((t.ts, o.ts), (Some(t), Some(o)) if t > o));
    if renamed && !js::trim(title).is_empty() {
        return short_title(title);
    }
    // A name the person typed (Rename) is theirs to the letter; only the app's is cut.
    if ours.is_some_and(|o| o.source == "founder") { label.to_string() } else { short_title(label) }
}

/// The AI teammate a thread is a run of, by its label (`teammateIdOf` in `shared/teammates.mts`).
pub fn teammate_id_of(i: &Item) -> Option<&str> {
    i.labels.iter().find_map(|l| l.strip_prefix("teammate:")).filter(|id| !id.is_empty())
}

/// The name on the row (`rowTitle` in `renderer/src/list-rules.ts`). A teammate's thread can
/// start with no words of the person's to name it by, so it is named by its answer's first line.
pub fn row_title(i: &Item) -> String {
    let name = shown_title(i);
    if name.is_empty() && teammate_id_of(i).is_some() {
        return i.result.as_deref().unwrap_or("").split('\n').map(|l| js::trim(&search_text(l)).to_string()).find(|l| !l.is_empty()).unwrap_or_default();
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(v: serde_json::Value) -> Item {
        serde_json::from_value(v).unwrap()
    }

    // The cases of the web app's own tests for this rule (tests/a-thread-title-is-a-few-words-
    // never-the-sentence and its neighbours), so the port answers what the original answers.
    #[test]
    fn a_sentence_becomes_a_few_words() {
        for (opening, want) in [
            ("Two patients at the Riverside clinic both got a confirmation for the 9:30 slot on Friday and showed up at the same time, which was really awkward for the front desk.", "Two patients at the Riverside clinic"),
            ("> Patients in Austin say they got appointment reminders at 3am.", "Patients in Austin got appointment reminders"),
            ("The reminders fire twice and nobody knows why.", "The reminders fire twice"),
            ("Hey Maya, could you call the Riverside clinic manager and tell her about the double booking fix before Thursday, since she asked us to let her know personally.", "Call the Riverside clinic manager"),
            ("Can you look at why the reminder emails are going out at the wrong time for clinics outside Lisbon?", "Why the reminder emails"),
            ("The checkout page is down for everyone.", "The checkout page is down for everyone"),
            ("I can no longer open the settings page.", "Cannot open the settings page"),
            ("Book Dr. Ames for the group class on Thursday morning.", "Book Dr. Ames"),
            ("Please write a launch post for the clinic booking widget and schedule it for Tuesday morning.", "Write a launch post"),
            ("**Fix** the `slot_race` bug in the booking flow before the Northgate demo on Monday", "Fix the slot_race bug"),
            ("- move the clinic calendar sync off the cron box and onto the worker queue", "Move the clinic calendar sync"),
            ("Build the seat picker for group bookings, 2 to 20 seats, and show it on a phone.", "Build the seat picker for group bookings"),
            ("Draw onboarding direction B as one page, in our team look.", "Draw onboarding direction B as one page"),
            ("Find why Northside Clinic sees double bookings on group slots.", "Find why Northside Clinic sees double bookings"),
            ("Write the words for a slot with fewer than 2 seats left, by noon.", "Write the words for a slot"),
            ("Fix the flaky calendar-sync test.", "Fix the flaky calendar-sync test"),
            ("Find why the reminder emails go out twice for clinics outside Lisbon on Mondays.", "Find why the reminder emails go out twice"),
            ("Call the Riverside clinic manager and tell her.", "Call the Riverside clinic manager"),
        ] {
            assert_eq!(short_title(opening), want, "{opening}");
        }
    }

    #[test]
    fn a_short_line_stays_as_typed() {
        for typed in ["Make tasks from findings 1 to 3", "Why is CI red?", "Fix login on Safari", "iOS build", "Ship the fix before", "Wait for it...", "OK button", "Ratio 3:2 images", "Why is it slow?", "M19-B1: fix login", "Later: ok"] {
            assert_eq!(short_title(typed), typed);
        }
        assert_eq!(short_title("M16 list:"), "M16 list");
        assert_eq!(short_title("Three fixes :"), "Three fixes");
        assert_eq!(short_title("Fix the header;"), "Fix the header");
        assert_eq!(short_title("Fix the header,"), "Fix the header");
        assert_eq!(short_title(""), "");
    }

    #[test]
    fn a_spoken_opener_goes_and_a_tag_stays() {
        assert_eq!(short_title("Ok so the onboarding flow is confusing and the pricing page is wrong"), "The onboarding flow is confusing");
        for said in ["okay so the sign up email never arrives", "So basically the export button does nothing", "um so the export button does nothing", "Alright the export button does nothing", "Ok, so the export button does nothing"] {
            let t = short_title(said);
            assert!(t.starts_with("The sign up email never arrives") || t.starts_with("The export button does nothing"), "{said}: {t}");
        }
        assert!(short_title("So many errors in the deploy log").starts_with("So many errors"));
        assert!(short_title("Right sidebar is too wide on small screens").starts_with("Right sidebar"));
        assert_eq!(short_title("M19-B1: add a line to the README saying hello from the phone"), "M19-B1: Add a line");
        assert_eq!(short_title("Bug: the reply box covers the last message on small phones"), "Bug: The reply box covers the last message");
        assert_eq!(short_title("M16 single: add a README line saying hello"), "M16 single");
        assert_ne!(short_title("https://example.com/a/very/long/path is broken for everyone who opens it today"), "https:");
    }

    // What the web app's `shortTitle` answers for these, asked of it directly.
    #[test]
    fn an_accented_word_is_looked_up_without_its_accent() {
        for (opening, want) in [
            // "são" is looked up as "so", which ends a clause.
            ("Os relatórios da clínica do centro são enviados duas vezes por semana até segunda", "Os relatórios da clínica do centro"),
            // "aí" ends a word at its accent, so "So" before it is a spoken opener.
            ("So aí está o problema do relatório semanal", "Aí está o problema do relatório semanal"),
            ("ok então o botão de exportar não faz nada quando clico", "Então o botão de exportar não"),
            ("Preciso que você verifique por que os e-mails de lembrete saem duas vezes às segundas", "Preciso que você verifique por que"),
            ("Não", "Não"),
        ] {
            assert_eq!(short_title(opening), want, "{opening}");
        }
    }

    #[test]
    fn other_openings_as_the_web_app_names_them() {
        for (opening, want) in [
            ("Fix the 🚀 launch page before the demo on Monday morning please", "Fix the 🚀 launch page"),
            ("🚀 launch", "🚀 launch"),
            ("   ", ""),
            ("a.b.c", "a.b.c"),
            ("1. first thing to do is check the logs and then restart the worker pool", "First thing to do is check"),
            ("Hey, can you please look into why the export fails?", "Why the export fails"),
            ("i still can't open the page", "i still can't open the page"),
            ("Find out whether the cron ran", "Find out whether the cron ran"),
            ("See how it goes", "See how it goes"),
            ("\"Quoted title here that is quite long and goes on and on for a while\"", "Quoted title"),
            ("What about `code_like_this` and __bold__ words in a longer sentence here", "What about code_like_this"),
            ("They said that they got two emails when they signed up yesterday morning", "They got two emails"),
            ("Check [the doc](https://example.com/doc) and tell me what is wrong with the second paragraph", "Check the doc"),
            ("one two three four five six seven eight nine ten", "One two three four five six"),
            ("one two three four five six seven", "one two three four five six seven"),
            ("one two three and four five six seven eight", "One two three"),
            ("Make it so the app is faster when it has many rows to draw on screen", "Make it so the app is faster"),
            ("Line one\nLine two", "Line one Line two"),
        ] {
            assert_eq!(short_title(opening), want, "{opening}");
        }
    }

    #[test]
    fn a_picture_or_a_file_never_shows_its_brackets() {
        let pic = "![shot.png](attachments/shot.png)";
        assert_eq!(short_title(&format!("See this\n\n{pic}")), "See this");
        assert_eq!(short_title(&format!("{pic}\n\nSee this")), "See this");
        assert_eq!(short_title(pic), "shot.png");
        assert_eq!(without_files("Read [notes.pdf](attachments/notes.pdf) first"), "Read notes.pdf first");
        assert_eq!(without_files("the [guide](https://example.com/guide)"), "the [guide](https://example.com/guide)");
        assert_eq!(without_files("one\n\ntwo "), "one\n\ntwo ");
        assert_eq!(without_files("say `![a](b.png)` to draw one"), "say `![a](b.png)` to draw one");
    }

    #[test]
    fn the_label_names_the_thread_unless_it_was_renamed_after() {
        let labelled = item(json!({ "title": "Two patients at the Riverside clinic both got a confirmation.", "label": "Riverside double booking fix" }));
        assert_eq!(shown_title(&labelled), "Riverside double booking fix");
        let renamed = item(json!({ "title": "Front desk double bookings", "label": "Riverside double booking fix",
            "wrote": { "title": { "ts": 20, "source": "founder" }, "label": { "ts": 10, "source": "system" } } }));
        assert_eq!(shown_title(&renamed), "Front desk double bookings");
        // A label the person typed is theirs to the letter, however long.
        let typed = item(json!({ "title": "x", "label": "Every single booking that the Riverside clinic lost in March, listed",
            "wrote": { "label": { "ts": 10, "source": "founder" } } }));
        assert_eq!(shown_title(&typed), "Every single booking that the Riverside clinic lost in March, listed");
        let blank = item(json!({ "title": "Fix login on Safari", "label": "   " }));
        assert_eq!(shown_title(&blank), "Fix login on Safari");
    }

    #[test]
    fn a_teammates_run_with_no_words_is_named_by_its_answer() {
        let run = item(json!({ "title": "", "labels": ["teammate:t-1"], "result": "\n## Monday's numbers\nBookings are up." }));
        assert_eq!(row_title(&run), "Monday's numbers");
        let other = item(json!({ "title": "", "result": "Bookings are up." }));
        assert_eq!(row_title(&other), "");
    }
}
