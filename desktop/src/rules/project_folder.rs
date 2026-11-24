//! A new project from a folder picked in the Mac's own folder window: which folders may be one
//! and the sentence when one may not (`tidyFolder`, `insideGitFolder`, `folderFromChooser`,
//! `checkProjectFolder` and `chooserStartsIn` in `shared/project-folder-check.mts`, which the
//! app's `zero:choose-folder` runs on the path the window hands back), what the project is
//! called (`projectFromFolder` in `renderer/src/project-folder.ts`, `nameFromFolder` in
//! `renderer/src/onboarding.ts`, `titleFromFileName` in `shared/name-words.mts`), and what is
//! said once the store has made it or turned it down (the `NewProject` call in `App.tsx`,
//! `shortFolder`, `whyNotMade`).

use serde_json::{Value, json};

use super::js::{self, re};

/// The app's name, as `shared/product-name.mjs` spells it at the start of a sentence.
const NAME: &str = "Mercury";

/// The channel a project is made on.
pub const CREATE_PRODUCT: &str = "zero:create-product";

/// The guarded folders, by their name directly under the home folder (`GUARDED`).
const GUARDED: [&str; 8] = ["Desktop", "Documents", "Downloads", "Movies", "Music", "Pictures", "Library", "Public"];

/// Top-level directories that are the Mac and not anybody's work (`SYSTEM`).
const SYSTEM: [&str; 16] = ["/", "/Applications", "/Library", "/System", "/Users", "/Volumes", "/private", "/tmp", "/var", "/etc", "/usr", "/bin", "/sbin", "/opt", "/net", "/cores"];

/// The folders people keep code in, tried in this order when there is nothing to learn from (`USUAL`).
const USUAL: [&str; 8] = ["dev", "code", "src", "Projects", "projects", "Developer", "repos", "workspace"];

/// The folder a path names, with `.`, `..`, doubled and trailing slashes gone; `~` left as it
/// is (`tidyFolder`).
pub fn tidy_folder(p: &str) -> String {
    let s = js::trim(p);
    if s.is_empty() {
        return String::new();
    }
    let root = if s.starts_with('~') { "~" } else if s.starts_with('/') { "/" } else { "" };
    let mut parts: Vec<&str> = Vec::new();
    for part in s[root.len()..].split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if !parts.is_empty() {
                parts.pop();
            } else if root.is_empty() {
                parts.push("..");
            }
            continue;
        }
        parts.push(part);
    }
    match root {
        "~" if parts.is_empty() => "~".to_string(),
        "~" => format!("~/{}", parts.join("/")),
        "/" => format!("/{}", parts.join("/")),
        _ => parts.join("/"),
    }
}

/// Whether a path is a `.git` folder or anywhere inside one (`insideGitFolder`).
pub fn inside_git_folder(p: &str) -> bool {
    tidy_folder(&p.replace('\\', "/")).split('/').any(|part| part == ".git")
}

fn under_home(p: &str, home: &str) -> Option<String> {
    let h = tidy_folder(home);
    if h.is_empty() {
        return None;
    }
    if p == h {
        return Some(String::new());
    }
    p.strip_prefix(&format!("{h}/")).map(str::to_string)
}

/// Whether a folder can be a project, and the sentence when it cannot (`checkProjectFolder`):
/// None for yes.
pub fn check_project_folder(folder: &str, home: &str) -> Option<String> {
    let p = tidy_folder(folder);
    if p.is_empty() {
        return None;
    }
    let rel = if p == "~" || p == "~/" { Some(String::new()) } else if let Some(rest) = p.strip_prefix("~/") { Some(rest.to_string()) } else { under_home(&p, home) };
    if rel.as_deref() == Some("") {
        return Some("That is your whole home folder. Agents would work across everything on this computer, and the system will ask you about Downloads, Music and every app you have. Pick the folder your project’s code is in.".to_string());
    }
    if let Some(rel) = rel.as_deref().filter(|rel| GUARDED.contains(rel)) {
        return Some(format!("{rel} holds a lot more than one project, and the system guards it, so choosing it means a run of permission panels. Pick the folder inside it that your project’s code is in."));
    }
    if SYSTEM.contains(&p.as_str()) {
        return Some(format!("{p} is part of the system rather than one of your projects. Pick the folder your project’s code is in."));
    }
    if inside_git_folder(&p) {
        return Some(format!("{p} is inside a repository’s .git folder, which is git’s own bookkeeping. Pick the folder your project’s code is in."));
    }
    if VOLUME.test(&p) {
        return Some(format!("{p} is a whole disk rather than one project. Pick the folder your project’s code is in."));
    }
    None
}

