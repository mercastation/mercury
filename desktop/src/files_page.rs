//! The Files place (`FilesPlace` in `renderer/src/files/FilesPlace.tsx`): its state, what it
//! writes, its keys and its strip, and the page of every file. It opens on its home
//! (`files_home.rs`): the project's folders as sections of tiles beside Recent, a folder's own
//! page, the files in no folder, the search's results, and the menus on a file, a folder and
//! New. The page of every file (`files-place.css`), behind "Every file, with sizes and the
//! Trash", is what this window drew before as the whole place: a tile for every file, the
//! search box, the kinds, Recent and Name, and grid or list.
//!
//! The list is the app's (`zero:files-place`), read when the page opens and again as the store
//! changes; a file's own lines come when its tile is pressed (`zero:files-place-open`). A change
//! made here (`zero:files-place-folder`: a folder made, renamed, deleted or put back, a file
//! moved, named or given an icon) is drawn over the list at once and stays drawn until a list
//! asked for after the app wrote it arrives (`actNow`, `withChanges`); New writes
//! `zero:files-place-new`, and Move to the Trash `zero:files-place-trash`, which Z takes back
//! with `zero:files-place-untrash`. Every one is a channel the web window calls. Upload from
//! computer opens the Mac's file window over this window and hands the app the path picked on
//! `zero:files-place-upload-here`, a channel of the app's local door alone that does what the web
//! window's `zero:files-place-upload` does once its own file window has picked.
//!
//! The page of every file has its tiles' and folder cards' ⋯ and menus (`MoveMenu`,
//! `FolderMenu`), the total and its menu (`TotalMenu`), and choosing files for the Trash
//! (`chosen`). Left out: dragging a tile onto a folder.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rune::file_dialog::FileDialogOptions;
use rune::{Cursor, El, Latest, SurfaceFit, TextEdit, TextStyle, Tx, Waker, col, el, icon, row, text, text_input};
use serde_json::{Value, json};

use crate::pictures::{Look, Want};
use crate::rules::files_home::{self as home, Act, Back, Making, Shown};
use crate::rules::files_shown::{self as rules, DocBlock, Folder, List, Placed, Place as Where, Sort};
use crate::source::{self, Source};
use crate::theme::*;
use crate::write::{self, Sent};
use crate::{State, View, chrome, feel, icons};

/// The keys of the search box and of the box a name is typed into.
pub const SEARCH: &str = "files-search";
pub const NAME_BOX: &str = "files-name-box";

/// What a menu of the home is open on (`TileMenu`, `FolderHead`'s); on the page of every file, a
/// tile's (`MoveMenu`), a folder card's (`FolderMenu`), the check pictures' line's (`MoveMenu`
/// with their keys) or the total's (`TotalMenu`).
#[derive(Clone, Debug, PartialEq)]
pub enum MenuOn {
    File(String),
    Folder(String),
    Scratch,
    Total,
}

/// Where a file's menu is: its first rows, the icons, the folders, or the box for a new one.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Step {
    #[default]
    Menu,
    Icon,
    Move,
    New,
}

/// An open menu: on what, at which step, and at the pointer when a right press opened it (else
/// under its ⋯).
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub on: MenuOn,
    pub step: Step,
    pub at: Option<(f32, f32)>,
}

/// A name being changed in place: a file's on its tile, a folder's in its heading.
#[derive(Clone, Debug, PartialEq)]
pub enum Renaming {
    File(String),
    Folder(String),
}

/// What the pointer is over, for the looks Rune's own hover cannot draw: a tile lit and a point
/// higher, a section's or a folder page's ⋯ shown.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hover {
    pub tile: Option<String>,
    pub sec: Option<String>,
    pub title: bool,
}

/// A change drawn over the list until the list has it (`pending`): `done` is when the app took
/// it, on the wall clock, `serial` which write it was.
#[derive(Clone, Debug, PartialEq)]
struct Pending {
    change: Shown,
    done: Option<f64>,
    serial: u64,
}

/// What a write of the Files place was, for what is done and said once the app answers.
#[derive(Clone, Debug, PartialEq)]
pub enum Wrote {
    /// A change to a folder or a file (`act`): what Z then puts back, and what follows it.
    Act { act: Act, serial: u64, undo: Option<Back>, then: Then },
    /// A document or a table made with New (`makeFile`).
    New { kind: Making },
    /// Files moved to the Trash, by the words that name them (`trash`).
    Trash { what: String },
    /// Z's own writes: a change put back, or files back from the Trash.
    Undone { acts: Vec<Act>, trash: bool },
    /// Files picked on this computer, added to a project (`upload`).
    Upload,
}

/// What follows a change once the app has it.
#[derive(Clone, Debug, PartialEq)]
pub enum Then {
    Nothing,
    /// A folder made from Move to…'s New folder…: the file moves into it (`moveToNew`).
    MoveInto(String),
    /// A folder deleted: said, and Z puts it back (`removeFolder`).
    Deleted { name: String, files: usize, project: String },
    /// A folder made from the check pictures' Move menu: they all move into it.
    MoveAllInto(Vec<String>),
    /// One of several files moved at once (`moveAll`): Z puts back those the app took, once
    /// every one has been answered.
    Group(u64),
}

/// Several files moved at once (`moveAll`): where they were, where they go, and what the app
/// has said of each so far.
#[derive(Clone, Debug, PartialEq)]
struct Group {
    id: u64,
    before: Vec<Placed>,
    folders: Vec<Folder>,
    to: Option<String>,
    keys: Vec<String>,
    answered: Vec<(String, bool)>,
}

/// The order the tiles and the project cards were last drawn in, held while the pointer moves
/// over them (`held`).
#[derive(Clone, Debug, Default)]
struct Held {
    tiles: Vec<String>,
    projects: Vec<String>,
}

/// The page's own state: the list as last read, and how the person narrowed and ordered it.
#[derive(Default)]
pub struct Place {
    list: Option<Rc<List>>,
    reading: Latest<Result<Value, String>>,
    /// When the list was last asked for, on the window's clock in milliseconds.
    asked_at: f64,
    /// The list is owed a reading: the page opened, or the store changed.
    stale: bool,
    /// The file a pressed tile opens, while the app is asked for its lines.
    opening: Latest<(String, Result<Value, String>)>,
    pub query: TextEdit,
    pub kind: Option<&'static str>,
    /// Only the files with comments open and nobody's approval (`review`).
    pub review: bool,
    pub sort: Sort,
    pub as_list: bool,
    /// The folder the person went into (`folderId`): a folder's id, or `not-in-a-folder`.
    pub folder: Option<String>,
    /// How many tiles are drawn, which grows as the foot comes near (`TILES_AT_ONCE`).
    drawn: usize,
    /// The check pictures agents took, folded under their line until it is pressed.
    scratch_open: bool,
    /// The page of every file, rather than the home (`everything`).
    pub everything: bool,
    /// Changes made here, drawn over the list until it has them.
    pending: Vec<Pending>,
    serial: u64,
    /// The menu of a file or a folder that is open.
    pub menu: Option<Menu>,
    /// New's menu, when open: its first rows, or the projects for what was chosen.
    pub new_menu: Option<Option<Making>>,
    /// A folder being named from New, in this project (`naming`).
    pub naming: Option<String>,
    pub renaming: Option<Renaming>,
    /// The box a name is typed into, and whether it had the keys at the last turn, so leaving
    /// it keeps the name as the web window's `onBlur` does.
    pub name_box: TextEdit,
    name_had_keys: bool,
    /// How many tiles a folder's page draws (`Shelf`'s `limit`).
    pub shelf: usize,
    pub hover: Hover,
    /// The files chosen for the Trash, in the order they were chosen, while choosing (`chosen`).
    pub chosen: Option<Vec<String>>,
    /// What Files keeps in the cloud (`zero:files-cloud`), read once the page has opened.
    cloud: Option<Value>,
    cloud_reading: Latest<Result<Value, String>>,
    cloud_asked: bool,
    /// The box a Move menu finds a folder with (`.fp-menu-find`).
    pub menu_find: TextEdit,
    /// The pointer moved over the tiles lately: their order holds (`HOLD_STILL_MS`), until when.
    pub holding: bool,
    hold_until: f64,
    held: RefCell<Held>,
    /// Several files moved at once, waiting on the app's answers.
    groups: Vec<Group>,
    /// What Z's writes drew over the list, until the app answers them.
    undo_serials: Vec<u64>,
}

/// How long the tiles keep their order after the pointer last moved over them (`HOLD_STILL_MS`).
const HOLD_STILL_MS: f64 = 1500.0;

impl Place {
    pub fn set_waker(&self, wake: Waker) {
        self.reading.set_waker(wake.clone());
        self.cloud_reading.set_waker(wake.clone());
        self.opening.set_waker(wake);
    }
    /// The projects the place has listed, once it has been read: the command bar's Files rows.
    pub fn projects(&self) -> Vec<String> {
        self.list.as_ref().map(|l| l.projects.clone()).unwrap_or_default()
    }
    /// Whether a box of the home is taking a name.
    fn naming_something(&self) -> bool {
        self.naming.is_some() || self.renaming.is_some() || self.menu.as_ref().is_some_and(|m| m.step == Step::New)
    }
}

/// The list the page draws: the fixture's for a picture, else the app's as last read, with the
/// changes made here drawn over it.
pub fn list_of(s: &State) -> Option<Rc<List>> {
    let list = match &s.files.list {
        Some(l) => l.clone(),
        None => Rc::new(serde_json::from_value::<List>(s.fixture.get("filesPlace")?.clone()).ok()?),
    };
    if s.files.pending.is_empty() {
        return Some(list);
    }
    let changes: Vec<Shown> = s.files.pending.iter().map(|p| p.change.clone()).collect();
    Some(Rc::new(home::with_changes(&list, &changes)))
}

/// The files the search leaves while its box has words (`found`): every file under the page,
/// in the list's order, check pictures and all.
pub fn found<'a>(s: &State, list: &'a List) -> Vec<&'a Placed> {
    let place = Where { project: project_of(s), folder: real_folder(s, list) };
    let below = rules::files_below(&list.files, &list.folders, &place);
    rules::files_shown(&below, &s.files.query.text, s.files.kind, &list.folders).into_iter().filter(|f| !s.files.review || rules::needs_review(f)).collect()
}

fn project_of(s: &State) -> Option<String> {
    match &s.view {
        View::Files { project } => project.clone(),
        _ => None,
    }
}

/// The folder the page is in when it names a folder of the list (`folder`).
fn real_folder(s: &State, list: &List) -> Option<String> {
    s.files.folder.clone().filter(|id| list.folders.iter().any(|f| &f.id == id))
}

/// The window's turn: the list is asked for when the page is open and owes a reading (at most
/// every two seconds, as a burst of the store's changes reads once), what was read lands, and
/// a pressed tile opens once its lines have come; a name box the keys left keeps its name.
/// Says whether to draw again.
pub fn tend(s: &mut State, now: f64, stale: bool, tx: &mut Tx) -> bool {
    let on_files = matches!(s.view, View::Files { .. });
    // The command bar lists each project's page too, so it reads the list as well.
    let wanted = on_files || s.palette.open;
    s.files.stale |= stale && wanted;
    if wanted
        && (s.files.list.is_none() || s.files.stale)
        && !s.files.reading.is_pending()
        && now - s.files.asked_at >= 2000.0
        && let Source::Door(door) = &s.source
        && let Some((url, token)) = door.now()
    {
        s.files.asked_at = now;
        s.files.stale = false;
        // The first reading of a window asks for the list the app made last, at once (`soon`).
        let ask = if s.files.list.is_none() { json!({ "soon": true }) } else { json!({}) };
        s.files.reading.request((url, token), move |(url, token)| source::call(&url, &token, "zero:files-place", &ask));
    }
    // What the cloud keeps, for the total's Keep files in the cloud: read once the page of every
    // file is open (`api.filesCloud()` as `FilesPlace` mounts).
    if on_files
        && every_page(s)
        && !s.files.cloud_asked
        && let Source::Door(door) = &s.source
        && let Some((url, token)) = door.now()
    {
        s.files.cloud_asked = true;
        s.files.cloud_reading.request((url, token), |(url, token)| source::call(&url, &token, "zero:files-cloud", &Value::Null));
    }
    let mut changed = false;
    if let Some(Ok(answer)) = s.files.cloud_reading.take() {
        s.files.cloud = Some(answer);
        changed = true;
    }
    // The pointer has been still a moment: the tiles take their own order again.
    if s.files.holding && now >= s.files.hold_until {
        s.files.holding = false;
        changed = true;
    }
    if let Some(Ok(answer)) = s.files.reading.take()
        && let Ok(list) = serde_json::from_value::<List>(answer)
    {
        // A change stays drawn until a list asked for after the app wrote it.
        let asked = s.files.asked_at;
        s.files.pending.retain(|p| p.done.is_none_or(|done| done > asked));
        s.files.list = Some(Rc::new(list));
        changed = true;
    }
    if let Some((key, Ok(view))) = s.files.opening.take()
        && on_files
    {
        s.open_placed(&key, &view);
        changed = true;
    }
    // A kind no file here is any more lets go, so the page is not left empty under it
    // (`if (kind && !kinds.includes(kind)) setKind(null)`).
    if on_files
        && let Some(k) = s.files.kind
        && let Some(list) = list_of(s)
    {
        let place = Where { project: project_of(s), folder: real_folder(s, &list) };
        if !rules::kinds_in(&rules::files_below(&list.files, &list.folders, &place)).contains(&k) {
            s.files.kind = None;
            changed = true;
        }
    }
    // Leaving a box that takes a name keeps the name (`onBlur`).
    let has_keys = s.files.name_box.focused;
    if s.files.name_had_keys && !has_keys && s.files.naming_something() {
        s.name_done(tx);
        changed = true;
    }
    s.files.name_had_keys = has_keys && s.files.naming_something();
    changed
}

