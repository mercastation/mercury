//! The sheet worked by the hands, on an invented sheet: what each press, key and menu row does,
//! and what is written. Each test names the lines of `docs/parity-tables.md` it holds.

use rune::{Key, KeyEvent, Mods, PhysicalInput, Runtime};

use super::*;
use crate::feel::tests::{press, rect_of, run, with_a_picture};

/// An invented list of leads: a name, two kinds of tags, a day, a site, an email, an amount
/// and a long note.
pub(crate) const LEADS: &str = "Company,Status,Owner,Next call,Website,Email,ARR,Note\n\
Acme Robotics,Meeting,Dana,2026-11-24,acme-robotics.example,dana@acme-robotics.example,\"$12,000\",Asked for a demo of the export and wants pricing for a team of forty before the end of the month\n\
Globex,Replied,Sam,2026-11-26,globex.example,sam@globex.example,\"$8,400\",Replied to the second note\n\
Initech,New,Dana,,initech.example,,,\n\
Umbrella Labs,Contacted,Ari,2026-12-02,umbrella.example,ari@umbrella.example,\"$3,150\",Opened the email twice\n\
Hooli,Lost,Sam,,hooli.example,,$0,Went with another tool\n\
Vandelay,Meeting,Ari,2026-11-25,vandelay.example,ari@vandelay.example,\"$22,500\",Intro through a mutual friend\n";

/// A conversation naming `exports/leads.csv`, the sheet open beside it, saves going to nobody.
fn open(name: &str, csv: &str) -> (Runtime<State>, std::path::PathBuf) {
    let csv = csv.to_string();
    let (mut rt, shot) = with_a_picture(name, move |shot| {
        let dir = std::path::Path::new(shot).parent().and_then(std::path::Path::parent).expect("its folder");
        std::fs::create_dir_all(dir.join("exports")).expect("a folder for the sheet");
        std::fs::write(dir.join("exports/leads.csv"), &csv).expect("an invented sheet");
        "The list is in exports/leads.csv.".to_string()
    });
    let dir = shot.parent().and_then(std::path::Path::parent).expect("its folder").to_path_buf();
    // Wide enough that every column of the invented sheet is in view beside the conversation.
    rt.set_viewport((2400.0, 900.0), 2.0);
    rt.frame(1.0 / 60.0);
    rt.update(|s, _| {
        s.tables.test_door = true;
        s.open_beside("exports/leads.csv", false);
    });
    run(&mut rt, 0.1);
    (rt, dir)
}

fn sheet(rt: &Runtime<State>) -> &Sheet {
    rt.state.tables.pane.as_ref().expect("the sheet is open beside the conversation")
}

/// The middle of a cell, in window points, as it is drawn now.
fn cell_at(rt: &Runtime<State>, r: usize, c: usize) -> (f32, f32) {
    let [vx, vy, _, _] = rect_of(rt, "tbl-scroll-pane").expect("the grid is drawn");
    let sh = sheet(rt);
    let (x, w) = draw::column_x(&rt.state, sh, c);
    let at = scrolled(Where::Pane);
    let left = if c == sh.first() { (x - at.offset[0]).max(sh.rn_w()) } else { x - at.offset[0] };
    (vx + left + (w / 2.0).min(30.0), vy + HEAD_H + r as f32 * sh.row_h() + sh.row_h() / 2.0 - at.offset[1])
}

fn row_number_at(rt: &Runtime<State>, r: usize) -> (f32, f32) {
    let [vx, vy, _, _] = rect_of(rt, "tbl-scroll-pane").expect("the grid is drawn");
    let sh = sheet(rt);
    (vx + 20.0, vy + HEAD_H + r as f32 * sh.row_h() + sh.row_h() / 2.0 - scrolled(Where::Pane).offset[1])
}

fn header_at(rt: &Runtime<State>, c: usize) -> (f32, f32) {
    let [vx, vy, _, _] = rect_of(rt, "tbl-scroll-pane").expect("the grid is drawn");
    let sh = sheet(rt);
    let (x, _) = draw::column_x(&rt.state, sh, c);
    (vx + x - scrolled(Where::Pane).offset[0] + 24.0, vy + HEAD_H / 2.0)
}

fn hold(rt: &mut Runtime<State>, mods: Mods) {
    rt.physical_input(PhysicalInput::Modifiers(mods));
}

fn press_with(rt: &mut Runtime<State>, at: impl Fn(&Runtime<State>) -> (f32, f32), mods: Mods) {
    let (x, y) = at(rt);
    hold(rt, mods);
    press(rt, x, y);
    hold(rt, Mods::default());
}

/// A press where `at` finds, as the sheet is drawn now.
fn tap(rt: &mut Runtime<State>, at: impl Fn(&Runtime<State>) -> (f32, f32)) {
    let (x, y) = at(rt);
    press(rt, x, y);
}

fn key(rt: &mut Runtime<State>, key: Key) {
    key_with(rt, key, Mods::default());
}

fn key_with(rt: &mut Runtime<State>, key: Key, mods: Mods) {
    rt.key(KeyEvent { key, mods });
    rt.frame(1.0 / 60.0);
}

fn shift() -> Mods {
    Mods { shift: true, ..Mods::default() }
}
fn cmd() -> Mods {
    Mods { cmd: true, logo: true, ..Mods::default() }
}
fn alt() -> Mods {
    Mods { alt: true, ..Mods::default() }
}

fn typed(rt: &mut Runtime<State>, words: &str) {
    for c in words.chars() {
        key(rt, Key::Text(c.to_string()));
    }
}

/// What the sheet holds at a place in the view and a column.
fn value(rt: &Runtime<State>, r: usize, c: usize) -> String {
    sheet(rt).value(At { r, c }).to_string()
}

/// What the last save sent the app, once its 400 ms have gone by.
fn written(rt: &mut Runtime<State>) -> Value {
    run(rt, 0.5);
    rt.state.tables.wrote.last().cloned().expect("a save went out")
}

fn done(dir: std::path::PathBuf) {
    let _ = std::fs::remove_dir_all(dir);
}

