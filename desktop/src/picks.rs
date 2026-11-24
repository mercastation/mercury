//! What the person last picked in the box a thread is started in: the project, and for each
//! coding agent its model and how hard it thinks; how the Inbox is viewed and filtered; and the
//! stopped cloud accounts the cloud menu has already been opened on. The web window keeps these
//! in its own storage (`zero.lastProduct`, `zero.lastModel`, `zero.lastEffort`,
//! `threads.display.inbox`, `zero.cloudBrokenSeen`), which this window cannot read, so it keeps
//! its own: in memory, and in one small file beside the note about the app's door, written
//! when a pick changes and read when the window opens.
//!
//! A picture drawn from a fixture never reads or writes that file: its picks are the fixture's
//! own (`picks`), or none. Nor does a window pointed by hand at some engine (`--door`), which
//! may be a scratch one: its picks last as long as the window, unless `--picks <file>` (or
//! `MERCURY_PICKS`) says where to keep them.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The file's name, beside `local-door.json` in the app's own folder.
const FILE: &str = "native-picks.json";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Picks {
    /// The project last picked or sent to, by its slug.
    pub project: Option<String>,
    /// The model last picked for each coding agent, by the agent's id. An agent left on its
    /// default has no entry.
    pub model: BTreeMap<String, String>,
    /// And the level, the same way.
    pub effort: BTreeMap<String, String>,
    /// The Inbox's View and filters, in the shape the web window keeps its own.
    pub display: crate::rules::display::Display,
    /// The cloud accounts that stopped working and that the cloud menu was opened on, as
    /// `engine:key`: the cloud button stops asking for their sign-in.
    pub cloud_broken_seen: Vec<String>,
    /// Two of Settings' switches that each window keeps for itself, as the web window keeps
    /// its own (`zero.hints`, `agentbox.sidebarShrinks`): whether a key hint comes under a
    /// pointer that waits, and whether the rail folds to its marks. Both are on until turned off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_hints: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rail_shrinks: Option<bool>,
    /// The rounds of an agent's choices folded away by the little arrow in their corner, the
    /// newest hundred, as the web window keeps its own (`one.choicesFoldedOn`, `foldKey`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices_folded: Vec<String>,
    /// How much of the body a file beside a conversation takes, where a hand last dragged its
    /// divider (`zero.docPane.split.v1`); none until one has.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_split: Option<f32>,
    /// What each sheet keeps, by `product/path`: its sort and filter, the widths a hand set and
    /// what was chosen that a sheet cannot hold (`agentbox.table-view`, `-widths`, `-looks`).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub tables: BTreeMap<String, crate::table::Kept>,
    /// Where the person was in each conversation they left, the newest fifty, as the web window
    /// keeps its own across a restart (`zero.threadPlaces`, `thread-place.ts`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub thread_places: Vec<ThreadPlace>,
    /// The sidebar's sections folded away by a press on their name, as the web window keeps its
    /// own (`agentbox.sidebarFolds`): `workspace`, `recent`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rail_folds: Vec<String>,
}

/// Where the person was in one conversation when they left it (`ThreadPlace`): its newest answer
/// then, how far they had scrolled it themselves, and what they had unfolded in it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThreadPlace {
    /// `product/id`.
    pub thread: String,
    pub news: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<f32>,
    /// Where they had scrolled it to, as the row of the conversation at the top of the page and
    /// how far the page's top was into it: rows away from the screen are not laid out, so how
    /// far down a row stands is not known until it is, and an offset alone would land on
    /// another line. Kept beside `top`, which a place written before has alone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub into: Option<f32>,
    /// The earlier messages shown, and the middle asked back.
    pub earlier: bool,
    pub whole: bool,
    /// The folds of what a run ran that were open, and the lines in them and how far, by name.
    pub runs: Vec<String>,
    pub lines: Vec<(String, u8)>,
}

/// The picks, and the file they are kept in when there is one.
#[derive(Debug, Default)]
pub struct Kept {
    picks: Picks,
    file: Option<PathBuf>,
}

impl Kept {
    /// The picks kept in `file`. A file that is missing or cannot be read is no picks: a window
    /// that will not open over a bad file is worse than one that opens on the defaults.
    pub fn at(file: Option<PathBuf>) -> Kept {
        let picks = file.as_ref().and_then(|file| std::fs::read_to_string(file).ok()).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default();
        Kept { picks, file }
    }

    /// Picks that live only as long as the window does: a fixture's.
    pub fn only_here(picks: Picks) -> Kept {
        Kept { picks, file: None }
    }

    pub fn get(&self) -> &Picks {
        &self.picks
    }

    /// Changes the picks, and writes them down when they did change. A file that cannot be
    /// written is not a reason to lose the pick: it stands for as long as this window does.
    pub fn change(&mut self, change: impl FnOnce(&mut Picks)) {
        let was = self.picks.clone();
        change(&mut self.picks);
        let (Some(file), true) = (&self.file, self.picks != was) else {
            return;
        };
        // Written beside itself and moved into place, so a window opening meanwhile never
        // reads half a file.
        let draft = file.with_extension(format!("{}.tmp", std::process::id()));
        if let Some(folder) = file.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        let written = serde_json::to_string(&self.picks).map_err(|e| e.to_string()).and_then(|text| std::fs::write(&draft, text).map_err(|e| e.to_string()));
        if written.is_ok() {
            let _ = std::fs::rename(&draft, file);
        }
    }
}

