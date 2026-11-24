//! New project: the Mac's own folder window, one pick, and the project made in that folder
//! with that folder's name (`renderer/src/components/NewProject.tsx` and its call in `App.tsx`).
//! Cancel makes nothing and says nothing; there is nothing to fill in.
//!
//! The web window asks the app for the folder window (`zero:choose-folder`), which hangs it on
//! the web window itself, as a sheet, and so would bring that window to the front over this
//! one. So this window opens the same folder window as a sheet of its own (Rune's
//! `Tx::open_folder`), where the app's would open (`chooserStartsIn`, the first of them that is
//! there), and runs on the folder picked what the app runs on it before it hands it back
//! (`folderFromChooser`): the folders that are not projects are turned down with the app's
//! own sentence. Then it writes what the web window writes (`zero:create-product`) and says
//! what the web window says. The rules are `rules/project_folder.rs`.
//!
//! Every door of the web window's that this window has opens it: the project menu of the box a
//! thread is started in (`compose.rs`), the command bar's first row (`palette.rs`) and New
//! project on Settings › Projects (`settings/projects.rs`).

use std::path::{Path, PathBuf};

use rune::Tx;
use rune::file_dialog::FileDialogOptions;
use serde_json::Value;

use crate::rules::project_folder as rules;
use crate::{State, write};

/// The home folder, which the folder checks are made against (`os.homedir()`).
fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

fn is_dir(p: &str) -> bool {
    Path::new(p).is_dir()
}

impl State {
    /// Opens the Mac's folder window over this window, where the projects already connected
    /// are. The window's own title and button are the system's: see `docs/parity-inbox-bars.md`.
    pub fn new_project(&mut self, tx: &mut Tx) {
        let known: Vec<String> = self.snap.products.iter().filter_map(|p| p.rest.get("repoPath").and_then(Value::as_str)).map(str::to_string).collect();
        let start = rules::chooser_starts_in(None, &known, &home()).into_iter().find(|p| is_dir(p));
        let options = FileDialogOptions { directory: start.map(PathBuf::from), ..FileDialogOptions::default() };
        tx.open_folder(options, |s: &mut State, picked, tx| match picked {
            Ok(Some(path)) => s.make_project_in(&path.to_string_lossy(), tx),
            // Cancelling is not an answer: nothing is made and nothing is said.
            Ok(None) => {}
            Err(e) => s.tell(&format!("The folder window did not open: {e}"), tx),
        });
    }

    /// The folder picked, checked as the app checks it, and made a project of: or the sentence
    /// that says why it cannot be one.
    pub(crate) fn make_project_in(&mut self, picked: &str, tx: &mut Tx) {
        let folder = match rules::folder_from_chooser(picked, &home(), is_dir) {
            Ok(folder) => folder,
            Err(say) => return self.tell(&say, tx),
        };
        let Some((name, repo)) = rules::project_from_folder(&folder) else {
            return;
        };
        let (channel, payload) = rules::create(&name, &repo);
        let sent = write::Sent::Project { name, repo };
        // A picture has no store behind it: its fixture carries what the page's was answered, or
        // the refusal its bridge threw (`newProjectRefused`).
        let pictured = match (self.fixture.get(rules::CREATE_PRODUCT), self.fixture.get("newProjectRefused").and_then(Value::as_str)) {
            (_, Some(why)) => Some(Err(why.to_string())),
            (Some(made), None) => Some(Ok(made.clone())),
            (None, None) => None,
        };
        if let Some(answer) = pictured {
            return landed(self, write::Landed { sent, answer, snapshot: None }, tx);
        }
        if !self.send(channel, payload, sent) {
            let why = self.refused.take().unwrap_or_default();
            self.tell(&why, tx);
        }
    }
}

/// What the store said: made, and the next thread is addressed to it, as the box remembers by
/// project (`rememberProject`, `setPickProject`); or the walk's own words for why not.
pub fn landed(s: &mut State, landed: write::Landed, tx: &mut Tx) {
    let write::Sent::Project { name, repo } = &landed.sent else {
        return;
    };
    if let Some(why) = landed.refusal() {
        return s.tell(&rules::why_not_made(&why, name), tx);
    }
    if let Some(slug) = landed.answer.as_ref().ok().and_then(|made| made.get("slug")).and_then(Value::as_str).map(str::to_string) {
        s.picks.change(|picks| picks.project = Some(slug.clone()));
        s.compose.project = Some(slug);
    }
    s.tell(&rules::made_line(name, repo), tx);
}

#[cfg(test)]
mod tests {
    use rune::{Key, KeyEvent, Mods};
    use serde_json::json;

    use crate::feel::tests::{key, run, window};
    use crate::rules::project_folder::CREATE_PRODUCT;
    use crate::write::{Landed, Sent};