// A2 A4 A5 C1 C9 D1 D2 D7 D14 D19 E1 E2 F1 F2 F6 F13 F14 (and A6 by its own test below)
#[test]
fn a_sheet_beside_a_conversation_is_picked_as_a_spreadsheet_is() {
    let (mut rt, dir) = open("tbl-pick", LEADS);
    let sh = sheet(&rt);
    assert!(sh.editable, "a sheet inside the project's own folders can be changed");
    assert_eq!(sh.table().columns.iter().map(|c| c.kind).collect::<Vec<_>>(), [Kind::Name, Kind::Tag, Kind::Tag, Kind::Date, Kind::Link, Kind::Email, Kind::Number, Kind::Text]);
    assert_eq!(draw::count_words(sh), "6 rows");
    assert!(rect_of(&rt, "tbl-search-pane").is_some(), "the bar has its search box");
    // A press picks a cell; Shift stretches the block; the count says so.
    tap(&mut rt, |rt| cell_at(rt, 1, 4));
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 1, c: 4 })));
    press_with(&mut rt, |rt| cell_at(rt, 2, 5), shift());
    assert_eq!(sheet(&rt).sel(), Some(Block { r0: 1, r1: 2, c0: 4, c1: 5 }));
    assert_eq!(draw::count_words(sheet(&rt)), "4 cells picked · 6 rows");
    // The arrows move it, Shift stretching the far end, ⌘ going to the edge.
    key(&mut rt, Key::Right);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 1, c: 5 })));
    key_with(&mut rt, Key::Down, shift());
    assert_eq!(sheet(&rt).sel(), Some(Block { r0: 1, r1: 2, c0: 5, c1: 5 }));
    key_with(&mut rt, Key::Down, cmd());
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 5, c: 5 })));
    key_with(&mut rt, Key::End, cmd());
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 5, c: 7 })));
    key(&mut rt, Key::Home);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 5, c: 0 })));
    key(&mut rt, Key::Tab);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 5, c: 1 })));
    key_with(&mut rt, Key::Tab, shift());
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 5, c: 0 })));
    // A row number picks whole rows; a header picks its column.
    tap(&mut rt, |rt| row_number_at(rt, 2));
    assert!(sheet(&rt).whole_rows());
    assert_eq!(draw::count_words(sheet(&rt)), "1 row picked · 6 rows", "one row is eight cells");
    press_with(&mut rt, |rt| row_number_at(rt, 3), shift());
    assert_eq!(draw::count_words(sheet(&rt)), "2 rows picked · 6 rows");
    tap(&mut rt, |rt| header_at(rt, 3));
    assert!(sheet(&rt).whole_cols());
    assert_eq!(sheet(&rt).sel(), Some(Block { r0: 0, r1: 5, c0: 3, c1: 3 }));
    // Escape: the block back to where it began, then nothing picked with the keys kept, then
    // it goes on to put the file down.
    key(&mut rt, Key::Escape);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 0, c: 3 })));
    key(&mut rt, Key::Escape);
    assert_eq!(sheet(&rt).pick, None);
    assert!(rt.state.beside().is_some());
    // With nothing picked an arrow picks the first cell again.
    key(&mut rt, Key::Down);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 0, c: 0 })));
    key(&mut rt, Key::Escape);
    key(&mut rt, Key::Escape);
    assert!(rt.state.beside().is_none(), "Escape with nothing picked goes on to the pane");
    done(dir);
}

// F3 G1 G2 G3 G8 O1 O2 O3 N6
#[test]
fn a_cell_typed_over_is_written_back_in_the_files_own_shape() {
    let (mut rt, dir) = open("tbl-type", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 2, 7));
    key(&mut rt, Key::Enter);
    assert!(sheet(&rt).editing.is_some(), "Enter edits");
    typed(&mut rt, "Called back");
    key(&mut rt, Key::Enter);
    assert!(sheet(&rt).editing.is_none());
    assert_eq!(value(&rt, 2, 7), "Called back");
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 3, c: 7 })), "Enter keeps the words and goes down");
    let mtime = sheet(&rt).mtime;
    let sent = written(&mut rt);
    let text = sent["text"].as_str().expect("the file's words");
    assert!(text.contains("Initech,New,Dana,,initech.example,,,Called back\n"));
    assert_eq!(text, edit::write_table(sheet(&rt).table(), &edit::shape_of(LEADS, "leads.csv")));
    assert_eq!((sent["product"].as_str(), sent["src"].as_str(), sent["mtime"].as_f64()), (Some("storefront"), Some("exports/leads.csv"), Some(mtime)));
    assert!(sent.get("thread").is_none(), "beside a conversation the save names no thread");
    // A plain number in a column of amounts is written as the column writes them; ⌘Enter stays.
    let (x, y) = cell_at(&rt, 2, 6);
    press(&mut rt, x, y);
    key(&mut rt, Key::Function(2));
    typed(&mut rt, "7200");
    key_with(&mut rt, Key::Enter, cmd());
    assert_eq!(value(&rt, 2, 6), "$7,200");
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 2, c: 6 })));
    // Escape drops what was typed; Tab keeps it and goes right.
    key(&mut rt, Key::Enter);
    typed(&mut rt, "9");
    key(&mut rt, Key::Escape);
    assert_eq!(value(&rt, 2, 6), "$7,200");
    key(&mut rt, Key::Enter);
    key(&mut rt, Key::Backspace);
    typed(&mut rt, "1");
    key(&mut rt, Key::Tab);
    assert_eq!(value(&rt, 2, 6), "$7,201");
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 2, c: 7 })));
    done(dir);
}