/// Where the picks are kept: where the command line says; else beside the note the window
/// found the app's door by; else nowhere.
pub fn file(args: &[String], note: Option<&str>) -> Option<PathBuf> {
    let said = args.iter().position(|a| a == "--picks").and_then(|at| args.get(at + 1)).cloned().or_else(|| std::env::var("MERCURY_PICKS").ok());
    said.map(PathBuf::from).or_else(|| note.and_then(|note| std::path::Path::new(note).parent().map(|folder| folder.join(FILE))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("mercury-picks-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        folder.join(FILE)
    }

    #[test]
    fn a_pick_is_written_when_it_changes_and_read_by_the_next_window() {
        let file = scratch("kept");
        let mut kept = Kept::at(Some(file.clone()));
        assert_eq!(kept.get(), &Picks::default());
        // Nothing changed, nothing written: the folder is not even made.
        kept.change(|_| {});
        assert!(!file.exists());
        kept.change(|picks| {
            picks.project = Some("shop".to_string());
            picks.model.insert("claude".to_string(), "sonnet".to_string());
            picks.effort.insert("codex".to_string(), "high".to_string());
        });
        let next = Kept::at(Some(file.clone()));
        assert_eq!(next.get(), kept.get());
        assert_eq!(next.get().model.get("claude").map(String::as_str), Some("sonnet"));
        // The Inbox's display and the stopped accounts looked at go in the same file.
        let mut kept = next;
        kept.change(|picks| {
            picks.display = picks.display.toggled_project("shop");
            picks.cloud_broken_seen = vec!["codex:a-2".to_string()];
        });
        let next = Kept::at(Some(file.clone()));
        assert_eq!(next.get().display.projects, ["shop"]);
        assert_eq!(next.get().cloud_broken_seen, ["codex:a-2"]);
        // A file written before either was kept reads them as none.
        std::fs::write(&file, r#"{"project":"shop","model":{},"effort":{}}"#).unwrap();
        let older = Kept::at(Some(file.clone()));
        assert_eq!((older.get().project.as_deref(), older.get().display.is_filtered(), older.get().cloud_broken_seen.len()), (Some("shop"), false, 0));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn a_file_that_cannot_be_read_is_no_picks_and_a_window_with_no_file_keeps_them_to_itself() {
        let file = scratch("bad");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "not json").unwrap();
        assert_eq!(Kept::at(Some(file.clone())).get(), &Picks::default());
        let mut here = Kept::only_here(Picks { project: Some("shop".to_string()), ..Picks::default() });
        here.change(|picks| picks.project = Some("app".to_string()));
        assert_eq!(here.get().project.as_deref(), Some("app"));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }

    #[test]
    fn the_picks_sit_beside_the_doors_note_unless_the_command_line_says_where() {
        let none: [String; 0] = [];
        // No note and nothing said: a door given by hand, or a fixture. Nothing is kept on disk.
        if std::env::var("MERCURY_PICKS").is_err() {
            assert_eq!(file(&none, None), None);
            assert_eq!(file(&none, Some("/somewhere/Mercury/local-door.json")), Some(PathBuf::from("/somewhere/Mercury/native-picks.json")));
        }
        let said = ["--picks".to_string(), "/tmp/elsewhere.json".to_string()];
        assert_eq!(file(&said, Some("/somewhere/Mercury/local-door.json")), Some(PathBuf::from("/tmp/elsewhere.json")));
    }

    /// `tests/coming-back-to-a-thread-puts-you-where-you-were.test.mts`, across a restart: the
    /// place a conversation was left at is read back by the next window.
    #[test]
    fn where_a_conversation_was_left_is_read_by_the_next_window() {
        let file = scratch("places");
        let place = ThreadPlace { thread: "shop/w-1".into(), news: 7.0, top: Some(900.0), row: Some("said>m5".into()), into: Some(40.0), earlier: true, whole: true, runs: vec!["shop/w-1|run>m5".into()], lines: vec![("shop/w-1|line>w9:ran:npm test".into(), 2)] };
        Kept::at(Some(file.clone())).change(|p| p.thread_places.push(place.clone()));
        assert_eq!(Kept::at(Some(file.clone())).get().thread_places, vec![place]);
        // A place written before rows were kept has its offset alone, and still reads.
        std::fs::write(&file, r#"{"thread_places":[{"thread":"shop/w-1","news":7.0,"top":900.0,"earlier":true,"whole":false,"runs":[],"lines":[]}]}"#).unwrap();
        let older = Kept::at(Some(file.clone())).get().thread_places.clone();
        assert_eq!((older[0].top, older[0].row.as_deref(), older[0].into), (Some(900.0), None, None));
        let _ = std::fs::remove_dir_all(file.parent().unwrap());
    }
}