impl State {
    /// The rail's Files: All files, or a project's page. A project's page opens on its home, out
    /// of any folder, with the box for a new folder kept only when it is that project's.
    pub fn open_files_place(&mut self, project: Option<String>) {
        self.files.folder = None;
        self.files.everything = false;
        self.files.shelf = home::SHELF_STEP;
        if self.files.naming.as_ref() != project.as_ref() {
            self.files.naming = None;
        }
        self.close_files_menus();
        self.files.renaming = None;
        self.files.drawn = rules::TILES_AT_ONCE;
        self.files.stale = true;
        if matches!(self.view, View::Files { .. }) {
            // Another place of Files is the same page: no step back to the last. Files being
            // chosen for the Trash stay chosen, as the web window's page stays the same page.
            self.view = View::Files { project };
        } else {
            // Come to from elsewhere, the page is new: nothing is chosen (`FilesPlace` mounts).
            self.files.chosen = None;
            self.files.cloud_asked = false;
            self.go(View::Files { project });
        }
    }
    /// The project's folders, for Move to… (`folders` handed to the home): none on All files.
    pub fn files_folders<'a>(&self, list: &'a List) -> Vec<&'a Folder> {
        match project_of(self) {
            Some(p) => list.folders.iter().filter(|f| f.project.to_lowercase() == p.to_lowercase()).collect(),
            None => Vec::new(),
        }
    }
    /// A section's name, See all, or a folder's crumb: that folder's page, or the files in none.
    pub fn go_folder(&mut self, folder: Option<String>) {
        self.close_files_menus();
        self.files.folder = folder;
        self.files.shelf = home::SHELF_STEP;
    }
    /// A crumb of the home's path back: All files, a project's home, or a folder's page.
    pub fn go_crumb(&mut self, project: Option<String>, folder: Option<String>) {
        if project != project_of(self) {
            self.open_files_place(project);
        }
        self.go_folder(folder);
    }
    /// "Every file, with sizes and the Trash": the page of every file.
    pub fn files_everything(&mut self) {
        self.close_files_menus();
        self.files.everything = true;
        self.files.drawn = rules::TILES_AT_ONCE;
    }
    /// Closes whatever menu of the home is open.
    pub fn close_files_menus(&mut self) {
        if self.files.menu.as_ref().is_some_and(|m| m.step == Step::New) {
            self.files.name_box = TextEdit::new("");
        }
        self.files.menu = None;
        self.files.new_menu = None;
        // A Move menu closes with its search emptied (`setQuery('')`).
        self.files.menu_find = TextEdit::new("");
    }
    /// The check pictures' ⋯ (`MoveMenu` with their keys), or the total (`TotalMenu`): its menu,
    /// or closed when it was open.
    pub fn toggle_every_menu(&mut self, on: MenuOn) {
        let was = self.files.menu.as_ref().is_some_and(|m| m.on == on);
        self.close_files_menus();
        if !was {
            self.files.menu = Some(Menu { on, step: Step::Menu, at: None });
            // The box that finds a folder takes the keys as it is drawn (`autoFocus`).
            self.files.menu_find.focused = true;
        }
    }
    /// A tile's or a folder card's ⋯ on the page of every file: as the home's, and the box that
    /// finds a folder takes the keys.
    pub fn toggle_every_file_menu(&mut self, on: MenuOn) {
        match on {
            MenuOn::File(key) => self.toggle_file_menu(&key, None),
            MenuOn::Folder(id) => self.toggle_folder_menu(&id),
            other => return self.toggle_every_menu(other),
        }
        if self.files.menu.is_some() {
            self.files.menu_find.focused = true;
        }
    }
    /// The total's "Choose files to move to the Trash": every tile Files may move gets a ring,
    /// and a press chooses it (`setChosen(new Set())`).
    pub fn start_choosing(&mut self) {
        self.close_files_menus();
        self.files.chosen = Some(Vec::new());
    }
    /// A tile pressed while choosing: chosen, or chosen no more. One Files may not move is
    /// passed over (`f.trashable ? choose(f) : undefined`).
    pub fn choose_file(&mut self, key: &str) {
        let Some(chosen) = self.files.chosen.as_mut() else { return };
        match chosen.iter().position(|k| k == key) {
            Some(at) => {
                chosen.remove(at);
            }
            None => chosen.push(key.to_string()),
        }
    }
    /// Cancel, or Escape while choosing.
    pub fn stop_choosing(&mut self) {
        self.files.chosen = None;
    }
    /// Move to the Trash while choosing: the chosen files, named by the one's name or how many.
    pub fn trash_chosen(&mut self, tx: &mut Tx) {
        let Some(keys) = self.files.chosen.take() else { return };
        if keys.is_empty() {
            self.files.chosen = Some(keys);
            return;
        }
        let what = if keys.len() == 1 { self.file(&keys[0]).map(|f| rules::shown_name(&f)).unwrap_or_default() } else { format!("{} files", keys.len()) };
        self.trash(keys, what, false, tx);
    }
    /// The total's Move older versions to the Trash: every file here with an older version.
    pub fn trash_older(&mut self, tx: &mut Tx) {
        self.close_files_menus();
        let Some(list) = list_of(self) else { return };
        let place = Where { project: project_of(self), folder: real_folder(self, &list) };
        let keys: Vec<String> = rules::files_below(&list.files, &list.folders, &place).into_iter().filter(|f| f.older.unwrap_or(0.0) > 0.0).map(|f| f.key.clone()).collect();
        self.trash(keys, "the older versions".to_string(), true, tx);
    }
    /// The total's Keep files in the cloud: Settings › Files on a paid plan, else the plans.
    pub fn files_cloud_settings(&mut self, tx: &mut Tx) {
        self.close_files_menus();
        let paid = self.files.cloud.as_ref().and_then(|c| c.get("cap")).is_some_and(|cap| !cap.is_null() && cap.as_f64() != Some(0.0));
        crate::settings::open(self, if paid { crate::rules::settings::PageId::Files } else { crate::rules::settings::PageId::Cloud }, tx);
    }
    /// A file's ⋯, or a right press on its tile: its menu, or closed when it was open there.
    pub fn toggle_file_menu(&mut self, key: &str, at: Option<(f32, f32)>) {
        let on = MenuOn::File(key.to_string());
        let was = self.files.menu.as_ref().is_some_and(|m| m.on == on && at.is_none());
        self.close_files_menus();
        if !was {
            self.files.menu = Some(Menu { on, step: Step::Menu, at });
        }
    }
    /// A folder's ⋯: its menu, or closed.
    pub fn toggle_folder_menu(&mut self, id: &str) {
        let on = MenuOn::Folder(id.to_string());
        let was = self.files.menu.as_ref().is_some_and(|m| m.on == on);
        self.close_files_menus();
        if !was {
            self.files.menu = Some(Menu { on, step: Step::Menu, at: None });
        }
    }
    /// New: its menu, or closed.
    pub fn toggle_new_menu(&mut self) {
        let was = self.files.new_menu.is_some();
        self.close_files_menus();
        if !was {
            self.files.new_menu = Some(None);
        }
    }
    /// Change icon… or Move to…: the same menu at its next step.
    pub fn menu_step(&mut self, step: Step) {
        if let Some(m) = self.files.menu.as_mut() {
            m.step = step;
        }
    }
    /// Move to…'s New folder…: the box for its name at the menu's foot, the keys in it.
    pub fn move_menu_new_folder(&mut self, tx: &mut Tx) {
        self.menu_step(Step::New);
        self.files.name_box = TextEdit::new("");
        self.files.name_box.focused = true;
        self.files.name_had_keys = false;
        tx.focus_when_shown(NAME_BOX, false);
    }
    /// The file whose menu is open, from the list as drawn.
    fn file(&self, key: &str) -> Option<Placed> {
        list_of(self)?.files.iter().find(|f| f.key == key).cloned()
    }
    /// Rename on a file's menu: its name a box over its own line, all of it chosen.
    pub fn rename_file_start(&mut self, key: &str, tx: &mut Tx) {
        self.close_files_menus();
        let Some(f) = self.file(key) else { return };
        self.start_naming(&self.name_start(&f), tx);
        self.files.renaming = Some(Renaming::File(key.to_string()));
    }
    /// The name a file's box starts from: its tile's name, which on the home is its document's
    /// title when it has one, and on the page of every file the name the page shows.
    fn name_start(&self, f: &Placed) -> String {
        if every_page(self) { rules::shown_name(f) } else { crate::files_home::name_of(f) }
    }
    /// Rename on a folder's menu: its name a box where it stood.
    pub fn rename_folder_start(&mut self, id: &str, tx: &mut Tx) {
        self.close_files_menus();
        let Some(name) = list_of(self).and_then(|l| l.folders.iter().find(|f| f.id == id).map(|f| f.name.clone())) else { return };
        self.start_naming(&name, tx);
        self.files.renaming = Some(Renaming::Folder(id.to_string()));
    }
    fn start_naming(&mut self, start: &str, tx: &mut Tx) {
        self.files.name_box = TextEdit::new(start);
        self.files.name_box.set_selection(0, start.len());
        self.files.name_box.focused = true;
        self.files.name_had_keys = false;
        tx.focus_when_shown(NAME_BOX, false);
    }
    /// Return in a name box, or the keys leaving it: the name is kept when it is new and not
    /// blank, else nothing changes (`NameInput`'s `done`, `NameBox`'s).
    pub fn name_done(&mut self, tx: &mut Tx) {
        let typed = self.files.name_box.text.trim().to_string();
        self.files.name_had_keys = false;
        if let Some(project) = self.files.naming.take() {
            self.files.name_box = TextEdit::new("");
            if !typed.is_empty() {
                let parent = list_of(self).and_then(|l| real_folder(self, &l));
                self.act(Act::Make { project, name: typed, parent }, None, Then::Nothing, tx);
            }
            return;
        }
        if let Some(m) = self.files.menu.clone().filter(|m| m.step == Step::New) {
            self.close_files_menus();
            if typed.is_empty() {
                return;
            }
            match &m.on {
                MenuOn::File(key) => {
                    let project = self.file(key).and_then(|f| f.project).or_else(|| project_of(self)).unwrap_or_default();
                    self.act(Act::Make { project, name: typed, parent: None }, None, Then::MoveInto(key.clone()), tx);
                }
                // The check pictures' menu makes the folder in the first one's project and moves
                // them all into it.
                MenuOn::Scratch => {
                    let scratch = scratch_here(self);
                    let project = scratch.first().and_then(|f| f.project.clone()).unwrap_or_default();
                    let keys = scratch.iter().map(|f| f.key.clone()).collect();
                    self.act(Act::Make { project, name: typed, parent: None }, None, Then::MoveAllInto(keys), tx);
                }
                _ => {}
            }
            return;
        }
        match self.files.renaming.take() {
            Some(Renaming::File(key)) => {
                let Some(f) = self.file(&key) else { return };
                let start = self.name_start(&f);
                if !typed.is_empty() && typed != start {
                    let undo = home::file_rename_undo(&f, &typed);
                    self.act(Act::Name { key, name: typed }, undo, Then::Nothing, tx);
                }
            }
            Some(Renaming::Folder(id)) => {
                let Some(list) = list_of(self) else { return };
                let Some(start) = list.folders.iter().find(|f| f.id == id).map(|f| f.name.clone()) else { return };
                if !typed.is_empty() && typed != start {
                    let undo = home::rename_undo(&list.folders, &id, &typed);
                    self.act(Act::Rename { id, name: typed }, undo, Then::Nothing, tx);
                }
            }
            None => {}
        }
    }
    /// Escape in a name box: nothing changes; Move to…'s box goes back to the folders.
    pub fn name_given_up(&mut self, _tx: &mut Tx) {
        self.files.name_had_keys = false;
        let back = if every_page(self) { Step::Menu } else { Step::Move };
        if let Some(m) = self.files.menu.as_mut().filter(|m| m.step == Step::New) {
            // The home's goes back to the folders; the page of every file's to its whole menu
            // (`setNaming(false)`).
            m.step = back;
            self.files.name_box = TextEdit::new("");
            return;
        }
        self.files.naming = None;
        self.files.renaming = None;
        self.files.name_box = TextEdit::new("");
    }
    /// A mark picked in Change icon…, or Automatic (`''`).
    pub fn pick_icon(&mut self, key: &str, icon: &str, tx: &mut Tx) {
        self.close_files_menus();
        self.act(Act::Icon { key: key.to_string(), icon: icon.to_string() }, None, Then::Nothing, tx);
    }
    /// A folder picked in Move to…, or Not in a folder (`move`): Z puts it back where it was.
    pub fn move_file(&mut self, key: &str, to: Option<String>, tx: &mut Tx) {
        self.close_files_menus();
        let undo = list_of(self).and_then(|l| home::move_undo(&l.files, &[key.to_string()], to.as_deref(), &l.folders));
        self.act(Act::Move { key: key.to_string(), folder: to }, undo, Then::Nothing, tx);
    }
    /// Delete folder: its files go to no folder, it is said, and Z puts it back. Deleting the
    /// folder whose page is open goes back to the home.
    pub fn remove_folder(&mut self, id: &str, tx: &mut Tx) {
        self.close_files_menus();
        let Some(list) = list_of(self) else { return };
        let Some(f) = list.folders.iter().find(|f| f.id == id) else { return };
        let files = rules::files_below(&list.files, &list.folders, &Where { project: project_of(self), folder: Some(id.to_string()) }).len();
        let then = Then::Deleted { name: f.name.clone(), files, project: f.project.clone() };
        if self.files.folder.as_deref() == Some(id) {
            self.files.folder = None;
        }
        self.act(Act::Remove { id: id.to_string() }, None, then, tx);
    }
    /// Several files moved at once (`moveAll`), each its own write; Z puts back those the app
    /// took once it has answered every one. One file is `move_file`.
    pub fn move_all(&mut self, keys: Vec<String>, to: Option<String>, tx: &mut Tx) {
        self.close_files_menus();
        if keys.len() == 1 {
            return self.move_file(&keys[0], to, tx);
        }
        let Some(list) = list_of(self) else { return };
        self.files.serial += 1;
        let id = self.files.serial;
        self.files.groups.push(Group { id, before: list.files.clone(), folders: list.folders.clone(), to: to.clone(), keys: keys.clone(), answered: Vec::new() });
        for key in keys {
            self.act(Act::Move { key, folder: to.clone() }, None, Then::Group(id), tx);
        }
    }
    /// Move to the Trash on a file's menu (`trash`).
    pub fn trash_file(&mut self, key: &str, tx: &mut Tx) {
        self.close_files_menus();
        let Some(f) = self.file(key) else { return };
        self.trash(vec![key.to_string()], rules::shown_name(&f), false, tx);
    }
    /// Files to the Trash, or only their older versions (`trash`): said while it goes, and what
    /// emptying the Trash gives back once the app has done it.
    pub fn trash(&mut self, keys: Vec<String>, what: String, older: bool, tx: &mut Tx) {
        self.tell(&format!("Moving {what} to the Trash…"), tx);
        if !self.send(home::TRASH, json!({ "keys": keys, "older": older }), Sent::Files(Wrote::Trash { what })) {
            self.tell("Those files could not be moved to the Trash.", tx);
        }
    }
    /// Upload from computer (`upload`): the Mac's own file window over this window, and what is
    /// picked added to the project, in the folder on screen. The web window asks the app for that
    /// window (`zero:files-place-upload`), which would open it over the app's own window; so this
    /// one opens it itself and hands the app the paths on the door's own channel.
    pub fn upload_from_computer(&mut self, project: &str, folder: Option<String>, tx: &mut Tx) {
        let options = FileDialogOptions { title: format!("Add files to {project}"), ..FileDialogOptions::default() };
        let project = project.to_string();
        tx.open_file(options, move |s: &mut State, picked, tx| match picked {
            Ok(Some(path)) => s.upload_paths(&project, folder.clone(), vec![path.to_string_lossy().into_owned()], tx),
            // Nothing picked: nothing is added and nothing is said (`r.canceled`).
            Ok(None) => {}
            Err(e) => s.tell(&format!("The file picker did not open: {e}"), tx),
        });
    }
    /// The paths picked, handed to the app to add (`zero:files-place-upload-here`).
    pub fn upload_paths(&mut self, project: &str, folder: Option<String>, paths: Vec<String>, tx: &mut Tx) {
        if !self.send(home::UPLOAD_HERE, json!({ "project": project, "folder": folder, "paths": paths }), Sent::Files(Wrote::Upload)) {
            self.tell(home::UPLOAD_REFUSED, tx);
        }
    }
    /// A row of New's menu: made at once in the project on screen, or in the one project of
    /// the person's own; else the menu asks which (`choose`).
    pub fn choose_new(&mut self, what: Making, tx: &mut Tx) {
        let mine = list_of(self).map(|l| l.mine.clone()).unwrap_or_default();
        match home::new_goes_in(project_of(self).as_deref(), &mine) {
            Some(p) => self.make_new(what, &p, tx),
            None => self.files.new_menu = Some(Some(what)),
        }
    }
    /// What New makes, in `project` (`pick`): a project picked from All files opens its home; a
    /// folder's name box stands on it; a document or a table is made and opens big.
    pub fn make_new(&mut self, what: Making, project: &str, tx: &mut Tx) {
        self.close_files_menus();
        let here = project_of(self).as_deref() == Some(project);
        let folder = if here { list_of(self).and_then(|l| real_folder(self, &l)) } else { None };
        if !here {
            self.open_files_place(Some(project.to_string()));
        }
        match what {
            Making::Folder => {
                self.files.naming = Some(project.to_string());
                self.files.name_box = TextEdit::new("");
                self.files.name_box.focused = true;
                self.files.name_had_keys = false;
                tx.focus_when_shown(NAME_BOX, false);
            }
            Making::Document | Making::Table => {
                let kind = if what == Making::Table { "table" } else { "document" };
                if !self.send(home::NEW_FILE, json!({ "project": project, "name": "", "kind": kind, "folder": folder }), Sent::Files(Wrote::New { kind: what })) {
                    self.tell(&format!("That {kind} could not be made."), tx);
                }
            }
            Making::Upload => self.upload_from_computer(project, folder, tx),
        }
    }
    /// A change written, and drawn over the list from now on unless the app refuses it
    /// (`actNow`). A folder made is drawn once the app gives its id; one put back takes back a
    /// deletion still only drawn here.
    fn act(&mut self, act: Act, undo: Option<Back>, then: Then, tx: &mut Tx) {
        self.files.serial += 1;
        let serial = self.files.serial;
        if let Act::Restore { ids } = &act {
            self.files.pending.retain(|p| !matches!(&p.change, Shown::Act(Act::Remove { id }) if ids.contains(id)));
        }
        if !matches!(act, Act::Make { .. } | Act::Restore { .. }) {
            self.files.pending.push(Pending { change: Shown::Act(act.clone()), done: None, serial });
        }
        let payload = act.payload();
        if !self.send(home::FOLDER_ACT, payload, Sent::Files(Wrote::Act { act, serial, undo, then })) {
            self.files.pending.retain(|p| p.serial != serial);
            self.tell("That could not be done.", tx);
        }
    }
}