// E6 H1 H2 H3 H4 H5 H6 H7 I1 I2 I3
#[test]
fn a_tag_cell_opens_its_list_with_one_press_and_a_date_cell_its_calendar() {
    let (mut rt, dir) = open("tbl-tags", LEADS);
    let (x, y) = cell_at(&rt, 2, 1);
    press(&mut rt, x, y);
    assert!(matches!(sheet(&rt).editing.as_ref().and_then(|e| e.pop.as_ref()), Some(Pop::Tag(_))), "one press opens a tag cell's list");
    let column = sheet(&rt).table().columns[1].clone();
    assert_eq!(draw::tag_items(&column, "New", ""), [draw::Item::Tag("Meeting".into()), draw::Item::Tag("Replied".into()), draw::Item::Tag("New".into()), draw::Item::Tag("Contacted".into()), draw::Item::Tag("Lost".into()), draw::Item::Clear]);
    // The cell's own tag is lit; ↓ and Enter take the next one.
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Enter);
    assert_eq!(value(&rt, 2, 1), "Contacted");
    // A tag typed that the column has not had is added, in a colour none of its tags has.
    press(&mut rt, x, y);
    typed(&mut rt, "Paused later");
    key(&mut rt, Key::Enter);
    assert_eq!(value(&rt, 2, 1), "Paused later");
    let col = &sheet(&rt).table().columns[1];
    assert!(col.values.contains(&"Paused later".to_string()));
    assert_eq!(col.tone("Paused later"), Tone::Amber, "the colour its word means");
    // Its page: renamed in every cell that holds it, then deleted from them all.
    press(&mut rt, x, y);
    rt.update(|s, _| draw_open_page(s, "Meeting"));
    rt.frame(1.0 / 60.0);
    typed(&mut rt, "Demo booked");
    key(&mut rt, Key::Enter);
    assert_eq!((value(&rt, 0, 1), value(&rt, 5, 1)), ("Demo booked".to_string(), "Demo booked".to_string()));
    key(&mut rt, Key::Escape);
    assert!(sheet(&rt).editing.is_none(), "Escape closes the list");
    super::drop_tag(&mut rt.state, Where::Pane, 1, "demo booked");
    assert_eq!((value(&rt, 0, 1), value(&rt, 5, 1)), (String::new(), String::new()));
    // A date cell's calendar: what is typed is a day written as the column writes days.
    let (x, y) = cell_at(&rt, 2, 3);
    press(&mut rt, x, y);
    assert!(matches!(sheet(&rt).editing.as_ref().and_then(|e| e.pop.as_ref()), Some(Pop::Date(_))));
    typed(&mut rt, "nov 30, 2026");
    key(&mut rt, Key::Enter);
    assert_eq!(value(&rt, 2, 3), "2026-11-30");
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 3, c: 3 })), "Enter goes down");
    // ↓ moves the typed day by one.
    let (x, y) = cell_at(&rt, 2, 3);
    press(&mut rt, x, y);
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Enter);
    assert_eq!(value(&rt, 2, 3), "2026-12-01");
    done(dir);
}

/// Opens the page of a tag in the list open on the cell being edited.
fn draw_open_page(s: &mut State, tag: &str) {
    if let Some(Pop::Tag(t)) = s.tables.pane.as_mut().and_then(|sh| sh.editing.as_mut()).and_then(|e| e.pop.as_mut()) {
        // As its ⋯ opens it: the name selected whole.
        let mut name = TextEdit::new(tag);
        name.anchor = 0;
        name.focus = name.text.len();
        name.focused = true;
        t.find.focused = false;
        t.page = Some((tag.to_string(), name));
    }
}

// B19 B20 C3 C4 C5 C6 C7 C8 C9 L2 N6 N7
#[test]
fn the_filter_and_the_sort_narrow_and_order_the_rows_and_are_kept_for_the_file() {
    let (mut rt, dir) = open("tbl-filter", LEADS);
    // The Filter menu offers the tag columns; a tick narrows the rows and the menu stays open.
    let (fx, fy) = rect_of(&rt, "tbl-filter-pane").map(|[x, y, w, h]| (x + w / 2.0, y + h / 2.0)).expect("Filter in the bar");
    press(&mut rt, fx, fy);
    assert_eq!(sheet(&rt).bar, Some(Bar::Filter));
    super::toggle_filter(&mut rt.state, Where::Pane, 1, "Meeting");
    rt.frame(1.0 / 60.0);
    assert_eq!(sheet(&rt).order().len(), 2);
    assert_eq!(draw::count_words(sheet(&rt)), "2 of 6 rows");
    assert_eq!(sheet(&rt).bar, Some(Bar::Filter), "ticking leaves the menu open");
    key(&mut rt, Key::Escape);
    assert_eq!(sheet(&rt).bar, None, "Escape closes the menu");
    // Search narrows further, any case, and Escape takes the words away first.
    rt.update(|s, _| {
        if let Some(sh) = s.tables.pane.as_mut() {
            sh.search = TextEdit::new("VANDELAY");
        }
        super::searched(s, Where::Pane);
    });
    assert_eq!(sheet(&rt).order().len(), 1);
    key(&mut rt, Key::Escape);
    assert_eq!(sheet(&rt).search.text, "");
    assert!(rt.state.beside().is_some(), "Escape cleared the search, and put nothing down");
    super::clear_filters(&mut rt.state, Where::Pane);
    // A header's menu sorts; the sort and the filter are kept for the file by column name.
    super::set_sort(&mut rt.state, Where::Pane, Some(Sort { col: 6, up: false }));
    assert_eq!(value(&rt, 0, 0), "Vandelay", "the largest amount first");
    assert_eq!(value(&rt, 5, 0), "Initech", "and the empty one last either way");
    super::sort_by(&mut rt.state, Where::Pane, 6);
    assert_eq!(sheet(&rt).sort, None, "the column sorted downward turns off");
    super::sort_by(&mut rt.state, Where::Pane, 0);
    super::toggle_filter(&mut rt.state, Where::Pane, 2, "Sam");
    let kept = rt.state.picks.get().tables.get("storefront/exports/leads.csv").cloned().expect("kept for the file");
    assert_eq!(kept.sort, Some(KeptSort { name: "Company".into(), dir: "asc".into() }));
    assert_eq!(kept.picked.get("Owner"), Some(&vec!["Sam".to_string()]));
    done(dir);
}

// F8 N1 F5 O1
#[test]
fn undo_and_redo_walk_the_changes() {
    let (mut rt, dir) = open("tbl-undo", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 0, 7));
    key(&mut rt, Key::Backspace);
    assert_eq!(value(&rt, 0, 7), "");
    key_with(&mut rt, Key::Command('z'), cmd());
    assert!(value(&rt, 0, 7).starts_with("Asked for a demo"));
    key_with(&mut rt, Key::Command('z'), Mods { shift: true, ..cmd() });
    assert_eq!(value(&rt, 0, 7), "");
    key_with(&mut rt, Key::Command('z'), cmd());
    // Undo is written back too.
    let sent = written(&mut rt);
    assert_eq!(sent["text"].as_str(), Some(LEADS));
    done(dir);
}