    /// A folder of the system's temporary folder that is there, named as a project's would be.
    fn a_project_folder(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("w4c6-new-project-{}", std::process::id())).join(name);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap().to_string_lossy().into_owned()
    }

    #[test]
    fn the_command_bars_first_row_opens_the_folder_window_and_the_folder_picked_is_made_a_project() {
        let mut rt = window("new-project", &["--view", "inbox"]);
        rt.state.pretend_sent = true;
        run(&mut rt, 0.1);
        rt.key(KeyEvent { key: Key::Command('k'), mods: Mods { cmd: true, ..Mods::default() } });
        run(&mut rt, 0.1);
        // Its first row, lit as the bar opens.
        let rows: Vec<(String, f32)> = rt.elements().iter().filter_map(|e| Some((e.text.clone()?, e.rect[1]))).collect();
        let at = |w: &str| rows.iter().filter(|(t, _)| t == w).map(|(_, y)| *y).fold(f32::MIN, f32::max);
        assert!(at("New project…") > 0.0 && at("New project…") < at("New thread"), "{rows:?}");
        key(&mut rt, Key::Enter);
        run(&mut rt, 0.1);
        assert!(!rt.state.palette.open, "the bar closes first");
        let (kind, options) = rt.file_dialog().expect("the folder window is up");
        assert_eq!(kind, rune::file_dialog::FileDialogKind::Folder);
        // No title of its own: the app's window shows none on a Mac either.
        assert_eq!(options.title, "");
        let folder = a_project_folder("greet-js");
        rt.resolve_file_dialog(Some(folder.clone().into())).unwrap();
        run(&mut rt, 0.1);
        assert_eq!(rt.state.tried, vec![(CREATE_PRODUCT, json!({ "name": "Greet JS", "repoPath": folder }))]);
        // The store made it: it is said, and the next thread is addressed to it.
        rt.update(|s, tx| super::landed(s, Landed { sent: Sent::Project { name: "Greet JS".into(), repo: folder.clone() }, answer: Ok(json!({ "slug": "greet-js", "name": "Greet JS" })), snapshot: None }, tx));
        run(&mut rt, 0.1);
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.clone()), Some(format!("New project: Greet JS · its code is in {}", crate::rules::project_folder::short_folder(&folder))));
        assert_eq!(rt.state.compose.project.as_deref(), Some("greet-js"));
        assert_eq!(rt.state.picks.get().project.as_deref(), Some("greet-js"));
    }

    #[test]
    fn the_project_menu_of_the_box_opens_the_folder_window_and_makes_the_project() {
        // Home's box: its project menu, New project last.
        let mut rt = window("new-project-menu", &[]);
        rt.state.pretend_sent = true;
        run(&mut rt, 0.1);
        rt.update(|s, tx| s.toggle_menu(crate::compose::MenuKind::Project, tx));
        run(&mut rt, 0.1);
        let menu = rt.state.compose.menu.clone().expect("the project menu");
        let entries = rt.state.menu_entries(&menu);
        let last = entries.len() - 1;
        assert!(matches!(entries[last], crate::rules::compose::Entry::NewProject));
        assert!(rt.elements().iter().any(|e| e.text.as_deref() == Some("New project")));
        rt.update(|s, tx| s.press_entry(last, tx));
        run(&mut rt, 0.1);
        assert!(rt.state.compose.menu.is_none(), "the menu closes first");
        assert!(rt.state.refused.is_none(), "no line that it is made elsewhere");
        assert_eq!(rt.file_dialog().map(|(kind, _)| kind), Some(rune::file_dialog::FileDialogKind::Folder));
        let folder = a_project_folder("shop-front");
        rt.resolve_file_dialog(Some(folder.clone().into())).unwrap();
        run(&mut rt, 0.1);
        assert_eq!(rt.state.tried, vec![(CREATE_PRODUCT, json!({ "name": "Shop Front", "repoPath": folder }))]);
        // Settings › Projects' own button is `settings::tests::new_project_on_the_projects_page_opens_the_folder_window_over_settings`.
    }

    #[test]
    fn a_cancel_makes_nothing_and_a_folder_that_is_not_a_project_is_turned_down_in_the_apps_words() {
        let mut rt = window("new-project-no", &["--view", "inbox"]);
        rt.state.pretend_sent = true;
        run(&mut rt, 0.1);
        rt.update(|s, tx| s.new_project(tx));
        run(&mut rt, 0.1);
        rt.resolve_file_dialog(None).unwrap();
        run(&mut rt, 0.1);
        assert!(rt.state.tried.is_empty());
        assert!(rt.state.toast.is_none(), "nothing is said");
        rt.update(|s, tx| s.new_project(tx));
        run(&mut rt, 0.1);
        rt.resolve_file_dialog(Some("/Applications".into())).unwrap();
        run(&mut rt, 0.1);
        assert!(rt.state.tried.is_empty());
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("/Applications is part of the system rather than one of your projects. Pick the folder your project’s code is in."));
        // The store turned it down: its words, as the walk words them.
        rt.update(|s, tx| super::landed(s, Landed { sent: Sent::Project { name: "Shop".into(), repo: "/x/shop".into() }, answer: Err("product shop already exists".into()), snapshot: None }, tx));
        assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("You already have a project called Shop. Give this one another name."));
    }
}