/// The app's own words for a refusal, else the page's (`r.error || …`).
fn refused_words(answer: &Result<Value, String>, otherwise: &str) -> String {
    match answer {
        Ok(v) => v.get("error").and_then(Value::as_str).filter(|e| !e.is_empty()).unwrap_or(otherwise).to_string(),
        Err(_) => otherwise.to_string(),
    }
}

/// What the app answered a write of the Files place: the change kept drawn or taken back off,
/// what follows it, what is said, and what Z can then put back.
pub fn landed(s: &mut State, landed: write::Landed, tx: &mut Tx) {
    let write::Sent::Files(wrote) = landed.sent.clone() else { return };
    let refused = landed.refusal().is_some();
    let answer = landed.answer.as_ref().ok().cloned().unwrap_or(Value::Null);
    // What was written changes the list: read it again now, not after two seconds.
    s.files.stale = true;
    s.files.asked_at = 0.0;
    let now = write::wall_clock();
    match wrote {
        Wrote::Act { act, serial, undo, then } => {
            if let (Then::Group(id), Act::Move { key, .. }) = (&then, &act) {
                group_answered(s, *id, key, !refused);
            }
            if refused {
                s.files.pending.retain(|p| p.serial != serial);
                let words = refused_words(&landed.answer, "That could not be done.");
                s.tell(&words, tx);
                return;
            }
            for p in s.files.pending.iter_mut().filter(|p| p.serial == serial) {
                p.done = Some(now);
            }
            if let (Act::Make { project, name, parent }, Some(id)) = (&act, answer.get("folder").and_then(Value::as_str)) {
                let folder = Folder { id: id.to_string(), project: project.clone(), name: home::clean_name(name), parent: parent.clone(), at: now };
                s.files.serial += 1;
                let serial = s.files.serial;
                s.files.pending.push(Pending { change: Shown::Made(folder), done: Some(now), serial });
                if let Then::MoveInto(key) = &then {
                    s.move_file(key, Some(id.to_string()), tx);
                }
                if let Then::MoveAllInto(keys) = &then {
                    s.move_all(keys.clone(), Some(id.to_string()), tx);
                }
            }
            if let Some(back) = undo {
                s.undos.push(write::Undo::plain(back.label, back.undoes, back.back.iter().map(|a| (home::FOLDER_ACT, a.payload())).collect(), Sent::Files(Wrote::Undone { acts: back.back, trash: false }), (String::new(), String::new()), false));
            }
            if let Then::Deleted { name, files, project } = then {
                let ids: Vec<String> = answer.get("ids").and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
                if ids.is_empty() {
                    return;
                }
                let back = vec![Act::Restore { ids }];
                s.undos.push(write::Undo::plain(format!("Folder back: {name}"), format!("put {name} back"), back.iter().map(|a| (home::FOLDER_ACT, a.payload())).collect(), Sent::Files(Wrote::Undone { acts: back, trash: false }), (String::new(), String::new()), false));
                s.say(home::deleted_words(&name, files, &project), true, tx);
            }
        }
        Wrote::New { kind } => {
            let word = if kind == Making::Table { "table" } else { "document" };
            match answer.get("key").and_then(Value::as_str).filter(|_| !refused) {
                Some(key) => s.open_tile(key),
                None => {
                    let words = refused_words(&landed.answer, &format!("That {word} could not be made."));
                    s.tell(&words, tx);
                }
            }
        }
        Wrote::Trash { what } => {
            if refused {
                let words = refused_words(&landed.answer, "Those files could not be moved to the Trash.");
                return s.tell(&words, tx);
            }
            if answer.get("moved").and_then(Value::as_f64).unwrap_or(0.0) == 0.0 {
                return s.tell("Nothing here could be moved to the Trash.", tx);
            }
            let bytes = answer.get("bytes").and_then(Value::as_f64).unwrap_or(0.0);
            let undo = answer.get("undo").filter(|u| !u.is_null()).cloned();
            if let Some(undo) = &undo {
                s.undos.push(write::Undo::plain(format!("Back from the Trash: {what}"), format!("put {what} back"), vec![(home::UNTRASH, undo.clone())], Sent::Files(Wrote::Undone { acts: Vec::new(), trash: true }), (String::new(), String::new()), false));
            }
            let freed = if bytes > 0.0 { format!(" Empty the Trash to get the {} back.", rules::size_words(bytes)) } else { String::new() };
            s.say(format!("Moved {what} to the Trash.{freed}"), undo.is_some(), tx);
        }
        Wrote::Undone { acts: _, trash } => {
            // Z's changes were drawn as the key was pressed (`undoing`): kept until the list has
            // them, or taken back off.
            let drawn = std::mem::take(&mut s.files.undo_serials);
            if refused {
                s.files.pending.retain(|p| !drawn.contains(&p.serial));
            } else {
                for p in s.files.pending.iter_mut().filter(|p| drawn.contains(&p.serial)) {
                    p.done = Some(now);
                }
            }
            let Some(done) = s.answering.undoing.take() else { return };
            if refused {
                let otherwise = if trash { "Those files are in the Trash now. Put them back from there." } else { "That could not be put back." };
                let words = refused_words(&landed.answer, otherwise);
                return s.tell(&words, tx);
            }
            s.say(done.said, false, tx);
        }
        Wrote::Upload => {
            // An app started before it had the door's channel answers 404: it is restarted first.
            if let Some(why) = landed.refusal() {
                let words = if matches!(&landed.answer, Err(e) if e.contains("404")) { crate::trouble::refused(&why) } else { refused_words(&landed.answer, home::UPLOAD_REFUSED) };
                return s.tell(&words, tx);
            }
            let left: Vec<String> = answer.get("refused").and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()).unwrap_or_default();
            if let Some(words) = home::upload_left_words(&left) {
                s.tell(&words, tx);
            }
        }
    }
}

/// Z's writes on the Files place have gone: what they put back is drawn over the list now, as the
/// web window's `actNow` draws it at the key, a folder put back taking back a deletion still only
/// drawn here.
pub fn undoing(s: &mut State, sent: &Sent) {
    let Sent::Files(Wrote::Undone { acts, trash: false }) = sent else { return };
    for act in acts {
        if let Act::Restore { ids } = act {
            s.files.pending.retain(|p| !matches!(&p.change, Shown::Act(Act::Remove { id }) if ids.contains(id)));
        }
        if matches!(act, Act::Make { .. } | Act::Restore { .. }) {
            continue;
        }
        s.files.serial += 1;
        let serial = s.files.serial;
        s.files.pending.push(Pending { change: Shown::Act(act.clone()), done: None, serial });
        s.files.undo_serials.push(serial);
    }
}

/// One move of several answered: once all are, Z puts back the ones the app took (`moveAll`'s
/// `moveUndo` over the moves that were `ok`).
fn group_answered(s: &mut State, id: u64, key: &str, ok: bool) {
    let Some(at) = s.files.groups.iter().position(|g| g.id == id) else { return };
    s.files.groups[at].answered.push((key.to_string(), ok));
    if s.files.groups[at].answered.len() < s.files.groups[at].keys.len() {
        return;
    }
    let g = s.files.groups.remove(at);
    let took: Vec<String> = g.keys.iter().filter(|k| g.answered.iter().any(|(a, ok)| a == *k && *ok)).cloned().collect();
    if let Some(back) = home::move_undo(&g.before, &took, g.to.as_deref(), &g.folders) {
        s.undos.push(write::Undo::plain(back.label, back.undoes, back.back.iter().map(|a| (home::FOLDER_ACT, a.payload())).collect(), Sent::Files(Wrote::Undone { acts: back.back, trash: false }), (String::new(), String::new()), false));
    }
}

impl State {
    /// A tile pressed: the app is asked for the file's lines, and the file opens big when they
    /// come. A picture's page has them in its fixture.
    pub fn open_tile(&mut self, key: &str) {
        if let Some(view) = self.fixture.get("filesPlaceViews").and_then(|v| v.get(key)).cloned() {
            return self.open_placed(key, &view);
        }
        let Source::Door(door) = &self.source else { return };
        let Some((url, token)) = door.now() else { return };
        let key = key.to_string();
        self.files.opening.request((url, token, key), |(url, token, key)| {
            let view = source::call(&url, &token, "zero:files-place-open", &serde_json::json!({ "key": key }));
            (key, view)
        });
    }
}

/* ------------------------------ what a tile's face reads ------------------------------ */

/// A document's or a script's face, read once from the top of its file.
enum Face {
    Doc(Vec<DocBlock>),
    Code(String),
}

thread_local! {
    /// Faces read so far, by the file and what says it changed: its versions and its size.
    static FACES: RefCell<HashMap<(String, u32, u64), Rc<Face>>> = RefCell::new(HashMap::new());
}

/// The face of a document or a script: the top of the file, read off the disk once.
fn face_of(f: &Placed) -> Option<Rc<Face>> {
    let path = f.opened.clone()?;
    let key = (path.clone(), f.versions, f.bytes.unwrap_or(0.0) as u64);
    if let Some(face) = FACES.with(|faces| faces.borrow().get(&key).cloned()) {
        return Some(face);
    }
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(&path).ok()?.take(32 << 10).read_to_end(&mut bytes).ok()?;
    let words = String::from_utf8_lossy(&bytes);
    let face = Rc::new(match f.kind.as_deref() {
        Some("markdown") => Face::Doc(rules::doc_blocks(&words, 8)),
        _ => Face::Code(words.split('\n').take(12).collect::<Vec<_>>().join("\n")),
    });
    FACES.with(|faces| faces.borrow_mut().insert(key, face.clone()));
    Some(face)
}

/* ------------------------------ the tiles ------------------------------ */

/// A picture's face: its top filling the tile, or the whole of it in the middle when filling
/// would cut away too much of it to read (`coverFit`), read at the tile's size.
fn picture_face(s: &State, path: &str, w: f32, h: f32) -> El {
    let places = vec![path.to_string()];
    let want = Want::Cover(s.pictures.pixels(w), s.pictures.pixels(h));
    let drawn = match s.pictures.look(places.clone(), want) {
        Look::Here(held) => {
            let natural = (held.natural.0 as f32, held.natural.1 as f32);
            if rules::cover_fills(natural, (w, h)) {
                el().size(w, h).pixels_fit(&held.pixels, SurfaceFit::Fill).surface_anchor(rune::Align::Center, rune::Align::Start)
            } else {
                el().size(w, h).pixels_fit(&held.pixels, SurfaceFit::Fit)
            }
        }
        _ => el().size(w, h),
    };
    drawn.on_visible(move |s: &mut State, on, _| s.pictures.near(&places, want, on))
}

/// A document's face (`.fp-doc`): its title, headings, paragraphs and items on the page's paper,
/// fading out over its last quarter.
fn doc_face(blocks: &[DocBlock], w: f32, h: f32) -> El {
    let inner = w - 40.0;
    let words = |s: &str, size: f32, weight: f32, line: f32, c| text(s).font(TextStyle::new(size, weight).lh(line)).color(c).wrap().max_w(inner);
    let mut out = col().size(w, h).pt(20.0).px(20.0).gap(9.0).fill(surface()).clip_fade_linear(0.0, 0.0, 0.0, 0.28 * h);
    for b in blocks {
        out = out.child(match b {
            DocBlock::Title(t) => text(t.as_str()).font(TextStyle::new(15.0, 600.0).lh(19.5).tracking(-0.075)).color(ink()).wrap().max_w(inner).mb(2.0),
            DocBlock::Head(t) => words(t, 12.0, 600.0, kept(16.8), ink()).mt(4.0),
            DocBlock::Words(t) => words(t, 11.5, 400.0, kept(17.825), dim()),
            DocBlock::Item(t) => el().mt(-4.0).pl(14.0).child(el().absolute(3.0, 0.72 * 11.5 + (17.825 - 11.5 * 1.3) / 2.0).size(3.0, 3.0).fill(muted())).child(words(t, 11.5, 400.0, kept(17.825), dim()).max_w(inner - 14.0)),
        });
    }
    out.clip_rounded(9.0)
}

/// A script's or a table's face (`.fp-code-face`): its first twelve lines, small, on the paper.
fn code_face(words: &str, w: f32, h: f32) -> El {
    el().size(w, h).py(14.0).px(16.0).fill(surface()).clip_rounded(9.0).child(text(words).font(mono(10.5, 15.75)).color(dim()))
}

/// A word in the middle of an empty face (`.fp-face-empty`).
fn empty_face(words: &str, w: f32, h: f32) -> El {
    row().size(w, h).center().child(text(words).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()))
}

/// What a tile's face holds, by the file's kind (`Face` in `FilesPlace.tsx`). What this window
/// cannot draw small (a page, a PDF, a deck, a film's first frame) is its kind's word, and a
/// film its play mark.
fn face(s: &State, f: &Placed, w: f32, h: f32) -> El {
    let Some(opened) = f.opened.as_deref() else { return empty_face("Not on this computer yet", w, h) };
    match f.kind.as_deref() {
        Some("image") => picture_face(s, opened, w, h),
        Some("markdown" | "code" | "table") => match face_of(f).as_deref() {
            Some(Face::Doc(blocks)) => doc_face(blocks, w, h),
            Some(Face::Code(words)) => code_face(words, w, h),
            None => empty_face("Not on this computer yet", w, h),
        },
        Some("film") => {
            let play = row().absolute(8.0, h - 8.0 - 24.0).size(24.0, 24.0).radius(12.0).center().pl(1.0).fill(rgba(0, 0, 0, 0.62)).child(icon(&icons::PLAY, 10.0).color(rgb(255, 255, 255)));
            el().size(w, h).child(play)
        }
        kind => empty_face(rules::kind_one(kind), w, h),
    }
}

/// The marks after a tile's words: open comments, Approved, Changed since approved.
fn marks(f: &Placed) -> Vec<El> {
    let mut out = Vec::new();
    if f.open > 0 {
        out.push(row().gap(4.0).items_center().child(icon(&icons::COMMENT, 12.0).color(dim())).child(text(f.open.to_string()).font(figures(12.0, 400.0)).color(dim())));
    }
    if !f.approved_by.is_empty() {
        out.push(row().gap(6.0).items_center().child(el().size(6.0, 6.0).fill(rgb(0x6b, 0xc4, 0x8a))).child(label("Approved", 12.0, 400.0, dim())));
    } else if f.changed_since_approval.is_some() {
        out.push(label("Changed since approved", 12.0, 400.0, dim()));
    }
    out
}