// F9 F10 F11 G7
#[test]
fn copy_cut_paste_and_fill_down_speak_the_spreadsheet_clipboard() {
    let (mut rt, dir) = open("tbl-clip", LEADS);
    let clip = rune::clipboard::memory();
    rt.use_clipboard(&clip);
    tap(&mut rt, |rt| cell_at(rt, 0, 0));
    press_with(&mut rt, |rt| cell_at(rt, 1, 1), shift());
    key_with(&mut rt, Key::Command('c'), cmd());
    assert_eq!(clip.text().as_deref().map(String::as_str), Some("Acme Robotics\tMeeting\nGlobex\tReplied"));
    // Pasted at the foot, rows are added for it, and the pasted block is picked.
    tap(&mut rt, |rt| cell_at(rt, 5, 0));
    key_with(&mut rt, Key::Command('v'), cmd());
    rt.frame(1.0 / 60.0);
    assert_eq!(sheet(&rt).order().len(), 7);
    assert_eq!((value(&rt, 5, 0), value(&rt, 6, 1)), ("Acme Robotics".to_string(), "Replied".to_string()));
    assert_eq!(sheet(&rt).sel(), Some(Block { r0: 5, r1: 6, c0: 0, c1: 1 }));
    // Fill down: the top row of the block into the rows under it.
    tap(&mut rt, |rt| cell_at(rt, 0, 5));
    press_with(&mut rt, |rt| cell_at(rt, 2, 5), shift());
    key_with(&mut rt, Key::Command('d'), cmd());
    let first = "dana@acme-robotics.example".to_string();
    assert_eq!((value(&rt, 1, 5), value(&rt, 2, 5)), (first.clone(), first.clone()));
    // ⌘X copies and empties; ⌘A picks every cell.
    key_with(&mut rt, Key::Command('x'), cmd());
    assert_eq!(clip.text().as_deref().map(String::as_str), Some(format!("{first}\n{first}\n{first}").as_str()));
    assert_eq!(value(&rt, 1, 5), "");
    key_with(&mut rt, Key::Command('a'), cmd());
    assert_eq!(sheet(&rt).sel(), Some(Block { r0: 0, r1: 6, c0: 0, c1: 7 }));
    done(dir);
}

// D19 E3 F3 F12 M3 N2 N3 L3
#[test]
fn rows_and_columns_are_added_moved_and_taken_away() {
    let (mut rt, dir) = open("tbl-rows", LEADS);
    // Shift Enter puts a row in below and edits its first cell.
    tap(&mut rt, |rt| cell_at(rt, 0, 0));
    key_with(&mut rt, Key::Enter, shift());
    assert_eq!(sheet(&rt).order().len(), 7);
    assert!(sheet(&rt).editing.as_ref().is_some_and(|e| (e.r, e.c) == (1, 0)));
    typed(&mut rt, "Wayne Fabrics");
    key(&mut rt, Key::Enter);
    assert_eq!(value(&rt, 1, 0), "Wayne Fabrics");
    // ⌥↓ moves whole rows picked; the menu's Delete row takes them away.
    tap(&mut rt, |rt| row_number_at(rt, 1));
    key_with(&mut rt, Key::Down, alt());
    assert_eq!(value(&rt, 2, 0), "Wayne Fabrics");
    assert_eq!(sheet(&rt).sel().map(|b| b.r0), Some(2));
    super::drop_picked(&mut rt.state, Where::Pane);
    assert_eq!(sheet(&rt).order().len(), 6);
    assert_eq!(value(&rt, 2, 0), "Initech", "the pick stays on the row that comes into its place");
    // New row at the foot.
    rt.update(|s, tx| super::add_row_at(s, Where::Pane, 6, false, tx));
    assert_eq!(sheet(&rt).order().len(), 7);
    // A column inserted is named and renamed at once; moved, and taken away.
    tap(&mut rt, |rt| header_at(rt, 2));
    rt.update(|s, tx| {
        if let Some(sh) = s.tables.pane.as_mut() {
            sh.float = Some(Float { x: 0.0, y: 0.0, col: Some(2), page_type: false, lit: None });
        }
        super::draw::press_row(s, Where::Pane, super::draw::Act::InsertRight, tx)
    });
    assert_eq!(sheet(&rt).renaming, Some(3));
    assert_eq!(sheet(&rt).table().columns[3].name, "Column 9");
    rt.update(|s, tx| {
        if let Some(sh) = s.tables.pane.as_mut() {
            sh.rename = TextEdit::new("Region");
        }
        super::finish_rename(s, Where::Pane, true, tx);
    });
    assert_eq!(sheet(&rt).table().columns[3].name, "Region");
    tap(&mut rt, |rt| header_at(rt, 3));
    key_with(&mut rt, Key::Left, alt());
    assert_eq!(sheet(&rt).table().columns[2].name, "Region");
    rt.update(|s, tx| {
        if let Some(sh) = s.tables.pane.as_mut() {
            sh.float = Some(Float { x: 0.0, y: 0.0, col: Some(2), page_type: false, lit: None });
        }
        super::draw::press_row(s, Where::Pane, super::draw::Act::DeleteColumn, tx);
    });
    assert_eq!(sheet(&rt).table().columns.len(), 8);
    let sent = written(&mut rt);
    assert!(sent["text"].as_str().is_some_and(|t| t.starts_with("Company,Status,Owner,Next call,")));
    done(dir);
}

// L1 L2 L3 L5 M2 M3
#[test]
fn a_header_and_a_cell_open_their_menus_and_the_keys_walk_them() {
    let (mut rt, dir) = open("tbl-menus", LEADS);
    // A right press on a cell picks it and opens the cell's menu.
    let (x, y) = cell_at(&rt, 1, 4);
    rt.pointer_move(x, y);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(1.0 / 60.0);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 1, c: 4 })));
    let rows: Vec<String> = draw::float_rows(&rt.state, Where::Pane).iter().filter(|l| !l.sep).map(|l| l.words.clone()).collect();
    assert_eq!(rows, ["Open row", "Edit cell", "Copy", "Paste", "Open link", "Insert row above", "Insert row below", "Move up", "Move down", "Clear", "Delete row"]);
    // ↓ walks it, Enter presses the row reached.
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Enter);
    assert!(sheet(&rt).float.is_none());
    assert!(sheet(&rt).editing.is_some(), "Edit cell");
    key(&mut rt, Key::Escape);
    // A right press on a header opens its column's menu: the sort rows and the rest.
    let (x, y) = header_at(&rt, 6);
    rt.pointer_move(x, y);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(1.0 / 60.0);
    let rows: Vec<String> = draw::float_rows(&rt.state, Where::Pane).iter().filter(|l| !l.sep).map(|l| l.words.clone()).collect();
    assert_eq!(rows, ["Sort 1 to 9", "Sort 9 to 1", "Rename", "Type", "Insert column left", "Insert column right", "Move left", "Move right", "Fit to its contents", "Delete column"]);
    // Type opens the types, the column's ticked; Text turns the amounts into words.
    rt.update(|s, tx| super::draw::press_row(s, Where::Pane, super::draw::Act::TypePage, tx));
    let types: Vec<(String, bool)> = draw::float_rows(&rt.state, Where::Pane).iter().skip(1).map(|l| (l.words.clone(), l.ticked)).collect();
    assert_eq!(types.iter().find(|t| t.1).map(|t| t.0.as_str()), Some("Number"));
    rt.update(|s, tx| super::draw::press_row(s, Where::Pane, super::draw::Act::SetType(Kind::Text), tx));
    assert_eq!(sheet(&rt).table().columns[6].kind, Kind::Text);
    assert_eq!(rt.state.picks.get().tables.get("storefront/exports/leads.csv").and_then(|k| k.looks.kinds.get("ARR")), Some(&Kind::Text), "the type is kept for the file");
    // Escape closes a menu.
    rt.pointer_move(x, y);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(1.0 / 60.0);
    key(&mut rt, Key::Escape);
    assert!(sheet(&rt).float.is_none());
    done(dir);
}

