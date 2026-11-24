//! What the window checks about a key or a code a cloud run asked for, and where on this
//! computer such a key may be read from. A port of `renderer/src/key-asks.ts`, of the half of
//! `shared/cloud-keys.mts` the window reads (`keyAsks`, `keyWords`, `keyLabel`, `savedReply`),
//! and of `shared/cloud-key-sources.mts` with what it reads (`shared/sign-in-places.mts`,
//! `engineReads` in `shared/cloud-key-names.mts`), function by function. What a run's lines
//! ask for is `rules/needs.rs`.

use super::js::{self, group, re};
use super::needs;

/* ------------------------------ the older key line (`cloud-keys.mts`) ------------------------------ */

// The parts of these files no screen of this window reads, which the web window keeps for its
// engine's side or no longer draws (`savedReply`, `keySourceLine`, `keySourceWords`,
// `keyAsksAsBlocks`, `keyBlock`), are kept here with their cases as the web keeps them.

/// `KEY_ASK`.
#[allow(dead_code)]
pub const KEY_ASK: &str = "AGENTBOX-NEEDS-A-KEY:";
re!(ASK_LINE, r"^\s*AGENTBOX-NEEDS-A-KEY:\s*([A-Z][A-Z0-9_]{1,79})\b[\s:\u2014\u2013-]*(.*)$", "");
re!(OUTER_BACKTICKS, r"^`+|`+$", "g");
re!(TRAILING_BACKTICKS, r"`+$", "");

/// One key a message asks for on the older line (`KeyAsk`).
#[derive(Clone, Debug, PartialEq)]
pub struct KeyAsk {
    pub name: String,
    pub what: String,
}

/// The keys a message asks for on the older line, in order and once each (`keyAsks`).
pub fn key_asks(text: &str) -> Vec<KeyAsk> {
    let mut out: Vec<KeyAsk> = Vec::new();
    for line in text.split('\n') {
        let bare = OUTER_BACKTICKS.strip_all(line);
        if let Some(m) = ASK_LINE.exec(&bare) {
            let name = group(&m, 1);
            if !out.iter().any(|a| a.name == name) {
                out.push(KeyAsk { name: name.to_string(), what: js::trim(&TRAILING_BACKTICKS.strip(group(&m, 2))).to_string() });
            }
        }
    }
    out
}

/// Whether `line` is one of those asks (`isKeyAsk`).
#[allow(dead_code)]
pub fn is_key_ask(line: &str) -> bool {
    ASK_LINE.test(&OUTER_BACKTICKS.strip_all(line))
}

re!(WORDS_END, r",\s*from\b|\s+from\s+\S+\.\w|\s+from\s+`|\s*`|\s*\(|\.\s", "i");
re!(ARTICLE, r"^\s*(your|an?|the)\s+", "i");

/// What the agent said the key is, as a short name: "your Stripe secret key, from
/// dashboard.stripe.com/apikeys" is "Stripe secret key" (`keyWords`).
pub fn key_words(what: &str) -> Option<String> {
    let first = WORDS_END.split(what).into_iter().next().unwrap_or("");
    let words = ARTICLE.strip(first);
    let words = js::slice(js::trim(&words), 0, 80);
    (!words.is_empty()).then(|| words.to_string())
}

/// The same with its first letter raised (`keyLabel`).
pub fn key_label(what: &str) -> Option<String> {
    key_words(what).map(|t| js::upper_first(&t))
}