/// How a tile stands on the page of every file: chosen or not while files are chosen for the
/// Trash (`chosen`), passed over then if Files may not move it (`unmovable`), its name being
/// changed, its menu open or the pointer on it.
struct Stands {
    chosen: Option<bool>,
    unmovable: bool,
    renaming: bool,
    open: bool,
    lit: bool,
    /// Its ⋯: on a file in a project, while nothing is chosen and its name is not a box.
    more: bool,
}

fn stands(s: &State, f: &Placed) -> Stands {
    let choosing = s.files.chosen.is_some();
    // Only a file the app says Files may move is chosen (`f.trashable`).
    let movable = f.trashable == Some(true);
    let renaming = s.files.renaming == Some(Renaming::File(f.key.clone()));
    let open = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::File(f.key.clone()));
    Stands {
        chosen: s.files.chosen.as_ref().filter(|_| movable).map(|c| c.contains(&f.key)),
        unmovable: choosing && !movable,
        renaming,
        open,
        lit: open || s.files.hover.tile.as_deref() == Some(hover_key(&f.key).as_str()),
        more: f.project.is_some() && !choosing && !renaming,
    }
}

/// The page of every file's own name for a tile or a card under the pointer, apart from the
/// home's.
fn hover_key(key: &str) -> String {
    format!("fp/{key}")
}

/// Keeps which tile or card the pointer is on (`.fp-tile:hover`, `.fp-folder:hover`).
fn hovered(e: El, key: String, card: bool) -> El {
    e.on_hover_change(move |s: &mut State, inside, _| {
        let at = if card { &mut s.files.hover.sec } else { &mut s.files.hover.tile };
        if inside {
            *at = Some(key.clone());
        } else if at.as_deref() == Some(key.as_str()) {
            *at = None;
        }
    })
}

/// A ⋯ of the page (`.fp-more`): 26 points, the surface and the strong line, its dots dim and in
/// the ink under the pointer; not seen until its tile or card is under the pointer, its menu is
/// open, or it stands on the check pictures' line.
fn more_button(key: String, label: String, shown: bool, open: bool, does: impl Fn(&mut State, &mut Tx) + 'static) -> El {
    let b = row()
        .key(key)
        .label(label)
        .size(26.0, 26.0)
        .radius(6.0)
        .center()
        .fill(surface())
        .border(1.0, rule())
        .border_layout()
        .hover_group()
        .cursor(Cursor::Pointer)
        .expanded(open)
        .role("button")
        .child(icon(&icons::FH_DOTS, 14.0).color(dim()).group_color(ink()))
        .on_press_tx(does);
    if shown { b } else { b.opacity(0.0) }
}

/// The ring a tile wears while files are chosen (`.fp-check`): white and ticked once chosen.
fn check(chosen: bool) -> El {
    let ring = row().size(18.0, 18.0).radius(9.0).center().border(1.5, white(if chosen { 1.0 } else { 0.85 })).border_layout().fill(if chosen { white(1.0) } else { rgba(0, 0, 0, 0.35) });
    if chosen { ring.child(icon(&icons::FP_CHECK, 11.0).color(rgb(17, 17, 17))) } else { ring }
}

/// The box a name is typed into on this page (`.fp-name-box`): 24 tall, six in, the wash and the
/// strong line, 13 points.
fn fp_name_box(s: &State, w: f32, placeholder: &'static str) -> El {
    row().w(w).h(24.0).px(6.0).radius(6.0).items_center().fill(feel::wash()).border(1.0, rule()).border_layout().child(crate::files_home::name_box(s, sans(13.0, 400.0), placeholder))
}

/// What a press on a tile does: opens it; while choosing, chooses it, or nothing for one Files
/// may not move; nothing while its name is a box.
fn tile_pressed(e: El, f: &Placed, st: &Stands) -> El {
    if st.renaming || st.unmovable {
        return e;
    }
    let key = f.key.clone();
    if st.chosen.is_some() { e.on_press(move |s: &mut State| s.choose_file(&key)) } else { e.on_press(move |s: &mut State| s.open_tile(&key)) }
}

/// A tile (`.fp-tile`): the face, sixteen by ten, its edge brighter under the pointer; the name
/// a person reads; and the line under it. A press opens the file big. Its ⋯ at its corner opens
/// its menu; while files are chosen, a ring there instead.
fn tile(s: &State, f: &Placed, w: f32) -> El {
    let h = w * 10.0 / 16.0;
    let st = stands(s, f);
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let said = rules::tile_words(f, s.now, me, |_| "A teammate".to_string());
    let name = rules::shown_name(f);
    let inside = face(s, f, w - 2.0, h - 2.0);
    // A PDF's and a deck's face sit on the strong wash (`.fp-face-pdf`, `.fp-face-deck`).
    let ground = if matches!(f.kind.as_deref(), Some("pdf" | "deck")) { feel::wash_strong() } else { white(0.04) };
    let face = el()
        .size(w, h)
        .radius(10.0)
        .fill(ground)
        // A chosen file's face is edged in the dim ink (`.is-chosen .fp-face`).
        .border(1.0, if st.chosen == Some(true) { dim() } else { edge() })
        .border_layout()
        .child(inside.radius(9.0))
        .child(feel::edge_lit(10.0, rule(), true));
    let face = if f.cloud { face.opacity(0.45) } else { face };
    let meta = row().mt(3.0).gap(12.0).items_center().w(w).child(row().grow(1.0).min_w(0.0).child(text(said).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()).truncate())).children(marks(f));
    let key = f.key.clone();
    // Renaming, the name is a box over its own line (`.fp-name > .fp-name-box`).
    let named = if st.renaming { row().mt(10.0).w(w).h(18.0).child(fp_name_box(s, w, "Name").ml(-6.0).mt(-3.0)) } else { row().mt(10.0).w(w).child(text(name.clone()).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate()) };
    let mut tile = col().key(format!("files-tile-{key}")).w(w).hover_group().child(face).child(named).child(meta);
    if name != f.name && !st.renaming {
        tile = tile.help(f.name.clone());
    }
    if let Some(chosen) = st.chosen {
        tile = tile.child(check(chosen).absolute(w - 9.0 - 18.0, 9.0));
    }
    if st.more {
        let k = key.clone();
        tile = tile.child(more_button(format!("fp-more-{key}"), format!("More for {name}"), st.lit, st.open, move |s: &mut State, tx| s.press_every_more(MenuOn::File(k.clone()), tx)).absolute(w - 8.0 - 26.0, 8.0));
    }
    let tile = if st.unmovable { tile.opacity(0.4) } else { tile };
    tile_pressed(hovered(tile, hover_key(&key), false), f, &st)
}

/// A row of the list (`.fp-line`): a small thumbnail, the name, the marks and the line, washed
/// under the pointer; its ⋯ or its ring at its end.
fn line(s: &State, f: &Placed, w: f32) -> El {
    let st = stands(s, f);
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let said = rules::tile_words(f, s.now, me, |_| "A teammate".to_string());
    let thumb = match (f.kind.as_deref(), f.opened.as_deref()) {
        (Some("image"), Some(opened)) => picture_face(s, opened, 44.0, 30.0),
        (Some("film"), Some(_)) => el().size(44.0, 30.0),
        (kind, _) => row().size(44.0, 30.0).center().child(text(rules::thumb_word(kind)).font(TextStyle::new(9.5, 400.0).lh(12.0).tracking(0.38)).color(muted())),
    };
    let thumb = el().size(46.0, 32.0).radius(6.0).fill(white(0.04)).border(1.0, edge()).border_layout().clip_rounded(6.0).child(thumb);
    let key = f.key.clone();
    let wide = w + 12.0;
    let name = if st.renaming {
        row().grow(1.0).min_w(0.0).h(18.0).child(fp_name_box(s, wide - 6.0 - 46.0 - 12.0 - 44.0, "Name").ml(-6.0).mt(-3.0))
    } else {
        row().grow(1.0).min_w(0.0).child(text(rules::shown_name(f)).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate())
    };
    let mut out = row()
        .key(format!("files-line-{key}"))
        .w(wide)
        .h(44.0)
        .pl(6.0)
        .pr(44.0)
        .gap(12.0)
        .items_center()
        .radius(6.0)
        .hover(feel::wash())
        .child(thumb)
        .child(name)
        .child(row().gap(12.0).children(marks(f)))
        .child(text(said).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()));
    if let Some(chosen) = st.chosen {
        out = out.child(check(chosen).absolute(wide - 12.0 - 18.0, 13.0));
    }
    if st.more {
        let k = key.clone();
        out = out.child(more_button(format!("fp-more-{key}"), format!("More for {}", rules::shown_name(f)), st.lit, st.open, move |s: &mut State, tx| s.press_every_more(MenuOn::File(k.clone()), tx)).absolute(wide - 8.0 - 26.0, 9.0));
    }
    let out = if st.unmovable { out.opacity(0.4) } else { out };
    tile_pressed(hovered(out, hover_key(&key), false), f, &st)
}

/// A project's card on All files, or a folder's (`.fp-folder`): its mark, its name and how many
/// files are in it. A folder's has its ⋯ (`FolderMenu`) while the pointer is on it or its menu is
/// open; renaming, its words give way to a box with its name.
fn folder_card(s: &State, name: &str, count: usize, w: f32, folder: Option<&str>, goes: impl Fn(&mut State) + 'static) -> El {
    let words = if count == 1 { "1 file".to_string() } else { format!("{count} files") };
    let renaming = folder.is_some_and(|id| s.files.renaming == Some(Renaming::Folder(id.to_string())));
    let card = row()
        .key(format!("fp-folder-{}", folder.unwrap_or(name)))
        .w(w)
        .h(56.0)
        .px(14.0)
        .gap(11.0)
        .items_center()
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, edge())
        .border_layout()
        .hover_group()
        .child(feel::edge_lit(10.0, rule(), true))
        .child(icon(&icons::FOLDER, 16.0).color(muted()));
    if renaming {
        // The box fills the words' column, six out to its left (`.fp-folder-words .fp-name-box`).
        return card.child(fp_name_box(s, w - 2.0 - 28.0 - 16.0 - 11.0, "Folder name").ml(-6.0));
    }
    let mut card = card
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .child(text(name).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate())
                .child(text(words).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted())),
        )
        .on_press(goes);
    let Some(id) = folder else { return card };
    let open = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::Folder(id.to_string()));
    let shown = open || s.files.hover.sec.as_deref() == Some(hover_key(id).as_str());
    let i = id.to_string();
    card = card.child(more_button(format!("fp-fmore-{id}"), format!("More for {name}"), shown, open, move |s: &mut State, tx| s.press_every_more(MenuOn::Folder(i.clone()), tx)).absolute(w - 2.0 - 14.0 - 26.0, 14.0));
    hovered(card, hover_key(id), true)
}

/* ------------------------------ the page ------------------------------ */

/// How tall the bar's controls stand: the switches, the kinds and the search box (`.fp-kind`,
/// `.fp-find`), the height of the strip's buttons.
const CONTROL: f32 = 32.0;

/// Words of the bar and of the kinds under it, in the soft patch of the sky's own dark that every
/// word there wears, as the Inbox's tabs do (`.fp-bar`, `:is(.fp-bar, .fp-tools) button`).
/// `lit` is their colour under the pointer.
fn shaded_words(words: String, font: TextStyle, size: f32, tone: rune::Color, lit: Option<rune::Color>) -> El {
    crate::inbox::shaded(move |shadow| {
        let e = text(words.as_str()).font(font).color(if shadow { crate::inbox::under(1.0) } else { tone });
        pen(match lit.filter(|_| !shadow) {
            Some(lit) => e.group_color(lit),
            None => e,
        }, size)
    })
}

/// A filter or a switch (`.fp-kind`): quiet words, the one on in ink on the strong wash.
fn kind_button(words: &str, on: bool, does: impl Fn(&mut State) + 'static) -> El {
    let said = shaded_words(words.to_string(), sans(12.5, 400.0), 12.5, if on { ink() } else { muted() }, Some(if on { ink() } else { dim() }));
    let b = row().key(format!("fp-kind-{words}")).h(CONTROL).px(10.0).items_center().radius(6.0).cursor(Cursor::Pointer).child(said);
    let b = if on { b.fill(feel::wash_strong()) } else { b.hover_group() };
    // A kind, the review or the order changed draws the first sixty again (`setDrawn`'s effect).
    b.on_press(move |s: &mut State| {
        does(s);
        s.files.drawn = rules::TILES_AT_ONCE;
    })
}

/// Grid or list, by its mark (`.fp-view-icon`).
fn view_button(glyph: &'static rune::Icon, on: bool, help: &'static str, list: bool) -> El {
    let b = row().size(CONTROL, CONTROL).center().radius(6.0).cursor(Cursor::Pointer).help(help).child(icon(glyph, 13.0).color(if on { ink() } else { muted() }).group_color(if on { ink() } else { dim() }));
    let b = if on { b.fill(feel::wash_strong()) } else { b.hover_group() };
    b.on_press(move |s: &mut State| s.files.as_list = list)
}

/// The columns of the grid for a page `wide` points across: as many of at least 232 points as
/// fit with 22 between them, sharing the rest (`repeat(auto-fill, minmax(232px, 1fr))`).
pub fn columns(wide: f32) -> (usize, f32) {
    let n = (((wide + 22.0) / (232.0 + 22.0)).floor() as usize).max(1);
    (n, (wide - 22.0 * (n - 1) as f32) / n as f32)
}

