# The Files place as the web window draws it since 2026-11-23: line by line

The checklist of everything the web window's Files place does, read off its code, and whether
this window does the same. A line is ticked only once it is ported and checked, and how it was
checked is said on the line. A line left open says exactly why.

The web window's Files place changed on 2026-11-22 and 23 (e732e86da "Files opens on a home of
what the team kept, folder by folder", ff8d3d086, 6c6622b26, b109a1092 "Files' folders get their
own pages, a path back, menus and icons a person picks", cb3377e0d, 7deceafc6, 8a49d44f2 "A
change made in Files shows at once", 4696bc8ce "Files opens with Recent on the left, a day at a
time"). It now opens on a home; the page this window drew before is still there behind "Every
file, with sizes and the Trash".

The spec, by file (`R` is `renderer/src`):

- `R/files/FilesHome.tsx` and `R/files/files-home.css`: the home, a folder's page, the files in
  no folder, the search's results, the tiles, the menus.
- `R/files/files-home.ts`: their rules. `R/files/files-changes.ts`: a change drawn over the list
  at once. `R/files/files-undo.ts`: what Z puts back.
- `R/files/FilesPlace.tsx` and `R/files/files-place.css`: the state, the writes, New, and the
  page of every file (`everything`).
- `R/App.tsx`: `filesPage`, `openFiles`, `filesShown`, `skyOnPlace`, its keys, its title.
- The channels, in `preload.cjs` and `main/ipc.mts`: `zero:files-place`, `zero:files-place-open`,
  `zero:files-place-folder` (`do`: make, rename, move, name, icon, remove, restore),
  `zero:files-place-new`, `zero:files-place-upload`, `zero:files-place-trash`,
  `zero:files-place-untrash`, `zero:read-doc`, `zero:files-cloud`. The local door passes every
  one of them (it hands on every handler the app files, `main/main.mts`'s `phoneChannels`).
- The tests: `tests/files-home-shows-what-the-team-kept-and-sets-the-rest-apart.test.mts`,
  `files-recent-shows-what-changed-last-by-day`, `files-folders-have-their-own-pages-and-a-path-back`,
  `a-change-made-in-files-shows-at-once`, `z-undoes-a-folder-rename-or-a-file-move-on-the-files-page`.

Here: `src/files_page.rs` (the state, the writes, the keys, the strip, the page of every file),
`src/files_home.rs` (the home and its pages and menus), `src/rules/files_home.rs` (the rules, with
the web's cases), `src/rules/files_shown.rs` (its list gained `icon`, `trashable`, `movable` and a
folder's `at`), `tools/scenes/files*.mjs`.

Each line: **[x]** done, **[ ]** not done, **[~]** done with a deviation, said why. "Tests" are
`cargo test` (`rules::files_home::tests`, `files_home::tests`); "pictures" are the scenes, at
1280 by 760, 2x, measured by `measure.sh`, every keyed rect compared to the web window's dump.

## A. Opening, the strip, the keys (`App.tsx`)

- [x] A1 The rail's Files opens All files (`openFiles(null)`); the command bar's Go to Files and
  Files in {project} open All files or a project's home (`open_files_place`, as before). Test
  `the_rails_files_opens_the_files_place_…` (feel.rs, rewritten to press the file's Recent row).
- [~] A2 The strip says "Files", 44 into the card, over the sky in its soft patch, as the web
  window draws it with no arrows before it. The arrows themselves are drawn by
  `chrome::back_forward`, which this port leaves alone: the lead takes them off Files there
  (d1f5ef4a8's rule). Until then the native strip shows them over the title. Pictures: strip 0.14.
- [x] A3 The strip's right holds New only on the page of every file (`#fp-head-slot`): the strip's
  button look, its menu 38 under it. Picture `files-everything-new` (rects to 0.01).
- [x] A4 The Inbox's sky is drawn behind Files (as before).
- [x] A5 `/` puts the caret in the search box. Test `slash_puts_the_caret_in_the_search_…`.
- [x] A6 Escape (no box, no menu) leaves Files, from any of its pages. Test
  `escape_with_no_menu_leaves_files`.
- [x] A7 Z undoes on Files; N still starts a thread; other letters do nothing (`keys.rs`, as
  before). Tests of Z below.
- [x] A8 Opening another project resets the folder, `everything`, an open menu and a rename, and
  keeps a folder's name box only for its own project (`useEffect([project])`).

## B. The state and the list (`FilesPlace.tsx`)

- [x] B1 The list is read when the page opens (`soon` on the window's first reading) and again
  as the store changes, at most every two seconds; a write the app took reads it again at once
  (`readAgain`).
- [x] B2 A change is drawn over the list at once and kept until a list asked for after the app
  wrote it arrives; a refused one is taken off and said; a folder made is drawn once the app
  gives its id; a folder put back drops a pending deletion (`Pending`, `with_changes`). Tests
  `a_tiles_menu_moves_it_drawing_the_move_at_once…`, `a_change_the_app_refuses_…`,
  `new_makes_a_document…` (the folder made), rules' `with_changes` cases.
- [x] B3 Which page: a folder's when the id names a folder of a project, the files in no folder
  for `not-in-a-folder`, else the home. Pictures `files-folder`, `files-loose`.
- [x] B4 The path back (`crumbs_for`), each crumb opening its place. Test
  `a_projects_home_has_its_folders_in_sections…`.
- [x] B5 The title: the project, or Files. Pictures.
- [x] B6 `found`: every file under the page the search leaves, in the list's order, check pictures
  and all (`files_page::found`). Picture `files-search`; test.
- [x] B7 Recent: the project's, or every project's on All files, ten. Rules' cases; pictures.
- [x] B8 The project tiles (`shelves`). Rules' case; picture `files-kept`.
- [x] B9 The folders for Move to… are the project's only, none on All files (`files_folders`).
- [x] B10 Nothing empty is said until the list has come (`loaded`).

## C. The home's top (`FilesHome`, `.fh`, `.fh-top`)

- [x] C1 `.fh`: 1120 at most, in the middle, 16 above, 24 below. Rects to 0.01.
- [x] C2 `.fh-crumbs`: crumbs 24 tall, 6 in, corner 6, faint, the hand; under the pointer the
  ink on the wash; the 12-point chevron; -4 above, 6 below, -6 in. Picture `files-hover-crumb`.
- [x] C3 `.fh-title h1`: 30, 600, 36 tall, -0.6, cut with dots. Pictures.
- [x] C4 A folder's page: its name as the heading with its ⋯ (26), shown while the pointer is on
  the title or its menu is open; renaming, a 360 by 38 box of 22 points. Picture `files-folder`;
  test `delete_folder_…` presses it.
- [x] C5 The files in no folder: "Not in a folder". Picture `files-loose`.
- [x] C6 `.fh-find`: 320 by 34, corner 9, the sky's dark at 82% over the blurred sky, the wash
  under the pointer; magnifier 15; 13-point box; the `/` cap. Pictures (rects to 0.01).
- [x] C7 Escape in the box with words empties it and goes no further; with none it does
  nothing, the caret staying in the box (the page takes no key from a box). Test.
- [x] C8 New: 32 tall, 13 in, the strong edge drawn over the sky behind, corner 6, + and "New";
  the strong wash under the pointer and while open. Pictures `files-new`, rects to 0.01.
- [x] C9 New's menu: 180 wide, 40 under New's top, right-aligned; Document, Table, Folder, Upload
  from computer; with no project and two projects of one's own, "New document in" and the
  projects. Pictures `files-new` (0.33), `files-new-all` (0.22); test `new_makes_a_document…`.
- [x] C10 New is not drawn with no project and none of one's own. Picture `files-empty`.
- [~] C11 Document and Table write `zero:files-place-new` `{ project, name: '', kind, folder }`
  and open the file big once the app gives its key; a refusal says the app's words, else "That
  document could not be made.". The web window opens it ready to write in (`typing`): this
  window's view of a file cannot edit a document, so it opens to read. Test.
- [x] C12 Folder: from All files the project picked opens; a name box (`.fp-name-box`, 24 tall,
  the wash and the strong edge) stands as a section; Return or leaving the box makes it
  (`do: 'make'`, the folder as parent); Escape or a blank name gives up. Picture `files-naming`;
  test.
- [~] C13 Upload from computer: ported, part L below, with a channel of the door's own that takes
  the paths (`zero:files-place-upload-here`). One file a pick, and the window's button is Open
  (L4).
- [x] C14 New's menu closes on a press outside and on Escape. Test (Escape); picture.

## D. The home's body

- [x] D1 `.fh-split`: Recent 232 wide, 44 apart, the folders three across; without Recent one
  column four across; under 1100 the two stack, the folders still three across. Pictures at 1280
  and at 1000 wide (`files`, `files-project`: 0.21 whole), and 1600 by 1000 (0.22, 0.15).
- [x] D2 Recent's heading, 38 under the top. Pictures.
- [x] D3 A day's name: Today, Yesterday or "Nov 20", 12.5, faint, 20 tall. Rules' cases; picture
  `files-kept` (three days).
- [x] D4 A Recent row: 32 tall, 10 in, corner 9, its mark (19 in 20) and name (14, 500), the
  wash under the pointer, a press opens the file, its title the name. Picture `files-hover-row`;
  test (feel.rs).
- [x] D5 A section: 38 above; its heading row 28 tall, 14 over the tiles; the name a button
  underlined under the pointer, opening its page; the count; the ⋯ shown while the pointer is on
  the section; See all N at the row's end past six. Pictures `files-project`,
  `files-hover-section`; tests.
- [x] D6 Six tiles at first.
- [x] D7 "A, B and C are empty." Rules' case; picture.
- [x] D8 Not in a folder, its name opening its page. Picture; test.
- [x] D9 "Nothing in {title} yet." Picture `files-project-empty`.
- [x] D10 "Every file, with sizes and the Trash", 30 under, once loaded and not empty. Test.
- [x] D11 All files: "Projects" and a tile per project: the folder mark, the name, what it kept,
  the day of its newest; a press opens it; "Nothing in Files yet." with none. Pictures `files`,
  `files-kept`, `files-empty`; test.
- [x] D12 The search's results: "Matches" and the count, or "No file matches", up to sixty tiles
  four across. Pictures `files-search`, `files-search-none`; test.
- [x] D13 A folder's page and the files in no folder: every file, 22 under the top, sixty at a
  time with Show more; "Nothing in this folder yet." Pictures `files-folder`, `files-loose`.

## E. A tile (`Tile`)

- [x] E1 164 tall, corner 14, 16 18 14 inside, the ink at 4% and an inset `--line`; under the
  pointer or with its menu open the ink at 7%, an inset `--line-strong`, a point higher. Picture
  `files-hover-tile`.
- [x] E2 Its mark: the icon picked, a word of its name, its document's title or its folder's
  name (never Pinned's), else its kind. Rules' cases (every one of the web's).
- [x] E3 A picture shows itself in a 58-point strip, from its middle, unless an icon was picked;
  one that will not load wears its mark. Pictures (total-line-v2.png).
- [x] E4 Its name, 14.5, 600, 12 under the mark (10 under a picture), cut with dots; the accent
  dot on a Pinned file changed this week; the file's own name as its title when another is
  shown. Pictures (Rounding's dot).
- [x] E5 Its line: a document's first sentence, else the thread it came from or its kind, 13
  points 19 tall, two lines (one under a picture), cut as Chromium's `line-clamp` cuts: the last
  line as it wrapped with "…" after it (`clamped`). Pictures.
- [x] E6 Its foot: when, and who ("you" for You), 12, faint. Pictures.
- [x] E7 A press opens the file; a right press opens its menu four past the pointer, read in
  whole points as the page's `clientX` is. Picture `files-context` (rects to 0.01); test.
- [x] E8 Its ⋯: 28, 12 in from the top and right, corner 8, `--surface`, inset strong line, dim;
  shown under the pointer or while open; the strong wash and the ink under the pointer or while
  open; a press opens or closes the menu. Pictures `files-hover-tile`, `files-menu`.
- [~] E9 Dragging a tile or a Recent row onto a section, a folder's page or a crumb moves it
  there: left out. The browser draws the carried tile as its own translucent picture under the
  pointer, which this window cannot draw the same; Move to… does the same move.

## F. The menus

- [x] F1 `.fh-menu`: 210 (252 for icons), 5 in, corner 10, `--surface`, the strong ring and the
  soft shadow; kept 8 inside the window; under its button, a folder's from the button's left, or
  at the pointer; turned upward near the foot. Pictures `files-menu*`, `files-folder-menu`,
  `files-context` (rects to 0.01 or 0); rules' case `a_menu_stands_under_its_button…`.
- [~] F2 A press outside closes it; Escape closes it and goes no further. The web window lets
  the press that closed it also land on what is under it; this window's press only closes it,
  save one case followed since: a press on the ⋯ of the menu that is open opens it again at its
  first rows under the ⋯, as the web window's `mousedown` closes it and its `click` on the ⋯
  opens it (`setMenu(menu ? null : atButton(…))` seeing the menu already gone). Tests, and
  `a_press_on_the_dots_of_an_open_menu_opens_it_again_under_them`.
- [x] F3 A menu row: 30 tall, 8 in, corner 6, 13 points, the strong wash under the pointer, its
  mark faint, the tick at its end; words too long run under its edge and are cut, no ellipsis
  (the button's words are a flex item). Pictures.
- [x] F4 A file's menu: Open, Rename, Change icon…, Move to… (a file in a project), Move to the
  Trash (unless `trashable` is false). Test; picture `files-menu`.
- [x] F5 Rename: the name a box, all of it chosen; Return or leaving it writes `do: 'name'` when
  new; Escape gives up. Picture `files-rename`; test (Return, Escape and leaving).
- [x] F6 Change icon…: "Icon", the 22 marks six across, the one picked outlined; Automatic with
  its mark, ticked when none is picked; a pick writes `do: 'icon'`. Pictures `files-menu-icon`,
  `files-menu-icon-picked`; test.
- [x] F7 Move to…: the project's folders, subfolders 14 in, the file's ticked; Not in a folder;
  New folder… a box at the foot (181 wide, 7 out); a pick writes `do: 'move'`, a name
  `do: 'make'` then the move. Pictures `files-menu-move`, `files-menu-new`; tests.
- [x] F8 Move to the Trash: "Moving … to the Trash…", `zero:files-place-trash`
  `{ keys: [key], older: false }`, then what emptying gives back, and Z writes
  `zero:files-place-untrash` with the app's `undo`. Test.
- [x] F9 A folder's menu: Rename (a 280 by 30 box in a section, 360 by 38 as a page's heading),
  Delete folder (`do: 'remove'`, said with where its files went, Z writes `do: 'restore'`);
  deleting the open folder goes back to the home. Picture `files-folder-menu`; test.
- [x] F10 Z after a move or a rename puts it back, saying "Moved … back", "Renamed … back". Test;
  rules' cases. Z's writes are drawn over the list as the key is pressed (`actNow` on each of
  them), a folder put back taking back a deletion only drawn here, and taken back off if the app
  refuses (`files_page::undoing`, called by `answer.rs` once Z's writes have gone). Test
  `z_draws_what_it_puts_back_at_the_key_and_takes_it_off_if_the_app_refuses`.
- [x] F11 A refusal says the app's words, else "That could not be done." Test.

## G. The rules

- [x] G1 `whenWords`, `homeOf`, `isPinned`, `pinChangedLately`, `firstLine`, `docTop`, `tileName`,
  `tileLine`, `glyphFor`, `markOf`, `recentOf`, `recentByDay`, `crumbsFor`, `folderFiles`, the
  constants. `rules/files_home.rs`, every case of the web's two home tests, the path's and the
  folder page's.
- [x] G2 `withChanges`, `cleanName`: every case of `a-change-made-in-files-shows-at-once`.
- [x] G3 `renameUndo`, `fileRenameUndo`, `moveUndo`: their words and writes.

## H. The page of every file

- [x] H1 What this window drew before as the whole place. Pictures `files-everything` (10.6, the
  page's tile: see the README), `files-list` (0.52), `files-name`.
- [x] H2 Its crumbs go back to the home where they name no folder, the current one included
  (`go`, which the last crumb also runs). Test `the_files_in_no_folder_…`.
- [x] H3 New in the strip. Picture `files-everything-new`.
- [x] H4 A tile's menu (`MoveMenu`), the folder cards' menus, choosing files for the Trash, the
  total's menu: ported, line by line in part K below.

## I. The file opened big from Files

- [ ] I1 The open mark offers the apps the Mac would open the file with (`OpenWith`,
  `zero:open-with` then `zero:open-artifact` with `app`): not ported; this window's open mark
  still hands the file to its default app. The door passes both channels.
- [ ] I2 A file's comments handed to an agent (747da8461): not ported, as the view has no notes.
- [ ] I3 A document made with New opens ready to write in, and a file is renamed from its name in
  the view (`typing`, `onRename`): not ported; the view cannot edit (see C11).

I1 to I3 are the big view of a file, not the page of every file: I1 is the lead's to leave (the
menu of apps on a file opened big), and I2 and I3 wait on a view that keeps notes and edits a
document, which this window's view of a file does neither of.

## K. The page of every file, line by line (`FilesPlace`'s own page, `files-place.css`)

Read off `FilesPlace.tsx` from `const [chosen` to its last `</div>`, every rule of
`files-place.css`, and the web's tests of the page: `files-says-what-each-file-takes-and-moves-
old-ones-to-the-trash`, `the-files-page-sorts-filters-and-names-a-file-for-a-person`,
`a-file-arriving-does-not-move-the-tile-you-are-about-to-click`, `a-file-keeps-its-place-and-its-
own-name-whoever-names-it` (the Move menu's search), `agents-check-pictures-sit-under-one-line-in-
files`, `the-cheap-looking-things-in-files-and-threads-are-gone`, `files-and-team-controls-share-
one-height-and-one-corner`, `settings-and-side-pages-keep-the-accent-small-and-the-words-short`
(`showsTotal`), `z-undoes-from-the-files-page`. Here: `src/files_page.rs`, `src/rules/files_shown.rs`
(`older_of`, `folders_matching`, `in_order_of`, `chosen_words`, with those tests' cases), scenes
`files-everything*` at 1280 by 760, 2x, each with its regions (`tools/scenes/*.regions.json`).
The whole window is about 10.2 of 255 on all of them for H1's page tile; the numbers below are
each scene's own regions.

The state and what it does:

- [x] K1 The page is drawn while asked for or while files are chosen, whatever crumb is pressed
  (`if (!chosen && !everything) return <FilesHome…>`); come to from elsewhere, nothing is chosen
  (the page mounts afresh). `every_page`; test `files_chosen_for_the_trash_go_together…` (a crumb
  pressed while choosing).
- [x] K2 What the cloud keeps is read once as the page opens (`api.filesCloud()`), for the total's
  last row. `tend`; the fixture's `filesCloud`.
- [x] K3 While files are chosen, Escape stops choosing and goes no further, even from the search
  box (a listener on the window, in its capture phase). `menu_key`, the search box's own key;
  test `escape_or_cancel_stops_choosing_and_files_stays`.
- [x] K4 `trash(keys, what, older)`: "Moving {what} to the Trash…", then the app's refusal or
  "Those files could not be moved to the Trash.", "Nothing here could be moved to the Trash.",
  or "Moved {what} to the Trash." with "Empty the Trash to get the {size} back." and Z's promise;
  Z writes `zero:files-place-untrash` with the app's `undo` ("Back from the Trash: {what}", put
  back, else "Those files are in the Trash now. Put them back from there."). Tests
  `move_to_the_trash_on_a_tiles_menu_says_what_it_frees`, F8's.
- [x] K5 The sort: Recent, Name, and Size only while it is the sort (pressing Size goes back to
  Recent); Grid or List. As before (H1); a press on any of them, a kind or Needs review draws
  the first sixty again (`setDrawn(TILES_AT_ONCE)` on `[project, folder, query, kind, review,
  sort]`), and so does a folder card. Test `a_kind_no_file_here_is_any_more_lets_go…`.
- [x] K6 A kind no file here is any more lets go (`if (kind && !kinds.includes(kind))
  setKind(null)`). `tend`; the same test.
- [x] K7 Looking (words, a kind or Needs review) searches every folder under here; browsing stays
  in the one place; a search finds check pictures like any file, browsing keeps them under their
  line (`apartScratch`). As before (H1).
- [x] K8 The order holds while the pointer moves over the tiles or the cards, and while a tile's
  menu or name box is open (`HOLD_STILL_MS` 1500, `busy()`): tiles and project cards in the order
  they were last drawn, a new one after them (`inOrderOf`, every case of the web's test). `held`,
  `held_still`; test `the_tiles_hold_their_order_while_the_pointer_moves_over_them`.
- [x] K9 Sixty tiles at a time, the next as the foot nears (`useNear`, `moreTiles`): as before.
- [~] K10 A teammate's name in a tile's line (`nameOf`): "A teammate", as before (H1); this
  window draws no team.
- [x] K11 `act`, `undoable`, `moveAll` (each file its own `do: 'move'`, one Z for those the app
  took once every one is answered, `moveUndo`), `rename`, `renameFile`, `removeFolder` with its
  words and Z (`do: 'restore'`), `countIn`. Tests `the_check_pictures_move_all_at_once…`,
  `a_folder_cards_dots_rename_it_in_the_card_or_delete_it`, F's.
- [x] K12 The cards: a project's on All files (its count, a press opens it, no menu), a folder's
  on a project's (its count, a press opens it, its ⋯), and the folder being named from New as a
  card of its own (`is-new`). Pictures `files-everything*` (cards 0.29 to 0.45).
- [x] K13 New in the strip, its folder offered where folders may go (`project ? canNest :
  true`), Upload included (L). Picture `files-everything-new` (its menu 0.36).
- [x] K14 What is said when nothing is here: "No file matches.", "Nothing in this folder yet.",
  "No files in {project} yet.", "Nothing in Files yet.", never while a folder is being named. As
  before.
- [~] K15 The tile is a button named "Open {name}" (or "Choose"/"Unchoose" while choosing) with
  the file's own name as its title when another is shown; here the title stands on the whole
  tile, its ⋯ included, and the tile has no name of its own for a screen reader. The ⋯s are
  named as there ("More for {name}", "Move {n} check pictures…").

The tile's ⋯ and its menu (`MoveMenu`, `.fp-more-box`, `.fp-more`, `.fp-menu*`):

- [x] K16 `.fp-more`: 26 by 26, `--surface`, the strong line, corner 6, its dots 14 points dim,
  the ink under the pointer; eight in from the tile's top right (`.fp-more-box`), nine down on a
  row of the list (`.fp-line .fp-more-box`); seen while the pointer is on the tile or the row
  (`:hover`) or its menu is open; drawn only for a file in a project, while nothing is chosen
  and its name is not a box. Pictures `files-everything-hover-tile` (0.31), `-menu` (the open
  tile's corner 0.31); rects to 0.01.
- [~] K17 `:focus-within` shows the ⋯ while it has the keys; nothing here takes Tab to it.
- [x] K18 A press toggles the menu; `up` when the ⋯'s foot is under 300 points from the window's
  foot (`MENU_ROOM`); then `fitMenu(menu, the page's foot)`: kept inside the page side to side
  and top to foot, flipping and scrolling as `keep-in-window.ts` does (`rules::compose::fit_menu`).
  Pictures `files-everything-menu` (0.47), `-menu-low` (upward, 0.47); test
  `a_press_on_the_dots_of_an_open_menu_closes_it` (the ⋯ is inside the box that keeps it open).
- [x] K19 `.fp-menu`: 200 wide, 30 under the ⋯'s top, its right edge on the ⋯'s (`top: 30px;
  right: 0`), at most 280 tall and scrolling past it, five in, `--surface`, the strong line,
  corner 10, `--shadow-sm`; `.is-up` 30 over its foot. Rects to 0.01 (`fp-menu` 158.5, 334, 200,
  280).
- [x] K20 Rename (while the box for a new folder is not there), then a gap. Rename makes the
  tile's name a box over its own line (`.fp-name > .fp-name-box`: 24 tall, six out, three up and
  down, the wash and the strong line, 13 points), all of it chosen; Return or leaving it writes
  `do: 'name'` when new, Escape gives up. Picture `files-everything-rename` (the tile 0.26, the
  box 1.38, rect to 0.01); test `rename_on_the_page_of_every_file…`.
- [x] K21 `.fp-menu-head` "Move to": 12 points, faint, 5 8 4 in (24.5 tall).
- [x] K22 `.fp-menu-find` once the project has more than three folders: 26 tall, four in, the
  line (the strong line while it has the keys), 13 points, "Find a folder", the keys in it as
  it opens (`autoFocus`); every key stays in it; Return moves into the one folder its words
  leave; Escape closes the menu. Picture `files-everything-menu-find` (0.41); test
  `the_move_menus_search_leaves_the_folders_it_names…`.
- [x] K23 The list: "Top of {project}" while no words are typed and the file is in a folder (the
  check pictures' always); with words, the folders they name, flat (`foldersMatching`); else each
  top folder and its subfolders 22 in (`.is-sub`), each with the folder mark (16, faint, 8 apart);
  the folder the file is in faint and no choice (`:disabled`); "No folder called that" (6 8 in, 13
  points, faint). Test `a_file_in_a_folder_is_offered_the_top_and_not_its_own_folder`.
- [x] K24 `.fp-menu-item`: 28 tall, eight in, corner 6, 13 points, the strong wash under the
  pointer unless disabled, cut at its edge; `.fp-menu-aside` at its end, 16 clear, 12 points,
  faint, tabular. A press inside the menu that is on no row keeps it open.
- [x] K25 New folder: a box in its place (`.fp-menu-new`: 2 8 3 14 in, the box six out), Rename
  and Move to the Trash put away while it is there; Return makes it (`do: 'make'`) and moves the
  file in once the app gives its id; Escape gives up, back to the whole menu. Picture
  `files-everything-menu-new` (0.50); test `new_folder_in_the_move_menu…`.
- [x] K26 Move to the Trash, after a gap, for a file the app says Files may move (`trashable`
  true, absent being no), with what it frees (`movable`, when more than nothing). Test.
- [x] K27 The check pictures' line (`.fp-scratch-row`: 30 down, 8 between) with its ⋯ always
  shown, for a project's: their Move menu, no Rename and no Trash, "Top of {project}" always,
  no folder faint; New folder makes it in the first one's project and moves them all in. Its
  fold's chevron is the web window's (12, `M6 4l4 4-4 4` and `M4 10l4-4 4 4`), and the line no
  longer stands six out (`.fp-scratch-row .fp-scratch { margin: 0 }`). Test
  `the_check_pictures_move_all_at_once…`; no scene has check pictures.

A folder card's ⋯ (`FolderCard`, `FolderMenu`, `.fp-folder*`):

- [x] K28 `.fp-folder-more`: in the middle of the card, fourteen in from its right, seen while the
  pointer is on the card or its menu is open. Picture `files-everything-hover-folder` (0.30),
  rect to 0.01.
- [x] K29 Its menu: Rename and Delete folder, no gap; 200 wide (`.fp-folder-menu`'s 160 loses to
  `.fp-menu`'s 200, which the stylesheet sets after it), 30 under the ⋯'s top, its right edge on
  the ⋯'s. Picture `files-everything-folder-menu` (the menu 0.35); rects to 0.01.
- [x] K30 Rename: the card's name and count give way to a box with its name, all of it chosen,
  six out, as wide as the words were, no ⋯; Return or leaving it writes `do: 'rename'` with Z.
  Picture `files-everything-folder-rename` (the box 1.13, rect to 0.01); test.
- [x] K31 Delete folder: `do: 'remove'`, "Deleted the folder {name}." with where its files went
  and Z. Test.
- [~] K32 A file dragged onto a card lands in that folder (`dropsFile`, `.fp-folder.is-over`):
  left out with E9.

The total and its menu (`TotalMenu`, `.fp-total*`):

- [x] K33 `.fp-total`, drawn once the files here take a megabyte (`showsTotal`, the web's cases)
  and nothing is chosen: 26 tall, nine in, the disk mark 13 and 7 apart, "{size} on this Mac",
  12.5 points, faint and tabular, dim under the pointer, the ink on the strong wash while open.
  Rect to 0.01 (155 wide). Picture `files-everything-total`: the menu 0.69, the bar 1.78.
- [~] K34 Open, its words' dark patch over the strong wash is a little unlike the web window's
  text-shadow there: 6.49 of 255 on the button's 155 by 26 points.
- [x] K35 Its menu (`.fp-total-menu`): 300 wide, 30 under, its right edge on the total's, no
  height limit. "{where} uses {size} of this Mac" (the folder's name, else the project's, else
  Files); Show the biggest first, or Show the newest first while by size; a gap; Move older
  versions to the Trash with what they take, while there are some (`{ keys: every file here with
  an older version, older: true }`, "the older versions"); Choose files to move to the Trash; a
  gap; Keep files in the cloud with On, Off (a plan with a cap) or Pro, opening Settings › Files
  on a plan, else the cloud's plans (`onSettings(paid ? 'files' : 'cloud')`). Rects to 0.01;
  test `the_totals_menu_sorts_by_size_moves_older_versions_and_opens_the_clouds_plans`.

Choosing files for the Trash (`chosen`, `.fp-choosing`, `.fp-check`):

- [x] K36 The bar keeps only the way back and the search (`!chosen && <div className="fp-views">`).
- [x] K37 The row under it (`.fp-tools.fp-choosing`): "Choose files to move to the Trash" or "{n}
  chosen · {size}" (13 points, dim, tabular, no dark patch: it is not a button), then Cancel (a
  kind's button) and Move to the Trash (`.fp-trash-go`: 26 tall, eleven in, the strong line and
  wash, 12.5 points; faint and doing nothing with nothing chosen). Pictures
  `files-everything-choose` (the row 0.59), `-chosen` (0.77); rects to 0.01.
- [x] K38 Every tile Files may move wears a ring (`.fp-check`: 18, nine in from its top right,
  thirteen down and twelve in on a row, 1.5 of white at 85% over black at 35%), white and ticked
  once chosen (the tick 11, `#111`), its face edged in the dim ink (`.is-chosen .fp-face`); a tile
  it may not move is faded to 40% (`.is-unmovable`) and a press passes it by; a press on a tile
  chooses or unchooses it instead of opening it. Pictures (the tiles 0.36, 0.35; the rings' rects
  to 0.01); test.
- [x] K39 Move to the Trash sends the chosen in the order chosen, named by the one's name or "{n}
  files", and stops choosing; Cancel and Escape stop. Tests.
- [x] K40 No ⋯ on a tile while choosing; the check pictures' ⋯ stays (it asks nothing of
  `chosen`). Test.

The rest of `files-place.css`:

- [x] K41 `.fp-page`, `.fp-bar`, `.fp-crumbs`, `.fp-crumb*`, `.fp-find`, `.fp-tools`, `.fp-kinds`,
  `.fp-views`, `.fp-switch`, `.fp-kind`, `.fp-view-icon`, `.fp-new-box`, `.fp-empty`,
  `.fp-folders`, `.fp-folder`, `.fp-folder-open`, `.fp-folder-glyph`, `.fp-folder-words`,
  `.fp-folder-name`, `.fp-folder-count`, `.fp-grid`, `.fp-tile`, `.fp-open-file`, `.fp-face`,
  `.fp-lines`, `.fp-line`, `.fp-thumb*`, `.fp-name`, `.fp-meta*`, `.fp-open`, `.fp-approved`,
  `.fp-scratch`, `.fp-grid-scratch`, `.fp-code-face`, `.fp-doc*`, `.fp-face-empty`, `.fp-in-cloud`,
  `.is-in-cloud`: H1, as before.
- [x] K42 `.fp-face-pdf` and `.fp-face-deck` stand on the strong wash. `files-everything` 10.64 to
  10.24 whole.
- [~] K43 `.fp-page-face` (a page drawn small), `.fp-face-pdf .pdf-page`, `.fp-face-deck` slides,
  `.file-cover-play` on a film's first frame, `.fp-length`: left out as before (README), the
  page's tile being the 10 of 255 that stays on every scene of this page.
- [x] K44 `.fp-sent*`: the big view's foot (I).

## L. Upload from computer

From `FilesPlace.tsx` (`NewMenu`, `pick`, `upload`), `renderer/src/api.ts`
(`filesPlaceUpload`), `main/ipc.mts` (`zero:files-place-upload`) and the web's tests of adding
files (`a-document-made-in-files-is-there-to-write-in`, its `addFiles` cases, and
`a-file-you-attach-or-add-costs-no-second-copy-of-disk`).

- [x] L1 New's last row, "Upload from computer" with its mark, wherever a document may be made
  (`canDocument`): on the home's New and the strip's. Pictures `files-new`, `files-new-all`,
  `files-everything-new`; test `new_makes_a_document…`.
- [x] L2 In a project, at once; from All files with one project of one's own, in it; with more,
  "Add files to" and the projects; a project picked from All files opens its home and the files go
  to its top (`p === project ? folder : null`). Tests `upload_from_all_files_asks_which_project…`
  and `upload_from_computer_opens_the_macs_file_window_here…` (a folder's page: into the folder).
- [x] L3 The Mac's own file window, over this window (Rune's `Tx::open_file`), titled "Add files
  to {project}", files only; in a test it is the scripted one and never a real window.
- [~] L4 One file a pick (Rune's file window hands back one path; the web window's takes several,
  `multiSelections`), and its button is the system's Open, not "Add" (`buttonLabel`, which Rune's
  options do not carry).
- [~] L5 A window that will not open says "The file picker did not open: {why}", as the paper
  clip's does here; the app's own says "Adding files needs the Mercury app on this computer." for
  a copy with no file window at all, which this window never is.
- [x] L6 Cancelled: nothing written, nothing said (`{ ok: true, added: 0 }`). Test.
- [x] L7 The paths picked go to the app on `zero:files-place-upload-here` `{ project, folder,
  paths }`, a channel of the local door alone (`main/local-door.mts` `FILES_UPLOAD_HERE`,
  `pickedFiles`, `filesUploadHere`; not in the Map the phone's copy is made from), which runs the
  code `zero:files-place-upload` runs once its window has picked (`addPickedFiles` in
  `main/ipc.mts`, moved there from the handler, which now calls it) and answers as it answers:
  `{ ok: true, added, refused, keys }`, or `{ ok: false, error }`. It takes a project, a folder or
  none, and 1 to 500 absolute paths, and nothing else ("not files to add"). Web test
  `the-local-door-opens-only-with-its-key` (four cases: what it adds and answers, what it turns
  down, the app's words on a failure, no channel without it, the key, and the one piece of code);
  native test: the channel and the exact payload.
- [x] L8 The answer: the list read again at once (`readAgain`); a kind Files cannot show named
  ("tool.exe is a kind Files cannot show, so it was not added.", "a.exe, b.dmg are kinds…, so
  they were not added.", `rules::files_home::upload_left_words` with those cases); a refusal in
  the app's words, else "Those files could not be added."; an app started before it had the
  channel (404) "Not done: restart Mercury first, so it can take that from this window."
  (`trouble::refused`), where the web window's bridge says "Quit and reopen Mercury to add files
  here.". Test.

## J. Mechanical check of the spec's names

Every class `FilesHome.tsx` and `files-home.css` name, against the lines above: `fh` C1,
`fh-crumbs` `fh-crumb-box` `fh-crumb` `fh-chev` C2, `fh-top` `fh-title` C3–C5, `fh-find` `fp-find`
`fh-key` C6, `fh-sec` `fh-head` `fh-head-name` `fh-link` `fh-count` `fh-see-all` D5,
`fh-grid` D1, `fh-split` `fh-main` `fh-rail` D1–D2, `fh-rday-box` `fh-rday` D3, `fh-rrow`
`fh-rname` D4, `fh-empty` D7 D9 D11, `fh-every` D10, `fh-shelf` `fh-none` `fh-more` D13, `fh-tile`
`fh-open` E1, `fh-glyph` E2, `fh-preview` E3, `fh-name` `fh-dot` E4, `fh-line` E5, `fh-meta` E6,
`fh-more-btn` E8 C4 D5, `fh-name-box` F5 F7 F9, `fh-menu` `fh-menu-head` `fh-menu-gap`
`fh-menu-list` `fh-menu-new` F1 F7, `fh-item` `fh-check` F3, `fh-sub` F7, `fh-icons`
`fh-icon-pick` F6. Its handlers: `onClick` (14), `onPick` (11), `onDone`/`onCancel`/`onBlur`/
`onKeyDown`/`onFocus`/`onChange` (the name boxes and the search, C7 C12 F5), `onContextMenu`
E7, `onClose` F2, `onFolder` D5 D8, `onOpen` E7 D4, `onRename` F5, `onError` E3,
`onDragStart`/`onDragEnd` E9. Its calls: `api.readDoc` (a document's title and first line, read
here off the disk as the tiles' faces already were), and through `FilesPlace.tsx`
`api.filesPlace`, `filesPlaceOpen`, `filesPlaceFolder`, `filesPlaceNew`, `filesPlaceTrash`,
`filesPlaceUntrash`, `filesPlaceUpload` (C13), `filesCloud` (the page of every file's total
menu, H4), `filesPlaceEdit`/`filesPlaceMark`/`openArtifactExternally` (the view, I).

Every class `FilesPlace.tsx` and `files-place.css` name (84, listed with `grep -o` and each looked
for here), against the lines above: `fp-page` `fp-bar` `fp-crumbs` `fp-crumb-box` `fp-crumb-gap`
`fp-crumb` `fp-find` `fp-tools` `fp-kinds` `fp-views` `fp-switch` `fp-kind` `fp-view-icon` `is-on`
K41 H1; `fp-new-box` `th-new` K13 A3; `fp-empty` K14; `fp-folders` `fp-folder` `fp-folder-open`
`fp-folder-glyph` `fp-folder-words` `fp-folder-name` `fp-folder-count` `is-new` K12; `is-over`
K32; `fp-folder-more` `fp-folder-menu` K28–K29; `fp-name-box` `fp-name` K20 K30; `fp-grid`
`fp-tile` `fp-open-file` `fp-face` `fp-face-image` `fp-thumb` `fp-thumb-word` `fp-lines` `fp-line`
`fp-line-marks` `fp-meta` `fp-meta-words` `fp-open` `fp-approved` K41; `fp-more-box` `is-open`
`fp-more` K16–K18; `fp-menu` `is-up` `fp-menu-head` `fp-menu-find` `fp-menu-list` `fp-menu-none`
`fp-menu-item` `is-sub` `fp-menu-new` `fp-menu-gap` `fp-menu-aside` K19–K26; `fp-scratch-row`
`fp-scratch` `fp-grid-scratch` K27 K41; `fp-face-pdf` `fp-face-deck` K42; `pdf-page`
`file-cover-play` `fp-length` `fp-page-face` K43; `fp-code-face` `fp-doc` `fp-doc-li`
`fp-face-empty` `fp-in-cloud` `is-in-cloud` K41; `fp-sent` `fp-sent-label` `fp-sent-place` K44;
`fp-total-box` `fp-total` `fp-total-menu` K33–K35; `fp-choosing` `fp-chosen-words` `fp-trash-go`
`fp-check` `is-chosen` `is-unmovable` K36–K40. Its handlers on this page: `onClick` (each a line
above), `onChange`/`onKeyDown`/`onBlur`/`onFocus` (the boxes: K20 K22 K25 K30, C7), `onPointerMove`
K8, `onDragStart`/`onDragOver`/`onDragLeave`/`onDrop` K32 E9. Its calls: `api.filesCloud` K2,
`filesPlaceTrash`/`filesPlaceUntrash` K4, `filesPlaceFolder` K11, `filesPlaceNew` C11,
`filesPlaceUpload` L, `readDoc` H1, `filesPlaceOpen` and the view's I.

## Count

127 lines: 111 ticked [x], 13 with a deviation [~] (A2, C11, C13, E9, F2, K10, K15, K17, K32,
K34, K43, L4, L5), 3 open [ ] (I1, I2, I3: the big view of a file, each saying why). Parts K
and L (the page of every file and Upload from computer, 2026-11-24) are 52 of them: 44 ticked,
8 deviations, none open.