// O4 O5 O7
#[test]
fn a_save_the_app_refuses_says_so_and_save_mine_writes_it_anyway() {
    let (mut rt, dir) = open("tbl-stale", LEADS);
    rt.state.tables.answer = Some(serde_json::json!({ "ok": false, "stale": true, "error": "Something else changed this file while you were typing, so nothing was saved." }));
    tap(&mut rt, |rt| cell_at(rt, 0, 7));
    key(&mut rt, Key::Delete);
    written(&mut rt);
    rt.frame(1.0 / 60.0);
    assert_eq!(sheet(&rt).problem, Some(Problem { error: "This file changed on disk since it opened, so your edit is not saved yet.".into(), stale: true }));
    rt.state.tables.answer = None;
    super::flush(&mut rt.state, Where::Pane, true);
    run(&mut rt, 0.1);
    let sent = rt.state.tables.wrote.last().cloned().expect("written anyway");
    assert_eq!(sent["mtime"].as_f64(), Some(0.0), "Save mine carries no time, so the app writes it");
    assert_eq!(sheet(&rt).problem, None);
    // Another refusal says the app's own words, with Try again.
    rt.state.tables.answer = Some(serde_json::json!({ "ok": false, "error": "That is not a document this pane can open." }));
    key(&mut rt, Key::Delete);
    tap(&mut rt, |rt| cell_at(rt, 1, 7));
    key(&mut rt, Key::Delete);
    written(&mut rt);
    rt.frame(1.0 / 60.0);
    assert_eq!(sheet(&rt).problem.as_ref().map(|p| (p.error.as_str(), p.stale)), Some(("That is not a document this pane can open.", false)));
    done(dir);
}

// O6
#[test]
fn an_edit_not_yet_written_is_written_when_the_sheet_is_put_down() {
    let (mut rt, dir) = open("tbl-leave", LEADS);
    rt.state.tables.answer = Some(serde_json::json!({ "ok": false, "error": "No room left on the disk." }));
    tap(&mut rt, |rt| cell_at(rt, 0, 7));
    key(&mut rt, Key::Delete);
    assert!(rt.state.tables.wrote.is_empty(), "not before its 400 ms");
    rt.update(|s, _| s.close_beside());
    run(&mut rt, 0.1);
    assert_eq!(rt.state.tables.wrote.len(), 1, "written as it was put down");
    assert_eq!(rt.state.toast.as_ref().map(|t| t.line.as_str()), Some("Your last edit to leads.csv was not saved. No room left on the disk."));
    done(dir);
}

// F4 K1 K3 K4 E10
#[test]
fn space_opens_the_row_whole_and_its_keys_walk_the_rows() {
    let (mut rt, dir) = open("tbl-row", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 1, 4));
    key(&mut rt, Key::Text(" ".into()));
    assert_eq!(sheet(&rt).sheet, Some(1));
    assert!(rect_of(&rt, "tbl-sheet-pane").is_some(), "drawn at the table's right");
    key(&mut rt, Key::Down);
    assert_eq!(sheet(&rt).sheet, Some(2));
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 2, c: 4 })), "the pick follows");
    key(&mut rt, Key::Escape);
    assert_eq!(sheet(&rt).sheet, None);
    // A double press on a row number opens it too.
    let (x, y) = row_number_at(&rt, 4);
    press(&mut rt, x, y);
    press(&mut rt, x, y);
    assert_eq!(sheet(&rt).sheet, Some(4));
    done(dir);
}

// A4 G5 G6 L4 M2 M3 D19 F5: a sheet that cannot be changed is read, and offers nothing that
// changes it.
#[test]
fn a_sheet_that_cannot_be_changed_is_read_and_never_written() {
    let (mut rt, dir) = open("tbl-read-only", LEADS);
    assert!(rect_of(&rt, "tbl-add-pane").is_some(), "New row, where the sheet can be changed");
    rt.update(|s, _| {
        if let Some(sh) = s.tables.pane.as_mut() {
            sh.editable = false;
        }
    });
    run(&mut rt, 0.05);
    assert!(rect_of(&rt, "tbl-add-pane").is_none(), "no New row");
    // Enter opens the cell's words read only.
    tap(&mut rt, |rt| cell_at(rt, 1, 7));
    key(&mut rt, Key::Enter);
    assert!(sheet(&rt).editing.is_some(), "Read cell");
    key(&mut rt, Key::Escape);
    // A tag cell never opens, and Delete empties nothing.
    tap(&mut rt, |rt| cell_at(rt, 1, 1));
    key(&mut rt, Key::Enter);
    assert!(sheet(&rt).editing.is_none());
    key(&mut rt, Key::Delete);
    assert_eq!(value(&rt, 1, 1), "Replied");
    // The menus hold only what reads.
    let (x, y) = cell_at(&rt, 1, 4);
    rt.pointer_move(x, y);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(1.0 / 60.0);
    let rows: Vec<String> = draw::float_rows(&rt.state, Where::Pane).iter().filter(|l| !l.sep).map(|l| l.words.clone()).collect();
    assert_eq!(rows, ["Open row", "Read cell", "Copy", "Open link"]);
    key(&mut rt, Key::Escape);
    let (x, y) = header_at(&rt, 7);
    rt.pointer_move(x, y);
    rt.secondary_down(x, y);
    rt.secondary_up(x, y);
    rt.frame(1.0 / 60.0);
    let rows: Vec<String> = draw::float_rows(&rt.state, Where::Pane).iter().filter(|l| !l.sep).map(|l| l.words.clone()).collect();
    assert_eq!(rows, ["Sort A to Z", "Sort Z to A", "Fit to its contents", "Wrap text"]);
    key(&mut rt, Key::Escape);
    run(&mut rt, 0.5);
    assert!(rt.state.tables.wrote.is_empty(), "nothing is written");
    done(dir);
}