/// The page, under the card's strip, scrolled with the card's other pages: Files' home and its
/// pages (`files_home.rs`), or the page of every file.
pub fn view(s: &State) -> El {
    let View::Files { project } = &s.view else { return el() };
    // While files are chosen for the Trash this page stays, whatever crumb was pressed
    // (`if (!chosen && !everything) return <FilesHome …>`).
    if !every_page(s) {
        let list = list_of(s);
        return crate::files_home::view(s, list.as_deref());
    }
    let (page_w, _) = s.page_size();
    let wide = page_w - 44.0 - 32.0;
    let Some(list) = list_of(s) else { return col().pt(18.0).pl(44.0) };
    let place = Where { project: project.clone(), folder: real_folder(s, &list) };
    let below = rules::files_below(&list.files, &list.folders, &place);
    let here = rules::files_in_place(&list.files, &list.folders, &place);
    let query = s.files.query.text.as_str();
    let looking = !query.trim().is_empty() || s.files.kind.is_some() || s.files.review;
    let kinds = rules::kinds_in(&below);
    let reviewing = below.iter().filter(|f| rules::needs_review(f)).count();
    // Looking for something goes through every folder under here; only browsing stays in the one place.
    let found = if looking { rules::files_shown(&below, query, s.files.kind, &list.folders).into_iter().filter(|f| !s.files.review || rules::needs_review(f)).collect() } else { here };
    let (files, scratch) = if query.trim().is_empty() { rules::apart_scratch(found) } else { (found, Vec::new()) };
    let sorted = rules::sort_files(&files, s.files.sort);
    // A file arriving while the pointer is on its way to a tile would move that tile: the order
    // holds while the pointer moves over the files, and while a tile's menu or name box is open
    // (`holding`, `busy`).
    let busy = s.files.menu.as_ref().is_some_and(|m| matches!(m.on, MenuOn::File(_) | MenuOn::Scratch)) || matches!(s.files.renaming, Some(Renaming::File(_)));
    let holding = s.files.holding || busy;
    let shown = if holding { rules::in_order_of(&sorted, &s.files.held.borrow().tiles, |f| f.key.clone()) } else { sorted };
    let choosing = s.files.chosen.is_some();

    // The way back, and the box that searches (`.fp-bar`).
    let crumbs = rules::crumbs_to(&list.folders, &place);
    let last = crumbs.len() - 1;
    let crumbs = row().grow(1.0).min_w(0.0).items_center().children(crumbs.into_iter().enumerate().map(|(i, (name, to))| {
        let on = i == last;
        let crumb = row().key(format!("fp-crumb-{i}")).h(28.0).items_center().child(shaded_words(name, sans(14.0, 400.0), 14.0, if on { ink() } else { muted() }, Some(ink())));
        // The place it is on keeps the arrow and its look, and still takes a press: the
        // project's own crumb goes back to its home (`go`, `.fp-crumb.is-on { cursor: default }`).
        let crumb = if on { crumb } else { crumb.hover_group().cursor(Cursor::Pointer) };
        let crumb = crumb.on_press(move |s: &mut State| s.go_files_place(&to));
        let gap = (i > 0).then(|| crate::inbox::shaded(|shadow| text("/").font(sans(14.0, 400.0)).color(if shadow { crate::inbox::under(1.0) } else { muted() })).mx(8.0));
        row().items_center().children(gap).child(crumb)
    }));
    let search = text_input(&s.files.query, sans(13.0, 400.0))
        .key("files-search")
        .single_line()
        .grow(1.0)
        .color(ink())
        .placeholder("Search files", muted())
        .caret(ink(), rgba(110, 165, 255, 0.3))
        .on_edit(|s: &mut State| {
            s.files.drawn = rules::TILES_AT_ONCE;
            &mut s.files.query
        })
        .on_input_key(|s: &mut State, e: &rune::KeyEvent, tx: &mut Tx| {
            // Escape takes back what is typed; with nothing typed it does nothing, the page
            // taking no key from a box (`inInput` in `App.tsx`'s keydown). While files are
            // chosen it stops the choosing first, and goes no further (its listener comes first).
            if e.key != rune::edit::Key::Escape {
                return false;
            }
            if s.files.chosen.is_some() {
                s.stop_choosing();
            } else if !s.files.query.text.is_empty() {
                s.files.query = TextEdit::new("");
                s.files.query.focused = true;
            }
            let _ = tx;
            true
        });
    let find = row()
        .w(220.0)
        .h(CONTROL)
        .px(10.0)
        .gap(8.0)
        .items_center()
        .radius(6.0)
        .fill(crate::inbox::under(0.82))
        .border(1.0, if s.files.query.focused { rule() } else { edge() })
        .border_layout()
        .cursor(Cursor::Text)
        .child(icon(&icons::FILES_SEARCH, 15.0).color(muted()))
        .child(search)
        .on_press(|s: &mut State| s.files.query.focused = true);

    // How the files are ordered and drawn, in the bar between the way back and the search
    // (`.fp-views`): the total once it says something, Recent and Name, grid or list.
    let used = rules::on_mac_of(&below);
    let mut views = row().gap(12.0).items_center();
    if rules::shows_total(used) {
        // The quiet total (`.fp-total`): its words faint, dim under the pointer, in the ink on
        // the strong wash while its menu is open.
        let open = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::Total);
        let tone = if open { ink() } else { muted() };
        let lit = if open { ink() } else { dim() };
        let total = row()
            .key(TOTAL)
            .h(26.0)
            .px(9.0)
            .gap(7.0)
            .items_center()
            .radius(6.0)
            .cursor(Cursor::Pointer)
            .hover_group()
            .expanded(open)
            .child(icon(&icons::DISK, 13.0).color(tone).group_color(lit))
            .child(shaded_words(format!("{} on this {}", rules::size_words(used), machine()), figures(12.5, 400.0), 12.5, tone, Some(lit)))
            .on_press_tx(|s: &mut State, tx| s.press_every_more(MenuOn::Total, tx));
        views = views.child(if open { total.fill(feel::wash_strong()) } else { total });
    }
    let sort = s.files.sort;
    let mut sorts = row().gap(2.0).child(kind_button("Recent", sort == Sort::Recent, |s: &mut State| s.files.sort = Sort::Recent)).child(kind_button("Name", sort == Sort::Name, |s: &mut State| s.files.sort = Sort::Name));
    if sort == Sort::Size {
        sorts = sorts.child(kind_button("Size", true, |s: &mut State| s.files.sort = Sort::Recent));
    }
    views = views.child(sorts).child(row().gap(2.0).child(view_button(&icons::GRID, !s.files.as_list, "Grid", false)).child(view_button(&icons::FILES_LIST, s.files.as_list, "List", true)));
    // While files are chosen the bar holds only the way back and the search (`!chosen &&`).
    let mut bar = row().w(wide).h(CONTROL).gap(24.0).items_center().child(crumbs);
    if !choosing {
        bar = bar.child(views);
    }
    let bar = bar.child(find);

    // The kinds, under the bar while there is more than one or something needs review
    // (`.fp-tools`); while files are chosen, what is chosen and the way to the Trash.
    let tools_shown = kinds.len() > 1 || reviewing > 0 || s.files.review;
    let mut kind_row = row().gap(2.0).items_center();
    if kinds.len() > 1 {
        kind_row = kind_row.child(kind_button("All", s.files.kind.is_none() && !s.files.review, |s: &mut State| {
            s.files.kind = None;
            s.files.review = false;
        }));
        for k in kinds {
            let on = s.files.kind == Some(k);
            kind_row = kind_row.child(kind_button(rules::kind_plural(k), on, move |s: &mut State| s.files.kind = if s.files.kind == Some(k) { None } else { Some(k) }));
        }
    }
    if reviewing > 0 || s.files.review {
        kind_row = kind_row.child(kind_button("Needs review", s.files.review, |s: &mut State| s.files.review = !s.files.review));
    }
    let mut page = col().w(page_w).pt(18.0).pr(32.0).pb(48.0).pl(44.0).child(bar);
    if let Some(chosen) = &s.files.chosen {
        page = page.child(choosing_row(chosen, &below, wide));
    } else if tools_shown {
        page = page.child(row().w(wide).min_h(26.0).mt(14.0).items_center().child(kind_row));
    }
    let (n, column) = columns(wide);

    // A card for each project on All files, for each folder on a project's page (`.fp-folders`).
    let mut cards: Vec<El> = Vec::new();
    let mut held_projects = Vec::new();
    if !looking {
        if project.is_none() {
            // The project cards hold their order with the tiles' (`held.current.projects`).
            let projects = if holding { rules::in_order_of(&list.projects, &s.files.held.borrow().projects, |p| p.clone()) } else { list.projects.clone() };
            for p in &projects {
                let count = rules::files_below(&list.files, &list.folders, &Where { project: Some(p.clone()), folder: None }).len();
                let to = p.clone();
                cards.push(folder_card(s, p, count, column, None, move |s: &mut State| s.open_files_place(Some(to.clone()))));
            }
            held_projects = projects;
        } else {
            for f in rules::folders_in_place(&list.folders, &place) {
                let count = rules::files_below(&list.files, &list.folders, &Where { project: project.clone(), folder: Some(f.id.clone()) }).len();
                let id = f.id.clone();
                cards.push(folder_card(s, &f.name, count, column, Some(&f.id), move |s: &mut State| {
                    s.files.folder = Some(id.clone());
                    s.files.drawn = rules::TILES_AT_ONCE;
                }));
            }
        }
    }
    *s.files.held.borrow_mut() = Held { tiles: shown.iter().map(|f| f.key.clone()).collect(), projects: held_projects };
    // A folder being named from New stands as a card of its own after the others (`is-new`).
    if s.files.naming.is_some() && project.is_some() {
        let b = row().h(24.0).ml(-6.0).px(6.0).radius(6.0).items_center().grow(1.0).min_w(0.0).fill(feel::wash()).border(1.0, rule()).border_layout().child(crate::files_home::name_box(s, sans(13.0, 400.0), "Folder name"));
        cards.push(row().w(column).h(56.0).px(14.0).gap(11.0).items_center().radius(10.0).fill(white(0.04)).border(1.0, edge()).border_layout().child(icon(&icons::FOLDER, 16.0).color(muted())).child(b));
    }
    let had_cards = !cards.is_empty();
    if had_cards {
        page = page.child(held_still(grid(cards, n, 22.0, 10.0).mt(18.0)));
    }
    if shown.is_empty() && scratch.is_empty() && !had_cards {
        let empty = if looking {
            "No file matches.".to_string()
        } else if place.folder.is_some() {
            "Nothing in this folder yet.".to_string()
        } else {
            match project {
                Some(p) => format!("No files in {p} yet."),
                None => "Nothing in Files yet.".to_string(),
            }
        };
        page = page.child(text(empty).font(TextStyle::new(13.5, 400.0).lh(20.0)).color(muted()).mt(26.0));
    }
    // The tiles, a step at a time: a foot past the drawn ones asks for the next step as it nears.
    let drawn = s.files.drawn.max(rules::TILES_AT_ONCE);
    let total = shown.len() + if s.files.scratch_open { scratch.len() } else { 0 };
    let tiles: Vec<El> = shown.iter().take(drawn).map(|f| if s.files.as_list { line(s, f, wide) } else { tile(s, f, column) }).collect();
    if !tiles.is_empty() {
        page = page.child(held_still(if s.files.as_list { col().mt(18.0).ml(-6.0).children(tiles) } else { grid(tiles, n, 22.0, 30.0).mt(22.0) }));
    }
    // The check pictures agents took wait under one line, which opens them (`.fp-scratch`), and
    // a ⋯ beside it moves them all (`.fp-scratch-row`).
    if !scratch.is_empty() && drawn >= shown.len() {
        let open = s.files.scratch_open;
        let fold = row()
            .h(26.0)
            .pl(6.0)
            .pr(8.0)
            .gap(6.0)
            .radius(7.0)
            .items_center()
            .hover(feel::wash())
            .cursor(Cursor::Pointer)
            .child(icon(if open { &icons::FP_FOLD_OPEN } else { &icons::FP_FOLD_SHUT }, 12.0).color(dim()))
            .child(label(&rules::scratch_line(scratch.len()), 13.0, 400.0, dim()))
            .on_press(|s: &mut State| s.files.scratch_open = !s.files.scratch_open);
        let mut fold_row = row().mt(30.0).gap(8.0).items_center().child(fold);
        if scratch[0].project.is_some() {
            let open_menu = s.files.menu.as_ref().is_some_and(|m| m.on == MenuOn::Scratch);
            fold_row = fold_row.child(more_button(SCRATCH_MORE.to_string(), format!("Move {}", rules::scratch_line(scratch.len())), true, open_menu, |s: &mut State, tx| s.press_every_more(MenuOn::Scratch, tx)));
        }
        page = page.child(fold_row);
        if open {
            let more: Vec<El> = scratch.iter().take(drawn - shown.len()).map(|f| if s.files.as_list { line(s, f, wide) } else { tile(s, f, column) }).collect();
            page = page.child(if s.files.as_list { col().mt(16.0).ml(-6.0).children(more) } else { grid(more, n, 22.0, 30.0).mt(16.0) });
        }
    }
    if drawn < total {
        page = page.child(el().h(1.0).on_visible(move |s: &mut State, on, _| {
            if on {
                s.files.drawn = rules::more_tiles(drawn, total);
            }
        }));
    }
    page
}

/// The total's key, and the check pictures' ⋯'s.
pub const TOTAL: &str = "fp-total";
pub const SCRATCH_MORE: &str = "fp-more-scratch";

/// Whether the page of every file is drawn: asked for, or files are being chosen on it.
pub fn every_page(s: &State) -> bool {
    s.files.everything || s.files.chosen.is_some()
}

/// The computer's word (`MACHINE`): the app runs on Linux too, where "this Mac" would be untrue.
fn machine() -> &'static str {
    if cfg!(target_os = "macos") { "Mac" } else { "computer" }
}

/// The check pictures the page holds apart under their line, as it draws them.
fn scratch_here(s: &State) -> Vec<Placed> {
    let Some(list) = list_of(s) else { return Vec::new() };
    let place = Where { project: project_of(s), folder: real_folder(s, &list) };
    let below = rules::files_below(&list.files, &list.folders, &place);
    let query = s.files.query.text.trim();
    if !query.is_empty() {
        return Vec::new();
    }
    let looking = s.files.kind.is_some() || s.files.review;
    let found = if looking { rules::files_shown(&below, "", s.files.kind, &list.folders).into_iter().filter(|f| !s.files.review || rules::needs_review(f)).collect() } else { rules::files_in_place(&list.files, &list.folders, &place) };
    rules::apart_scratch(found).1.into_iter().cloned().collect()
}

/// The tiles and the cards keep their order while the pointer moves over them (`onPointerMove=
/// {pointerMoved}`).
fn held_still(e: El) -> El {
    e.on_pointer_move_local(|s: &mut State, _, tx| {
        let was = s.files.holding;
        s.files.holding = true;
        s.files.hold_until = crate::clock() + HOLD_STILL_MS;
        tx.wake_after(std::time::Duration::from_millis(HOLD_STILL_MS as u64));
        if was {
            tx.live_only();
        }
    })
}

/// While files are chosen for the Trash, under the bar (`.fp-tools.fp-choosing`): how many and
/// what they take, Cancel, and Move to the Trash, which waits for one.
fn choosing_row(chosen: &[String], below: &[&Placed], wide: f32) -> El {
    // The files chosen, as the page counts them: those here (`below.filter(chosen.has)`).
    let bytes: f64 = below.iter().filter(|f| chosen.contains(&f.key)).map(|f| f.movable.unwrap_or(0.0)).sum();
    let words = pen(text(rules::chosen_words(chosen.len(), bytes)).font(figures(13.0, 400.0)).color(dim()), 13.0);
    let some = !chosen.is_empty();
    let go = row()
        .key("fp-trash-go")
        .h(26.0)
        .px(11.0)
        .items_center()
        .radius(6.0)
        .border(1.0, rule())
        .border_layout()
        .child(shaded_words("Move to the Trash".to_string(), sans(12.5, 400.0), 12.5, if some { ink() } else { muted() }, None));
    let go = if some { go.fill(feel::wash_strong()).cursor(Cursor::Pointer).on_press_tx(|s: &mut State, tx| s.trash_chosen(tx)) } else { go };
    let cancel = kind_button("Cancel", false, |s: &mut State| s.stop_choosing());
    row().key("fp-choosing").w(wide).min_h(26.0).mt(14.0).gap(16.0).items_center().child(row().grow(1.0).min_w(0.0).child(words)).child(row().gap(12.0).items_center().child(cancel).child(go))
}

impl State {
    /// A ⋯ of the page of every file, or its total: the menu opens, or closes when it was open;
    /// a Move menu's box that finds a folder takes the keys as it opens (`autoFocus`).
    pub fn press_every_more(&mut self, on: MenuOn, tx: &mut Tx) {
        self.toggle_every_file_menu(on);
        let finds = self.files.menu.as_ref().is_some_and(|m| matches!(m.on, MenuOn::File(_) | MenuOn::Scratch)) && list_of(self).is_some_and(|l| self.files_folders_of(&l, self.files.menu.as_ref()).len() > 3);
        if finds {
            tx.focus_when_shown(MENU_FIND, false);
        } else {
            self.files.menu_find.focused = false;
        }
    }
    /// The folders a Move menu offers: its file's project's (`here`).
    fn files_folders_of<'a>(&self, list: &'a List, menu: Option<&Menu>) -> Vec<&'a Folder> {
        let project = match menu.map(|m| &m.on) {
            Some(MenuOn::File(key)) => list.files.iter().find(|f| &f.key == key).and_then(|f| f.project.clone()),
            Some(MenuOn::Scratch) => scratch_here(self).first().and_then(|f| f.project.clone()),
            _ => None,
        }
        .unwrap_or_default();
        list.folders.iter().filter(|f| f.project.to_lowercase() == project.to_lowercase()).collect()
    }
}

