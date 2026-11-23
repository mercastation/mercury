# A table file, behaviour by behaviour

Every behaviour of a `.csv` or `.tsv` opened in the web window, read off its code, so the native
window can be held to the whole of it and not to its look alone. Read off Mercury `3d1b1383a`:
`renderer/src/components/TableView.tsx` (the table), `renderer/src/table-file.ts` (reading a
sheet), `renderer/src/table-edit.ts` (what the hands do to it and how it is written back),
`renderer/src/components/table-view.css` and the shared menu look (`.when-menu` in
`renderer/src/styles.css`), and where a table is shown: `DocPane.tsx` (beside a conversation,
`mode="pane"`), `ArtifactStage.tsx`'s `TableStage` (big, `mode="full"`), and `main/ipc.mts`
`zero:write-doc` with `main/doc-file.mts` `writeDoc` (the save).

`R` is `renderer/src`. Each line carries where it is in the web's code, whether the native window
does it, and how that was checked. A line not done says what stands in the way.

How lines were checked:

- A test named plainly is in `src/table/tests.rs`: the native window built headless on an
  invented sheet, pressed, typed into and walked with the keys, its state and what it would
  write read back.
- `rules:` names a test in `src/rules/table_file.rs`, `src/rules/table_edit.rs` or
  `src/rules/pane.rs`, each ported case for case from the web's own test file named over it.
- A scene is `tools/scenes/<name>.mjs`: the web window photographed after the same presses and
  keys, the native window driven through them by `tools/table.py` and photographed, the number
  the mean difference of 255 in the sheet's card (fonts and anti-aliasing alone come to about
  0.2 to 0.4). Where a scene writes, what each window would have sent the app is compared word
  for word.