// E7 E11: a double press on a cell edits it; a press elsewhere on the table ends the edit,
// keeping the words, and picks what it pressed.
#[test]
fn a_double_press_edits_and_a_press_elsewhere_keeps_the_words() {
    let (mut rt, dir) = open("tbl-double", LEADS);
    let (x, y) = cell_at(&rt, 2, 0);
    press(&mut rt, x, y);
    press(&mut rt, x, y);
    assert!(sheet(&rt).editing.is_some(), "a double press edits");
    typed(&mut rt, " Inc");
    tap(&mut rt, |rt| cell_at(rt, 4, 7));
    assert!(sheet(&rt).editing.is_none());
    assert_eq!(value(&rt, 2, 0), "Initech Inc");
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 4, c: 7 })));
    done(dir);
}

// G4: rows copied from a spreadsheet, pasted into an empty cell's box, go across the table.
#[test]
fn rows_pasted_into_an_empty_box_go_across_the_table() {
    let (mut rt, dir) = open("tbl-paste-box", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 2, 5));
    key(&mut rt, Key::Enter);
    assert!(sheet(&rt).editing.is_some());
    rt.paste("ines@initech.example\t$900\nari@umbrella.example\t$3,150\n");
    rt.frame(1.0 / 60.0);
    assert!(sheet(&rt).editing.is_none(), "the box closes");
    assert_eq!([value(&rt, 2, 5), value(&rt, 2, 6), value(&rt, 3, 5)], ["ines@initech.example", "$900", "ari@umbrella.example"]);
    // Into a box that holds words, a paste is the box's own.
    tap(&mut rt, |rt| cell_at(rt, 0, 0));
    key(&mut rt, Key::Enter);
    rt.paste("a\tb");
    rt.frame(1.0 / 60.0);
    assert!(sheet(&rt).editing.as_ref().is_some_and(|e| e.draft.text == "Acme Roboticsa\tb"));
    done(dir);
}

// F7: a letter is the app's, not the cell's.
#[test]
fn a_letter_goes_on_to_the_conversation() {
    let (mut rt, dir) = open("tbl-letter", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 1, 0));
    key(&mut rt, Key::Text("r".into()));
    assert!(rt.state.replying, "R opens the reply box");
    assert_eq!(value(&rt, 1, 0), "Globex", "and types nothing into the cell");
    done(dir);
}

// J1
#[test]
fn a_cut_off_cell_picked_shows_its_words_whole_until_escape() {
    let (mut rt, dir) = open("tbl-peek", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 0, 7));
    assert!(draw::peek(&rt.state, sheet(&rt)).is_some_and(|p| p.text.ends_with("end of the month")));
    assert!(rect_of(&rt, "tbl-peek-pane").is_some(), "drawn in the frame the press drew");
    key(&mut rt, Key::Escape);
    assert!(rect_of(&rt, "tbl-peek-pane").is_none(), "put away");
    assert!(sheet(&rt).pick.is_some(), "the cell still picked");
    // A short cell shows nothing more.
    tap(&mut rt, |rt| cell_at(rt, 1, 0));
    assert!(draw::peek(&rt.state, sheet(&rt)).is_none());
    // E13: a press on the words shown whole goes to the cell under it.
    tap(&mut rt, |rt| cell_at(rt, 0, 7));
    let [x, y, _, h] = rect_of(&rt, "tbl-peek-pane").expect("shown again for another pick");
    let (_, below) = cell_at(&rt, 1, 7);
    assert!(y + h > below - 20.0, "the words reach over the row below");
    press(&mut rt, x + 20.0, y + h - 4.0);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 1, c: 7 })));
    done(dir);
}

// A3 C9 D21: big, from the pane's corners, the same table.
#[test]
fn a_sheet_opened_big_is_the_full_table() {
    let (mut rt, dir) = open("tbl-big", LEADS);
    rt.update(|s, _| s.pane_full());
    run(&mut rt, 0.1);
    let big = rt.state.tables.big.as_ref().expect("the sheet open big");
    assert_eq!(draw::count_words(big), "6 rows · 8 columns");
    assert!(rect_of(&rt, "tbl-scroll-big").is_some());
    let [x, y, _, _] = rect_of(&rt, "tbl-scroll-big").expect("the grid");
    press(&mut rt, x + 100.0, y + HEAD_H + 46.0 * 1.5);
    assert_eq!(rt.state.tables.big.as_ref().and_then(|b| b.pick).map(|p| p.a), Some(At { r: 1, c: 0 }));
    key(&mut rt, Key::Delete);
    run(&mut rt, 0.5);
    assert_eq!(rt.state.tables.wrote.last().and_then(|w| w["thread"].as_str()), Some("w-feel0"), "big, the save names the thread");
    done(dir);
}

// A6: read again when an agent writes the file, the pick let go.
#[test]
fn a_sheet_rewritten_on_disk_is_read_again() {
    let (mut rt, dir) = open("tbl-follow", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 1, 0));
    let file = dir.join("exports/leads.csv");
    std::fs::write(&file, LEADS.replace("Globex", "Globex West")).expect("rewritten");
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
    std::fs::File::options().write(true).open(&file).and_then(|f| f.set_modified(later)).expect("a newer time");
    rt.update(|s, _| s.reread_beside());
    run(&mut rt, 0.1);
    assert_eq!(value(&rt, 1, 0), "Globex West");
    assert_eq!(sheet(&rt).pick, None);
    done(dir);
}