/// The key of a Move menu's box that finds a folder.
pub const MENU_FIND: &str = "fp-menu-find";

/* ------------------------------ the page's menus ------------------------------ */

/// A row of one of the page's menus (`.fp-menu-item`): 28 tall, eight in, 13 points, the strong
/// wash under the pointer; its mark faint, and what it frees or says at its end. One that cannot
/// be pressed is faint and takes nothing (`:disabled`).
fn fp_item(words: &str, inner: f32, mark: Option<El>, aside: Option<String>, enabled: bool, does: impl Fn(&mut State, &mut Tx) + 'static) -> El {
    let mut r = row().key(format!("fp-item-{words}")).w(inner).min_h(28.0).px(8.0).gap(8.0).radius(6.0).items_center().role("menuitem");
    if let Some(mark) = mark {
        r = r.child(mark);
    }
    r = r.child(pen(text(words).font(sans(13.0, 400.0)).color(if enabled { ink() } else { muted() }), 13.0));
    if let Some(aside) = aside {
        r = r.child(el().grow(1.0)).child(pen(text(aside).font(figures(12.0, 400.0)).color(muted()), 12.0).pl(16.0));
    }
    let r = r.clip_rounded(6.0);
    if enabled { r.hover(feel::wash_strong()).on_press_tx(does) } else { r }
}

/// A menu's quiet head (`.fp-menu-head`): 12 points, faint, 24.5 tall.
fn fp_head(words: &str) -> El {
    row().px(8.0).pt(5.0).pb(4.0).child(pen(text(words).font(sans(12.0, 400.0)).color(muted()), 12.0))
}

/// The line between a menu's groups (`.fp-menu-gap`).
fn fp_gap() -> El {
    el().h(1.0).mx(6.0).my(5.0).fill(edge())
}

const ITEM: f32 = 28.0;
const GAP_H: f32 = 11.0;
const HEAD: f32 = 24.5;
/// A menu's padding and edge, top and foot together.
const SHELL: f32 = 12.0;
/// A Move menu stops growing here and scrolls (`.fp-menu { max-height: 280px }`).
const MENU_MOST: f32 = 280.0;

/// A Move menu (`MoveMenu`): on a file, Rename, then where it can go in its project, New folder,
/// and Move to the Trash with what that frees; on the check pictures' line, where they all can
/// go. Its rows and their height.
fn move_menu(s: &State, list: &List, on: &MenuOn, step: Step) -> Option<(El, f32)> {
    let inner = 188.0;
    let (file, keys): (Option<&Placed>, Vec<String>) = match on {
        MenuOn::File(key) => {
            let f = list.files.iter().find(|f| &f.key == key)?;
            (Some(f), vec![key.clone()])
        }
        MenuOn::Scratch => (None, scratch_here(s).iter().map(|f| f.key.clone()).collect()),
        _ => return None,
    };
    let project = match file {
        Some(f) => f.project.clone(),
        None => scratch_here(s).first().and_then(|f| f.project.clone()),
    }
    .unwrap_or_default();
    let here = s.files_folders_of(list, s.files.menu.as_ref());
    let naming = step == Step::New;
    let query = s.files.menu_find.text.clone();
    let mut body = col();
    let mut h = SHELL;
    if let (Some(f), false) = (file, naming) {
        let k = f.key.clone();
        body = body.child(fp_item("Rename", inner, None, None, true, move |s: &mut State, tx| s.rename_file_start(&k, tx))).child(fp_gap());
        h += ITEM + GAP_H;
    }
    body = body.child(fp_head("Move to"));
    h += HEAD;
    if here.len() > 3 {
        let found_keys = keys.clone();
        let input = text_input(&s.files.menu_find, sans(13.0, 400.0))
            .key(MENU_FIND)
            .single_line()
            .grow(1.0)
            .min_w(0.0)
            .color(ink())
            .placeholder("Find a folder", muted())
            .caret(ink(), rgba(110, 165, 255, 0.3))
            .on_edit(|s: &mut State| &mut s.files.menu_find)
            // Return moves it into the one folder the words leave (`found.length === 1`).
            .on_submit(move |s: &mut State, tx: &mut Tx| {
                let Some(list) = list_of(s) else { return };
                let here = s.files_folders_of(&list, s.files.menu.as_ref());
                let found = rules::folders_matching(&here, &s.files.menu_find.text);
                if let [one] = found.as_slice() {
                    let to = one.id.clone();
                    s.move_all(found_keys.clone(), Some(to), tx);
                }
            })
            // Every key stays in the box; Escape closes the menu (`useClosesOutside`'s listener).
            .on_input_key(|s: &mut State, e: &rune::KeyEvent, _tx: &mut Tx| {
                if e.key == rune::edit::Key::Escape {
                    s.close_files_menus();
                }
                e.key == rune::edit::Key::Escape
            });
        let edge_color = if s.files.menu_find.focused { rule() } else { edge() };
        body = body.child(row().w(inner - 8.0).h(26.0).mx(4.0).mb(4.0).px(8.0).radius(6.0).items_center().border(1.0, edge_color).border_layout().cursor(Cursor::Text).child(input));
        h += 30.0;
    }
    let mut rows = col();
    let mut n = 0.0;
    let item = |f: &Folder, sub: bool| {
        let (ks, to) = (keys.clone(), f.id.clone());
        let here_now = file.is_some_and(|x| x.folder.as_deref() == Some(f.id.as_str()));
        let mark = row().pl(if sub { 14.0 } else { 0.0 }).child(icon(&icons::FOLDER, 16.0).color(muted()));
        fp_item(&f.name, inner, Some(mark), None, !here_now, move |s: &mut State, tx| s.move_all(ks.clone(), Some(to.clone()), tx))
    };
    if query.is_empty() && (file.is_none() || file.is_some_and(|f| f.folder.is_some())) {
        let ks = keys.clone();
        rows = rows.child(fp_item(&format!("Top of {project}"), inner, None, None, true, move |s: &mut State, tx| s.move_all(ks.clone(), None, tx)));
        n += 1.0;
    }
    if !query.is_empty() {
        let found = rules::folders_matching(&here, &query);
        for f in &found {
            rows = rows.child(item(f, false));
            n += 1.0;
        }
        if found.is_empty() {
            rows = rows.child(row().px(8.0).py(6.0).child(pen(text("No folder called that").font(sans(13.0, 400.0)).color(muted()), 13.0)));
            h += 29.0;
        }
    } else {
        for t in here.iter().filter(|f| f.parent.is_none()) {
            rows = rows.child(item(t, false));
            n += 1.0;
            for sub in here.iter().filter(|f| f.parent.as_deref() == Some(t.id.as_str())) {
                rows = rows.child(item(sub, true));
                n += 1.0;
            }
        }
    }
    body = body.child(rows);
    h += n * ITEM;
    if naming {
        // `.fp-menu-new`: the box at the foot, fourteen in and six out.
        body = body.child(row().pt(2.0).pb(3.0).pl(14.0).pr(8.0).child(fp_name_box(s, inner - 22.0, "Folder name").ml(-6.0)));
        h += 29.0;
    } else {
        body = body.child(fp_item("New folder", inner, None, None, true, |s: &mut State, tx| {
            s.menu_step(Step::New);
            s.files.name_box = TextEdit::new("");
            s.files.name_box.focused = true;
            s.files.menu_find.focused = false;
            tx.focus_when_shown(NAME_BOX, false);
        }));
        h += ITEM;
    }
    if let (Some(f), false) = (file, naming)
        && f.trashable == Some(true)
    {
        let k = f.key.clone();
        let frees = f.movable.filter(|m| *m > 0.0).map(rules::size_words);
        body = body.child(fp_gap()).child(fp_item("Move to the Trash", inner, None, frees, true, move |s: &mut State, tx| s.trash_file(&k, tx)));
        h += GAP_H + ITEM;
    }
    Some((body, h))
}

/// A folder card's menu (`FolderMenu`): Rename and Delete folder.
fn folder_menu(id: &str) -> (El, f32) {
    let (a, b) = (id.to_string(), id.to_string());
    let body = col()
        .child(fp_item("Rename", 188.0, None, None, true, move |s: &mut State, tx| s.rename_folder_start(&a, tx)))
        .child(fp_item("Delete folder", 188.0, None, None, true, move |s: &mut State, tx| s.remove_folder(&b, tx)));
    (body, SHELL + 2.0 * ITEM)
}

/// The total's menu (`TotalMenu`): what this place takes, the biggest first or the newest, the
/// older versions to the Trash, choosing files for it, and keeping files in the cloud.
fn total_menu(s: &State, list: &List) -> (El, f32) {
    let inner = 288.0;
    let place = Where { project: project_of(s), folder: real_folder(s, list) };
    let below = rules::files_below(&list.files, &list.folders, &place);
    let (used, older) = (rules::on_mac_of(&below), rules::older_of(&below));
    let place_name = place.folder.as_ref().and_then(|id| list.folders.iter().find(|f| &f.id == id)).map(|f| f.name.clone()).or_else(|| place.project.clone()).unwrap_or_else(|| "Files".to_string());
    let sort_words = if s.files.sort == Sort::Size { "Show the newest first" } else { "Show the biggest first" };
    let mut body = col().child(fp_head(&format!("{place_name} uses {} of this {}", rules::size_words(used), machine()))).child(fp_item(sort_words, inner, None, None, true, |s: &mut State, _| {
        s.close_files_menus();
        s.files.sort = if s.files.sort == Sort::Size { Sort::Recent } else { Sort::Size };
        s.files.drawn = rules::TILES_AT_ONCE;
    }));
    let mut h = SHELL + HEAD + ITEM + GAP_H;
    body = body.child(fp_gap());
    if older > 0.0 {
        body = body.child(fp_item("Move older versions to the Trash", inner, None, Some(rules::size_words(older)), true, |s: &mut State, tx| s.trash_older(tx)));
        h += ITEM;
    }
    // Pro until a plan with a cap is had; then whether it is on (`paid ? (on ? 'On' : 'Off') : 'Pro'`).
    let cloud = s.files.cloud.as_ref().or_else(|| s.fixture.get("filesCloud"));
    let paid = cloud.and_then(|c| c.get("cap")).is_some_and(|cap| !cap.is_null() && cap.as_f64() != Some(0.0));
    let on = cloud.and_then(|c| c.get("on")).and_then(Value::as_bool).unwrap_or(false);
    let says = if paid { if on { "On" } else { "Off" } } else { "Pro" };
    body = body
        .child(fp_item("Choose files to move to the Trash", inner, None, None, true, |s: &mut State, _| s.start_choosing()))
        .child(fp_gap())
        .child(fp_item("Keep files in the cloud", inner, None, Some(says.to_string()), true, |s: &mut State, tx| s.files_cloud_settings(tx)));
    h += ITEM + GAP_H + ITEM;
    (body, h)
}

/// The shell every menu of the page wears (`.fp-menu`): the surface, the strong line, corner 10,
/// the small shadow; five in. `most` is how tall it may stand before it scrolls inside.
fn fp_shell(body: El, w: f32, h: f32, most: Option<f32>) -> El {
    let shown = most.map_or(h, |m| h.min(m));
    let inside = col().padding(5.0).child(body);
    let inside = if shown < h { col().w(w - 2.0).h(shown - 2.0).clip_rounded(9.0).scroll(rune::ScrollSpec { stick_bottom: false }).child(inside) } else { inside };
    col()
        .key("fp-menu")
        .w(w)
        .h(shown)
        .radius(10.0)
        .fill(surface())
        .border(1.0, rule())
        .border_layout()
        .box_shadow(shadow(0.0, 1.0, 2.0, 0.0, rgba(0, 0, 0, 0.3), false))
        .role("menu")
        .child(inside)
        // A press inside the menu, on a row that cannot be pressed or between rows, keeps it
        // open (`box.current.contains(e.target)`).
        .on_press(|_: &mut State| {})
}

/// Whatever menu of the page of every file stands open, placed: under its ⋯ (30 points down,
/// its right edge on the ⋯'s), or over it when the ⋯ is under 300 points from the window's foot,
/// then kept inside the page (`fitMenu`); the total's under the total.
pub fn every_layer(s: &State) -> Option<El> {
    let list = list_of(s)?;
    let m = s.files.menu.as_ref()?;
    let (anchor, body, w, h, fits) = match &m.on {
        MenuOn::File(key) => {
            let (body, h) = move_menu(s, &list, &m.on, m.step)?;
            (format!("fp-more-{key}"), body, 200.0, h, true)
        }
        MenuOn::Scratch => {
            let (body, h) = move_menu(s, &list, &m.on, m.step)?;
            (SCRATCH_MORE.to_string(), body, 200.0, h, true)
        }
        MenuOn::Folder(id) => {
            let (body, h) = folder_menu(id);
            (format!("fp-fmore-{id}"), body, 200.0, h, false)
        }
        MenuOn::Total => {
            let (body, h) = total_menu(s, &list);
            (TOTAL.to_string(), body, 300.0, h, false)
        }
    };
    let [x, y, bw, bh] = rune::memo::window_layout(&anchor)?;
    let right = x + bw;
    if !fits {
        let most = (w == 200.0).then_some(MENU_MOST);
        return Some(fp_shell(body, w, h, most).absolute(right - w, y + 30.0));
    }
    // The page's own foot, above the window's: the page clips what runs past it.
    let page_foot = s.size.1 - chrome::CARD[3];
    let tall = h.min(MENU_MOST);
    let up = s.size.1 - (y + bh) < 300.0;
    let hang = crate::rules::compose::Hang { below: !up, gap: 30.0 - bh, dx: bw - w };
    let sides = (chrome::card_left(s), s.size.0 - chrome::CARD[2]);
    let fit = crate::rules::compose::fit_menu([x, y, bw, bh], hang, (w, tall), page_foot, sides);
    let most = fit.max_height.map_or(MENU_MOST, |m| m.min(MENU_MOST));
    Some(fp_shell(body, w, h, Some(most)).absolute(fit.left, fit.top))
}

/// Rows of `n` cells, `gap` across and `rows_gap` down, as the page's grids lay them out.
fn grid(cells: Vec<El>, n: usize, gap: f32, rows_gap: f32) -> El {
    let mut out = col().gap(rows_gap);
    let mut cells = cells.into_iter().peekable();
    while cells.peek().is_some() {
        out = out.child(row().gap(gap).items_start().children(cells.by_ref().take(n)));
    }
    out
}

impl State {
    /// A crumb of the page of every file pressed: All files, a project's page, or a folder in
    /// it; a place with no folder is the home again (`setEverything(false)`).
    fn go_files_place(&mut self, to: &Where) {
        let View::Files { project } = &self.view else { return };
        if *project != to.project {
            self.open_files_place(to.project.clone());
        }
        self.files.folder = to.folder.clone();
        if to.folder.is_none() {
            self.files.everything = false;
        }
    }
}

/* ------------------------------ the strip and the keys ------------------------------ */