re!(VOLUME, r"^/Volumes/[^/]+$", "");

/// What a folder window's answer means as a project (`folderFromChooser`): a pick ending in
/// `.git` is the repository above it; a folder that may not be one, or does not exist, is
/// turned down with its sentence.
pub fn folder_from_chooser(raw: &str, home: &str, is_dir: impl Fn(&str) -> bool) -> Result<String, String> {
    let mut picked = tidy_folder(raw);
    if picked.split('/').next_back() == Some(".git") {
        picked = picked[..picked.len() - "/.git".len()].to_string();
        if picked.is_empty() {
            picked = "/".to_string();
        }
    }
    if let Some(say) = check_project_folder(&picked, home) {
        return Err(say);
    }
    if !is_dir(&picked) {
        return Err(format!("{picked} does not exist. Pick the folder your project’s code is in."));
    }
    Ok(picked)
}

/// Where the folder window opens, best first (`chooserStartsIn`): where it was asked to, then
/// beside the projects already connected, commonest parent first, then the folders people keep
/// code in, and the home folder last.
pub fn chooser_starts_in(start_in: Option<&str>, known: &[String], home: &str) -> Vec<String> {
    let h = tidy_folder(home);
    let mut counts: Vec<(String, usize)> = Vec::new();
    for f in known {
        let tidied = tidy_folder(f);
        let p = GIT_TAIL.strip(&tidied);
        let Some(cut) = p.rfind('/').filter(|cut| *cut > 0) else { continue };
        let parent = &p[..cut];
        if check_project_folder(parent, &h).is_some() {
            continue;
        }
        match counts.iter_mut().find(|(seen, _)| seen == parent) {
            Some(entry) => entry.1 += 1,
            None => counts.push((parent.to_string(), 1)),
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.len().cmp(&b.0.len())));
    let mut order = vec![tidy_folder(start_in.unwrap_or(""))];
    order.extend(counts.into_iter().map(|(parent, _)| parent));
    if !h.is_empty() {
        order.extend(USUAL.iter().map(|name| format!("{h}/{name}")));
    }
    order.push(h);
    let mut seen = Vec::new();
    for folder in order {
        if !folder.is_empty() && !seen.contains(&folder) {
            seen.push(folder);
        }
    }
    seen
}

re!(GIT_TAIL, r"/\.git(/.*)?$", "");

/// The short words kept in capitals in a name (`SHOUTED` in `shared/name-words.mts`).
const SHOUTED: [&str; 31] = ["ai", "api", "cdn", "cli", "cms", "crm", "css", "db", "html", "http", "ios", "js", "json", "jsx", "llm", "mcp", "npm", "php", "pr", "qa", "sdk", "seo", "sql", "svg", "ts", "tsx", "tv", "ui", "ux", "xml", "yaml"];

/// One word of a name: shouted if it is on the list, else its first letter up (`nameWord`).
fn name_word(word: &str) -> String {
    if SHOUTED.contains(&word.to_lowercase().as_str()) {
        return word.to_uppercase();
    }
    // `/^\w/`: a first letter, digit or underscore, raised.
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() || c == '_' => c.to_ascii_uppercase().to_string() + chars.as_str(),
        _ => word.to_string(),
    }
}

re!(JOINERS, r"[-_.]+", "");

/// A file or folder name as a name: `greet-js` reads "Greet JS" (`titleFromFileName`).
pub fn title_from_file_name(name: &str) -> String {
    let spaced = JOINERS.replace_all(name, |_| " ".to_string());
    spaced.split_whitespace().map(name_word).collect::<Vec<_>>().join(" ")
}

/// A project's name from its folder (`nameFromFolder`).
pub fn name_from_folder(path: &str) -> String {
    let path = path.trim_end_matches('/');
    title_from_file_name(path.rsplit('/').next().unwrap_or(""))
}

