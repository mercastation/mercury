//! What a tool call reads as in a conversation: a plain verb and what it acted on, shortened
//! without being wrong (`shared/work-lines.mts`), and the words the fold of a run of them wears
//! (`renderer/src/activity-summary.ts`), and what the thread says of what it did not keep
//! (`shared/agents.mts`). The whole command is always kept beside the words.
//!
//! What the web's file also has and this one leaves: `workSubject` and `fullSubject`, which
//! read a transcript's tool input rather than a log's line, and `runSummary`, which is only
//! the fold's hover title.

use super::js::{self, re};

/// Plain words for a tool, not its name. Edit is "changed": a reader who is not a programmer
/// takes "edited" to mean a document was revised.
fn verb_of(name: &str) -> Option<&'static str> {
    Some(match name {
        "Bash" => "ran",
        "Read" => "read",
        "Write" => "wrote",
        "Edit" | "MultiEdit" | "NotebookEdit" => "changed",
        "Glob" => "looked for",
        "Grep" => "searched for",
        "WebFetch" => "fetched",
        "WebSearch" => "searched the web for",
        "Task" => "sent off a helper",
        "TodoWrite" => "kept its list",
        "ListAgents" => "listed the other sessions",
        "SendMessage" => "messaged",
        "ToolSearch" => "looked up a tool",
        "Skill" => "used a skill",
        "AskUserQuestion" => "asked a question",
        "BashOutput" => "checked on a command",
        "KillShell" => "stopped a command",
        "ExitPlanMode" => "finished planning",
        _ => return None,
    })
}

re!(UNDERSCORES, r"_+", "g");

/// An MCP tool's name is `mcp__<server>__<tool>`, and the server's own name may hold
/// underscores, so the tool is what follows the last double underscore (`workVerb`).
pub fn work_verb(name: &str) -> String {
    let n = js::trim(name);
    if n.is_empty() {
        return "did something".to_string();
    }
    if let Some(verb) = verb_of(n) {
        return verb.to_string();
    }
    let tool = if n.starts_with("mcp__") { &n[n.rfind("__").map_or(0, |at| at + 2)..] } else { n };
    let words = js::trim(&UNDERSCORES.replace_all(tool, |_| " ".to_string())).to_lowercase();
    if words.is_empty() { "did something".to_string() } else { words }
}

re!(PRIVATE_TMP, r"^\/private\/tmp\/", "");
re!(TRAILING_SLASHES, r"\/+$", "");

/// A path shortened from its end, where the file's name is (`shortPath`): inside `cwd` it is
/// relative to it, under `home` it starts with a tilde, and three segments hold most of the
/// rest. With no home given, a path is never shortened against a guess at whose Mac this is.
pub fn short_path(raw: &str, cwd: &str, home: &str) -> String {
    let s = js::trim(raw);
    if s.is_empty() {
        return String::new();
    }
    if !s.starts_with('/') && !s.starts_with('~') {
        return s.to_string();
    }
    // On macOS `/tmp` is a link to `/private/tmp`; show the one people type.
    let mut p = PRIVATE_TMP.replace(s, |_| "/tmp/".to_string());
    let base = TRAILING_SLASHES.strip(&PRIVATE_TMP.replace(cwd, |_| "/tmp/".to_string()));
    if !base.is_empty() && (p == base || p.starts_with(&format!("{base}/"))) {
        let rel = if p == base { "" } else { &p[base.len() + 1..] };
        return if rel.is_empty() { "./".to_string() } else { trim_segments(rel) };
    }
    let h = TRAILING_SLASHES.strip(home);
    if !h.is_empty() && p.starts_with(&format!("{h}/")) {
        p = format!("~/{}", &p[h.len() + 1..]);
    }
    trim_segments(&p)
}

/// The leading `…/` says segments were dropped; nothing trimmed keeps the lead, because
/// `/x/store.mjs` and `x/store.mjs` are different files (`trimSegments`).
fn trim_segments(p: &str) -> String {
    let lead = if p.starts_with("~/") { "~/" } else if p.starts_with('/') { "/" } else { "" };
    let parts: Vec<&str> = p[lead.len()..].split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() <= 3 {
        return format!("{lead}{}", parts.join("/"));
    }
    format!("…/{}", parts[parts.len() - 3..].join("/"))
}