/// The page's name in the card's strip, over the sky as the Inbox's is, in a soft patch of the
/// sky's own dark (`.workspace-title`): 44 points into the card, as the web window draws it on
/// Files, where it draws no arrows before it (`backForward` is the Inbox's alone).
pub fn strip(s: &State) -> El {
    let (card_w, _) = s.card_size();
    let under = crate::inbox::under;
    let words = |shadow: bool| pen(text("Files").font(sans(17.0, 400.0).tracking(-0.51)).color(if shadow { under(1.0) } else { ink() }), 17.0);
    let shadows = [8.0, 5.0, 3.0, 1.5, 0.5].map(|sigma| el().absolute(0.0, 0.0).blur(sigma).child(words(true)));
    let patch = el().pin(Some(0.0), Some(0.0), None, None).cache().layer_opacity(0.999).inert().a11y_hidden().children(shadows);
    let title = el().child(patch).child(words(false));
    let strip = row().absolute(chrome::card_left(s), chrome::CARD[1]).size(card_w, 64.0).pl(44.0).pr(28.0).pb(1.0).items_center().window_drag().child(title);
    // On the page of every file New stands at the strip's right (`#fp-head-slot`), a button of
    // the strip like the Inbox's New thread, its menu under it.
    let offers = every_page(s) && list_of(s).is_some_and(|l| crate::files_home::new_offers(s, &l).is_some());
    if !offers {
        return strip;
    }
    let open = s.files.new_menu.is_some();
    let new = crate::files_home::over_the_sky(s, STRIP_NEW, 6.0, 1.0, open, feel::wash_strong())
        .h(32.0)
        .gap(8.0)
        .px(13.0)
        .items_center()
        .cursor(Cursor::Pointer)
        .expanded(open)
        .child(icon(&icons::FH_PLUS, 13.0).color(dim()))
        .child(crate::inbox::shaded(|shadow| pen(label("New", 13.5, 400.0, if shadow { crate::inbox::under(1.0) } else { ink() }), 13.5)))
        .on_press(|s: &mut State| s.toggle_new_menu());
    strip.child(el().grow(1.0)).child(new)
}

/// New in the strip, on the page of every file.
pub const STRIP_NEW: &str = "files-strip-new";

/// A window opened straight onto the Files place (`--view files`) came there from Home; one
/// opened for a picture says where in its fixture: a project's home (`filesProject`), a folder's
/// page (`filesFolder`), the page of every file (`filesEverything`) and how it lists them
/// (`filesAsList`, `filesSort`), words in the search box (`filesQuery`), and what stands open: a
/// menu (`filesMenu`: `{ file | folder, step, at }`), New's (`filesNew`: true, or what it asks a
/// project for), the box for a new folder (`filesNaming`) or a name being changed
/// (`filesRenaming`).
pub fn pictured(s: &mut State) {
    let View::Files { .. } = s.view else { return };
    let project = s.fixture.get("filesProject").and_then(Value::as_str).map(str::to_string);
    s.view = View::Files { project: project.clone() };
    s.history.push(View::Home);
    let said = |k: &str| s.fixture.get(k).cloned().unwrap_or(Value::Null);
    let text_of = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
    s.files.folder = said("filesFolder").as_str().map(str::to_string);
    s.files.everything = said("filesEverything").as_bool().unwrap_or(false);
    s.files.shelf = home::SHELF_STEP;
    if let Some(q) = said("filesQuery").as_str() {
        s.files.query = TextEdit::new(q);
        s.files.query.set_caret(q.len());
        s.files.query.focused = true;
    }
    let menu = said("filesMenu");
    if menu.is_object() {
        let flag = |k: &str| menu.get(k).and_then(Value::as_bool) == Some(true);
        let on = match (text_of(&menu, "file"), text_of(&menu, "folder")) {
            (Some(key), _) => Some(MenuOn::File(key)),
            (None, Some(id)) => Some(MenuOn::Folder(id)),
            _ if flag("total") => Some(MenuOn::Total),
            _ if flag("scratch") => Some(MenuOn::Scratch),
            _ => None,
        };
        // A Move menu's box that finds a folder has the keys as it opens, and what was typed.
        if let Some(find) = text_of(&menu, "find") {
            s.files.menu_find = TextEdit::new(&find);
            s.files.menu_find.set_caret(find.len());
        }
        s.files.menu_find.focused = s.files.everything && matches!(on, Some(MenuOn::File(_) | MenuOn::Scratch)) && text_of(&menu, "step").as_deref() != Some("new");
        let step = match text_of(&menu, "step").as_deref() {
            Some("icon") => Step::Icon,
            Some("move") => Step::Move,
            Some("new") => Step::New,
            _ => Step::Menu,
        };
        let at = menu.get("at").and_then(Value::as_array).and_then(|a| Some((a.first()?.as_f64()? as f32, a.get(1)?.as_f64()? as f32)));
        if let Some(on) = on {
            s.files.menu = Some(Menu { on, step, at });
        }
        if step == Step::New {
            s.files.name_box.focused = true;
        }
    }
    match said("filesNew") {
        Value::Bool(true) => s.files.new_menu = Some(None),
        Value::String(what) => {
            s.files.new_menu = Some(match what.as_str() {
                "table" => Some(Making::Table),
                "folder" => Some(Making::Folder),
                "upload" => Some(Making::Upload),
                _ => Some(Making::Document),
            })
        }
        _ => {}
    }
    if said("filesNaming").as_bool() == Some(true) {
        s.files.naming = project.clone();
        s.files.name_box.focused = true;
    }
    if let Some(key) = text_of(&said("filesRenaming"), "file") {
        let name = list_of(s).and_then(|l| l.files.iter().find(|f| f.key == key).map(crate::files_home::name_of)).unwrap_or_default();
        s.files.name_box = TextEdit::new(name.clone());
        s.files.name_box.set_selection(0, name.len());
        s.files.name_box.focused = true;
        s.files.renaming = Some(Renaming::File(key));
    }
    if let Some(id) = text_of(&said("filesRenaming"), "folder") {
        let name = list_of(s).and_then(|l| l.folders.iter().find(|f| f.id == id).map(|f| f.name.clone())).unwrap_or_default();
        s.files.name_box = TextEdit::new(name.clone());
        s.files.name_box.set_selection(0, name.len());
        s.files.name_box.focused = true;
        s.files.renaming = Some(Renaming::Folder(id));
    }
    // Files chosen for the Trash (`filesChosen`), in the order they were chosen.
    if let Some(chosen) = said("filesChosen").as_array() {
        s.files.chosen = Some(chosen.iter().filter_map(|k| k.as_str().map(str::to_string)).collect());
    }
    s.files.cloud = s.fixture.get("filesCloud").cloned();
    s.files.as_list = s.fixture.get("filesAsList").and_then(Value::as_bool).unwrap_or(false);
    s.files.sort = match s.fixture.get("filesSort").and_then(Value::as_str) {
        Some("name") => Sort::Name,
        Some("size") => Sort::Size,
        _ => Sort::Recent,
    };
    s.files.drawn = rules::TILES_AT_ONCE;
    // A headless picture of a door's Files cannot wait for the list, so it asks for it here.
    if let (Source::Door(door), Some(_)) = (&s.source, rune::tooling::headless_run())
        && let Some((url, token)) = door.now()
        && let Ok(list) = source::call(&url, &token, "zero:files-place", &serde_json::json!({})).and_then(|v| serde_json::from_value::<List>(v).map_err(|e| e.to_string()))
    {
        s.files.list = Some(Rc::new(list));
    }
}

/// Escape while a menu of the home stands open closes it and goes no further (`Menu`'s and
/// `useClosesOutside`'s listeners, which take it first). Every other key goes on.
pub fn menu_key(s: &mut State, e: &rune::KeyEvent) -> Option<bool> {
    if !matches!(s.view, View::Files { .. }) || (s.files.menu.is_none() && s.files.new_menu.is_none() && s.files.chosen.is_none()) {
        return None;
    }
    if e.key == rune::edit::Key::Escape {
        // A menu's listener and the choosing's both hear it (`useClosesOutside`, `chosen`'s).
        s.close_files_menus();
        s.stop_choosing();
        return Some(true);
    }
    None
}

