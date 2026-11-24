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
- [~] C9 New's menu: 180 wide, 40 under New's top, right-aligned; Document, Table, Folder; with
  no project and two projects of one's own, "New document in" and the projects. Upload from
  computer is left out (C13), so the menu is a row shorter than the web window's. Pictures
  `files-new`, `files-new-all`; test `new_makes_a_document…`.
- [x] C10 New is not drawn with no project and none of one's own. Picture `files-empty`.
- [~] C11 Document and Table write `zero:files-place-new` `{ project, name: '', kind, folder }`
  and open the file big once the app gives its key; a refusal says the app's words, else "That
  document could not be made.". The web window opens it ready to write in (`typing`): this
  window's view of a file cannot edit a document, so it opens to read. Test.
- [x] C12 Folder: from All files the project picked opens; a name box (`.fp-name-box`, 24 tall,
  the wash and the strong edge) stands as a section; Return or leaving the box makes it
  (`do: 'make'`, the folder as parent); Escape or a blank name gives up. Picture `files-naming`;
  test.
- [~] C13 Upload from computer: left out. `zero:files-place-upload` answers with the system's
  file window (`dialog.showOpenDialog(window, …)`), which the app opens over its own window, not
  this one. Needs a channel that takes the paths (see the report).
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
  the press that closed it also land on what is under it; this window's press only closes it.
  Tests.
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
  rules' cases. Z's writes are drawn over the list once the app has them, not at the press (the
  pile is the window's, `answer.rs`, which this port leaves alone).
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
- [ ] H4 A tile's menu (`MoveMenu`), the folder cards' menus, choosing files for the Trash, the
  total's menu: still not drawn, as before this port. Every channel they need is passed by the
  door; left for want of time, and listed in the README.

## I. The file opened big from Files

- [ ] I1 The open mark offers the apps the Mac would open the file with (`OpenWith`,
  `zero:open-with` then `zero:open-artifact` with `app`): not ported; this window's open mark
  still hands the file to its default app. The door passes both channels.
- [ ] I2 A file's comments handed to an agent (747da8461): not ported, as the view has no notes.
- [ ] I3 A document made with New opens ready to write in, and a file is renamed from its name in
  the view (`typing`, `onRename`): not ported; the view cannot edit (see C11).

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

## Count

75 lines: 65 ticked [x] (F10's timing noted on its tick), 6 with a deviation [~] (A2, C9, C11,
C13, E9, F2), 4 open [ ] (H4, I1, I2, I3), each open line saying why.