/// What a picked folder makes: its name and path, or nothing for nothing (`projectFromFolder`).
pub fn project_from_folder(picked: &str) -> Option<(String, String)> {
    let repo = js::trim(picked).trim_end_matches('/').to_string();
    if repo.is_empty() {
        return None;
    }
    let name = name_from_folder(&repo);
    (!name.is_empty()).then_some((name, repo))
}

/// What is written to make it (`api.createProduct({ name, repoPath })`).
pub fn create(name: &str, repo: &str) -> (&'static str, Value) {
    (CREATE_PRODUCT, json!({ "name": name, "repoPath": repo }))
}

re!(USERS, r"^/Users/[^/]+", "");

/// The home folder as `~`, the way the rail and Settings write a path (`shortFolder`).
pub fn short_folder(full: &str) -> String {
    USERS.replace(full, |_| "~".to_string())
}

/// What is said once the store has made it.
pub fn made_line(name: &str, repo: &str) -> String {
    format!("New project: {name} · its code is in {}", short_folder(repo))
}

re!(ALREADY, r"already exists", "i");
re!(NEEDS_NAME, r"needs a name", "i");
re!(REMOTE, r"^Error invoking remote method '[^']*':\s*", "i");
re!(ERROR, r"^Error:\s*", "i");
re!(ENDS, r"[.!?]$", "");