/// "a, b and c" (`listed`, and the same list in `savedReply` and `keySavedWords`).
pub fn listed(words: &[String]) -> String {
    match words {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// What goes back to a thread once every key its answer asked for is saved, in the agent's
/// words for each, its variable's name where it said nothing (`savedReply`). A bare name is
/// an ask with nothing said of it.
#[allow(dead_code)]
pub fn saved_reply(asks: &[KeyAsk]) -> String {
    let words: Vec<String> = asks.iter().map(|a| key_words(&a.what).map_or_else(|| a.name.clone(), |w| format!("the {w}"))).collect();
    format!("I saved {}. Carry on.", listed(&words))
}

/* ----------------------- where a key is on this computer (`cloud-key-sources.mts`) ----------------------- */

/// A Keychain item, a file under the home folder (whole, or one line of an .env file), or a
/// variable in the person's login shell (`KeySource`).
#[derive(Clone, Debug, PartialEq)]
pub struct KeySource {
    pub place: Place,
    pub from: String,
    pub entry: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Place {
    Keychain,
    File,
    Shell,
}

impl Place {
    pub fn word(self) -> &'static str {
        match self {
            Place::Keychain => "keychain",
            Place::File => "file",
            Place::Shell => "shell",
        }
    }
    fn of(word: &str) -> Option<Place> {
        match word {
            "keychain" => Some(Place::Keychain),
            "file" => Some(Place::File),
            "shell" => Some(Place::Shell),
            _ => None,
        }
    }
}

re!(FOLDERS, r"(^|\/)(\.ssh|\.aws|\.gnupg|\.netrc|\.npmrc|\.pypirc|\.docker|\.kube|\.claude[^/]*|\.codex|\.agentbox|Keychains|\.config\/gh|\.config\/gcloud|\.password-store|\.git-credentials|\.config\/git\/credentials)(\/|$)", "i");
// This copy's own home, `~/.mercury`, holds the cloud key and the cloud sign-ins; it has had
// no other name (`APP_HOMES`, of `nameSlug` and `WAS` in `shared/product-name.mjs`).
re!(APP_HOMES, r"(^|/)\.(mercury)(/|$)", "i");
re!(KEYCHAIN_ITEMS, r"^(claude code\b|codex\b|gh:)|\bsafe storage$", "i");
re!(ENGINE, r"^(?:ANTHROPIC|CLAUDE|CODEX|AGENTBOX|ZERO)_", "");
re!(VARIABLE, r"^[A-Za-z_][A-Za-z0-9_]{0,79}$", "");

/// Where a sign-in lives (`pathHoldsSignIns`).
pub fn path_holds_sign_ins(path: &str) -> bool {
    FOLDERS.test(path) || APP_HOMES.test(path)
}

/// A Keychain item that holds a sign-in (`keychainItemHoldsSignIns`).
pub fn keychain_item_holds_sign_ins(item: &str) -> bool {
    KEYCHAIN_ITEMS.test(js::trim(item))
}

/// Whether Claude Code or Codex reads `name` for its own sign-in or billing (`engineReads`).
pub fn engine_reads(name: &str) -> bool {
    let n = name.to_uppercase();
    ENGINE.test(&n) || n == "OPENAI_API_KEY" || n == "OPENAI_BASE_URL"
}

/// `/^[^\0-\x1f\x7f]{1,200}$/`: one to two hundred UTF-16 units, none of them a control.
fn item_shaped(s: &str) -> bool {
    (1..=200).contains(&js::len(s)) && !s.chars().any(|c| c <= '\u{1f}' || c == '\u{7f}')
}

/// Whether the place asked for is where a sign-in lives, which no card may read
/// (`sourceHoldsSignIns`).
pub fn source_holds_sign_ins(place: &str, from: &str, entry: &str) -> bool {
    let at = js::trim(from);
    (place == "keychain" && keychain_item_holds_sign_ins(at)) || (place == "file" && (path_holds_sign_ins(at) || engine_reads(js::trim(entry)))) || (place == "shell" && engine_reads(at))
}

/// The place, when it is one a key can be read from (`keySource`). `entry` is None where the
/// web window's is absent, null or empty.
pub fn key_source(place: &str, from: &str, entry: Option<&str>) -> Option<KeySource> {
    let at = js::trim(from);
    if source_holds_sign_ins(place, at, entry.unwrap_or("")) {
        return None;
    }
    let kind = Place::of(place)?;
    let plain = |kind| Some(KeySource { place: kind, from: at.to_string(), entry: None });
    match kind {
        Place::Keychain => item_shaped(at).then_some(()).and(plain(kind)),
        Place::Shell => VARIABLE.test(at).then_some(()).and(plain(kind)),
        Place::File => {
            if !item_shaped(at) || !(at.starts_with("~/") || at.starts_with('/')) {
                return None;
            }
            match entry.filter(|e| !e.is_empty()) {
                None => plain(kind),
                Some(entry) => VARIABLE.test(entry).then(|| KeySource { place: kind, from: at.to_string(), entry: Some(entry.to_string()) }),
            }
        }
    }
}

re!(SOURCE_WORDS, r"^\s*(keychain|file|shell)\s*:?\s+(.+?)\s*$", "i");
re!(FILE_WORDS, r"^(\S+)(?:\s+([A-Za-z_][A-Za-z0-9_]*))?$", "");

/// The third part of an ask line: `keychain posthog-read-key`, `file ~/app/.env POSTHOG_KEY`,
/// `shell POSTHOG_KEY` (`keySourceFromWords`).
pub fn key_source_from_words(text: &str) -> Option<KeySource> {
    let m = SOURCE_WORDS.exec(text)?;
    let place = group(&m, 1).to_lowercase();
    if place != "file" {
        return key_source(&place, group(&m, 2), None);
    }
    let f = FILE_WORDS.exec(group(&m, 2))?;
    key_source("file", group(&f, 1), f.get(2))
}

/// The same place written back as that third part (`keySourceLine`).
#[allow(dead_code)]
pub fn key_source_line(src: &KeySource) -> String {
    [Some(src.place.word()), Some(src.from.as_str()), src.entry.as_deref()].into_iter().flatten().filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" ")
}

/// Where it is, as the card says it (`keySourceWords`).
#[allow(dead_code)]
pub fn key_source_words(src: &KeySource) -> String {
    match (src.place, &src.entry) {
        (Place::Keychain, _) => format!("your Keychain, as {}", src.from),
        (Place::Shell, _) => format!("{} in your shell", src.from),
        (Place::File, Some(entry)) => format!("{entry} in {}", src.from),
        (Place::File, None) => src.from.clone(),
    }
}

/* ------------------------------------ the window's checks (`key-asks.ts`) ------------------------------------ */

/// `NEED_BLOCK`, `KEY_BLOCK`.
pub const NEED_BLOCK: &str = "agentbox-need";
#[allow(dead_code)]
pub const KEY_BLOCK: &str = "agentbox-key";

/// A message with each thing a cloud run asked for turned into a block the thread draws as a
/// card (`needsAsBlocks`).
pub fn needs_as_blocks(text: &str) -> String {
    if !text.contains("AGENTBOX-NEEDS") {
        return text.to_string();
    }
    text.split('\n')
        .map(|line| match needs::needs_in(line).first() {
            Some(need) => ["", &format!("```{NEED_BLOCK}"), &needs::need_json(need), "```", ""].join("\n"),
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The same for the older key line, as a block of its name and what it is (`keyAsksAsBlocks`).
/// The web window no longer draws these blocks (`Focus.tsx` reads `NEED_BLOCK` alone, and
/// `needsIn` reads the older line too); kept, as the web keeps it, with its cases.
#[allow(dead_code)]
pub fn key_asks_as_blocks(text: &str) -> String {
    if !text.contains(KEY_ASK) {
        return text.to_string();
    }
    text.split('\n')
        .map(|line| {
            if !is_key_ask(line) {
                return line.to_string();
            }
            let ask = key_asks(line).remove(0);
            ["", &format!("```{KEY_BLOCK}"), &ask.name, &ask.what, "```", ""].join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The block's name and what the agent said it is (`keyBlock`).
#[allow(dead_code)]
pub fn key_block(source: &str) -> KeyAsk {
    let source = source.strip_suffix('\n').unwrap_or(source);
    let mut lines = source.split('\n');
    let name = js::trim(lines.next().unwrap_or("")).to_string();
    let what = js::trim(&lines.collect::<Vec<_>>().join(" ")).to_string();
    KeyAsk { name, what }
}

/// What `keySavedWords` is told. `connected` is its `done: 'connected'`, a sign-in's.
#[derive(Clone, Debug, Default)]
pub struct Saving<'a> {
    pub asks: &'a [String],
    pub saved: &'a [String],
    pub freed: &'a [String],
    pub thread: &'a str,
    pub cloud_off: bool,
    pub connected: bool,
    pub left: Option<&'a [String]>,
}

/// What the screen says once a key is saved: whether this thread carries on, and how many
/// others with it (`keySavedWords`).
pub fn key_saved_words(said: &str, o: &Saving) -> String {
    let head = format!("{said} {}.", if o.connected { "connected" } else { "saved" });
    if o.cloud_off {
        return format!("{head} Threads use it once the cloud is on.");
    }
    // Everything the ask still waits on, files and codes too.
    if let Some(left) = o.left.filter(|l| !l.is_empty()) {
        return format!("{head} Still to give: {}.", listed(left));
    }
    let keys_left = o.asks.iter().filter(|n| !o.saved.contains(n)).count();
    if keys_left > 0 {
        return format!("{head} {} to go.", if keys_left == 1 { "One more key".to_string() } else { format!("{keys_left} more keys") });
    }
    let others = o.freed.iter().filter(|id| *id != o.thread).count();
    if !o.freed.iter().any(|id| id == o.thread) {
        return if others > 0 { format!("{head} {others} waiting thread{} carry on.", if others == 1 { "" } else { "s" }) } else { head };
    }
    if others > 0 { format!("{head} This thread and {others} other{} carry on.", if others == 1 { "" } else { "s" }) } else { format!("{head} This thread carries on.") }
}

re!(A_LINK, r"^https?:\/\/", "i");
re!(A_PATH_OR_FILE, r"^\.{0,2}\/|\.[A-Za-z]{1,5}$", "");
re!(HAS_DIGIT_OR_DASH, r"[0-9_-]", "");
re!(HAS_LETTER, r"[A-Za-z]", "");

/// Not a link, a path or a file name, which are long and carry dashes too (`keyShaped`).
fn key_shaped(w: &str) -> bool {
    js::len(w) >= 12 && !A_LINK.test(w) && !A_PATH_OR_FILE.test(w) && HAS_DIGIT_OR_DASH.test(w) && HAS_LETTER.test(w)
}

re!(SAYS_WHERE, r"\bkeychain\b|~\/|\.env\b|\$[A-Z][A-Z0-9_]+|\bshell\b", "i");
re!(WORD_EDGES, r#"^[`'"(<]+|[`'".,)>:;!]+$"#, "g");

/// A key pasted into the reply box (`keyFromReply`): the name to save it under, the key, and
/// whether there are words besides it to send.
#[derive(Clone, Debug, PartialEq)]
pub struct Pasted {
    pub name: String,
    pub value: String,
    pub words: bool,
}

/// A key pasted into the reply box of a thread that asked for one, or None when the reply
/// reads as words. A short reply ("here you go sk_live_…") with one key-shaped run in it
/// counts (`keyFromReply`).
pub fn key_from_reply(text: &str, asks: &[String], saved: &[String]) -> Option<Pasted> {
    let open: Vec<&String> = asks.iter().filter(|n| !saved.contains(n)).collect();
    if open.len() != 1 {
        return None;
    }
    // Where the key is kept, not the key: the agent can ask this computer for it.
    if SAYS_WHERE.test(text) {
        return None;
    }
    let words: Vec<String> = js::words(js::trim(text)).into_iter().map(|w| WORD_EDGES.strip_all(w)).filter(|w| !w.is_empty()).collect();
    let keys: Vec<&String> = words.iter().filter(|w| key_shaped(w)).collect();
    if keys.len() != 1 {
        return None;
    }
    let (name, key) = (open[0].clone(), keys[0].clone());
    if words.len() <= 12 && (words.len() == 1 || js::len(&key) >= 16) {
        return Some(Pasted { name, value: key, words: false });
    }
    // In a longer reply, one marked as the asked key's own is kept, and the words go on without it.
    begins(&name).filter(|b| b.test(&key)).map(|_| Pasted { name, value: key, words: true })
}

re!(GIST, r"^##\s*Gist\s*\n+", "i");
re!(SPACES, r"\s+", "g");

/// Whether a message the thread draws is the thread's newest answer, though it is drawn with
/// its Options taken off, from a run's log that drops blank lines, or in part
/// (`isNewestAnswer`).
pub fn is_newest_answer(text: &str, result: Option<&str>) -> bool {
    let plain = |t: &str| js::trim(&SPACES.replace_all(&GIST.strip(&super::format::strip_options_section(t)), |_| " ".to_string())).to_string();
    let shown = plain(text);
    result.is_some_and(|r| !r.is_empty() && !shown.is_empty() && plain(r).contains(&shown))
}

re!(STRIPE_SECRET, r"^(sk|rk)_(live|test)_", "");
re!(STRIPE_PUBLISHABLE, r"^pk_(live|test)_", "");
re!(SUPABASE_TOKEN, r"^sbp_", "");
re!(GITHUB_TOKEN, r"^(ghp_|gho_|ghs_|github_pat_)", "");
re!(SENTRY_TOKEN, r"^sntry[su]_", "");
re!(AWS_KEY_ID, r"^(AKIA|ASIA)[0-9A-Z]{16}$", "");
re!(SLACK_TOKEN, r"^xox[abposr]-", "");
re!(ANTHROPIC_KEY, r"^sk-ant-", "");
re!(OPENAI_KEY, r"^sk-", "");
re!(JWT, r"^eyJ[A-Za-z0-9_-]+\.", "");
re!(STRIPE_NAME, r"^STRIPE_", "");
re!(STRIPE_PUBLISHABLE_NAME, r"^STRIPE_(PUBLISHABLE|PUBLIC)", "");
re!(SUPABASE_NAME, r"^SUPABASE_", "");
re!(GITHUB_NAME, r"^(GH_|GITHUB_)", "");
re!(SENTRY_NAME, r"^SENTRY_", "");
re!(AWS_KEY_ID_NAME, r"^AWS_ACCESS_KEY_ID$", "");
re!(SLACK_NAME, r"^SLACK_", "");
re!(OPENAI_NAME, r"^OPENAI_", "");
re!(LIVE_STRIPE, r"^(sk|rk)_live_", "");
re!(SPACE, r"\s", "");

/// Keys whose issuers give them a shape, and the names each belongs under (`SHAPES`). An
/// Anthropic key belongs under none (`/^(?!)/`), a project's own Supabase key under any but the
/// account token (`/^(?!SUPABASE_ACCESS_TOKEN$)/`).
/// A shape, what a key of it is, and which names it belongs under.
type Shape = (&'static js::Re, &'static str, fn(&str) -> bool);

fn shapes() -> [Shape; 10] {
    [
        (&STRIPE_SECRET, "a Stripe secret key", |n| STRIPE_NAME.test(n)),
        (&STRIPE_PUBLISHABLE, "a Stripe publishable key", |n| STRIPE_PUBLISHABLE_NAME.test(n)),
        (&SUPABASE_TOKEN, "a Supabase access token", |n| SUPABASE_NAME.test(n)),
        (&GITHUB_TOKEN, "a GitHub token", |n| GITHUB_NAME.test(n)),
        (&SENTRY_TOKEN, "a Sentry token", |n| SENTRY_NAME.test(n)),
        (&AWS_KEY_ID, "an AWS access key ID", |n| AWS_KEY_ID_NAME.test(n)),
        (&SLACK_TOKEN, "a Slack token", |n| SLACK_NAME.test(n)),
        (&ANTHROPIC_KEY, "an Anthropic key", |_| false),
        (&OPENAI_KEY, "an OpenAI key", |n| OPENAI_NAME.test(n)),
        (&JWT, "a Supabase project key (anon or service role)", |n| n != "SUPABASE_ACCESS_TOKEN"),
    ]
}

re!(BEGINS_SUPABASE, r"^sbp_", "");
re!(BEGINS_GH, r"^(ghp_|gho_|ghu_|ghs_|github_pat_)", "");
re!(BEGINS_STRIPE, r"^(sk|rk)_(live|test)_", "");
re!(BEGINS_NPM, r"^npm_", "");
re!(BEGINS_LINEAR, r"^lin_api_", "");
re!(BEGINS_AWS, r"^(AKIA|ASIA)", "");

/// How the keys whose issuers mark them begin (`BEGINS`).
fn begins(name: &str) -> Option<&'static js::Re> {
    Some(match name {
        "SUPABASE_ACCESS_TOKEN" => &BEGINS_SUPABASE,
        "GH_TOKEN" => &BEGINS_GH,
        "STRIPE_SECRET_KEY" => &BEGINS_STRIPE,
        "NPM_TOKEN" => &BEGINS_NPM,
        "LINEAR_API_KEY" => &BEGINS_LINEAR,
        "AWS_ACCESS_KEY_ID" => &BEGINS_AWS,
        _ => return None,
    })
}

/// Why a pasted key looks like it belongs under another name, or none (`keyLooksWrong`).
pub fn key_looks_wrong(name: &str, value: &str, what: Option<&str>) -> Option<String> {
    let v = js::trim(value);
    let wanted = key_words(what.unwrap_or("")).map_or_else(|| name.to_string(), |w| format!("the {w}"));
    if LIVE_STRIPE.test(v) && STRIPE_NAME.test(name) {
        return Some("That is a live Stripe key: every cloud thread could make real charges with it. Press Save again to keep it anyway.".to_string());
    }
    if let Some((_, like, under)) = shapes().into_iter().find(|(shape, _, _)| shape.test(v))
        && !under(name)
    {
        return Some(format!("That looks like {like}, not {wanted}. Press Save again to keep it anyway."));
    }
    let marked = begins(name);
    if SPACE.test(v) || js::len(v) < 16 || marked.is_some_and(|b| !b.test(v)) {
        return Some(format!("That does not look like {wanted}. Press Save again to keep it anyway."));
    }
    None
}

re!(SIX_DIGITS, r"^\d{6}$", "");
re!(DIGITS, r"^\d+$", "");

/// Why a typed one-time code looks wrong, or none: these codes are six digits
/// (`codeLooksWrong`).
pub fn code_looks_wrong(code: &str) -> Option<String> {
    let c = js::trim(code);
    if SIX_DIGITS.test(c) {
        return None;
    }
    Some(if DIGITS.test(c) { format!("That is {} digits; these codes are 6. Press Send again to send it anyway.", js::len(c)) } else { "These codes are 6 digits. Press Send again to send it anyway.".to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    const ASK: &str = "AGENTBOX-NEEDS-A-KEY: SUPABASE_ACCESS_TOKEN your Supabase access token, from supabase.com/dashboard/account/tokens";

    fn ask(name: &str, what: &str) -> KeyAsk {
        KeyAsk { name: name.to_string(), what: what.to_string() }
    }

    // a-cloud-run-missing-a-key-asks-for-it-on-the-thread: the line a run asks on.
    #[test]
    fn the_older_line_is_read_with_its_name_and_what_it_is() {
        assert_eq!(key_asks(&format!("**Paste your token.**\n\nMore words.\n\n{ASK}\n\n## Options\n1. Do it from my computer")), vec![ask("SUPABASE_ACCESS_TOKEN", "your Supabase access token, from supabase.com/dashboard/account/tokens")]);
        for line in [format!("`{KEY_ASK} VERCEL_TOKEN a Vercel token`"), format!("{KEY_ASK} VERCEL_TOKEN \u{2014} a Vercel token"), format!("{KEY_ASK} VERCEL_TOKEN: a Vercel token"), format!("   {KEY_ASK} VERCEL_TOKEN a Vercel token")] {
            assert_eq!(key_asks(&line), vec![ask("VERCEL_TOKEN", "a Vercel token")], "{line}");
        }
        let text = [format!("{KEY_ASK} AWS_ACCESS_KEY_ID"), format!("{KEY_ASK} AWS_SECRET_ACCESS_KEY the secret"), format!("{KEY_ASK} AWS_ACCESS_KEY_ID again")].join("\n");
        assert_eq!(key_asks(&text), vec![ask("AWS_ACCESS_KEY_ID", ""), ask("AWS_SECRET_ACCESS_KEY", "the secret")]);
        for line in [format!("I will end with {KEY_ASK} VERCEL_TOKEN if needed"), format!("{KEY_ASK} vercel_token"), KEY_ASK.to_string(), "AGENTBOX-NEEDS-THE-COMPUTER: your AWS sign-in".to_string()] {
            assert!(key_asks(&line).is_empty() && !is_key_ask(&line), "{line}");
        }
    }

    #[test]
    fn saving_says_the_same_words_the_thread_does() {
        let bare = |n: &[&str]| n.iter().map(|n| ask(n, "")).collect::<Vec<_>>();
        assert_eq!(saved_reply(&bare(&["A_KEY", "B_KEY", "C_KEY"])), "I saved A_KEY, B_KEY and C_KEY. Carry on.");
        assert_eq!(saved_reply(&bare(&["SUPABASE_ACCESS_TOKEN"])), "I saved SUPABASE_ACCESS_TOKEN. Carry on.");
        assert_eq!(saved_reply(&bare(&["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"])), "I saved AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY. Carry on.");
        assert_eq!(saved_reply(&[ask("SUPABASE_ACCESS_TOKEN", "your Supabase access token, from supabase.com/dashboard/account/tokens")]), "I saved the Supabase access token. Carry on.");
    }

    #[test]
    fn the_older_line_is_a_block_the_thread_reads_back() {
        let out = key_asks_as_blocks(&format!("**Paste it.**\n\n{ASK}\n\nAfter."));
        assert!(out.contains(&format!("```{KEY_BLOCK}\nSUPABASE_ACCESS_TOKEN\nyour Supabase access token, from supabase.com/dashboard/account/tokens\n```")));
        assert!(out.starts_with("**Paste it.**\n\n") && js::trim_end(&out).ends_with("After."));
        assert_eq!(key_asks_as_blocks("No asks here."), "No asks here.");
        assert_eq!(key_block("SUPABASE_ACCESS_TOKEN\nyour token\n"), ask("SUPABASE_ACCESS_TOKEN", "your token"));
        assert_eq!(key_block("EXPO_TOKEN\n"), ask("EXPO_TOKEN", ""));
    }

    // a-cloud-run-missing-a-key-asks-for-it-on-the-thread, a-cloud-ask-is-answered-once-and-checked.
    #[test]
    fn a_key_pasted_into_the_reply_box_is_the_one_key_asked_for() {
        let asks = names(&["SUPABASE_ACCESS_TOKEN"]);
        let saved = |value: &str, words: bool| Some(Pasted { name: "SUPABASE_ACCESS_TOKEN".to_string(), value: value.to_string(), words });
        for (text, value) in [
            ("sbp_0123456789abcdef0123", "sbp_0123456789abcdef0123"),
            ("  ghp_abcdefghijklmnop123  ", "ghp_abcdefghijklmnop123"),
            ("sk_live-abc-123-def-456", "sk_live-abc-123-def-456"),
            ("here you go sk_live_51Habcdef0123456789", "sk_live_51Habcdef0123456789"),
            ("`sbp_0123456789abcdef0123`", "sbp_0123456789abcdef0123"),
            ("token: sbp_0123456789abcdef0123.", "sbp_0123456789abcdef0123"),
        ] {
            assert_eq!(key_from_reply(text, &asks, &[]), saved(value, false), "{text}");
        }
        for text in ["please use the token I saved", "https://supabase.com/dashboard/account/tokens", "abc_123", "abcdefghijklmnopqrstuvwxyz", "sbp_0123456789abcdef0123 sbp_9876543210fedcba9876", "use src/lib/rate-limit.ts please"] {
            assert_eq!(key_from_reply(text, &asks, &[]), None, "{text}");
        }
        assert_eq!(key_from_reply("no key on me right now, skip the deploy step and just write the github action yaml for sbp_0123456789abcdef0123", &asks, &[]), saved("sbp_0123456789abcdef0123", true));
        assert_eq!(key_from_reply("sbp_0123456789abcdef0123", &asks, &asks), None);
        assert_eq!(key_from_reply("AKIAIOSFODNN7EXAMPLE", &names(&["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"]), &[]), None); // public-check: allow (AWS's own documented example ID)
        assert_eq!(key_from_reply("sbp_0123456789abcdef0123", &[], &[]), None);
        // In a sentence: kept only when marked as the asked key's own.
        let long = "here you go sntrys_ana_test_44aa just use that, and dont forget the npm part after";
        assert_eq!(key_from_reply(&format!("{long} please and thanks a lot"), &asks, &[]), None);
        assert_eq!(key_from_reply("here you go sbp_0123456789abcdef just use that, and dont forget the npm part after", &asks, &[]), Some(Pasted { name: "SUPABASE_ACCESS_TOKEN".into(), value: "sbp_0123456789abcdef".into(), words: true }));
        assert_eq!(key_from_reply("here is the build id abc123def456ghi789 and a lot more words that are about other things", &names(&["VERCEL_TOKEN"]), &[]), None);
        assert_eq!(key_from_reply("sbp_0123456789abcdef", &asks, &[]), Some(Pasted { name: "SUPABASE_ACCESS_TOKEN".into(), value: "sbp_0123456789abcdef".into(), words: false }));
    }

    // the-key-cards-say-where-on-this-computer-the-key-comes-from: a reply that says where the key is.
    #[test]
    fn a_reply_that_says_where_the_key_is_goes_as_words() {
        let asks = names(&["POSTHOG_PERSONAL_KEY"]);
        for reply in ["use posthog-read-key from keychain for the cloud thread", "posthog-read-key in my Keychain", "it is RESEND_API_KEY in ~/app/.env", "use $MIXPANEL_API_SECRET from my shell"] {
            assert_eq!(key_from_reply(reply, &asks, &[]), None, "{reply}");
        }
        let pasted = key_from_reply("here you go phx_abc123def456ghi789", &asks, &[]).unwrap();
        assert_eq!((pasted.name.as_str(), pasted.value.as_str()), ("POSTHOG_PERSONAL_KEY", "phx_abc123def456ghi789"));
    }

    #[test]
    fn what_saving_says() {
        let o = |asks: &[&str], saved: &[&str], freed: &[&str]| (names(asks), names(saved), names(freed));
        let said = |name: &str, (asks, saved, freed): (Vec<String>, Vec<String>, Vec<String>), cloud_off: bool| key_saved_words(name, &Saving { asks: &asks, saved: &saved, freed: &freed, thread: "w-1", cloud_off, ..Saving::default() });
        let token = ["SUPABASE_ACCESS_TOKEN"];
        assert_eq!(said("Supabase access token", o(&token, &token, &["w-1"]), false), "Supabase access token saved. This thread carries on.");
        assert_eq!(said("Supabase access token", o(&[], &[], &[]), true), "Supabase access token saved. Threads use it once the cloud is on.");
        assert_eq!(said("SUPABASE_ACCESS_TOKEN", o(&token, &token, &["w-1"]), false), "SUPABASE_ACCESS_TOKEN saved. This thread carries on.");
        assert_eq!(said("SUPABASE_ACCESS_TOKEN", o(&token, &token, &["w-1", "w-2"]), false), "SUPABASE_ACCESS_TOKEN saved. This thread and 1 other carry on.");
        assert_eq!(said("SUPABASE_ACCESS_TOKEN", o(&token, &token, &["w-2", "w-3"]), false), "SUPABASE_ACCESS_TOKEN saved. 2 waiting threads carry on.");
        assert_eq!(said("SUPABASE_ACCESS_TOKEN", o(&token, &token, &[]), false), "SUPABASE_ACCESS_TOKEN saved.");
        assert_eq!(said("AWS_ACCESS_KEY_ID", o(&["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY"], &["AWS_ACCESS_KEY_ID"], &[]), false), "AWS_ACCESS_KEY_ID saved. One more key to go.");
        assert_eq!(said("A", o(&["A_KEY", "B_KEY", "C_KEY"], &["A_KEY"], &[]), false), "A saved. 2 more keys to go.");
        assert_eq!(said("SUPABASE_ACCESS_TOKEN", o(&token, &token, &["w-2"]), false), "SUPABASE_ACCESS_TOKEN saved. 1 waiting thread carry on.");
        assert_eq!(said("SUPABASE_ACCESS_TOKEN", o(&token, &token, &["w-1", "w-2", "w-3"]), false), "SUPABASE_ACCESS_TOKEN saved. This thread and 2 others carry on.");
        // a-cloud-ask-is-answered-once-and-checked: connected once, and everything still to give.
        let stripe = names(&["STRIPE_SECRET_KEY"]);
        let w1 = names(&["w-1"]);
        assert_eq!(key_saved_words("Stripe", &Saving { asks: &stripe, saved: &stripe, freed: &w1, thread: "w-1", connected: true, ..Saving::default() }), "Stripe connected. This thread carries on.");
        assert_eq!(key_saved_words("Sentry auth token", &Saving { thread: "w-1", ..Saving::default() }), "Sentry auth token saved.");
        let left = names(&["launch-film-original.mov"]);
        assert_eq!(key_saved_words("Stripe", &Saving { thread: "w-1", connected: true, left: Some(&left), ..Saving::default() }), "Stripe connected. Still to give: launch-film-original.mov.");
        let three = names(&["GitHub", "a.mov", "the npm code"]);
        assert_eq!(key_saved_words("Stripe", &Saving { thread: "w-1", connected: true, left: Some(&three), ..Saving::default() }), "Stripe connected. Still to give: GitHub, a.mov and the npm code.");
        assert_eq!(key_saved_words("Stripe", &Saving { freed: &w1, thread: "w-1", connected: true, left: Some(&[]), ..Saving::default() }), "Stripe connected. This thread carries on.");
    }

    #[test]
    fn the_ask_that_takes_a_key_is_the_newest_answer() {
        let result = format!("**Paste it.**\n\n{ASK}\n\n## Options\n1. Do it from my computer");
        assert!(is_newest_answer(&result, Some(&result)));
        assert!(is_newest_answer(&format!("**Paste it.**\n\n{ASK}"), Some(&result)));
        assert!(is_newest_answer(&format!("## Gist\n\n**Paste it.**\n\n{ASK}\n"), Some(&format!("## Gist\n\n**Paste it.**\n\n{ASK}\n\n## Options\n1. Do it from my computer"))));
        assert!(is_newest_answer(&format!("**Paste it.**\n{ASK}\n## Options\n1. Do it from my computer"), Some(&result)));
        assert!(is_newest_answer(&format!("{ASK}\n"), Some(&result)));
        assert!(!is_newest_answer(&format!("**Paste it.**\n\n{ASK}"), Some("Used your saved key. Done.")));
        assert!(!is_newest_answer(&result, None));
    }

    #[test]
    fn a_key_pasted_under_the_wrong_name_is_warned_about() {
        for (name, value, what) in [
            ("SENTRY_AUTH_TOKEN", "sk_live_51Habcdef0123456789", "a Stripe secret key"),
            ("STRIPE_SECRET_KEY", "pk_live_51Habcdef0123456789", "a Stripe publishable key"),
            ("VERCEL_TOKEN", "sbp_0123456789abcdef0123", "a Supabase access token"),
            ("AWS_SECRET_ACCESS_KEY", "AKIAIOSFODNN7EXAMPLE", "an AWS access key ID"), // public-check: allow (AWS's own documented example ID)
            ("STRIPE_SECRET_KEY", "sk-ant-api03-abcdef", "an Anthropic key"),
            ("SUPABASE_ACCESS_TOKEN", "eyJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiYW5vbiJ9.sig", "a Supabase project key (anon or service role)"),
        ] {
            assert_eq!(key_looks_wrong(name, value, None), Some(format!("That looks like {what}, not {name}. Press Save again to keep it anyway.")), "{name} {value}");
        }
        assert_eq!(key_looks_wrong("STRIPE_SECRET_KEY", "pk_live_51Habcdef0123456789", Some("your Stripe secret key, from dashboard.stripe.com/apikeys")).as_deref(), Some("That looks like a Stripe publishable key, not the Stripe secret key. Press Save again to keep it anyway."));
        assert_eq!(key_looks_wrong("SOME_SERVICE_TOKEN", "eyJhbGciOiJIUzI1NiJ9.eyJ4IjoxfQ.sig", None), None);
        for (name, value) in [("STRIPE_SECRET_KEY", "sk_test_51Habcdef0123456789"), ("STRIPE_PUBLISHABLE_KEY", "pk_test_51Habcdef0123456789"), ("SUPABASE_ACCESS_TOKEN", "sbp_0123456789abcdef0123"), ("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE"), ("VERCEL_TOKEN", "q1w2e3r4t5y6u7i8o9p0")] { // public-check: allow (AWS's own documented example ID)
            assert_eq!(key_looks_wrong(name, value, None), None, "{name} {value}");
        }
        // a-cloud-ask-is-answered-once-and-checked: what is pasted or typed is checked.
        for (name, value) in [("VERCEL_TOKEN", "hunter2!"), ("VERCEL_TOKEN", "my password is long enough"), ("SUPABASE_ACCESS_TOKEN", "q1w2e3r4t5y6u7i8o9p0"), ("GH_TOKEN", "q1w2e3r4t5y6u7i8o9p0")] {
            let said = key_looks_wrong(name, value, None).unwrap();
            assert!(said.starts_with("That does not look like ") && said.ends_with(" Press Save again to keep it anyway."), "{said}");
        }
        for (name, value) in [("VERCEL_TOKEN", "q1w2e3r4t5y6u7i8o9p0abcd"), ("GH_TOKEN", "github_pat_11ABCDEFG0123456789"), ("SENTRY_AUTH_TOKEN", "sntrys_eyJpYXQiOjE3MDAwMDAwMDB9")] {
            assert_eq!(key_looks_wrong(name, value, None), None, "{name}");
        }
        assert!(key_looks_wrong("STRIPE_SECRET_KEY", "sk_live_51Habcdef0123456789", None).unwrap().starts_with("That is a live Stripe key"));
        assert_eq!(key_looks_wrong("STRIPE_SECRET_KEY", "sk_test_51Habcdef0123456789", None), None);
        // Named the way the label says it.
        assert_eq!(key_looks_wrong("VERCEL_TOKEN", "hunter2!", Some("Vercel token")).as_deref(), Some("That does not look like the Vercel token. Press Save again to keep it anyway."));
    }

    #[test]
    fn a_code_that_is_not_six_digits_is_asked_about_twice() {
        assert_eq!(code_looks_wrong("482139"), None);
        assert_eq!(code_looks_wrong("48213").as_deref(), Some("That is 5 digits; these codes are 6. Press Send again to send it anyway."));
        assert!(code_looks_wrong("4821390").unwrap().starts_with("That is 7 digits"));
        assert_eq!(code_looks_wrong("abc-def").as_deref(), Some("These codes are 6 digits. Press Send again to send it anyway."));
    }

    #[test]
    fn a_key_is_named_without_what_comes_after() {
        assert_eq!(key_words("the treg agent token from `treg org agent-new x-replies`, so cloud runs can search X (about $0.13 a day)").as_deref(), Some("treg agent token"));
        assert_eq!(key_words("your Stripe secret key, from dashboard.stripe.com/apikeys").as_deref(), Some("Stripe secret key"));
        assert_eq!(key_words("a Sentry auth token, from sentry.io/settings/account/api/auth-tokens").as_deref(), Some("Sentry auth token"));
        assert_eq!(key_words("ElevenLabs API key, from elevenlabs.io/app/settings/api-keys").as_deref(), Some("ElevenLabs API key"));
        assert_eq!(key_words(""), None);
        assert_eq!(key_label("the secret").as_deref(), Some("Secret"));
    }

    // a-cloud-run-gets-a-key-from-this-computer-only-after-a-yes, a-key-card-never-reads-a-sign-in-the-cloud-may-not-have.
    #[test]
    fn a_key_is_read_only_from_a_place_that_holds_no_sign_in() {
        let src = |place: Place, from: &str, entry: Option<&str>| Some(KeySource { place, from: from.to_string(), entry: entry.map(str::to_string) });
        assert_eq!(key_source("keychain", " posthog-read-key ", None), src(Place::Keychain, "posthog-read-key", None));
        assert_eq!(key_source("file", "~/.config/gilded/openai.key", None), src(Place::File, "~/.config/gilded/openai.key", None));
        assert_eq!(key_source("file", "~/app/.env", Some("POSTHOG_KEY")), src(Place::File, "~/app/.env", Some("POSTHOG_KEY")));
        assert_eq!(key_source("shell", "POSTHOG_KEY", None), src(Place::Shell, "POSTHOG_KEY", None));
        for (place, from, entry) in [("browser", "x", None), ("keychain", "", None), ("keychain", "a\nb", None), ("file", "relative/key.txt", None), ("file", "~/app/.env", Some("not a name")), ("shell", "POSTHOG_KEY; rm -rf ~", None)] {
            assert_eq!(key_source(place, from, entry), None, "{place} {from}");
        }
        assert_eq!(key_source_from_words("keychain posthog-read-key"), src(Place::Keychain, "posthog-read-key", None));
        assert_eq!(key_source_from_words("Keychain: Supabase CLI"), src(Place::Keychain, "Supabase CLI", None));
        assert_eq!(key_source_from_words("file ~/app/.env POSTHOG_KEY"), src(Place::File, "~/app/.env", Some("POSTHOG_KEY")));
        assert_eq!(key_source_from_words("shell POSTHOG_KEY"), src(Place::Shell, "POSTHOG_KEY", None));
        assert_eq!(key_source_from_words("from posthog.com/settings"), None);
        assert_eq!(key_source_from_words(""), None);
        for (place, from) in [
            ("file", "~/.codex/auth.json"),
            ("file", "~/.claude/.credentials.json"),
            ("file", "~/.claude.json"),
            ("file", "/Users/x/.ssh/id_ed25519"),
            ("file", "~/.aws/credentials"),
            ("file", "~/.config/gh/hosts.yml"),
            ("file", "~/.mercury/cloud-key"),
            ("keychain", "Claude Code-credentials"),
            ("keychain", "Claude Code-credentials-1a2b3c4d"),
            ("keychain", "Claude Code"),
            ("keychain", "Codex Auth"),
            ("keychain", "Agentbox Safe Storage"),
            ("keychain", "Chrome Safe Storage"),
        ] {
            assert_eq!(key_source(place, from, None), None, "{place} {from}");
            assert_eq!(key_source_from_words(&format!("{place} {from}")), None, "{place} {from}");
        }
        for (place, from, entry) in [("file", "~/app/.env", Some("POSTHOG_KEY")), ("file", "~/projects/claude-notes/.env", Some("KEY")), ("file", "~/codex-tools/key.txt", None), ("keychain", "posthog-read-key", None), ("keychain", "OPENAI_API_KEY", None), ("keychain", "claude-api-notes", None), ("shell", "POSTHOG_KEY", None)] {
            assert!(key_source(place, from, entry).is_some(), "{place} {from}");
        }
        // An entry or a variable an agent's own engine reads is never read.
        assert_eq!(key_source("shell", "ANTHROPIC_API_KEY", None), None);
        assert_eq!(key_source("file", "~/app/.env", Some("OPENAI_API_KEY")), None);
        assert_eq!(key_source_words(&src(Place::Keychain, "posthog-read-key", None).unwrap()), "your Keychain, as posthog-read-key");
        assert_eq!(key_source_words(&src(Place::File, "~/app/.env", Some("POSTHOG_KEY")).unwrap()), "POSTHOG_KEY in ~/app/.env");
        assert_eq!(key_source_words(&src(Place::File, "~/.config/k.key", None).unwrap()), "~/.config/k.key");
        assert_eq!(key_source_words(&src(Place::Shell, "POSTHOG_KEY", None).unwrap()), "POSTHOG_KEY in your shell");
        assert_eq!(key_source_line(&src(Place::File, "~/app/.env", Some("POSTHOG_KEY")).unwrap()), "file ~/app/.env POSTHOG_KEY");
        assert_eq!(key_source_line(&src(Place::Keychain, "posthog-read-key", None).unwrap()), "keychain posthog-read-key");
    }
}