// The preludes that come off a command's front, one at a time, until what is left is the
// command that ran (`PRELUDE`).
re!(PRELUDE_CD, r#"^cd\s+(?:"[^"]*"|'[^']*'|[^\s;&|]+)\s*(?:&&|;|\n)\s*"#, "");
re!(PRELUDE_EXPORT, r#"^export\s+[A-Za-z_][A-Za-z0-9_]*=(?:"[^"]*"|'[^']*'|[^\s;&]*)\s*(?:&&|;|\n)\s*"#, "");
re!(PRELUDE_SET, r#"^[A-Za-z_][A-Za-z0-9_]*=(?:"[^"]*"|'[^']*'|[^\s;&]*)\s*(?:&&|;|\n)\s*"#, "");
re!(PRELUDE_SOURCE, r#"^(?:source|\.)\s+(?:"[^"]*"|'[^']*'|[^\s;&|]+)\s*(?:&&|;|\n)\s*"#, "");
re!(PRELUDE_BANNER, r#"^echo\s+(?:"[^"]*(?:===|---)[^"]*"|'[^']*(?:===|---)[^']*')\s*(?:&&|;|\n)\s*"#, "");
// From a heredoc's marker on, the script becomes an ellipsis. `[\s\S]*$` in the web's, which
// is any character to the end: `s` and `.` say the same here.
re!(HEREDOC, r#"<<-?\s*(['"]?)([A-Za-z_][A-Za-z0-9_]*)\1(.*)$"#, "s");

/// When the command before a heredoc's marker is this short (`python3 -`), the script's first
/// line comes with it, because it usually names the file.
const HEREDOC_HEAD: usize = 12;

fn first_script_line(body: &str, marker: &str) -> String {
    body.split('\n').map(js::trim).find(|t| !t.is_empty() && *t != marker && !t.starts_with('#') && !t.starts_with("//")).unwrap_or("").to_string()
}

/// A command with the scaffolding off its front (`plainCommand`): the interesting part of a
/// command is never at the front, and an ellipsis cuts from the right.
pub fn plain_command(raw: &str) -> String {
    let mut s = js::trim(raw).to_string();
    if s.is_empty() {
        return String::new();
    }
    let whole = s.clone();
    // Repeated, because `cd x && export Y=z && npm test` has two.
    for _ in 0..6 {
        let before = s.clone();
        for re in [&*PRELUDE_CD, &*PRELUDE_EXPORT, &*PRELUDE_SET, &*PRELUDE_SOURCE, &*PRELUDE_BANNER] {
            s = re.strip(&s);
        }
        if s == before {
            break;
        }
    }
    if let Some(doc) = HEREDOC.exec(&s) {
        let head = js::trim(&s[..doc.span(0).map_or(0, |(start, _)| start)]).to_string();
        let first = if js::len(&head) <= HEREDOC_HEAD { first_script_line(js::group(&doc, 3), js::group(&doc, 2)) } else { String::new() };
        s = if first.is_empty() { format!("{head} …") } else { format!("{head} … {first}") };
    }
    let flat = |t: &str| js::trim(&SPACES.replace_all(t, |_| " ".to_string())).to_string();
    let s = flat(&s);
    if s.is_empty() { flat(&whole) } else { s }
}

re!(SPACES, r"\s+", "g");

/// The store's own bookkeeping, which is how a run talks to the app and not the work: taking
/// the thread, writing its status back, listing what is there (`isBookkeeping`).
pub fn is_bookkeeping(name: &str) -> bool {
    let n = js::trim(name);
    if n.is_empty() {
        return false;
    }
    if n == "ToolSearch" {
        return true;
    }
    let tool = if n.starts_with("mcp__") { &n[n.rfind("__").map_or(0, |at| at + 2)..] } else { n };
    matches!(tool, "claim_work_item" | "release_work_item" | "update_work_item" | "list_work_items" | "list_documents" | "list_products" | "list_skills" | "list_chats" | "get_product")
}

/// The path a tool call changed, whole, or nothing when it changed none (`changedFile`).
pub fn changed_file(name: &str, file_path: &str) -> String {
    if matches!(name, "Edit" | "Write" | "MultiEdit" | "NotebookEdit") { js::trim(file_path).to_string() } else { String::new() }
}

/* ------------------------------ a shell command in words ------------------------------ */

/// What a command did, in words (`CommandWork`).
#[derive(Clone, Debug, PartialEq)]
pub struct CommandWork {
    pub verb: String,
    pub doing: String,
    pub subject: String,
}

/// How a family of commands names what it acted on (`take`).
#[derive(Clone, Copy, PartialEq)]
enum Take {
    Files,
    First,
    Named,
    Url,
}

/// One kind of command and its words (`Family`).
struct Family {
    verb: &'static str,
    doing: &'static str,
    heads: &'static [&'static str],
    take: Take,
    skip: &'static [&'static str],
    many: Option<&'static str>,
    empty: Option<&'static str>,
    plural: Option<(&'static str, &'static str)>,
}

const FAMILIES: [Family; 11] = [
    Family { verb: "read", doing: "Reading", heads: &["cat", "head", "tail", "less", "more", "nl", "bat"], take: Take::Files, skip: &["-n", "-c", "-m"], many: None, empty: None, plural: None },
    Family { verb: "counted the lines in", doing: "Counting the lines in", heads: &["wc"], take: Take::Files, skip: &[], many: None, empty: None, plural: None },
    Family { verb: "searched for", doing: "Searching for", heads: &["grep", "rg", "ag", "ack"], take: Take::First, skip: &["-e", "-m", "-A", "-B", "-C"], many: Some("things"), empty: None, plural: None },
    Family { verb: "listed", doing: "Listing", heads: &["ls"], take: Take::Files, skip: &[], many: Some("folders"), empty: Some("./"), plural: None },
    Family { verb: "looked for", doing: "Looking for", heads: &["find", "fd"], take: Take::Named, skip: &[], many: Some("things"), empty: None, plural: None },
    Family { verb: "copied", doing: "Copying", heads: &["cp"], take: Take::Files, skip: &[], many: None, empty: None, plural: None },
    Family { verb: "moved", doing: "Moving", heads: &["mv"], take: Take::Files, skip: &[], many: None, empty: None, plural: None },
    Family { verb: "deleted", doing: "Deleting", heads: &["rm"], take: Take::Files, skip: &[], many: None, empty: None, plural: None },
    Family { verb: "made a folder", doing: "Making a folder", heads: &["mkdir"], take: Take::Files, skip: &[], many: Some("folders"), empty: None, plural: Some(("made folders", "Making folders")) },
    Family { verb: "made a file", doing: "Making a file", heads: &["touch"], take: Take::Files, skip: &[], many: None, empty: None, plural: Some(("made files", "Making files")) },
    Family {
        verb: "fetched",
        doing: "Fetching",
        heads: &["curl", "wget"],
        take: Take::Url,
        skip: &["-o", "-w", "-H", "-X", "-d", "-u", "-A", "-e", "-F", "--output", "--header", "--data"],
        many: Some("pages"),
        empty: None,
        plural: None,
    },
];

/// The git subcommands that have words; any other keeps its command (`GIT`).
fn git_phrase(sub: &str) -> Option<(&'static str, &'static str)> {
    Some(match sub {
        "status" | "diff" => ("checked what changed", "Checking what changed"),
        "log" | "show" | "blame" => ("read the code history", "Reading the code history"),
        "add" | "commit" => ("saved the changes", "Saving the changes"),
        "branch" => ("looked at the branches", "Looking at the branches"),
        "push" => ("pushed the branch", "Pushing the branch"),
        "fetch" | "pull" => ("fetched the latest code", "Fetching the latest code"),
        "merge" => ("merged the branch", "Merging the branch"),
        "checkout" | "switch" => ("switched branch", "Switching branch"),
        _ => return None,
    })
}

/// One stage of a command, in words: a verb and what it named (`StageWords`).
#[derive(Clone, Debug)]
struct Stage {
    verb: String,
    doing: String,
    names: Vec<String>,
    many: Option<&'static str>,
    plural: Option<(&'static str, &'static str)>,
    whole: bool,
}

impl Stage {
    fn of(verb: &str, doing: &str, names: Vec<String>) -> Stage {
        Stage { verb: verb.to_string(), doing: doing.to_string(), names, many: None, plural: None, whole: false }
    }
    fn many(mut self, many: &'static str) -> Stage {
        self.many = Some(many);
        self
    }
}

/// What a stage reads as: words, scaffolding, or a shape this does not know.
enum Read {
    Words(Stage),
    Noise,
    Unknown,
}

const TESTS: (&str, &str, &str) = ("ran the tests", "Running the tests", "test files");
const SCRIPT: (&str, &str, &str) = ("ran a script", "Running a script", "scripts");

re!(QUIET_REDIRECT, r"^[0-9]*>>?&?(?:[0-9]+|\/dev\/null)$", "");
re!(UNREADABLE, r"[<>`]", "");
re!(ALL_DIGITS, r"^[0-9]+$", "");
re!(SHELL_STRING, r"^-[a-z]*c$", "");
re!(SCRIPT_FILE, r"\.(?:sh|mjs|js|py)$", "");
re!(TEST_FILE, r"\.(?:test|spec)\.[a-z]+$", "");
re!(URL_LIKE, r"^(?:https?:\/\/|www\.|localhost|127\.0\.0\.1)", "");
re!(GLOB, r"[*?]", "");
re!(FILE_LIKE, r"[./]", "");
re!(SED_SCRIPT, r#"^['"]?[0-9,$]*[a-zA-Z]?[/#|]"#, "");
re!(VERSION, r"@[^@/]*$", "");

fn last_segment(file: &str) -> String {
    file.split('/').filter(|s| !s.is_empty()).next_back().unwrap_or("").to_string()
}

fn unquote(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() > 1 && (chars[0] == '"' || chars[0] == '\'') && chars[chars.len() - 1] == chars[0] {
        return chars[1..chars.len() - 1].iter().collect();
    }
    token.to_string()
}

/// Splits only outside quotes: a `;` inside a sed script is not a new command (`splitOutside`).
fn split_outside(text: &str, separators: &[&str]) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut quote: Option<char> = None;
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if let Some(q) = quote {
            buffer.push(ch);
            if ch == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            buffer.push(ch);
            i += 1;
            continue;
        }
        let pair: String = chars[i..(i + 2).min(chars.len())].iter().collect();
        if separators.contains(&pair.as_str()) {
            out.push(std::mem::take(&mut buffer));
            i += 2;
            continue;
        }
        if separators.contains(&ch.to_string().as_str()) {
            out.push(std::mem::take(&mut buffer));
            i += 1;
            continue;
        }
        buffer.push(ch);
        i += 1;
    }
    out.push(buffer);
    out.iter().map(|part| js::trim(part).to_string()).filter(|part| !part.is_empty()).collect()
}

fn tokens(segment: &str) -> Vec<String> {
    split_outside(segment, &[" ", "\t"])
}

fn is_flag(token: &str) -> bool {
    token.starts_with('-') && token != "-"
}

fn looks_like_a_file(token: &str) -> bool {
    FILE_LIKE.test(token) && !token.starts_with('-')
}

/// The arguments of one stage, the quiet redirects and the flags' values dropped, or None when
/// something is there this cannot account for (`argumentsOf`).
fn arguments_of(words: &[String], skip: &[&str]) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let raw = &words[i];
        let quoted = raw.starts_with('"') || raw.starts_with('\'');
        let token = unquote(raw);
        i += 1;
        if !quoted {
            if QUIET_REDIRECT.test(&token) {
                continue;
            }
            if UNREADABLE.test(&token) || token.contains("$(") {
                return None;
            }
        }
        if is_flag(&token) {
            if skip.contains(&token.as_str()) {
                i += 1;
            }
            continue;
        }
        out.push(token);
    }
    Some(out)
}

/// One stage of one segment, as words, scaffolding, or unknown (`stageWords`).
fn stage_words(stage: &str, cwd: &str, home: &str) -> Read {
    let words = tokens(stage);
    let Some(first) = words.first() else { return Read::Noise };
    let head = unquote(first);
    let rest = &words[1..];
    if matches!(head.as_str(), "echo" | "cd" | "pwd" | "true" | "false" | ":" | "sleep" | "clear" | "date" | "printf" | "set") {
        // `echo something > a file` writes a file, so it is not scaffolding.
        return if arguments_of(rest, &[]).is_none() { Read::Unknown } else { Read::Noise };
    }
    // `timeout 300 npx vitest …` and `arch -arm64 node …` are the command behind a wrapper.
    if matches!(head.as_str(), "timeout" | "nice" | "arch" | "env") {
        let inner: Vec<&String> = rest.iter().filter(|w| !is_flag(&unquote(w)) && !ALL_DIGITS.test(&unquote(w))).collect();
        return if inner.is_empty() { Read::Unknown } else { stage_words(&inner.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(" "), cwd, home) };
    }
    // A shell asked to run a string: the string is the command.
    if matches!(head.as_str(), "sh" | "bash" | "zsh" | "/bin/sh" | "/bin/bash" | "/bin/zsh") {
        let Some(flag) = rest.iter().position(|w| SHELL_STRING.test(&unquote(w))) else { return Read::Unknown };
        let Some(script) = rest.get(flag + 1).filter(|s| !s.is_empty()) else { return Read::Unknown };
        return match command_work(&unquote(script), cwd, home) {
            Some(w) => Read::Words(Stage { names: if w.subject.is_empty() { vec![] } else { vec![w.subject.clone()] }, whole: true, ..Stage::of(&w.verb, &w.doing, vec![]) }),
            None => Read::Unknown,
        };
    }
    if head == "git" {
        return git_words(rest, cwd, home);
    }
    if head == "sed" || head == "perl" {
        return sed_words(rest, cwd, home);
    }
    if matches!(head.as_str(), "npx" | "npm" | "pnpm" | "yarn" | "bun") {
        return runner_words(rest, cwd, home);
    }
    if matches!(head.as_str(), "node" | "python" | "python3" | "ruby" | "osascript" | "deno" | "tsx") || SCRIPT_FILE.test(&head) {
        let Some(args) = arguments_of(rest, &[]) else { return Read::Unknown };
        // `python3 - <<'PY'` has no file's name: the script's first line, after the ellipsis
        // `plain_command` put there, tells these apart.
        if let Some(at) = stage.rfind('…') {
            let first = js::trim(&stage[at + '…'.len_utf8()..]);
            return Read::Words(Stage::of(SCRIPT.0, SCRIPT.1, if first.is_empty() { vec![] } else { vec![first.to_string()] }).many(SCRIPT.2));
        }
        let file = args.iter().find(|a| looks_like_a_file(a));
        return Read::Words(Stage::of(SCRIPT.0, SCRIPT.1, file.map(|f| vec![last_segment(f)]).unwrap_or_default()).many(SCRIPT.2));
    }
    if matches!(head.as_str(), "vitest" | "jest" | "pytest" | "mocha" | "ava") {
        return Read::Words(test_words(rest));
    }
    if head == "tsc" {
        return Read::Words(Stage::of("checked the types", "Checking the types", vec![]));
    }
    if head == "eslint" {
        return Read::Words(Stage::of("checked the code style", "Checking the code style", vec![]));
    }
    let Some(family) = FAMILIES.iter().find(|f| f.heads.contains(&head.as_str())) else { return Read::Unknown };
    let Some(args) = arguments_of(rest, family.skip) else { return Read::Unknown };
    Read::Words(Stage { many: family.many, plural: family.plural, ..Stage::of(family.verb, family.doing, names_for(family, &args, cwd, home)) })
}

fn names_for(family: &Family, args: &[String], cwd: &str, home: &str) -> Vec<String> {
    match family.take {
        Take::First => args.iter().take(1).cloned().collect(),
        // The address is not always the first argument (`curl -s -o /dev/null <url>`).
        Take::Url => args.iter().find(|a| URL_LIKE.test(a)).or(args.first()).into_iter().cloned().collect(),
        Take::Named => match args.iter().find(|a| GLOB.test(a)) {
            Some(named) => vec![named.clone()],
            None => args.iter().take(1).map(|a| short_path(a, cwd, home)).collect(),
        },
        Take::Files => {
            let files: Vec<String> = args.iter().filter(|a| looks_like_a_file(a)).map(|a| short_path(a, cwd, home)).collect();
            if files.is_empty() { family.empty.map(|e| vec![e.to_string()]).unwrap_or_default() } else { files }
        }
    }
}

/// Keeps what a git subcommand named, the file, the branch or the message (`gitWords`).
fn git_words(rest: &[String], cwd: &str, home: &str) -> Read {
    let mut words: Vec<String> = rest.to_vec();
    // `git -C <path> <subcommand>`: the folder is not the verb.
    while words.first().is_some_and(|w| is_flag(&unquote(w))) {
        let flag = unquote(&words.remove(0));
        if matches!(flag.as_str(), "-C" | "--git-dir" | "--work-tree") && !words.is_empty() {
            words.remove(0);
        }
    }
    let sub = unquote(words.first().map_or("", String::as_str));
    let Some((verb, doing)) = git_phrase(&sub) else { return Read::Unknown };
    Read::Words(Stage::of(verb, doing, git_names(&sub, words.get(1..).unwrap_or(&[]), cwd, home)))
}

fn git_names(sub: &str, rest: &[String], cwd: &str, home: &str) -> Vec<String> {
    if sub == "commit" {
        // The message is absent when it came in on stdin (`git commit -F -`).
        let message = rest.iter().position(|w| matches!(unquote(w).as_str(), "-m" | "--message")).map(|at| unquote(rest.get(at + 1).map_or("", String::as_str))).unwrap_or_default();
        return if message.is_empty() { vec![] } else { vec![message] };
    }
    let Some(args) = arguments_of(rest, &["-m", "--message", "-C", "-S"]) else { return vec![] };
    if matches!(sub, "push" | "merge" | "checkout" | "switch" | "fetch" | "pull") {
        let refs: Vec<&String> = args.iter().filter(|a| !a.is_empty() && !matches!(a.as_str(), "origin" | "upstream" | "." | "--" | "HEAD")).collect();
        return refs.last().map(|r| vec![(*r).clone()]).unwrap_or_default();
    }
    let files: Vec<String> = args.iter().filter(|a| looks_like_a_file(a)).map(|a| short_path(a, cwd, home)).collect();
    // A history command often names something that is not a path (`git log --grep <word>`).
    if !files.is_empty() {
        return files;
    }
    args.first().map(|a| vec![a.clone()]).unwrap_or_default()
}

/// `sed -i` changes a file and `sed -n 1,200p <file>` reads part of one (`sedWords`).
fn sed_words(rest: &[String], cwd: &str, home: &str) -> Read {
    let Some(args) = arguments_of(rest, &[]) else { return Read::Unknown };
    let flags: Vec<String> = rest.iter().map(|w| unquote(w)).filter(|w| is_flag(w)).collect();
    let in_place = flags.iter().any(|f| f.starts_with("-i"));
    // The file is the last argument that is not the script; a line cut short may have lost it.
    let file = args.iter().rev().find(|a| looks_like_a_file(a) && !SED_SCRIPT.test(a));
    if in_place {
        return file.map_or(Read::Unknown, |f| Read::Words(Stage::of("changed", "Changing", vec![short_path(f, cwd, home)])));
    }
    if !flags.iter().any(|f| matches!(f.as_str(), "-n" | "-e" | "-E")) {
        return Read::Unknown;
    }
    file.map_or(Read::Unknown, |f| Read::Words(Stage::of("read", "Reading", vec![short_path(f, cwd, home)])))
}

/// What a package runner ran (`runnerWords`).
fn runner_words(rest: &[String], cwd: &str, home: &str) -> Read {
    let words: Vec<String> = rest.iter().map(|w| unquote(w)).filter(|w| !matches!(w.as_str(), "--yes" | "-y" | "-s")).collect();
    let mut tool = words.first().cloned().unwrap_or_default();
    if tool == "run" || tool == "exec" {
        tool = words.get(1).cloned().unwrap_or_default();
    }
    // `npx vitest@4` names a version, not a tool.
    let tool = VERSION.strip(&tool);
    let after = words.get(1..).unwrap_or(&[]).to_vec();
    if tool == "test" || matches!(tool.as_str(), "vitest" | "jest" | "pytest" | "mocha" | "ava") {
        return Read::Words(test_words(&after));
    }
    if tool == "tsc" || tool == "typecheck" {
        return Read::Words(Stage::of("checked the types", "Checking the types", vec![]));
    }
    if tool == "eslint" || tool == "lint" {
        return Read::Words(Stage::of("checked the code style", "Checking the code style", vec![]));
    }
    if tool == "build" || (tool == "vite" && words.iter().any(|w| w == "build")) {
        return Read::Words(Stage::of("built the app", "Building the app", vec![]));
    }
    if matches!(tool.as_str(), "install" | "ci" | "i") {
        return Read::Words(Stage::of("installed the packages", "Installing the packages", vec![]));
    }
    if matches!(tool.as_str(), "node" | "python" | "python3" | "ruby" | "osascript" | "deno" | "tsx") {
        return stage_words(&words.join(" "), cwd, home);
    }
    Read::Unknown
}

/// The test files a run names: anything that looks like a path counts, since a log cuts a long
/// line and the last file often arrives without its ending (`testWords`).
fn test_words(rest: &[String]) -> Stage {
    let args: Vec<String> = rest.iter().map(|w| unquote(w)).filter(|w| !is_flag(w) && w != "run").collect();
    let files = args.iter().filter(|w| TEST_FILE.test(w) || w.contains('/')).map(|w| last_segment(w)).collect();
    Stage::of(TESTS.0, TESTS.1, files).many(TESTS.2)
}

/// Up to three names, then a count that says what it counted (`joinNames`).
fn join_names(names: &[String], many: &str) -> String {
    let mut list: Vec<&String> = Vec::new();
    for name in names.iter().filter(|n| !n.is_empty()) {
        if !list.contains(&name) {
            list.push(name);
        }
    }
    match list.len() {
        0 => String::new(),
        1 => list[0].clone(),
        n if n > 3 => format!("{n} {many}"),
        n => format!("{} and {}", list[..n - 1].iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "), list[n - 1]),
    }
}

/// What a shell command did, in words, or None to leave it as the command (`commandWork`).
/// Every segment has to be known and all have to agree on one verb: a wrong verb is worse
/// than the command.
pub fn command_work(raw: &str, cwd: &str, home: &str) -> Option<CommandWork> {
    let command = plain_command(raw);
    if command.is_empty() {
        return None;
    }
    let mut steps: Vec<Stage> = Vec::new();
    for segment in split_outside(&command, &["&&", "||", ";", "\n"]) {
        // A pipeline's first stage is what ran; the rest shapes its output.
        let stage = split_outside(&segment, &["|"]).into_iter().next().unwrap_or_default();
        match stage_words(&stage, cwd, home) {
            Read::Unknown => return None,
            Read::Noise => {}
            Read::Words(words) => steps.push(words),
        }
    }
    let first = steps.first()?;
    if steps.iter().any(|s| s.verb != first.verb) {
        return None;
    }
    if first.whole {
        return Some(CommandWork { verb: first.verb.clone(), doing: first.doing.clone(), subject: first.names.first().cloned().unwrap_or_default() });
    }
    let mut names: Vec<String> = Vec::new();
    for name in steps.iter().flat_map(|s| s.names.iter()).filter(|n| !n.is_empty()) {
        if !names.contains(name) {
            names.push(name.clone());
        }
    }
    // "Made a folder, 4 folders" would be the verb and the count disagreeing.
    let (verb, doing) = match first.plural {
        Some(plural) if names.len() > 1 => (plural.0.to_string(), plural.1.to_string()),
        _ => (first.verb.clone(), first.doing.clone()),
    };
    Some(CommandWork { verb, doing, subject: join_names(&names, first.many.unwrap_or("files")) })
}

/* ------------------------------------ what a run says ------------------------------------ */

/// What a fold says for the lines in it (`activitySummary`): one line's own words, or the kinds
/// of thing it did.
pub fn activity_summary<'a>(items: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let items: Vec<(&str, &str)> = items.into_iter().collect();
    if items.is_empty() {
        return String::new();
    }
    let text = if items.len() == 1 {
        js::trim(&format!("{} {}", items[0].0, items[0].1)).to_string()
    } else {
        let mut kinds: Vec<&str> = Vec::new();
        for (verb, _) in &items {
            let kind = match *verb {
                "ran" => "ran commands",
                "read" => "read files",
                "wrote" => "wrote files",
                "changed" => "changed files",
                "deleted" => "deleted files",
                "searched for" => "searched files",
                "looked for" => "looked for files",
                "fetched" => "read web pages",
                "moved" => "moved to this Mac",
                other => other,
            };
            if !kinds.contains(&kind) {
                kinds.push(kind);
            }
        }
        kinds.join(", ")
    };
    js::upper_first(&text)
}

/// A failure inside a fold is said on its closed line (`runFailures`).
pub fn run_failures(failed: usize) -> String {
    if failed == 0 { String::new() } else { format!("{failed} failed") }
}

/// What a run past its cap left out (`runOverflow`).
pub fn run_overflow(more: usize) -> String {
    match more {
        0 => String::new(),
        1 => "and 1 more thing it ran, not shown".to_string(),
        n => format!("and {n} more things it ran, not shown"),
    }
}

/// The line that stands for a long conversation's missing middle (`conversationGap`).
pub fn conversation_gap(omitted: usize) -> String {
    match omitted {
        0 => String::new(),
        1 => "Show the 1 message in between".to_string(),
        n => format!("Show the {n} messages in between"),
    }
}

/// The steps a live line lists (`LIVE_STEPS_SHOWN`).
pub const LIVE_STEPS_SHOWN: usize = 5;

// A model writing a tool call sometimes leaves the call's closing tags in a field as plain
// text, and everything from the first one on is cut (`LEAKED`). Written in two halves here so
// the source never holds the tag itself.
static LEAKED: std::sync::LazyLock<js::Re> = std::sync::LazyLock::new(|| {
    let ns = concat!("ant", "ml:");
    js::Re::new(&format!(r"<\/(?:{ns})?(?:result|invoke|parameter|function_calls)>|<(?:{ns})?parameter\s+name="), "")
});

/// Where tool-call scaffolding leaked into a message, outside its code spans (`firstLeak`).
fn first_leak(text: &str) -> Option<usize> {
    let mut code: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < text.len() {
        if text[i..].starts_with("```") {
            let stop = text[i + 3..].find("```").map_or(text.len(), |end| i + 3 + end + 3);
            code.push((i, stop));
            i = stop;
            continue;
        }
        if text[i..].starts_with('`') {
            let stop = text[i + 1..].find('`').map_or(i + 1, |end| i + 1 + end + 1);
            code.push((i, stop));
            i = stop;
            continue;
        }
        i += text[i..].chars().next().map_or(1, char::len_utf8);
    }
    LEAKED.matches(text).into_iter().map(|(start, _)| start).find(|at| !code.iter().any(|(s, e)| at >= s && at < e))
}

/// A message with leaked tool-call scaffolding cut off it (`unleaked`). It runs before two
/// messages are compared, because the leak makes the broken copy the longer one.
pub fn unleaked(text: &str) -> String {
    match first_leak(text) {
        Some(at) => js::trim_end(&text[..at]).to_string(),
        None => text.to_string(),
    }
}

/// The steps a live run took since it last said something, with the ones still running taken
/// out (`previousActivity`): a finished step whose words are a running one's is that one.
pub fn previous_activity<T>(history: Vec<T>, running: &[(String, Option<f64>)], words: impl Fn(&T) -> (String, Option<f64>)) -> Vec<T> {
    let mut previous = history;
    let normal = |s: &str| js::trim(&SPACES.replace_all(s, |_| " ".to_string())).to_string();
    for (detail, started_at) in running {
        for at in (0..previous.len()).rev() {
            let (row, row_at) = words(&previous[at]);
            if let (Some(row_at), Some(started_at)) = (row_at, started_at)
                && row_at < started_at - 1000.0
            {
                continue;
            }
            if normal(&row) == normal(detail) {
                previous.remove(at);
                break;
            }
        }
    }
    previous
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO: &str = "/Users/you/Desktop/dev/zero";

    fn say(command: &str) -> Option<String> {
        command_work(command, "", "").map(|w| js::trim(&format!("{} {}", w.verb, w.subject)).to_string())
    }

    /// `tests/the-work-lines-are-clean.test.mts`.
    #[test]
    fn what_it_did_in_her_words() {
        for (tool, verb) in [("Bash", "ran"), ("Read", "read"), ("Write", "wrote"), ("Edit", "changed"), ("Grep", "searched for"), ("WebFetch", "fetched"), ("", "did something")] {
            assert_eq!(work_verb(tool), verb);
        }
        assert_eq!(work_verb("mcp__plugin_paper-desktop_paper__write_html"), "write html");
        assert_eq!(work_verb("mcp__plugin_paper-desktop_paper__get_screenshot"), "get screenshot");
        assert_eq!(work_verb("mcp__agentbox__list_work_items"), "list work items");
        assert_eq!(work_verb("cnvs_open_browser"), "cnvs open browser");
        assert_eq!(work_verb("tabs_context_mcp"), "tabs context mcp");
        assert_eq!(work_verb("SomethingNew"), "somethingnew");
        assert_eq!(work_verb("Skill"), "used a skill");
        assert_eq!(work_verb("AskUserQuestion"), "asked a question");
    }

    #[test]
    fn a_path_is_shown_from_the_end() {
        assert_eq!(short_path(&format!("{ZERO}/renderer/src/components/Tweaks.tsx"), ZERO, ""), "…/src/components/Tweaks.tsx");
        assert_eq!(short_path(&format!("{ZERO}/shared/agents.mts"), ZERO, ""), "shared/agents.mts");
        assert_eq!(short_path(&format!("{ZERO}/CLAUDE.md"), ZERO, ""), "CLAUDE.md");
        assert_eq!(short_path("/Users/you/.claude/settings.json", ZERO, "/Users/you"), "~/.claude/settings.json");
        assert_eq!(short_path("/Users/leon/.claude/settings.json", "/Users/leon/dev/site", "/Users/leon"), "~/.claude/settings.json");
        assert_eq!(short_path("/private/tmp/claude-501/-Users-you-Desktop-dev-harbour/35c5c838/scratch/x.md", ZERO, ""), "…/35c5c838/scratch/x.md");
        assert_eq!(short_path("agentbox-a1", "", ""), "agentbox-a1");
        assert_eq!(short_path("", "", ""), "");
        assert_eq!(short_path("/x/store.mjs", "", ""), "/x/store.mjs");
        assert_eq!(short_path("/etc/hosts", "", ""), "/etc/hosts");
    }

    #[test]
    fn a_command_has_its_scaffolding_off_the_front() {
        assert_eq!(plain_command(&format!("cd {ZERO}/renderer/src && python3 - <<'PY'\np='components/Tweaks.tsx'\nPY")), "python3 - … p='components/Tweaks.tsx'");
        assert_eq!(plain_command(&format!("cd {ZERO} && git add renderer/src/components/List.tsx tests/day-groups.test.mjs")), "git add renderer/src/components/List.tsx tests/day-groups.test.mjs");
        assert_eq!(plain_command("export PATH=\"$HOME/.nvm/versions/node/$(ls ~/.nvm/versions/node | tail -1)/bin:$PATH\"; npx electron ."), "npx electron .");
        assert_eq!(plain_command("export PATH=\"/Users/you/.nvm/versions/node/v22.14.0/bin:$PATH\"; cd /x/y && npx vitest run"), "npx vitest run");
        assert_eq!(plain_command("cat <<'EOF'\n  a\n  b\nEOF"), "cat … a");
        assert_eq!(plain_command("node -e \"x\" <<PY\nstuff\nPY"), "node -e \"x\" … stuff");
        assert_eq!(plain_command(&format!("cd {ZERO}")), format!("cd {ZERO}"));
        assert_eq!(plain_command(""), "");
        assert_eq!(plain_command("echo \"=== main-process files modified ===\" && git status --short main/"), "git status --short main/");
        assert_eq!(plain_command("echo \"=== running electron ===\"; ps -o pid= -p 123"), "ps -o pid= -p 123");
        assert_eq!(plain_command("echo \"launched pid $!\""), "echo \"launched pid $!\"");
        assert_eq!(plain_command("echo hello && ls"), "echo hello && ls");
        assert_eq!(plain_command("cd /x && python3 - <<'PY'\np='styles.css'; s=open(p).read()\nPY"), "python3 - … p='styles.css'; s=open(p).read()");
        assert_eq!(plain_command("python3 - <<'PY'\n# what this does\nimport re\nPY"), "python3 - … import re");
        assert_eq!(plain_command("npx some-long-tool --with --flags <<'PY'\np='x'\nPY"), "npx some-long-tool --with --flags …");
    }

    /// `tests/a-thread-reads-in-plain-words.test.mts`.
    #[test]
    fn the_stores_bookkeeping_is_not_the_work() {
        for name in ["mcp__agentbox__claim_work_item", "mcp__agentbox__list_documents", "mcp__agentbox__update_work_item", "ToolSearch", "mcp__daydream__claim_work_item", "mcp__daydream__release_work_item", "mcp__agentbox__list_work_items"] {
            assert!(is_bookkeeping(name), "{name}");
        }
        for name in ["mcp__agentbox__read_document", "mcp__agentbox__write_document", "mcp__agentbox__create_work_item", "mcp__agentbox__look_at_page", "mcp__claude_ai_Linear__list_teams", "Bash", "Read", ""] {
            assert!(!is_bookkeeping(name), "{name}");
        }
    }

    #[test]
    fn a_command_in_plain_words() {
        assert_eq!(command_work("cat README.md; echo ---; cat package.json", "", ""), Some(CommandWork { verb: "read".into(), doing: "Reading".into(), subject: "README.md and package.json".into() }));
        for (command, said) in [
            ("sed -n 1,200p renderer/src/item-thread.ts", "read renderer/src/item-thread.ts"),
            ("grep -rn \"workVerb\" shared/agents.mts", "searched for workVerb"),
            ("ls renderer/src", "listed renderer/src"),
            ("ls", "listed ./"),
            ("npx vitest run tests/one-door-one-list.test.mts --maxWorkers=1", "ran the tests one-door-one-list.test.mts"),
            ("npm test", "ran the tests"),
            ("npx tsc -p renderer --noEmit", "checked the types"),
            ("git status", "checked what changed"),
            ("git -C /tmp/w log -5", "read the code history"),
            ("git add -A && git commit -m \"a change\"", "saved the changes a change"),
            ("sed -i \"\" s/a/b/ src/cart.js", "changed src/cart.js"),
            ("mkdir -p scripts/scratch", "made a folder scripts/scratch"),
            ("rm -rf /tmp/w/frames", "deleted /tmp/w/frames"),
            ("node scripts/scratch/measure.mjs", "ran a script measure.mjs"),
            ("python3 - <<'PY'\nimport re\nPY", "ran a script import re"),
            // A developer keeps the detail they read the line for.
            ("git push origin agentbox/w-1 2>&1 | tail -5", "pushed the branch agentbox/w-1"),
            ("git merge --no-edit agentbox/w-1", "merged the branch agentbox/w-1"),
            ("git commit -m \"Say what a command did\"", "saved the changes Say what a command did"),
            ("git add renderer/src/App.tsx", "saved the changes renderer/src/App.tsx"),
            ("git diff renderer/src/styles.css", "checked what changed renderer/src/styles.css"),
            ("npx vitest run tests/team-one-page-s", "ran the tests team-one-page-s"),
            ("node -e \"console.log(1)\"", "ran a script console.log(1)"),
            ("git status --short", "checked what changed"),
        ] {
            assert_eq!(say(command).as_deref(), Some(said), "{command}");
        }
        // Nothing rather than the wrong thing.
        for command in ["ffmpeg -y -i in.mov out.mp4", "docker ps --format \"{{.Names}}\"", "cat a.md && ffmpeg -i in.mov out.mp4", "cat a.md && rm b.md", ""] {
            assert_eq!(command_work(command, "", ""), None, "{command}");
        }
        // A few files named, many counted.
        assert_eq!(command_work("cat a.md b.md", "", "").unwrap().subject, "a.md and b.md");
        assert_eq!(command_work("cat a.md b.md c.md", "", "").unwrap().subject, "a.md, b.md and c.md");
        assert_eq!(command_work("cat a.md b.md c.md d.md", "", "").unwrap().subject, "4 files");
        assert_eq!(command_work(&format!("cd {ZERO} && git status"), "", "").unwrap().verb, "checked what changed");
        assert_eq!(command_work("sed -i \"\" s/a/b/ src/cart.js", "", "").unwrap().verb, "changed");
    }

    #[test]
    fn a_run_of_them_folds_into_words_and_not_tool_names() {
        assert_eq!(activity_summary([("changed", "src/cart.js")]), "Changed src/cart.js");
        assert_eq!(activity_summary([("changed", "a"), ("read", "b")]), "Changed files, read files");
        // `tests/past-activity-folds-into-a-quiet-summary.test.mts`.
        assert_eq!(activity_summary([("read", ""), ("ran", ""), ("read", "")]), "Read files, ran commands");
        assert_eq!(activity_summary([("ran", "npm test")]), "Ran npm test");
        assert_eq!(activity_summary(std::iter::empty()), "");
        assert_eq!(activity_summary([("looked up a tool", ""), ("looked up a tool", "")]), "Looked up a tool");
        assert_eq!(run_failures(1), "1 failed");
        assert_eq!(run_failures(0), "");
    }

    #[test]
    fn what_a_thread_did_not_keep_is_said_out_loud() {
        assert_eq!(conversation_gap(0), "");
        assert_eq!(conversation_gap(1), "Show the 1 message in between");
        assert_eq!(conversation_gap(63), "Show the 63 messages in between");
        assert_eq!(run_overflow(3), "and 3 more things it ran, not shown");
    }

    #[test]
    fn leaked_tool_call_scaffolding_is_cut_off_a_message() {
        let close = concat!("</", "ant", "ml:parameter>");
        assert_eq!(unleaked(&format!("Done, all green.\n{close}\nmore")), "Done, all green.");
        assert_eq!(unleaked(&format!("The tag `{close}` is fine in code.")), format!("The tag `{close}` is fine in code."));
        assert_eq!(unleaked("Nothing leaked."), "Nothing leaked.");
    }

    /// `tests/live-steps-reset-when-the-agent-speaks.test.mts`: a step still running is not
    /// listed twice.
    #[test]
    fn a_running_step_is_not_also_a_finished_one() {
        let history = vec![("npm test".to_string(), Some(10_000.0)), ("ls".to_string(), Some(12_000.0))];
        let left = previous_activity(history, &[("npm  test".to_string(), Some(9_500.0))], |(detail, at)| (detail.clone(), *at));
        assert_eq!(left, [("ls".to_string(), Some(12_000.0))]);
    }
}