/// The keys on the Files page (`filesShown` in `renderer/src/App.tsx`'s keydown): `/` puts the
/// caret in the search box, Escape leaves the page. N still starts a thread.
pub fn key(s: &mut State, e: &rune::KeyEvent, tx: &mut Tx) -> Option<bool> {
    use rune::edit::Key;
    if !matches!(s.view, View::Files { .. }) {
        return None;
    }
    match &e.key {
        Key::Text(t) if t == "/" => {
            s.files.query.focused = true;
            tx.focus_when_shown(SEARCH, false);
            Some(true)
        }
        Key::Escape => {
            if s.history.is_empty() { s.go(View::Inbox) } else { s.back() }
            Some(true)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feel::tests::key;
    use crate::files_home::tests::{files, folder_of, lands, list, press_on, press_shown, says, toast};
    use crate::rules::files_home::{FOLDER_ACT, TRASH, UPLOAD_HERE};
    use crate::write::Landed;
    use rune::Runtime;
    use rune::edit::Key;

    const MB: f64 = 1024.0 * 1024.0;

    /// The home tests' invented list, each file taking two megabytes on the Mac, week-32.pdf one
    /// more in an older version, and failed-orders.csv in a code folder, which Files counts but
    /// never moves.
    fn heavy() -> Value {
        let mut l = list();
        for f in l["files"].as_array_mut().unwrap() {
            let name = f["name"].as_str().unwrap().to_string();
            f["onMac"] = json!(if name == "week-32.pdf" { 3.0 * MB } else { 2.0 * MB });
            f["older"] = json!(if name == "week-32.pdf" { MB } else { 0.0 });
            f["movable"] = json!(if name == "failed-orders.csv" { 0.0 } else { 2.0 * MB });
            if name == "failed-orders.csv" {
                f["trashable"] = json!(false);
            }
        }
        l
    }

    /// The page of every file of Storefront, at its top unless `more` says a folder.
    fn every(name: &str, more: Value) -> Runtime<State> {
        let mut fixture = json!({ "filesEverything": true, "filesPlace": heavy() });
        for (k, v) in more.as_object().cloned().unwrap_or_default() {
            fixture[k] = v;
        }
        files(name, Some("Storefront"), fixture)
    }

    fn tile_menu(rt: &mut Runtime<State>, file: &str) {
        press_shown(rt, &format!("files-tile-{file}"), &format!("fp-more-{file}"));
    }

    #[test]
    fn a_tiles_dots_open_its_move_menu_and_a_folder_picked_moves_it_drawn_at_once() {
        let mut rt = every("fp-move", json!({}));
        assert!(rt.state.files.menu.is_none());
        tile_menu(&mut rt, "storefront/week-32.pdf");
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.on.clone()), Some(MenuOn::File("storefront/week-32.pdf".into())));
        for words in ["Rename", "Move to", "Pinned", "Receipts", "Calls", "Brand kit", "New folder", "Move to the Trash", "2 MB"] {
            assert!(says(&rt, words), "{words:?} is not on the menu");
        }
        // At the project's top already: no way to the top (`file.folder` is null).
        assert!(!says(&rt, "Top of Storefront"));
        // More than three folders: the box that finds one, with the keys.
        assert!(rt.find(MENU_FIND).is_some() && rt.state.files.menu_find.focused);
        press_on(&mut rt, "fp-item-Receipts");
        assert!(rt.state.files.menu.is_none());
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/week-32.pdf", "folder": "rec" }))));
        assert_eq!(folder_of(&rt, "storefront/week-32.pdf").as_deref(), Some("rec"), "drawn there before the app answers");
        lands(&mut rt, json!({ "ok": true, "folder": null }));
        assert_eq!(rt.state.undos.last().map(|u| u.said.as_str()), Some("Moved week-32.pdf back"));
    }

    #[test]
    fn a_press_on_the_dots_of_an_open_menu_closes_it() {
        let mut rt = every("fp-again", json!({}));
        tile_menu(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fp-more-storefront/week-32.pdf");
        assert!(rt.state.files.menu.is_none(), "the ⋯ is inside the box that keeps the menu open, so its press toggles it shut");
    }

    #[test]
    fn the_move_menus_search_leaves_the_folders_it_names_and_return_moves_into_the_one() {
        let mut rt = every("fp-find", json!({}));
        tile_menu(&mut rt, "storefront/week-32.pdf");
        key(&mut rt, Key::Text("re".into()));
        assert_eq!(rt.state.files.menu_find.text, "re");
        assert!(rt.find("fp-item-Receipts").is_some() && rt.find("fp-item-Pinned").is_none() && rt.find("fp-item-Calls").is_none());
        assert!(rt.find("fp-item-Top of Storefront").is_none(), "words in the box take the top away");
        key(&mut rt, Key::Text("zz".into()));
        assert!(says(&rt, "No folder called that"));
        rt.update(|s, _| s.files.menu_find = TextEdit::new("cal"));
        rt.frame(1.0 / 60.0);
        rt.update(|s, _| s.files.menu_find.focused = true);
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/week-32.pdf", "folder": "calls" }))));
        // Escape in the box closes the menu, and no more.
        tile_menu(&mut rt, "storefront/failed-orders.csv");
        key(&mut rt, Key::Escape);
        assert!(rt.state.files.menu.is_none() && matches!(rt.state.view, View::Files { .. }));
    }

    #[test]
    fn a_file_in_a_folder_is_offered_the_top_and_not_its_own_folder() {
        let mut rt = every("fp-own", json!({ "filesFolder": "rec" }));
        tile_menu(&mut rt, "storefront/receipt-plan.md");
        assert!(says(&rt, "Top of Storefront"));
        let sent = rt.state.tried.len();
        press_on(&mut rt, "fp-item-Receipts");
        assert_eq!(rt.state.tried.len(), sent, "the folder it is in is not a choice (`disabled`)");
        press_on(&mut rt, "fp-item-Top of Storefront");
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/receipt-plan.md", "folder": null }))));
    }

    #[test]
    fn new_folder_in_the_move_menu_makes_it_then_moves_the_file_and_escape_goes_back() {
        let mut rt = every("fp-new-folder", json!({}));
        tile_menu(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fp-item-New folder");
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.step), Some(Step::New));
        assert!(!says(&rt, "Rename") && !says(&rt, "Move to the Trash"), "put away while the box is there");
        key(&mut rt, Key::Escape);
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.step), Some(Step::Menu), "back to the whole menu");
        press_on(&mut rt, "fp-item-New folder");
        key(&mut rt, Key::Text("Launch".into()));
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "make", "project": "Storefront", "name": "Launch", "parent": null }))));
        lands(&mut rt, json!({ "ok": true, "folder": "launch-1" }));
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/week-32.pdf", "folder": "launch-1" }))));
    }

    #[test]
    fn rename_on_the_page_of_every_file_makes_the_name_a_box_and_return_writes_it() {
        let mut rt = every("fp-rename", json!({}));
        tile_menu(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fp-item-Rename");
        assert_eq!(rt.state.files.renaming, Some(Renaming::File("storefront/week-32.pdf".into())));
        assert_eq!(rt.state.files.name_box.selected(), "week-32.pdf");
        assert!(rt.find("fp-more-storefront/week-32.pdf").is_none(), "no ⋯ while the name is a box");
        key(&mut rt, Key::Text("Week 32".into()));
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "name", "key": "storefront/week-32.pdf", "name": "Week 32" }))));
    }

    #[test]
    fn move_to_the_trash_on_a_tiles_menu_says_what_it_frees() {
        let mut rt = every("fp-trash", json!({}));
        tile_menu(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fp-item-Move to the Trash");
        assert_eq!(rt.state.tried.last(), Some(&(TRASH, json!({ "keys": ["storefront/week-32.pdf"], "older": false }))));
        lands(&mut rt, json!({ "ok": true, "moved": 1, "bytes": 2048, "undo": { "token": "t-2" } }));
        assert_eq!(toast(&rt).as_deref(), Some("Moved week-32.pdf to the Trash. Empty the Trash to get the 2 KB back."));
        // A file Files may not move has no such row.
        tile_menu(&mut rt, "storefront/failed-orders.csv");
        assert!(!says(&rt, "Move to the Trash"));
    }

    #[test]
    fn a_folder_cards_dots_rename_it_in_the_card_or_delete_it() {
        let mut rt = every("fp-card", json!({}));
        press_shown(&mut rt, "fp-folder-rec", "fp-fmore-rec");
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.on.clone()), Some(MenuOn::Folder("rec".into())));
        assert!(says(&rt, "Rename") && says(&rt, "Delete folder"));
        press_on(&mut rt, "fp-item-Rename");
        assert_eq!(rt.state.files.renaming, Some(Renaming::Folder("rec".into())));
        assert_eq!(rt.state.files.name_box.selected(), "Receipts");
        key(&mut rt, Key::Text("Bills".into()));
        key(&mut rt, Key::Enter);
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "rename", "id": "rec", "name": "Bills" }))));
        press_shown(&mut rt, "fp-folder-brand", "fp-fmore-brand");
        press_on(&mut rt, "fp-item-Delete folder");
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "remove", "id": "brand" }))));
        lands(&mut rt, json!({ "ok": true, "folder": null, "ids": ["brand"] }));
        assert_eq!(toast(&rt).as_deref(), Some("Deleted the folder Brand kit."));
    }

    #[test]
    fn the_totals_menu_sorts_by_size_moves_older_versions_and_opens_the_clouds_plans() {
        let mut rt = every("fp-total", json!({ "filesCloud": { "cap": null, "on": false } }));
        assert!(says(&rt, "11 MB on this Mac"));
        press_on(&mut rt, TOTAL);
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.on.clone()), Some(MenuOn::Total));
        for words in ["Storefront uses 11 MB of this Mac", "Show the biggest first", "Move older versions to the Trash", "1 MB", "Choose files to move to the Trash", "Keep files in the cloud", "Pro"] {
            assert!(says(&rt, words), "{words:?} is not on the menu");
        }
        press_on(&mut rt, "fp-item-Show the biggest first");
        assert_eq!(rt.state.files.sort, Sort::Size);
        assert!(says(&rt, "Size"), "the sort's own Size, on");
        press_on(&mut rt, TOTAL);
        assert!(says(&rt, "Show the newest first"));
        press_on(&mut rt, "fp-item-Move older versions to the Trash");
        assert_eq!(rt.state.tried.last(), Some(&(TRASH, json!({ "keys": ["storefront/week-32.pdf"], "older": true }))));
        assert_eq!(toast(&rt).as_deref(), Some("Moving the older versions to the Trash…"));
        // No paid plan: the plans, on Settings' cloud page.
        press_on(&mut rt, TOTAL);
        press_on(&mut rt, "fp-item-Keep files in the cloud");
        assert!(rt.state.settings.open);
        assert_eq!(rt.state.settings.pane, crate::rules::settings::Pane::Page(crate::rules::settings::PageId::Cloud));
        // On a plan with a cap, Settings › Files, and the row says whether it is on.
        let mut rt = every("fp-total-paid", json!({ "filesCloud": { "cap": 100.0, "on": true } }));
        press_on(&mut rt, TOTAL);
        assert!(says(&rt, "On"));
        press_on(&mut rt, "fp-item-Keep files in the cloud");
        assert_eq!(rt.state.settings.pane, crate::rules::settings::Pane::Page(crate::rules::settings::PageId::Files));
    }

    #[test]
    fn files_chosen_for_the_trash_go_together_in_the_order_they_were_chosen() {
        let mut rt = every("fp-choose", json!({}));
        press_on(&mut rt, TOTAL);
        press_on(&mut rt, "fp-item-Choose files to move to the Trash");
        assert_eq!(rt.state.files.chosen, Some(vec![]));
        assert!(says(&rt, "Choose files to move to the Trash") && says(&rt, "Cancel"));
        assert!(rt.find(TOTAL).is_none() && rt.find("fp-more-storefront/week-32.pdf").is_none(), "no total and no ⋯ while choosing");
        // Nothing chosen: Move to the Trash does nothing.
        let sent = rt.state.tried.len();
        press_on(&mut rt, "fp-trash-go");
        assert_eq!(rt.state.tried.len(), sent);
        // A press chooses a tile instead of opening it; one Files may not move is passed over.
        press_on(&mut rt, "files-tile-storefront/week-32.pdf");
        press_on(&mut rt, "files-tile-storefront/failed-orders.csv");
        assert_eq!(rt.state.files.chosen, Some(vec!["storefront/week-32.pdf".to_string()]));
        assert!(says(&rt, "1 chosen · 2 MB"));
        // Pressed again, it is chosen no more; pressed once more, chosen again.
        press_on(&mut rt, "files-tile-storefront/week-32.pdf");
        assert_eq!(rt.state.files.chosen, Some(vec![]));
        press_on(&mut rt, "files-tile-storefront/week-32.pdf");
        // Chosen elsewhere: a folder's page is still the page of every file, the choice kept.
        rt.update(|s, _| s.files.folder = Some("pin".into()));
        rt.frame(1.0 / 60.0);
        press_on(&mut rt, "files-tile-storefront/rounding.md");
        press_on(&mut rt, "fp-crumb-1");
        assert!(rt.state.files.chosen.as_ref().is_some_and(|c| c.len() == 2));
        press_on(&mut rt, "fp-trash-go");
        assert_eq!(rt.state.tried.last(), Some(&(TRASH, json!({ "keys": ["storefront/week-32.pdf", "storefront/rounding.md"], "older": false }))));
        assert_eq!(toast(&rt).as_deref(), Some("Moving 2 files to the Trash…"));
        assert!(rt.state.files.chosen.is_none());
    }

    #[test]
    fn escape_or_cancel_stops_choosing_and_files_stays() {
        let mut rt = every("fp-choose-escape", json!({ "filesChosen": ["storefront/week-32.pdf"] }));
        key(&mut rt, Key::Escape);
        assert!(rt.state.files.chosen.is_none());
        assert!(matches!(rt.state.view, View::Files { .. }));
        let mut rt = every("fp-choose-cancel", json!({ "filesChosen": [] }));
        press_on(&mut rt, "fp-kind-Cancel");
        assert!(rt.state.files.chosen.is_none());
    }

    #[test]
    fn upload_from_computer_opens_the_macs_file_window_here_and_hands_the_app_the_path() {
        let mut rt = files("fp-upload", Some("Storefront"), json!({ "filesFolder": "rec" }));
        press_on(&mut rt, crate::files_home::NEW);
        press_on(&mut rt, "fp-menu-item-Upload from computer");
        let (kind, options) = rt.file_dialog().expect("the file window is up");
        assert_eq!((kind, options.title.as_str()), (rune::file_dialog::FileDialogKind::Open, "Add files to Storefront"));
        rt.resolve_file_dialog(Some("/Users/you/Desktop/invented-plan.pdf".into())).unwrap();
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried.last(), Some(&(UPLOAD_HERE, json!({ "project": "Storefront", "folder": "rec", "paths": ["/Users/you/Desktop/invented-plan.pdf"] }))));
        assert_eq!(rt.state.last_sent, Some(Sent::Files(Wrote::Upload)));
        // Added: the list is read again, and nothing is said.
        lands(&mut rt, json!({ "ok": true, "added": 1, "refused": [], "keys": ["storefront/invented-plan.pdf"] }));
        assert!(toast(&rt).is_none() && rt.state.files.stale);
        // What Files cannot show is named.
        lands(&mut rt, json!({ "ok": true, "added": 0, "refused": ["tool.exe"], "keys": [] }));
        assert_eq!(toast(&rt).as_deref(), Some("tool.exe is a kind Files cannot show, so it was not added."));
        // The app's own words for a refusal, else the page's.
        lands(&mut rt, json!({ "ok": false, "error": "That project is gone." }));
        assert_eq!(toast(&rt).as_deref(), Some("That project is gone."));
        lands(&mut rt, json!({ "ok": false }));
        assert_eq!(toast(&rt).as_deref(), Some("Those files could not be added."));
        // An app started before the door had the channel.
        rt.update(|s, tx| s.landed(Landed { sent: Sent::Files(Wrote::Upload), answer: Err("http://127.0.0.1:1/api/zero%3Afiles-place-upload-here: status code 404".into()), snapshot: None }, tx));
        assert_eq!(toast(&rt).as_deref(), Some("Not done: restart Mercury first, so it can take that from this window."));
        // A cancelled window adds nothing and says nothing.
        let sent = rt.state.tried.len();
        rt.update(|s, _| s.toast = None);
        press_on(&mut rt, crate::files_home::NEW);
        press_on(&mut rt, "fp-menu-item-Upload from computer");
        rt.resolve_file_dialog(None).unwrap();
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried.len(), sent);
        assert!(toast(&rt).is_none());
    }

    #[test]
    fn upload_from_all_files_asks_which_project_and_adds_to_its_top() {
        let mut rt = files("fp-upload-all", None, json!({}));
        press_on(&mut rt, crate::files_home::NEW);
        press_on(&mut rt, "fp-menu-item-Upload from computer");
        assert!(says(&rt, "Add files to"));
        press_on(&mut rt, "fp-menu-item-Mobile");
        assert_eq!(rt.state.view, View::Files { project: Some("Mobile".into()) });
        assert_eq!(rt.file_dialog().map(|(_, o)| o.title.clone()).as_deref(), Some("Add files to Mobile"));
        rt.resolve_file_dialog(Some("/Users/you/Desktop/a.png".into())).unwrap();
        rt.frame(1.0 / 60.0);
        assert_eq!(rt.state.tried.last(), Some(&(UPLOAD_HERE, json!({ "project": "Mobile", "folder": null, "paths": ["/Users/you/Desktop/a.png"] }))));
    }

    #[test]
    fn the_tiles_hold_their_order_while_the_pointer_moves_over_them() {
        let mut rt = every("fp-hold", json!({}));
        let x_of = |rt: &Runtime<State>, k: &str| rt.find(&format!("files-tile-storefront/{k}")).unwrap().0;
        assert!(x_of(&rt, "failed-orders.csv") < x_of(&rt, "week-32.pdf"), "newest first");
        // The pointer moves over the tiles; a newer file arrives.
        let (x, y) = rt.find("files-tile-storefront/week-32.pdf").unwrap();
        rt.pointer_move(x, y);
        rt.frame(1.0 / 60.0);
        assert!(rt.state.files.holding);
        let mut l = heavy();
        l["files"].as_array_mut().unwrap().push(json!({ "key": "storefront/new.md", "project": "Storefront", "name": "new.md", "kind": "markdown", "versions": 1, "at": 1_790_000_000_000.0_f64, "open": 0, "approvedBy": [], "sentIn": [], "trashable": true }));
        rt.update(|s, _| s.files.list = Some(Rc::new(serde_json::from_value(l).unwrap())));
        rt.frame(1.0 / 60.0);
        assert!(x_of(&rt, "failed-orders.csv") < x_of(&rt, "week-32.pdf") && x_of(&rt, "week-32.pdf") < x_of(&rt, "new.md"), "the new one waits after the rest");
        let _ = y;
        // Still a moment: the sort is as before.
        rt.update(|s, tx| {
            tend(s, f64::MAX, false, tx);
        });
        rt.frame(1.0 / 60.0);
        assert!(!rt.state.files.holding);
        assert!(x_of(&rt, "new.md") < x_of(&rt, "failed-orders.csv"), "newest first again");
    }

    #[test]
    fn the_check_pictures_move_all_at_once_and_z_puts_back_those_the_app_took() {
        let mut l = heavy();
        for n in ["page-2026-10-31T17-42-40-037Z.png", "v2-check.png"] {
            l["files"].as_array_mut().unwrap().push(json!({ "key": format!("storefront/{n}"), "project": "Storefront", "name": n, "kind": "image", "versions": 1, "at": 1.0, "agent": "", "open": 0, "approvedBy": [], "sentIn": [], "trashable": true }));
        }
        let mut rt = every("fp-scratch", json!({ "filesPlace": l }));
        assert!(says(&rt, "2 check pictures agents took"));
        press_on(&mut rt, SCRATCH_MORE);
        assert_eq!(rt.state.files.menu.as_ref().map(|m| m.on.clone()), Some(MenuOn::Scratch));
        assert!(says(&rt, "Top of Storefront") && !says(&rt, "Rename") && !says(&rt, "Move to the Trash"));
        let undos = rt.state.undos.len();
        press_on(&mut rt, "fp-item-Brand kit");
        let n = rt.state.tried.len();
        assert_eq!(&rt.state.tried[n - 2..], &[
            (FOLDER_ACT, json!({ "do": "move", "key": "storefront/page-2026-10-31T17-42-40-037Z.png", "folder": "brand" })),
            (FOLDER_ACT, json!({ "do": "move", "key": "storefront/v2-check.png", "folder": "brand" })),
        ]);
        // Each answer lands; Z waits for both, and the refused one is not put back.
        let pending: Vec<(Act, u64)> = rt.state.files.pending.iter().filter_map(|p| match &p.change { Shown::Act(a) => Some((a.clone(), p.serial)), _ => None }).collect();
        let group = rt.state.files.groups[0].id;
        for (i, (act, serial)) in pending.into_iter().enumerate() {
            let answer = if i == 0 { json!({ "ok": true, "folder": null }) } else { json!({ "ok": false, "error": "That file is gone." }) };
            rt.update(|s, tx| s.landed(Landed { sent: Sent::Files(Wrote::Act { act: act.clone(), serial, undo: None, then: Then::Group(group) }), answer: Ok(answer), snapshot: None }, tx));
            if i == 0 {
                assert_eq!(rt.state.undos.len(), undos, "not before every move is answered");
            }
        }
        assert_eq!(rt.state.undos.len(), undos + 1);
        let back = &rt.state.undos.last().unwrap().writes;
        assert_eq!(back.len(), 1, "only the move the app took is put back: {back:?}");
    }

    #[test]
    fn z_draws_what_it_puts_back_at_the_key_and_takes_it_off_if_the_app_refuses() {
        let mut rt = every("fp-z", json!({}));
        tile_menu(&mut rt, "storefront/week-32.pdf");
        press_on(&mut rt, "fp-item-Brand kit");
        lands(&mut rt, json!({ "ok": true, "folder": null }));
        key(&mut rt, Key::Text("z".into()));
        assert_eq!(rt.state.tried.last(), Some(&(FOLDER_ACT, json!({ "do": "move", "key": "storefront/week-32.pdf", "folder": null }))));
        assert_eq!(folder_of(&rt, "storefront/week-32.pdf"), None, "back at the key, before the app answers");
        lands(&mut rt, json!({ "ok": false, "error": "That folder is gone." }));
        assert_eq!(folder_of(&rt, "storefront/week-32.pdf").as_deref(), Some("brand"), "the refused Z is taken back off");
        assert_eq!(toast(&rt).as_deref(), Some("That folder is gone."));
    }

    #[test]
    fn a_kind_no_file_here_is_any_more_lets_go_and_a_kind_draws_the_first_sixty_again() {
        let mut rt = every("fp-kind", json!({}));
        rt.update(|s, tx| {
            s.files.kind = Some("film");
            tend(s, 0.0, false, tx);
        });
        assert_eq!(rt.state.files.kind, None);
        rt.update(|s, _| s.files.drawn = 120);
        rt.frame(1.0 / 60.0);
        press_on(&mut rt, "fp-kind-PDFs");
        assert_eq!((rt.state.files.kind, rt.state.files.drawn), (Some("pdf"), rules::TILES_AT_ONCE));
        rt.update(|s, tx| {
            tend(s, 0.0, false, tx);
        });
        assert_eq!(rt.state.files.kind, Some("pdf"), "a kind still here stays");
    }

    #[test]
    fn the_grid_has_as_many_columns_of_232_points_as_fit() {
        // The page at 1280 by 760 is 1100 points across: four tiles of 258.5.
        assert_eq!(columns(1100.0), (4, 258.5));
        assert_eq!(columns(700.0).0, 2);
        assert_eq!(columns(200.0), (1, 200.0));
    }
}