- "Pointer" is the pointer's shape compared at the points the web window's dump lists.
- The saving path was also run once on a scratch engine (a store of its own in a temporary
  folder, never the app's), and the file it left on disk was the web's, byte for byte.

Out of scope here, as no table file: a markdown table inside a message (`InlineTable`, `mode="inline"`)
and the folded preview of a sheet under a teammate's answer (`OneThingAtATime`, `mode="preview"`).

## A. Which files, where, and when they can be changed

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| A1 | A table is a path ending `.csv` or `.tsv`, any case, a query or a fragment after it ignored | `isTablePath` | done | rules: `which_files_are_tables`, `the_pane_knows_a_kind_by_the_address_first_then_the_ending` |
| A2 | Opened beside a conversation (a mention, a row of the thread's Files list, the newest answer naming it on a line of its own) as the pane's table | `DocPane` `kind === 'table'` | done | `a_sheet_beside_a_conversation_is_picked_as_a_spreadsheet_is`; scene `table` (0.33) opens it from its mention |
| A3 | Opened big (a tile of the Files place; the pane's corners) as the full table, not as its lines | `TableStage` | done | `a_sheet_opened_big_is_the_full_table` (from the pane's corners); scene `file-table` (a tile of Files: whole 0.45, the table the same, the web's notes column aside) |
| A4 | Beside a conversation it can be changed unless it is a teammate's file (`readOnly`) or lies outside the project's own folders (`paneCanSave`); big, unless it is a teammate's (`readText`). Otherwise it is read only: nothing is written, the editor says Read cell, there is no New row, and the menus have no rows that change it | `onSave` given or not | done | `a_sheet_that_cannot_be_changed_is_read_and_never_written`; rules: `saves_only_a_file_inside_the_project`. This window opens no teammate's files, so that half never arises |
| A5 | Read through the app (`zero:read-doc`), which answers its words and its time on disk; while it is read the pane waits; a file that cannot be read says so (pane: the app's words, else "That file could not be read."; big: "<name> could not be opened.") | `DocPane`, `TableStage` | done | every test reads the invented sheet; `a_sheet_too_big_to_open_says_so_and_is_never_worked` (over the app's 1.5 MB: the app's words, no sheet, so nothing cut short is ever saved; big, "could not be opened."). This window reads the file off the disk itself, by the app's rule, where the web window asks the app; other refusals say "<name> could not be read." beside a conversation, as this window's pane says for any file |
| A6 | Beside a conversation it is asked of the disk while the app changes, at most every three seconds, and read again when its time on disk is newer; the new words give a new table: the pick, an edit, the open row, a menu, a rename and a problem all go, and so does what undo held | `DocPane` follow, `TableView` `[base]` effects | done | `a_sheet_rewritten_on_disk_is_read_again` |
| A7 | "Load the new one" reads it again (pane and big) | `onReload` | done | `load_the_new_one_reads_the_file_again` |
| A8 | A file with no header at all draws "This table has no rows yet." | `!table.columns.length` | done | `a_view_with_no_rows_says_why` |

## B. Reading a sheet (`R/table-file.ts`)

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| B1 | RFC 4180 with the separator given: quotes hold separators, doubled quotes and line breaks; a quote opens after the space a person types after a separator; every cell trimmed; a byte order mark dropped; CRLF read; rows with nothing in them dropped | `parseDelimited` | done | rules: `the_text_is_split_as_a_spreadsheet_splits_it`, `files_the_way_agents_really_write_them` |
| B2 | The separator: a tab for `.tsv`; else whichever of comma, tab and semicolon the first line holds most of, outside quotes | `sepFor` | done | rules: `the_text_is_split_as_a_spreadsheet_splits_it` |
| B3 | A markdown table saved as `.csv` (`\| a \| b \|` lines and a `\|---\|` rule) is read as its table; `\|` inside a cell is a pipe | `isMarkdownTable`, `parseMarkdownTable` | done | rules: `files_the_way_agents_really_write_them`, `what_the_last_power_user_loop_found`, `a_save_writes_the_file_back_the_way_it_came` |
| B4 | The header row names the columns: a row shorter than the widest is filled, a column with no header and nothing in it (a trailing separator) is dropped, a blank header is "Column N", a repeated name becomes "Name 2" (and on, never two alike); 150,000 rows read without running out of stack | `tableFromGrid` | done | rules: `each_column_is_told_what_it_holds`, `what_a_second_model_found` |
| B5 | A first column headed `#`, `No.`, `N°`, `Rank`, `Ranking`, `Position` or `Pos.` over numbers of up to four digits is the file's own rank: it stands where the row number goes, is a number, and the name is the column after it | `RANK_HEADER`, `ranked` | done | rules: `a_rows_name_opens_its_one_link`; `a_ranked_file_shows_its_rank_as_the_row_number` |
| B6 | What each column holds (`kindOf`), cells saying nothing (`n/a`, `none`, `tbd`, `-`, `?`…) aside: the first column (or the one after a rank) is the name; 80% markdown links is a link column; 80% emails an email column; 80% addresses that are not numbers a link column; 90% amounts a number column unless the header names a label (phone, zip, id, year) and no amount (revenue, per, count…); 80% days that carry other words is text; 80% plain days a date column where the header names a time or the values have a month, a slash or an ISO day; short values (28 characters, four words) of 30 kinds or fewer under a header naming a choice (status, stage, type, owner…) are tags, and of 12 kinds or fewer that repeat (no more than 0.6 kinds a value) under a header that is not a name, a place or a note are tags; anything else is text | `kindOf`, the header tables | done | rules: `each_column_is_told_what_it_holds`, `files_the_way_agents_really_write_them`, `what_a_second_model_found`, `what_the_last_power_user_loop_found` |
| B7 | An amount as an agent writes one: `$1.2M`, `~$1M`, `about 5k`, `$7B+`, `$250k-$1M` (the low end, sharing the high end's scale), `€80.000`, `1.250.000,50`; an ISO day is no amount | `amountOf`, `oneAmount` | done | rules: `what_the_last_power_user_loop_found`, `words_are_ordered_as_a_person_reads_them` |
| B8 | A day: ISO (a day not on the calendar is none), `m/d/yyyy` and `d/m/yyyy` (day first when the first is over 12; two-figure years in the 2000s; dots too), and words with a month in them as the browser's `Date.parse` reads them, this year when no year is written; one day is one number (UTC midnight) | `dateOf`, `realDay` | done | rules: `a_column_of_names_with_a_day_in_them_is_not_dates`, `what_the_last_power_user_loop_found`, `files_the_way_agents_really_write_them` (the browser's reading of words emulated case for case) |
| B9 | A day typed into a date cell: also today, tomorrow and yesterday, and `12/3` as this year's | `dayTyped` | done | rules: `today_tomorrow_and_yesterday_are_days` |
| B10 | A day picked is written the way the column writes its days: ISO, `m/d/yyyy`, or words with a weekday where most of the column has one and a year where most has one (or the column is empty) | `dateWords` | done | rules: `a_day_picked_is_written_the_way_the_column_writes_dates` |
| B11 | Names with a day in them ("Cloud Thu Nov 12") are not taken for dates | `onlyADate` | done | rules: `a_column_of_names_with_a_day_in_them_is_not_dates` |
| B12 | A cell that is wholly `[text](url)` is a link: `mailto:` with an email, `http(s)` with no space, or an address with no scheme (given `https://`); one level of brackets in the address; any other scheme is words | `markdownLink` | done | rules: `a_cell_written_text_and_url_reads_as_its_text_and_address` |
| B13 | What a cell reads as: a markdown link's text, else the cell | `shownText` | done | rules: `a_cell_written_text_and_url_reads_as_its_text_and_address` |
| B14 | A tag's colour: High, Medium and Low by whether the column measures something bad; yes and no under a header naming something bad; the words a pipeline uses (grey, green, rose, purple, blue, amber lists, also with `_` and `-` read as spaces); under a header about how something went (or none), words that mean later, bad or good; any other word the next of blue, purple, amber, grey by its place | `tagTone` | done | rules: `tag_colours`; scene `table-tags` (0.81) |
| B15 | A tag column's values as first written, one per spelling whatever its case, coloured in the order they first appear | `tableFromGrid` | done | rules: `tag_colours`, `each_column_is_told_what_it_holds` |
| B16 | The logo column: a link column not named for LinkedIn, Twitter, GitHub or Crunchbase whose sites are four fifths not social networks' pages, a column named website, domain, site or homepage first; each row's logo is its host | `logoColumn`, `logoFor` | done | rules: `each_column_is_told_what_it_holds`, `files_the_way_agents_really_write_them`; on the scratch engine public sites' logos drew and a social network's site kept its letter |
| B17 | A link cell's address gets `https://`, and reads with no scheme, no `www.` and no trailing slash | `hrefOf`, `shownLink` | done | rules: `a_cell_written_text_and_url_reads_as_its_text_and_address`; scenes show the sites as the web does |
| B18 | Where a cell goes: its own markdown link; a link column's address; an email column's `mailto:`; a name cell's bare `http(s)` address, or the table's one link column's address in its row (none when there are two link columns) | `cellLink`, `nameLinkColumn` | done | rules: `a_rows_name_opens_its_one_link` |
| B19 | Search: the words anywhere in a row, any case; a filter keeps per column the rows whose value is one of those ticked, any case; both together | `filterRows` | done | rules: `searching_filtering_and_sorting`; `the_filter_and_the_sort_narrow_and_order_the_rows_and_are_kept_for_the_file`; scenes `table-searched` (0.21), `table-filtered` (0.24) |
| B20 | Sort: numbers by amount, days by day, everything else by the words shown as a person orders them (figures as numbers, capitals and accents aside); empty, saying-nothing and unreadable cells last either way; rows that tie keep their order | `sortRows` | done | rules: `searching_filtering_and_sorting`, `words_are_ordered_as_a_person_reads_them` (the browser's collation emulated); scene `table-sorted` (0.36) |

## C. The bar over the table

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| C1 | The search box: a magnifier, "Search 1,234 rows" ("row" for one), 280 points at most and 140 at least; what is typed narrows the rows | `.tbl-search` | done | `a_sheet_beside_a_conversation_is_picked_as_a_spreadsheet_is`; scenes `table` (0.33), `table-searched` (0.21) |
| C2 | Escape in the search box while it holds words takes them away, unless a menu is open, something is picked or edited, a row or a rename is open, or the keys are in a box outside the table; with no words, Escape goes on to whatever the table is in | `query` effect, the input's `onKeyDown` | done | `the_filter_and_the_sort_narrow_and_order_the_rows_and_are_kept_for_the_file` |
| C3 | Filter, only when some column can be filtered: its mark and "Filter", or "Filtered by A", "Filtered by A and B", "Filtered by A and B and 2 more"; lit while filtering or open | `filterLabel`, `Menu` | done | same test; scene `table-filtered` (0.24) |
| C4 | Its menu, 240 wide and no taller than 480 or seven tenths of the window: a box "Find a value" (the keys in it) when the columns offer more than twelve values; "Show every row" while filtering; per column its name as a heading and a row per value, ticked or not, the value a tag in its colour, "Empty" for the empty cells, or its words, and how many rows hold it; a press ticks or unticks and the menu stays open; the box narrows the values, and a column with none left loses its heading | `FilterMenu` | done | same test; scene `table-filter-menu` (0.37) |
| C5 | Which columns it offers: every tag column with values, and any other column (not a name, not a number) whose values repeat from 2 to 40 kinds, at most 0.6 kinds a filled cell, in A to Z order; the empty cells as one more value where there are some | `filterChoices` | done | rules: `the_filter_menu_offers_any_column_with_a_short_list_of_values` |
| C6 | Sort: its mark and "Sort", or "Sorted by Name, A to Z" (Z to A, 1 to 9, 9 to 1, oldest first, newest first); lit while sorted | `Menu` label | done | the filter and sort test; scene `table-sorted` (0.36) |
| C7 | Its menu: "As the file has it" (lit with no sort; a press takes the sort off and closes), then every column (lit when sorted by it, with its direction's words): a column not sorted by sorts it upward and closes the menu; the column sorted by turns downward, then off, the menu staying open | the Sort `Menu` | done | the filter and sort test; scene `table-sort-menu` (0.32) |
| C8 | A bar menu opens and closes with a press on its button, closes on a press anywhere else, and on Escape, after which the keys are the table's again | `Menu` | done | the filter and sort test |
| C9 | The count at the bar's end: "8 rows", or "3 of 8 rows" when narrowed; with more than one cell picked, "6 cells picked · " or "2 rows picked · " (whole rows) before it; big, " · 5 columns" after it | `.tbl-count` | done | the first test, `a_sheet_opened_big_is_the_full_table`; scenes `table-block` (0.38), `table-rows` (0.39) |
| C10 | A save that did not happen says so where the count was: a rose dot, the reason, and "Save mine" and "Load the new one" when the file moved on, else "Try again" | `.tbl-problem` | done | `a_save_the_app_refuses_says_so_and_save_mine_writes_it_anyway`, `load_the_new_one_reads_the_file_again` |
| C11 | Teammates' faces in the bar, in a thread's file the team has open | `LiveFaces` | not done | Not Rune: this window has no live connection to teammates at all (no shared copies, no faces, no team rows anywhere in it), so a table has none to show. It needs that connection first |

## D. How the table looks

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| D1 | The header row stays at the top while the rows scroll; each header the mark of what its column holds and its name in small spaced capitals, a number column's to the right, the sorted column's brighter with ↑ or ↓, a picked column's brighter | `.tbl-th` | done | the first test; every scene; `table-sorted` (0.36) for the arrow |
| D2 | The row numbers, 44 points wide (58 from 1,000 rows), in small figures to the right; the corner over them picks every cell | `.tbl-rn` | done | the first test, `a_long_sheet_builds_only_the_rows_in_view` |
| D3 | A ranked file's rank stands where the row number goes, and its header is `#`, which sorts by it | `table.ranked` | done | `a_ranked_file_shows_its_rank_as_the_row_number` |
| D4 | The row numbers and the first column stay put while the rest scrolls sideways, on the table's ground | `.tbl-tr > :nth-child(1/2)` | done | scene `table-sideways` (0.99) |
| D5 | A wide column scrolled half under them keeps its header's name in view beside them | `.tbl-th > .tbl-ell` sticky | done | read against the web; not photographed: the sideways settling leaves no column half under in a scene |
| D6 | Scrolling sideways settles on a column's edge, none left half under the frozen ones | `scroll-snap-type: x proximity` | done | scene `table-sideways`: a wheel of 150 points, the web settled at 129 and the native window on the same column edge. Rune has no scroll snapping, so the window settles the sheet itself once it stands still, on the nearest column edge within a third of the view (Chromium's proximity is its own guess) |
| D7 | A row 46 points tall (86 for every row while any column wraps), its cells 16 in, a line on their right and under them | `.tbl-td` | done | the first test; every scene; `a_column_set_to_wrap_makes_every_row_three_lines_tall` |
| D8 | The name cell: the row's logo or, where it has none, the first letter of the name on a small tile, where the table has a logo column; then the name, or the link it opens | `CellBody`, `Logo` | done | every scene (letters: invented sites have no logos); logos seen on the scratch engine, asked of the same favicon service, a 16-point globe taken as none |
| D9 | A tag cell: a square tag with a dot, in its colour (seven colours), as the column first wrote it | `TagChip` | done | scenes `table` (0.33), `table-tags` (0.81) |
| D10 | A number cell to the right in even figures; a date cell dimmer in even figures | `.tbl-num`, `.tbl-date` | done | scenes `table`, `table-date` (0.39) |
| D11 | A link or an email cell: its words dim, underlined under the pointer, its title the address | `.tbl-link`, `hoverText` | done | scene `table-sideways` (the pointer resting on a link: underlined, its address as its title); pointer |
| D12 | Words on one line, cut with dots; the title carries a name, and text over 40 characters, whole | `.tbl-ell`, `hoverText` | done | every scene; titles read against the web |
| D13 | Wrap text: a column of words shows three lines a cell | `.tbl-td.wrap` | done | `a_column_set_to_wrap_makes_every_row_three_lines_tall` |
| D14 | Each column's width from the whole file (sampled at most 3,000 rows): its longest words up to 220 points (300 big), or its widest tag, or its header, with a logo's room and the cell's edges; a width the hand set wins; the longest column of text takes what is spare, else the last column | `columnWidths`, `tracks` | done | the first test; every scene's columns stand where the web's do |
| D15 | Only the rows in view are drawn, and twelve each side, so a sheet of 150,000 rows scrolls | `SPARE`, `start`/`end` | done | `a_long_sheet_builds_only_the_rows_in_view` |
| D16 | "This table has no rows yet." under the header with no rows; "No rows match." when the search or the filter leaves none | `.tbl-none` | done | `a_view_with_no_rows_says_why` |
| D17 | While a row is under the pointer, the end of its first cell offers "Open this row (Space)" | `.tbl-open` | done | scene `table-sideways` (the pointer's row shows it); pointer |
| D18 | While a header is under the pointer, its menu's caret shows by its name, and its right edge is a grip with a line under the pointer | `.tbl-th-menu`, `.tbl-resize` | done | pointer; `a_headers_grip_widens_its_column_and_a_double_press_fits_it` |
| D19 | "+ New row" at the foot while the last rows are in view, in a table that can be changed, washed under the pointer, its title "New row (Shift+Enter)" | `.tbl-add` | done | every scene; `a_sheet_that_cannot_be_changed_is_read_and_never_written` |
| D20 | What is picked: a blue tint on the block, a ring on the cell the keys act on, the row numbers of whole rows and the headers of whole columns lit; with the keys elsewhere the ring is a faint line | `.sel`, `.at`, `.holding` | done | scenes `table-picked` (0.35), `table-block` (0.38), `table-rows` (0.39), `table-searched` (the keys in the search box) |
| D21 | Beside a conversation it stands 28 points in under a bar 4 points down; big, 48 in and 8 down; square inside | `.tbl-pane`, `.tbl-full` | done | every scene (pane); `a_sheet_opened_big_is_the_full_table` |
| D22 | The header's mark and name carry the column's type as their title | `title={typeName(c.kind)}` | done | read against the web; a title shows after the pointer rests, as the system's does |

## E. The pointer

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| E1 | A press picks a cell; with Shift it stretches the block from where it began; dragging stretches it | `onDown`, `move` | done | the first test; scene `table-block` (0.38) |
| E2 | A press on a row number picks whole rows, on a header the whole column; Shift and dragging stretch them; a header picks nothing in a table with no rows | `pickFor` | done | the first test; scene `table-rows` (0.39) |
| E3 | Whole rows picked, pressed again in a table that can be changed and is not sorted, are picked up: a line shows where they will land as the pointer moves, letting go moves them there, Escape puts them down where they were, letting go where they started picks the pressed row | `moverows` | done | `rows_picked_whole_are_dragged_to_where_they_land` |
| E4 | The same for whole columns picked | `movecols` | done | `columns_picked_whole_are_dragged_to_where_they_land` |
| E5 | Dragging near the foot or under the header scrolls the table 18 points a move; moving columns near the sides scrolls it sideways | `move` | done | `a_drag_near_the_foot_scrolls_the_table` |
| E6 | One press on a tag or a date cell opens its list, once the press ends where it began, in a table that can be changed, with no key held, unless that cell was being edited | `opensOnClick` | done | `a_tag_cell_opens_its_list_with_one_press_and_a_date_cell_its_calendar`; scenes `table-tags`, `table-date` |
| E7 | A double press on a cell edits it; on a row number opens the row | `onDouble` | done | `a_double_press_edits_and_a_press_elsewhere_keeps_the_words`, `space_opens_the_row_whole_and_its_keys_walk_the_rows` |
| E8 | A link in a cell opens only with ⌘ held (or from the menu): a plain press picks the cell | `linkOpens` | done | a plain press on a link cell picks it in the tests and scenes; the ⌘ press read against the web, not pressed in a test because it opens the browser |
| E9 | A right press on a header opens its column's menu at the pointer; anywhere else it picks what is under it unless that is already in the block, and opens the cell's menu at the pointer; an edit open is ended first | `onContext` | done | `a_header_and_a_cell_open_their_menus_and_the_keys_walk_them`; scenes `table-cell-menu`, `table-column-menu` |
| E10 | A press on a row's "Open this row" picks the row and opens it | `.tbl-open` | done | `space_opens_the_row_whole_and_its_keys_walk_the_rows` |
| E11 | A press on the table while a cell is being edited ends that edit, keeping the words (a tag's list closes with nothing picked) | `onDown` | done | `a_double_press_edits_and_a_press_elsewhere_keeps_the_words` |
| E12 | A header's grip dragged widens or narrows its column (64 points at least), kept for the file; a double press fits it to what it holds | `onGrip`, `fitColumn` | done | `a_headers_grip_widens_its_column_and_a_double_press_fits_it` |
| E13 | A press on a cell's words shown whole goes to the cell under it, unless words were selected there; a double press edits | `.tbl-peek` | done | `a_cut_off_cell_picked_shows_its_words_whole_until_escape` |
| E14 | The pointer: the pointing hand on buttons and the header's caret, the grab hand over row numbers and headers that can be picked up, the closed hand while moving, the resize pointer on a grip, the I-beam over words shown whole | `cursor` rules | done | pointer, at every point of the web's dump over the sheet |

## F. The keys

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| F1 | The arrows move the pick, Shift stretching the block's far end, ⌘ going to the edge; Home and End to the row's ends, with ⌘ to the table's corners; Page Up and Page Down by the rows in view less one | `moved` | done | the first test; rules: `a_pick_moves_the_way_arrows_move_in_a_spreadsheet` |
| F2 | Tab and Shift Tab move right and left | `onKeys` | done | the first test |
| F3 | Enter edits; Shift Enter puts in a row below and edits it; F2 edits | `onKeys` | done | `a_cell_typed_over_is_written_back_in_the_files_own_shape`, `rows_and_columns_are_added_moved_and_taken_away`; scene `table-typed` |
| F4 | Space opens the row | `onKeys` | done | `space_opens_the_row_whole_and_its_keys_walk_the_rows`; scene `table-row` (0.80) |
| F5 | Backspace and Delete empty the picked cells | `clearPicked` | done | `undo_and_redo_walk_the_changes` |
| F6 | Escape: a block goes back to the cell it began on; a cell's words shown whole are put away; then nothing is picked and the table keeps the keys; with nothing picked, Escape goes on to whatever the table is in | `onKeys` | done | the first test; the words shown whole test |
| F7 | A letter is the app's (R replies, E closes, J and K go to the next thread), never typed into a cell | `passKey` | done | `a_letter_goes_on_to_the_conversation` |
| F8 | ⌘Z takes back, ⌘⇧Z and ⌘Y do again, 200 steps, also with nothing picked while the table has the keys | `undo`, `redo`, `onRoot` | done | `undo_and_redo_walk_the_changes` |
| F9 | ⌘A picks every cell | `onKeys` | done | `copy_cut_paste_and_fill_down_speak_the_spreadsheet_clipboard` |
| F10 | ⌘C copies the picked cells as a spreadsheet does (tabs and lines, a cell holding a tab, a line break or a quote quoted); ⌘X copies and empties them; ⌘V pastes: one value fills the pick, a block that fits it a whole number of times is repeated, a block is laid out from the pick's top left, columns past the last left off, rows past the end added; the pasted block is picked | `toClipboard`, `fromClipboard`, `pasteInto` | done | same test; rules: `copy_and_paste_speak_the_spreadsheet_clipboard` |
| F11 | ⌘D fills the block's top row down into the rows under it | `onKeys` | done | same test |
| F12 | ⌥↑ and ⌥↓ move whole rows picked (not under a sort), ⌥← and ⌥→ whole columns picked; on anything else they do nothing | `onKeys` | done | `rows_and_columns_are_added_moved_and_taken_away` |
| F13 | With nothing picked and the keys on the table, an arrow, Tab, Enter, Home, End or a page key picks the first cell | `onKeys`, `onRoot` | done | the first test |
| F14 | The moving end is kept in view: down and up a row at a time under the header, sideways past the frozen columns, a cell wider than the view showing its start | `follow` effect | done | the first test |

## G. A cell edited

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| G1 | A box opens over the cell, never over the row number and the first column (the first column's own box stands right of the row number), as wide as the cell, 380 points while it holds more than 32 characters or a line break, inside the view; it grows with the words up to 320 points, then scrolls; the caret at the end | `.tbl-edit`, `place` | done | `a_cell_typed_over_is_written_back_in_the_files_own_shape`; scene `table-editing` (0.36) |
| G2 | Enter keeps the words and moves down; ⌘Enter keeps them and stays; Shift or ⌥ Enter is a new line; Tab and Shift Tab keep them and move right or left; Escape drops them; leaving the box keeps them | `GrowingText` keys | done | same test; scene `table-typed` (the same four saves as the web's, word for word) |
| G3 | A plain number typed into a column of amounts is written as the column writes them (`7200` among `$1,200` is `$7,200`); a day typed in words into a date column is written as the column writes days, unless it is what was there | `likeItsColumn`, `asDate` | done | same test; rules: `a_number_typed_into_a_column_of_amounts_is_written_the_columns_way` |
| G4 | Rows copied from a spreadsheet pasted into an empty box go across the table, not into the one cell | the box's `onPaste` | done | `rows_pasted_into_an_empty_box_go_across_the_table` |
| G5 | In a table that cannot be changed the box opens read only, with a faint ring | `readOnly` | done | `a_sheet_that_cannot_be_changed_is_read_and_never_written` |
| G6 | A ranked file's rank is never edited; in a table that cannot be changed a tag cell never opens | `startEdit` | done | `a_ranked_file_shows_its_rank_as_the_row_number`, `a_sheet_that_cannot_be_changed_is_read_and_never_written` |
| G7 | Writing below the last row adds the rows it needs | `writeAt` | done | `copy_cut_paste_and_fill_down_speak_the_spreadsheet_clipboard` |
| G8 | After an edit, the keys are the table's again | `endEdit` | done | `a_cell_typed_over_is_written_back_in_the_files_own_shape` |

## H. A tag cell's list

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| H1 | It drops from the cell, 260 wide, or rises over it near the view's foot (less than 300 points under it and more over it) | `.tbl-pop` | done | `a_tag_cell_opens_its_list_with_one_press_and_a_date_cell_its_calendar`; scene `table-tags` (0.81) |
| H2 | A box "Find or add a tag" (holding a letter typed to open it); the column's tags each as its tag, narrowed by what is typed; "Add" and the new tag in the colour it will get when what is typed is new; "Clear" when the cell has a tag and nothing is typed; "No tags yet. Type one to add it." | `TagPicker` | done | same test and scene |
| H3 | The cell's own tag is lit at first and ringed; ↑ and ↓ move the light; Enter picks; Tab picks what is lit while something is typed, else keeps the cell, and moves right (Shift: left); Escape closes; a pointer moving onto a row lights it | `TagPicker` keys | done | same test; scene `table-typed` picks a tag with the keys |
| H4 | The lit row's "⋯" ("Rename, colour or delete") opens that tag's own page | `.tbl-pick-more` | done | same test; scene `table-tag-page` (0.57) |
| H5 | A tag's page: back ("Back (esc)"), its name in a box, all selected; Enter, Escape, back or leaving the box rename it; "Colour" and seven swatches, the tag's own ringed, a press recolouring it; "Delete tag" in rose with how many rows hold it, which empties them all | `TagEditor` | done | same test and scene |
| H6 | A tag renamed in every cell that holds it, whatever its case; a name another tag has merges the two; the colour follows; a filter on it follows the new name, or lets go of a tag deleted, and undo takes the filter back too | `renameTag`, `dropTag`, `refiltered` | done | same test; rules: `a_tag_is_renamed_merged_or_deleted_in_every_cell_that_holds_it` |
| H7 | A tag typed that the column has not had is learnt in the colour its word means, or one no tag in the column has yet; one deleted and typed again keeps its colour | `learnTag`, `newTone` | done | same test; rules: `a_colour_and_a_type_the_person_picked_come_back` |

## I. A date cell's calendar

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| I1 | It drops or rises as the tag list does (340 points); a box "Type a date", selected; the month and year ("November 2026") with ‹ and ›; S M T W T F S; whole weeks, the days of other months faded, today ringed, the cell's day filled | `DatePicker` | done | `a_tag_cell_opens_its_list_with_one_press_and_a_date_cell_its_calendar`; scene `table-date` (0.39) |
| I2 | What is typed is read as a day and the calendar turns to it; ↑ and ↓ move it a day; Enter writes it as the column writes days and moves down; Tab moves right or left; Escape closes; leaving with other words writes them | `DatePicker` keys | done | same test |
| I3 | Its foot: Today, and Clear when the cell has a day | `.tbl-cal-foot` | done | same test and scene |

## J. A cell's words shown whole

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| J1 | One cell picked whose words are cut short shows them whole over it, in the editor's place and look, read only, scrolling past 320 points, rising from the cell's foot near the view's; not for a tag, nor while a cell is edited, a row is open, a menu is open or something is moved; Escape puts it away until another cell is picked | `peek` | done | `a_cut_off_cell_picked_shows_its_words_whole_until_escape` (which found it stayed away for good after Escape; fixed) |

## K. A row opened whole

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| K1 | A panel on the table's right, 460 points (at most 92% of it): the row's name (its link, or "Untitled"), "3 of 8", and the row above, the row below (each off at its end) and close | `RowSheet` | done | `space_opens_the_row_whole_and_its_keys_walk_the_rows`; scene `table-row` (0.80) |
| K2 | Every column as a field: its mark and name, then a box that grows with its words and is written when left (Escape drops what was typed, ⌘Enter goes back to the panel), a tag's button with its list, a date's button with its calendar, or its words where it cannot be changed; "Empty", "Pick a tag", "Pick a date" | `Field` | done | `a_field_of_the_row_opened_whole_is_written_when_left`; scene `table-field` (0.92; the same save as the web's, word for word) |
| K3 | Opened from a cell, that cell's field is in the middle and its name brighter | `from` | done | `space_opens_the_row_whole_and_its_keys_walk_the_rows` |
| K4 | Escape closes it; ↑ and ↓ open the row above or below, the pick following; ⌘Z and ⌘⇧Z take back and do again | `RowSheet` keys | done | same test |

## L. A column's menu

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| L1 | Opened by the header's caret, under it, or by a right press, at the pointer; kept 8 points inside the window; 220 wide | `FloatMenu` | done | `a_header_and_a_cell_open_their_menus_and_the_keys_walk_them`; scene `table-column-menu` (0.44) |
| L2 | Sort A to Z and Sort Z to A (1 to 9 and 9 to 1 for numbers, Oldest first and Newest first for days); Stop sorting while sorted by it | the column `FloatMenu` | done | same test |
| L3 | Where it can be changed: Rename (the header becomes a box, all selected: Enter or leaving keeps it, Escape drops it; a name another column has gets a number; its looks follow); Type, saying the type, which opens the types (a back row with the column's name; Text, Tags, Number, Date, Link, Email, each with its mark, the column's ticked; the name column counts as Text; a rank has no Type); Insert column left and right (an empty text column, then renamed at once); Move left (⌥←) and Move right (⌥→), off at the ends; Fit to its contents; Wrap text for a column of words, ticked while on; Delete column, off with one column left | the column `FloatMenu` | done | same test, `rows_and_columns_are_added_moved_and_taken_away`; scene `table-type-menu` (0.63) |
| L4 | Where it cannot: Fit to its contents and Wrap text | the column `FloatMenu` | done | `a_sheet_that_cannot_be_changed_is_read_and_never_written` |
| L5 | ↑ and ↓ walk its rows around both ends past the ones that are off, Enter presses the one reached, Escape closes; a press anywhere else closes it, and so does the window losing the focus | `FloatMenu` keys | done | `a_header_and_a_cell_open_their_menus_and_the_keys_walk_them` |
| L6 | After a choice the keys are the table's again, unless the choice opened a box (Rename) | `closeFloat` | done | same test |

## M. A cell's menu

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| M1 | Comment, where the file takes notes, then a line | `onComment` | not done | Not Rune: this window has no notes on files at all (no notes column beside a file, here or for any other kind), so a Comment row would lead nowhere. Shown only once those notes are |
| M2 | Open row (Space); Edit cell or Read cell (↵); Copy (⌘C); Paste (⌘V, where it can be changed; "Press ⌘V to paste here." when the clipboard cannot be read); Open link or Write an email (⌘click) on a cell that goes somewhere | the cell `FloatMenu` | done | `a_header_and_a_cell_open_their_menus_and_the_keys_walk_them`, `a_sheet_that_cannot_be_changed_is_read_and_never_written`; scene `table-cell-menu` (1.51: the web's Comment row). The refusal words never show: Rune reads the clipboard without asking, so it is never refused |
| M3 | Where it can be changed: Insert row above; Insert row below (⇧↵); Move up and Move down (⌥↑, ⌥↓; under a sort "Sorted" and off; off at the ends); Clear (⌫); Delete row, or Delete 3 rows | the cell `FloatMenu` | done | same tests; `rows_and_columns_are_added_moved_and_taken_away` |

## N. Changes, undo, and what is kept

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| N1 | Each change is a new table, and undo and redo walk 200 of them; one that changed a filter takes it back too; undoing a column moved takes the sort and the filters back with it | `commit`, `travel` | done | `undo_and_redo_walk_the_changes` |
| N2 | A row put in among sorted rows goes after the row above it in the file; at the foot it goes last; in the view it stands where it was asked for | `addRowAt`, `orderAfterAdd` | done | `rows_and_columns_are_added_moved_and_taken_away`; rules: `rows_and_columns_added_and_taken_away` |
| N3 | Rows deleted leave the pick on the row that comes into their place | `dropPicked`, `orderAfterDrop` | done | same tests |
| N4 | A ranked file whose ranks count 1, 2, 3 is counted again after rows come, go or move; ranks in another order stay | `countsUp`, `renumbered` | done | `a_ranked_file_shows_its_rank_as_the_row_number`, `rows_picked_whole_are_dragged_to_where_they_land`; rules: `rows_and_columns_move_and_the_file_is_written_in_the_new_order` |
| N5 | The colours on screen are pinned for the file when its cells change, so moving rows does not deal them out again | `pinTones` | done | rules: `a_colour_and_a_type_the_person_picked_come_back` |
| N6 | A new search, filter or sort works the order out again, goes back to the top, and lets go of the pick, the edit and the open row, the keys staying with the table; an edit does not move its row out from under the caret | `viewOrder` effect | done | the filter and sort test, `a_cell_typed_over_is_written_back_in_the_files_own_shape` |
| N7 | The filter and the sort are kept for the file by column name, the widths a hand set too, and what the person chose that a sheet cannot hold (a column's type, a tag's colour, wrapping) | `keepView`, `keepWidths`, `keepLooks` | done | the filter and sort test, the grip test, the menus test (a type kept), the wrap test. Kept in this window's own picks file: the web window's are in its browser storage, which this window cannot read |
| N8 | Moving columns keeps the sort and the filters on their columns | `moveColsTo` | done | rules: `rows_and_columns_move_and_the_file_is_written_in_the_new_order` |

## O. Saving

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| O1 | A change to the cells or the headers is written 400 ms after the last one through the app, `zero:write-doc` with `{ product, src, text, mtime }` (and `thread` big); a colour, a type or wrapping is not written to the file | `commit`, `useTableSave` | done | `a_cell_typed_over_is_written_back_in_the_files_own_shape`, `undo_and_redo_walk_the_changes`; rules: `only_a_change_to_cells_or_headers_is_a_change_to_the_file`; scenes `table-typed`, `table-field` (what each window sent, word for word the same); the scratch engine |
| O2 | It is written in the file's own shape: its separator, its line ends, its byte order mark, a markdown table as a markdown table; the headers as the file had them; a cell quoted only when it holds the separator, a quote, a line break or space at either end (a header also when it holds a comma, a semicolon or a tab); a markdown cell with its line breaks as spaces and its pipes escaped; a line end at the end | `writeTable`, `shapeOf` | done | rules: `a_save_writes_the_file_back_the_way_it_came`; the scratch engine left the web's bytes on disk |
| O3 | It carries the time the file had when it was read, and the app refuses it when the file changed on disk since (by more than a second); after a save the time the save left is the one carried | `useTableSave`, `writeDoc` | done | `a_cell_typed_over_is_written_back_in_the_files_own_shape`; the scratch engine |
| O4 | Refused so: "This file changed on disk since it opened, so your edit is not saved yet.", Save mine (written whatever the disk holds) and Load the new one | `flush`, `.tbl-problem` | done | `a_save_the_app_refuses_says_so_and_save_mine_writes_it_anyway`, `load_the_new_one_reads_the_file_again` |
| O5 | Refused otherwise: the app's own words and Try again, the edit kept | `flush` | done | `a_save_the_app_refuses_says_so_and_save_mine_writes_it_anyway` |
| O6 | A table left with an edit not yet written writes it; refused then, a line says "Your last edit to leads.csv was not saved." with the reason | `flush` on unmount, `onNotice` | done | `an_edit_not_yet_written_is_written_when_the_sheet_is_put_down` |
| O7 | A save that goes through takes the problem away | `flush` | done | `a_save_the_app_refuses_says_so_and_save_mine_writes_it_anyway` |

## P. Teammates

| # | Behaviour | Web | Native | How checked |
|---|---|---|---|---|
| P1 | In a team, a thread's table is one live copy: everyone's cells merge, only a copy that can write the file saves, the others' picks are marked on the cells | `live`, `useCellMarks` | not done | Not Rune: as C11, this window has no live connection to teammates, so its table is always the one copy that writes the file. It needs that connection first |