// A7 O4: refused because the file moved on, Load the new one reads it again, the edit dropped.
#[test]
fn load_the_new_one_reads_the_file_again() {
    let (mut rt, dir) = open("tbl-load", LEADS);
    rt.state.tables.answer = Some(serde_json::json!({ "ok": false, "stale": true, "error": "changed" }));
    tap(&mut rt, |rt| cell_at(rt, 1, 7));
    key(&mut rt, Key::Delete);
    written(&mut rt);
    assert!(sheet(&rt).problem.as_ref().is_some_and(|p| p.stale));
    let file = dir.join("exports/leads.csv");
    std::fs::write(&file, LEADS.replace("Globex", "Globex West")).expect("rewritten");
    rt.state.tables.answer = None;
    rt.update(|s, _| super::reload(s, Where::Pane));
    run(&mut rt, 0.1);
    assert_eq!(value(&rt, 1, 0), "Globex West");
    assert_eq!(value(&rt, 1, 7), "Replied to the second note", "the edit refused is dropped");
    assert_eq!(sheet(&rt).problem, None);
    done(dir);
}

// A8 D16: a view with no rows says why.
#[test]
fn a_view_with_no_rows_says_why() {
    let shows = |rt: &Runtime<State>, words: &str| rt.elements().iter().any(|e| e.text.as_deref() == Some(words));
    let (mut rt, dir) = open("tbl-none", LEADS);
    rt.update(|s, _| {
        if let Some(sh) = s.tables.pane.as_mut() {
            sh.search = TextEdit::new("nothing like this");
        }
        super::searched(s, Where::Pane);
    });
    run(&mut rt, 0.05);
    assert!(shows(&rt, "No rows match."));
    done(dir);
    for (name, csv) in [("tbl-headed", "Name,Status\n"), ("tbl-blank", "")] {
        let (rt, dir) = open(name, csv);
        assert!(shows(&rt, "This table has no rows yet."), "{name}");
        done(dir);
    }
}

// D3 D2 B5: a ranked file's rank is the row number, and never edited.
#[test]
fn a_ranked_file_shows_its_rank_as_the_row_number() {
    let (mut rt, dir) = open("tbl-ranked", "#,Name,Fit\n1,Ada,High\n2,Bo,Low\n3,Cy,High\n");
    assert!(sheet(&rt).table().ranked);
    assert_eq!(sheet(&rt).first(), 1);
    tap(&mut rt, |rt| cell_at(rt, 0, 1));
    key(&mut rt, Key::Left);
    assert_eq!(sheet(&rt).pick, Some(Pick::one(At { r: 0, c: 1 })), "never onto the rank");
    // A row put in keeps it counting.
    key_with(&mut rt, Key::Enter, shift());
    key(&mut rt, Key::Escape);
    assert_eq!(sheet(&rt).table().rows.iter().map(|r| r.at(0).to_string()).collect::<Vec<_>>(), ["1", "2", "3", "4"]);
    done(dir);
}

/// A drag from `from` to `to`, in steps, as a hand drags.
fn drag(rt: &mut Runtime<State>, from: (f32, f32), to: (f32, f32)) {
    rt.pointer_move(from.0, from.1);
    rt.pointer_down(from.0, from.1);
    for k in 1..=8 {
        let f = k as f32 / 8.0;
        rt.pointer_move(from.0 + (to.0 - from.0) * f, from.1 + (to.1 - from.1) * f);
        rt.frame(1.0 / 60.0);
    }
    rt.pointer_up(to.0, to.1);
    rt.frame(1.0 / 60.0);
}

// E3 D20 N4: rows picked whole are picked up and moved by a drag, a line showing where.
#[test]
fn rows_picked_whole_are_dragged_to_where_they_land() {
    let (mut rt, dir) = open("tbl-drag", LEADS);
    tap(&mut rt, |rt| row_number_at(rt, 0));
    let from = row_number_at(&rt, 0);
    let to = row_number_at(&rt, 3);
    rt.pointer_move(from.0, from.1);
    rt.pointer_down(from.0, from.1);
    for k in 1..=8 {
        rt.pointer_move(from.0, from.1 + (to.1 + 10.0 - from.1) * k as f32 / 8.0);
        rt.frame(1.0 / 60.0);
    }
    assert!(sheet(&rt).dropping.is_some_and(|d| !d.cols && d.place == 4), "the line shows where they land");
    rt.pointer_up(to.0, to.1 + 10.0);
    rt.frame(1.0 / 60.0);
    assert_eq!((value(&rt, 0, 0), value(&rt, 3, 0)), ("Globex".to_string(), "Acme Robotics".to_string()));
    assert!(sheet(&rt).whole_rows() && sheet(&rt).sel().map(|b| b.r0) == Some(3), "and stay picked where they landed");
    done(dir);
}

// E5: a drag near the table's foot scrolls it 18 points a move, the block stretching with it.
#[test]
fn a_drag_near_the_foot_scrolls_the_table() {
    let long: String = std::iter::once("Name,Note\n".to_string()).chain((0..80).map(|i| format!("Lead {i},note {i}\n"))).collect();
    let (mut rt, dir) = open("tbl-drag-edge", &long);
    let from = cell_at(&rt, 0, 1);
    let [_, vy, _, vh] = rect_of(&rt, "tbl-scroll-pane").expect("the grid");
    rt.pointer_move(from.0, from.1);
    rt.pointer_down(from.0, from.1);
    for k in 0..6 {
        rt.pointer_move(from.0 + (k % 2) as f32, vy + vh - 6.0);
        rt.frame(1.0 / 60.0);
    }
    let down = scrolled(Where::Pane).offset[1];
    rt.pointer_up(from.0, vy + vh - 6.0);
    rt.frame(1.0 / 60.0);
    assert!(down >= 18.0 * 4.0, "scrolled {down}");
    assert!(sheet(&rt).sel().is_some_and(|b| b.r0 == 0 && b.r1 > 12), "the block reaches the rows scrolled into view");
    done(dir);
}

