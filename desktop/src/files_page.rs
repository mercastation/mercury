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
//! with `zero:files-place-untrash`. Every one is a channel the web window calls.
//!
//! Left out: Upload from computer (the app answers it with the system's file window, which
//! would open over the app's own window, not this one), dragging a tile onto a folder, and on the
//! page of every file the tiles' and folders' menus, choosing files for the Trash, the total's
//! menu and New in the strip.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

/// What a menu of the home is open on (`TileMenu`, `FolderHead`'s).
#[derive(Clone, Debug, PartialEq)]
pub enum MenuOn {
    File(String),
    Folder(String),
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
}

/// What follows a change once the app has it.
#[derive(Clone, Debug, PartialEq)]
pub enum Then {
    Nothing,
    /// A folder made from Move to…'s New folder…: the file moves into it (`moveToNew`).
    MoveInto(String),
    /// A folder deleted: said, and Z puts it back (`removeFolder`).
    Deleted { name: String, files: usize, project: String },
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
}

impl Place {
    pub fn set_waker(&self, wake: Waker) {
        self.reading.set_waker(wake.clone());
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
    let mut changed = false;
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
            // Another place of Files is the same page: no step back to the last.
            self.view = View::Files { project };
        } else {
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
        self.start_naming(&crate::files_home::name_of(&f), tx);
        self.files.renaming = Some(Renaming::File(key.to_string()));
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
            if let (MenuOn::File(key), false) = (&m.on, typed.is_empty()) {
                let project = self.file(key).and_then(|f| f.project).or_else(|| project_of(self)).unwrap_or_default();
                self.act(Act::Make { project, name: typed, parent: None }, None, Then::MoveInto(key.clone()), tx);
            }
            return;
        }
        match self.files.renaming.take() {
            Some(Renaming::File(key)) => {
                let Some(f) = self.file(&key) else { return };
                let start = crate::files_home::name_of(&f);
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
        if let Some(m) = self.files.menu.as_mut().filter(|m| m.step == Step::New) {
            m.step = Step::Move;
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
    /// Move to the Trash on a file's menu (`trash`).
    pub fn trash_file(&mut self, key: &str, tx: &mut Tx) {
        self.close_files_menus();
        let Some(f) = self.file(key) else { return };
        let what = rules::shown_name(&f);
        self.tell(&format!("Moving {what} to the Trash…"), tx);
        if !self.send(home::TRASH, json!({ "keys": [key], "older": false }), Sent::Files(Wrote::Trash { what })) {
            self.tell("Those files could not be moved to the Trash.", tx);
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
            // The app answers this with the system's file window, over its own window.
            Making::Upload => {}
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
        Wrote::Undone { acts, trash } => {
            let Some(done) = s.answering.undoing.take() else { return };
            if refused {
                let otherwise = if trash { "Those files are in the Trash now. Put them back from there." } else { "That could not be put back." };
                let words = refused_words(&landed.answer, otherwise);
                return s.tell(&words, tx);
            }
            for act in acts.into_iter().filter(|a| !matches!(a, Act::Make { .. } | Act::Restore { .. })) {
                s.files.serial += 1;
                let serial = s.files.serial;
                s.files.pending.push(Pending { change: Shown::Act(act), done: Some(now), serial });
            }
            s.say(done.said, false, tx);
        }
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

/// A tile (`.fp-tile`): the face, sixteen by ten, its edge brighter under the pointer; the name
/// a person reads; and the line under it. A press opens the file big.
fn tile(s: &State, f: &Placed, w: f32) -> El {
    let h = w * 10.0 / 16.0;
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let said = rules::tile_words(f, s.now, me, |_| "A teammate".to_string());
    let name = rules::shown_name(f);
    let inside = face(s, f, w - 2.0, h - 2.0);
    let face = el()
        .size(w, h)
        .radius(10.0)
        .fill(white(0.04))
        .border(1.0, edge())
        .border_layout()
        .child(inside.radius(9.0))
        .child(feel::edge_lit(10.0, rule(), true));
    let face = if f.cloud { face.opacity(0.45) } else { face };
    let meta = row().mt(3.0).gap(12.0).items_center().w(w).child(row().grow(1.0).min_w(0.0).child(text(said).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()).truncate())).children(marks(f));
    let key = f.key.clone();
    let mut tile = col().key(format!("files-tile-{key}")).w(w).hover_group().child(face).child(row().mt(10.0).w(w).child(text(name.clone()).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate())).child(meta);
    if name != f.name {
        tile = tile.help(f.name.clone());
    }
    tile.on_press(move |s: &mut State| s.open_tile(&key))
}

/// A row of the list (`.fp-line`): a small thumbnail, the name, the marks and the line, washed
/// under the pointer.
fn line(s: &State, f: &Placed, w: f32) -> El {
    let me = crate::rules::window::me(&s.snap).and_then(|m| m.get("id")).and_then(Value::as_str);
    let said = rules::tile_words(f, s.now, me, |_| "A teammate".to_string());
    let thumb = match (f.kind.as_deref(), f.opened.as_deref()) {
        (Some("image"), Some(opened)) => picture_face(s, opened, 44.0, 30.0),
        (Some("film"), Some(_)) => el().size(44.0, 30.0),
        (kind, _) => row().size(44.0, 30.0).center().child(text(rules::thumb_word(kind)).font(TextStyle::new(9.5, 400.0).lh(12.0).tracking(0.38)).color(muted())),
    };
    let thumb = el().size(46.0, 32.0).radius(6.0).fill(white(0.04)).border(1.0, edge()).border_layout().clip_rounded(6.0).child(thumb);
    let key = f.key.clone();
    row()
        .w(w + 12.0)
        .h(44.0)
        .pl(6.0)
        .pr(44.0)
        .gap(12.0)
        .items_center()
        .radius(6.0)
        .hover(feel::wash())
        .child(thumb)
        .child(row().grow(1.0).min_w(0.0).child(text(rules::shown_name(f)).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate()))
        .child(row().gap(12.0).children(marks(f)))
        .child(text(said).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted()))
        .on_press(move |s: &mut State| s.open_tile(&key))
}

/// A project's card on All files, or a folder's (`.fp-folder`): its mark, its name and how many
/// files are in it.
fn folder_card(name: &str, count: usize, w: f32, goes: impl Fn(&mut State) + 'static) -> El {
    let words = if count == 1 { "1 file".to_string() } else { format!("{count} files") };
    row()
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
        .child(icon(&icons::FOLDER, 16.0).color(muted()))
        .child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .child(text(name).font(TextStyle::new(13.5, 400.0).lh(18.0)).color(ink()).truncate())
                .child(text(words).font(TextStyle::new(12.0, 400.0).lh(16.0)).color(muted())),
        )
        .on_press(goes)
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
    let b = row().h(CONTROL).px(10.0).items_center().radius(6.0).cursor(Cursor::Pointer).child(said);
    let b = if on { b.fill(feel::wash_strong()) } else { b.hover_group() };
    b.on_press(does)
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
    if !s.files.everything {
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
    let shown = rules::sort_files(&files, s.files.sort);

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
            // Escape takes back what is typed first, then the keys go back to the page.
            if e.key != rune::edit::Key::Escape {
                return false;
            }
            if s.files.query.text.is_empty() {
                s.files.query.focused = false;
                tx.blur();
            } else {
                s.files.query = TextEdit::new("");
                s.files.query.focused = true;
            }
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
        let machine = if cfg!(target_os = "macos") { "Mac" } else { "computer" };
        views = views.child(row().h(26.0).px(9.0).gap(7.0).items_center().child(icon(&icons::DISK, 13.0).color(muted())).child(shaded_words(format!("{} on this {machine}", rules::size_words(used)), figures(12.5, 400.0), 12.5, muted(), None)));
    }
    let sort = s.files.sort;
    let mut sorts = row().gap(2.0).child(kind_button("Recent", sort == Sort::Recent, |s: &mut State| s.files.sort = Sort::Recent)).child(kind_button("Name", sort == Sort::Name, |s: &mut State| s.files.sort = Sort::Name));
    if sort == Sort::Size {
        sorts = sorts.child(kind_button("Size", true, |s: &mut State| s.files.sort = Sort::Recent));
    }
    views = views.child(sorts).child(row().gap(2.0).child(view_button(&icons::GRID, !s.files.as_list, "Grid", false)).child(view_button(&icons::FILES_LIST, s.files.as_list, "List", true)));
    let bar = row().w(wide).h(CONTROL).gap(24.0).items_center().child(crumbs).child(views).child(find);

    // The kinds, under the bar while there is more than one or something needs review
    // (`.fp-tools`).
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
    if tools_shown {
        page = page.child(row().w(wide).min_h(26.0).mt(14.0).items_center().child(kind_row));
    }
    let (n, column) = columns(wide);

    // A card for each project on All files, for each folder on a project's page (`.fp-folders`).
    let mut cards: Vec<El> = Vec::new();
    if !looking {
        if project.is_none() {
            for p in &list.projects {
                let count = rules::files_below(&list.files, &list.folders, &Where { project: Some(p.clone()), folder: None }).len();
                let to = p.clone();
                cards.push(folder_card(p, count, column, move |s: &mut State| s.open_files_place(Some(to.clone()))));
            }
        } else {
            for f in rules::folders_in_place(&list.folders, &place) {
                let count = rules::files_below(&list.files, &list.folders, &Where { project: project.clone(), folder: Some(f.id.clone()) }).len();
                let id = f.id.clone();
                cards.push(folder_card(&f.name, count, column, move |s: &mut State| s.files.folder = Some(id.clone())));
            }
        }
    }
    // A folder being named from New stands as a card of its own after the others (`is-new`).
    if s.files.naming.is_some() && project.is_some() {
        let b = row().h(24.0).ml(-6.0).px(6.0).radius(6.0).items_center().grow(1.0).min_w(0.0).fill(feel::wash()).border(1.0, rule()).border_layout().child(crate::files_home::name_box(s, sans(13.0, 400.0), "Folder name"));
        cards.push(row().w(column).h(56.0).px(14.0).gap(11.0).items_center().radius(10.0).fill(white(0.04)).border(1.0, edge()).border_layout().child(icon(&icons::FOLDER, 16.0).color(muted())).child(b));
    }
    let had_cards = !cards.is_empty();
    if had_cards {
        page = page.child(grid(cards, n, 22.0, 10.0).mt(18.0));
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
        page = page.child(if s.files.as_list { col().mt(18.0).ml(-6.0).children(tiles) } else { grid(tiles, n, 22.0, 30.0).mt(22.0) });
    }
    // The check pictures agents took wait under one line, which opens them (`.fp-scratch`).
    if !scratch.is_empty() && drawn >= shown.len() {
        let open = s.files.scratch_open;
        let fold = row()
            .mt(30.0)
            .ml(-6.0)
            .h(26.0)
            .pl(6.0)
            .pr(8.0)
            .gap(6.0)
            .radius(7.0)
            .items_center()
            .hover(feel::wash())
            .cursor(Cursor::Pointer)
            .child(text(if open { "⌃" } else { "›" }).font(TextStyle::new(12.0, 400.0).lh(12.0)).color(dim()))
            .child(label(&rules::scratch_line(scratch.len()), 13.0, 400.0, dim()))
            .on_press(|s: &mut State| s.files.scratch_open = !s.files.scratch_open);
        page = page.child(row().child(fold));
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
    let offers = s.files.everything && list_of(s).is_some_and(|l| crate::files_home::new_offers(s, &l).is_some());
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
        let on = match (text_of(&menu, "file"), text_of(&menu, "folder")) {
            (Some(key), _) => Some(MenuOn::File(key)),
            (None, Some(id)) => Some(MenuOn::Folder(id)),
            _ => None,
        };
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
    if !matches!(s.view, View::Files { .. }) || (s.files.menu.is_none() && s.files.new_menu.is_none()) {
        return None;
    }
    if e.key == rune::edit::Key::Escape {
        s.close_files_menus();
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

    #[test]
    fn the_grid_has_as_many_columns_of_232_points_as_fit() {
        // The page at 1280 by 760 is 1100 points across: four tiles of 258.5.
        assert_eq!(columns(1100.0), (4, 258.5));
        assert_eq!(columns(700.0).0, 2);
        assert_eq!(columns(200.0), (1, 200.0));
    }
}