/// What is said when the store did not make it, in the walk's own words (`whyNotMade`).
pub fn why_not_made(message: &str, name: &str) -> String {
    let said = js::trim(message);
    if ALREADY.test(said) {
        return format!("You already have a project called {name}. Give this one another name.");
    }
    if NEEDS_NAME.test(said) {
        return "A project needs a name with letters or numbers in it.".to_string();
    }
    let bare = ERROR.strip(&REMOTE.strip(said));
    let why = if bare.is_empty() { "It did not say why.".to_string() } else if ENDS.test(&bare) { bare } else { format!("{bare}.") };
    format!("{NAME} could not make the project. {why}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/Users/someone";

    /// `tests/a-path-ending-in-a-dot-is-the-folder-it-names.test.mts`.
    #[test]
    fn a_path_is_the_folder_it_names() {
        for (raw, tidied) in [
            ("/workspace/repos/range-utils-js/.", "/workspace/repos/range-utils-js"),
            ("/workspace/repos/.", "/workspace/repos"),
            ("/workspace/./repos", "/workspace/repos"),
            ("/workspace/repos/range-utils-js/..", "/workspace/repos"),
            ("/workspace/repos/../other", "/workspace/other"),
            ("/workspace/repos/", "/workspace/repos"),
            ("/workspace//repos///", "/workspace/repos"),
            ("/", "/"),
            ("/..", "/"),
            ("~", "~"),
            ("~/", "~"),
            ("", ""),
            ("~/dev/house/.", "~/dev/house"),
            ("~/dev/./house", "~/dev/house"),
        ] {
            assert_eq!(tidy_folder(raw), tidied, "{raw}");
        }
        assert!(check_project_folder(&format!("{HOME}/."), HOME).is_some_and(|say| say.starts_with("That is your whole home folder.")));
        assert!(check_project_folder("~/.", HOME).is_some());
        assert!(check_project_folder(&format!("{HOME}/Documents/my-app/.."), HOME).is_some_and(|say| say.starts_with("Documents holds a lot more than one project")));
    }

    /// `tests/the-setup-refuses-your-whole-mac.test.mts`, the sentences as the screens show them.
    #[test]
    fn the_folders_that_are_not_projects_are_turned_down_with_what_to_pick_instead() {
        assert_eq!(check_project_folder(&format!("{HOME}/dev/shop"), HOME), None);
        assert_eq!(check_project_folder(&format!("{HOME}/Documents/shop"), HOME), None);
        assert_eq!(check_project_folder("/Applications", HOME).as_deref(), Some("/Applications is part of the system rather than one of your projects. Pick the folder your project’s code is in."));
        assert_eq!(check_project_folder("/Volumes/Backup", HOME).as_deref(), Some("/Volumes/Backup is a whole disk rather than one project. Pick the folder your project’s code is in."));
        assert_eq!(check_project_folder(&format!("{HOME}/dev/shop/.git/hooks"), HOME).as_deref(), Some("/Users/someone/dev/shop/.git/hooks is inside a repository’s .git folder, which is git’s own bookkeeping. Pick the folder your project’s code is in."));
        assert_eq!(check_project_folder(&format!("{HOME}/Music"), HOME).as_deref(), Some("Music holds a lot more than one project, and the system guards it, so choosing it means a run of permission panels. Pick the folder inside it that your project’s code is in."));
        // What the window hands back: a repository's .git is the repository; what is not there
        // is turned down rather than made.
        let there = |p: &str| p == "/Users/someone/dev/shop";
        assert_eq!(folder_from_chooser("/Users/someone/dev/shop/.git", HOME, there), Ok("/Users/someone/dev/shop".to_string()));
        assert_eq!(folder_from_chooser("/Users/someone/dev/shop/", HOME, there), Ok("/Users/someone/dev/shop".to_string()));
        assert_eq!(folder_from_chooser("/Users/someone/dev/gone", HOME, there), Err("/Users/someone/dev/gone does not exist. Pick the folder your project’s code is in.".to_string()));
        assert!(folder_from_chooser(HOME, HOME, |_| true).is_err_and(|say| say.starts_with("That is your whole home folder.")));
    }

    /// `tests/the-folder-chooser-opens-where-the-projects-are.test.mts`.
    #[test]
    fn the_folder_window_opens_where_the_projects_are() {
        let home = "/home/box";
        let known = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(chooser_starts_in(Some("/home/box/work/app"), &[], home)[0], "/home/box/work/app");
        assert_eq!(chooser_starts_in(None, &known(&["/home/box/code/shop"]), home)[0], "/home/box/code");
        let order = chooser_starts_in(None, &known(&["/home/box/odd/one", "/home/box/code/a", "/home/box/code/b"]), home);
        assert_eq!(order[0], "/home/box/code");
        assert!(order.contains(&"/home/box/odd".to_string()));
        assert_eq!(order.last().map(String::as_str), Some(home));
        assert_eq!(chooser_starts_in(None, &[], home), ["dev", "code", "src", "Projects", "projects", "Developer", "repos", "workspace"].iter().map(|n| format!("{home}/{n}")).chain([home.to_string()]).collect::<Vec<_>>());
        assert!(chooser_starts_in(None, &[], "").is_empty());
        assert_eq!(chooser_starts_in(Some("/home/box/dev"), &known(&["/home/box/dev/a"]), home).iter().filter(|f| *f == "/home/box/dev").count(), 1);
        // A project kept as its .git folder opens beside the repository, not in it.
        assert_eq!(chooser_starts_in(None, &known(&["/home/box/code/shop/.git"]), home)[0], "/home/box/code");
    }

    #[test]
    fn a_project_is_named_after_its_folder_and_said_in_the_walks_words() {
        assert_eq!(project_from_folder("/Users/someone/Desktop/dev/agentbox-team/"), Some(("Agentbox Team".to_string(), "/Users/someone/Desktop/dev/agentbox-team".to_string())));
        assert_eq!(name_from_folder("/Users/someone/dev/greet-js"), "Greet JS");
        assert_eq!(name_from_folder("/Users/someone/dev/my_iOS.app"), "My IOS App");
        assert_eq!(project_from_folder("  "), None);
        assert_eq!(create("Shop", "/Users/someone/dev/shop"), (CREATE_PRODUCT, json!({ "name": "Shop", "repoPath": "/Users/someone/dev/shop" })));
        assert_eq!(made_line("Shop", "/Users/someone/dev/shop"), "New project: Shop · its code is in ~/dev/shop");
        assert_eq!(made_line("Shop", "/opt/shop"), "New project: Shop · its code is in /opt/shop");
        assert_eq!(why_not_made("product shop already exists", "Shop"), "You already have a project called Shop. Give this one another name.");
        assert_eq!(why_not_made("A project needs a name", "Shop"), "A project needs a name with letters or numbers in it.");
        assert_eq!(why_not_made("Error invoking remote method 'zero:create-product': Error: The disk is full", "Shop"), "Mercury could not make the project. The disk is full.");
        assert_eq!(why_not_made("", "Shop"), "Mercury could not make the project. It did not say why.");
    }
}