// E4: columns picked whole are picked up and moved by a drag the same way.
#[test]
fn columns_picked_whole_are_dragged_to_where_they_land() {
    let (mut rt, dir) = open("tbl-drag-cols", LEADS);
    tap(&mut rt, |rt| header_at(rt, 2));
    assert!(sheet(&rt).whole_cols(), "a header picks its column");
    let from = header_at(&rt, 2);
    let [vx, _, _, _] = rect_of(&rt, "tbl-scroll-pane").expect("the grid");
    let (x4, w4) = draw::column_x(&rt.state, sheet(&rt), 4);
    let to = (vx + x4 + w4 - 10.0, from.1);
    rt.pointer_move(from.0, from.1);
    rt.pointer_down(from.0, from.1);
    for k in 1..=8 {
        rt.pointer_move(from.0 + (to.0 - from.0) * k as f32 / 8.0, from.1);
        rt.frame(1.0 / 60.0);
    }
    assert!(sheet(&rt).dropping.is_some_and(|d| d.cols && d.place == 5), "the line shows where it lands");
    rt.pointer_up(to.0, to.1);
    rt.frame(1.0 / 60.0);
    let names: Vec<&str> = sheet(&rt).table().columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names[..5], ["Company", "Status", "Next call", "Website", "Owner"]);
    assert!(sheet(&rt).whole_cols() && sheet(&rt).sel().map(|b| b.c0) == Some(4), "and stays picked where it landed");
    done(dir);
}

// E12 N7: a header's grip widens the column, kept for the file; a double press fits it.
#[test]
fn a_headers_grip_widens_its_column_and_a_double_press_fits_it() {
    let (mut rt, dir) = open("tbl-grip", LEADS);
    let [vx, vy, _, _] = rect_of(&rt, "tbl-scroll-pane").expect("the grid");
    let (x, w) = draw::column_x(&rt.state, sheet(&rt), 1);
    let edge = (vx + x + w - 1.0, vy + HEAD_H / 2.0);
    drag(&mut rt, edge, (edge.0 + 60.0, edge.1));
    let wide = sheet(&rt).sized.get("Status").copied().expect("a width the hand set");
    assert!((wide - (w + 60.0)).abs() < 1.5, "{wide} against {}", w + 60.0);
    assert_eq!(rt.state.picks.get().tables.get("storefront/exports/leads.csv").and_then(|k| k.widths.get("Status")).copied(), Some(wide));
    let (x, w) = draw::column_x(&rt.state, sheet(&rt), 1);
    let edge = (vx + x + w - 1.0, vy + HEAD_H / 2.0);
    press(&mut rt, edge.0, edge.1);
    press(&mut rt, edge.0, edge.1);
    let fitted = sheet(&rt).sized.get("Status").copied().expect("fitted");
    assert!(fitted < wide, "fitted to what it holds: {fitted}");
    done(dir);
}

// D13 L3: Wrap text shows three lines a cell, every row taller, and is kept for the file.
#[test]
fn a_column_set_to_wrap_makes_every_row_three_lines_tall() {
    let (mut rt, dir) = open("tbl-wrap", LEADS);
    assert_eq!(sheet(&rt).row_h(), ROW_H);
    super::toggle_wrap(&mut rt.state, Where::Pane, 7);
    rt.frame(1.0 / 60.0);
    assert_eq!(sheet(&rt).row_h(), ROW_WRAPPED);
    assert_eq!(rt.state.picks.get().tables.get("storefront/exports/leads.csv").and_then(|k| k.looks.wrap.get("Note")).copied(), Some(true));
    super::toggle_wrap(&mut rt.state, Where::Pane, 7);
    assert_eq!(sheet(&rt).row_h(), ROW_H);
    done(dir);
}

// D15 D2: only the rows in view are built, however long the sheet; a thousand rows and more
// widen the row numbers.
#[test]
fn a_long_sheet_builds_only_the_rows_in_view() {
    let long: String = std::iter::once("Name,Status\n".to_string()).chain((0..5000).map(|i| format!("Lead {i},{}\n", if i % 3 == 0 { "New" } else { "Won" }))).collect();
    let (rt, dir) = open("tbl-long", &long);
    assert_eq!(sheet(&rt).order().len(), 5000);
    assert_eq!(sheet(&rt).rn_w(), RN_W_WIDE);
    let built = rt.elements().into_iter().filter(|e| e.path.ends_with(&format!("tbl-rows-pane/{}", e.key.as_deref().unwrap_or("-")))).count();
    assert!(built > 5 && built < 120, "{built} rows built");
    done(dir);
}

// K2: a field of the row opened whole is written when it is left.
#[test]
fn a_field_of_the_row_opened_whole_is_written_when_left() {
    let (mut rt, dir) = open("tbl-field", LEADS);
    tap(&mut rt, |rt| cell_at(rt, 1, 0));
    key(&mut rt, Key::Text(" ".into()));
    let [x, y, w, h] = rect_of(&rt, "tbl-field7-pane").expect("the note's box");
    press(&mut rt, x + w - 4.0, y + h / 2.0);
    key_with(&mut rt, Key::End, cmd());
    typed(&mut rt, " soon");
    assert_eq!(sheet(&rt).drafts[7].text, "Replied to the second note soon", "the field takes the typing");
    assert_eq!(value(&rt, 1, 7), "Replied to the second note", "nothing is written while the field has the keys");
    key(&mut rt, Key::Escape);
    run(&mut rt, 0.05);
    assert_eq!(value(&rt, 1, 7), "Replied to the second note", "Escape drops what was typed");
    assert_eq!(sheet(&rt).drafts[7].text, "Replied to the second note");
    let [x, y, w, h] = rect_of(&rt, "tbl-field7-pane").expect("the note's box");
    press(&mut rt, x + w - 4.0, y + h / 2.0);
    key_with(&mut rt, Key::End, cmd());
    typed(&mut rt, " again");
    key_with(&mut rt, Key::Enter, cmd());
    run(&mut rt, 0.05);
    assert_eq!(value(&rt, 1, 7), "Replied to the second note again", "leaving the box writes it");
    // Typed into and left by stepping to the next row: the press on the step takes the keys
    // first, so the field is written before the row below opens with its own words.
    let [x, y, w, h] = rect_of(&rt, "tbl-field7-pane").expect("the note's box");
    press(&mut rt, x + w - 4.0, y + h / 2.0);
    key_with(&mut rt, Key::End, cmd());
    typed(&mut rt, ", twice");
    let [x, y, w, h] = rect_of(&rt, "tbl-row-below-pane").expect("the step down");
    press(&mut rt, x + w / 2.0, y + h / 2.0);
    run(&mut rt, 0.05);
    assert_eq!(value(&rt, 1, 7), "Replied to the second note again, twice");
    assert_eq!(sheet(&rt).sheet, Some(2));
    assert_eq!(sheet(&rt).drafts[7].text, value(&rt, 2, 7), "the row below starts from its own words");
    done(dir);
}
