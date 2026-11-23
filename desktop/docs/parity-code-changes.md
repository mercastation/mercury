# The code a thread changed: every behaviour of the web window's, and this window's

The web window's view of the code a run changed, read off its source, line by line, so the port
can be held to all of it and not to its look alone. Each line names where the web window does
it. A line is ticked once this window does the same, with how that was checked; a line left
unticked says exactly what stops it.

Sources: `renderer/src/components/Focus.tsx` (the figures, the button, when they are read),
`renderer/src/threads/ThreadMenu.tsx` (the button, the menu row), `renderer/src/threads/summary.css`
(the button's look), `renderer/src/change-figures.ts`, `renderer/src/components/DocPane.tsx` (the
pane around a change), `renderer/src/components/CodeArtifact.tsx` (the view),
`renderer/src/code-artifact.ts`, `renderer/src/code-keys.ts`, `renderer/src/code-copy.ts`,
`renderer/src/code-edit.ts`, `renderer/src/styles.css` (`.code-*`, `.cr-*`, `.t-*`, `.doc-code`),
`renderer/src/workspace-navigation.css` (the narrow pane's picker), `renderer/src/App.tsx` (keys
while a change is open, the full workspace, Escape), `renderer/src/components/ItemThread.tsx` and
`Thread.tsx` (the chip on a run's step), `shared/work-lines.mts` (`fileInChange`),
`renderer/src/thread-commands.ts` (the command bar's row), `renderer/src/ship-offer.ts` (how long a
finished run's figures are read again), and on the engine's side `main/ipc.mts`
(`zero:code-change`, `zero:code-file`, `zero:save-code-file`), `main/code-change.mts`,
`main/git-change.mts` and `main/code-file.mts`.

How each line was checked is said after it: a test in `src/change.rs` or `src/rules/change.rs`
(named), a scene photographed beside the web window's (`thread-change*`, measured in the README),
the pointer walked over a scene (`MERCURY_FEEL`), the pointer resting on a control
(`tools/hover.py`), the scratch engine, or the code read against the web's where nothing else can
show it.

## A. The button in the head strip

- [x] A1. Drawn only when the thread's run left a change with something in it: the change file
  `runs/<thread id>/the-change-it-made.change` reads, holds one file or more, and adds or removes
  at least one line (`figuresFrom`). A thread that is one of the person's own agent sessions
  (`item.agent`) is never asked and never has one.
  *Checked:* `a_change_with_nothing_in_it_draws_no_button`, `the_figures_read_off_a_saved_change_or_say_nothing`,
  `an_agent_session_is_never_asked_and_has_no_button`.
- [x] A2. Where: in the strip's group after the Files button and before where it runs, Done or
  Reopen, and the dots (`ts-more-wrap`, ten points apart).
  *Checked:* scene `thread-change`, the strip 0.49 of 255 at 1280 by 760.
- [x] A3. What it says: the code mark (two chevrons, `m8.5 7-5 5 5 5`, `m15.5 7 5 5-5 5`, 16
  points, stroke 1.4) and the figures, `+36` in the added green and `−88` in the removed red,
  six points apart, in the mono face at 12.5 points, weight 560, tracking −0.015 em
  (`.ts-code-figures`). Thousands are separated (`+2,199`); a side that is zero is left out
  (`−0` is never drawn); the minus is U+2212 (`signed`, `figure`). The count of files is not on
  the button.
  *Checked:* the button 0.59 in `thread-change`; `the_figures_read_off_a_saved_change_or_say_nothing`.
- [x] A4. Its frame is Done's: 32 tall, nine in on the left and twelve on the right, eight
  between the mark and the figures, the strong line, the tag's corners, the dim ink
  (`.ts-done.ts-code`).
  *Checked:* the button 0.59 in `thread-change`.
- [x] A5. Under the pointer: the text's ink and the strong wash, at once both ways
  (`.ts-done:hover`, no transition). The pointing hand.
  *Checked:* `tools/hover.py` on `thread-change-hover`, 1.06 under the pointer against 0.58 at
  rest, the glyphs' edges; the hand by `MERCURY_FEEL`.
- [x] A6. Pressed look while the change is the file open beside the thread (`aria-pressed`): the
  text's ink and the strong wash, pointer or not.
  *Checked:* scene `thread-change-open`, the strip within the conversation's 1.11.
- [x] A7. A press opens the change beside the conversation; pressed while it is open, it puts the
  change away (`openChange`: `changeOpen ? onCloseDoc() : onOpenDoc(changePathFor(item.id))`).
  Opened from the button it opens at its top, never at a file.
  *Checked:* `the_button_says_the_figures_opens_the_change_and_puts_it_away`.
- [x] A8. It carries no key hint and no title; its name for assistive technology is "View code
  changes".
  *Checked:* read in `button()`: a label and no help.
- [x] A9. The thread's name in the strip gives way to it, cut short with dots, as it does to the
  Files button.
  *Checked:* the button stands in the strip's own group, as the Files button does (`thread.rs`),
  whose name gives way to the group. No scene has a title long enough to be cut.

## B. What is read, and when it is read again

- [x] B1. The button's figures: `zero:code-change` with `summary: true` (the change without its
  rows), asked when the conversation opens, again whenever the thread's row changes
  (`item.updatedAt`) and when a run starts on it (`session.startedAt`).
  *Checked:* `the_figures_are_asked_for_on_a_new_reading_and_every_four_seconds_while_the_folder_moves`;
  a new reading is another thread, its row's time or its run's start (`ask_for_the_summary`).
- [x] B2. While a run is up on the thread, asked again every four seconds; once it has ended,
  every four seconds for two minutes after its answer was written (`folderMayStillMove`,
  `FOLDER_SETTLES_MS` = 120 000), and then no more. A thread with neither is asked once.
  *Checked:* the same test, and `a_finished_runs_figures_are_read_for_two_minutes`.
- [x] B3. The chips on a run's steps (H below) read the same summary on the conversation's own
  beat: when it opens, on every change the engine pushes and every eight seconds while a run is
  up (`ItemThread.tsx`).
  *Checked:* one summary serves both, asked on every push while a run is up, in the same test.
  Its four seconds are at least as often as the chips' eight.
- [x] B4. The view itself: `zero:code-change` without `summary`, once, when the change opens
  beside the conversation (`DocPane`'s effect on `doc.src`). The engine's answer carries where
  the folder stands now (`withStandsNow`): "not committed" is read off that answer.
  *Checked:* `the_button_says_the_figures_opens_the_change_and_puts_it_away`,
  `a_change_is_read_off_the_engines_answer`, `the_words_on_the_change`.
- [x] B5. While the view is open it is not read again: an agent still writing shows its new lines
  only when the change is opened again. The button's figures do move (B1, B2).
  *Checked:* read: `change_opened` is called only when the pane opens the change.
- [x] B6. A file's lines between two hunks are read when that gap is pressed (`zero:code-file`),
  each time, never kept on disk.
  *Checked:* `a_gap_opens_onto_the_files_own_lines` (a picture's planted file); live, the engine
  is asked on the channel and with the words the save's read uses, which the scratch engine
  answered (`a_save_goes_through_a_running_engine`).
- [x] B7. Nothing is written until ⌘S (J below), and then only through `zero:code-file` (read)
  and `zero:save-code-file` (write).
  *Checked:* `a_line_takes_typing_keeps_it_and_says_unsaved` (the file untouched until ⌘S),
  `a_save_goes_through_a_running_engine`.

## C. The pane around a change

- [x] C1. A change opens in the pane beside the conversation, which narrows as it does for any
  file there (the split, the divider, its double press, the card under a thousand points taking
  the whole card); the pane's rules are shared with every other file.
  *Checked:* the pane's own tests; scenes `thread-change-open` at three sizes.
- [x] C2. Flush, not on the glass card: no air, no border, no corners; the code's own ground
  (`--code-ground`, #0e0f12 in the dark) under the head and the change; the divider's hairline in
  `--line` (`.doc-pane.doc-code`).
  *Checked:* the card 0.42 in `thread-change-open` at 1280 by 760, 0.61 at 1600 by 1000.
- [x] C3. The head: `--code-ground`, a line under it, twelve in on the right, eighteen on the
  left, twelve over and eleven under (`.doc-pane.doc-code .doc-head`).
  *Checked:* within the card's 0.42.
- [x] C4. The path says "The change" for a run's change (`runs/w-<hex>/the-change-it-made.change`);
  any other `.change` its own crumb.
  *Checked:* `opens_a_change_and_nothing_else`; the head in every scene.
- [x] C5. Its marks: the frame's corners ("Give this document the full workspace"), the open mark
  ("Open this file in your editor") and close ("Close this document (esc)"). No globe (a change
  is not shared, `canShareFile`) and no notes' mark (`takesNotesSidebar`).
  *Checked:* the head in the scenes; each mark's pointer by `MERCURY_FEEL`.
- [x] C6. The open mark opens the file of the change at the top of the column now (`fileAt`, from
  the scroll, E3), not the change file; before any file is current, the change file itself. Found
  as the engine finds it (`resolveArtifact`), handed to the app that owns it; a refusal is said in
  a line.
  *Checked:* `the_button_says_the_figures_opens_the_change_and_puts_it_away` (a deleted file
  refused in the engine's words, then the file the column stands on),
  `a_change_that_cannot_be_read_says_why_and_one_on_its_way_holds_its_place` (the change file).
- [x] C7. While the change is read: the skeleton that holds a document's place (`PreviewLoading`:
  a title bar, two lines, a block, two lines, breathing over 2.4 s, still under Reduce motion).
  *Checked:* the same test; scene `thread-change-loading`, the card 0.68.
- [x] C8. When the change cannot be read: the engine's reason, or "That change could not be read.",
  in the faint ink, 13.5 points, forty in (`.doc-missing`). When the change file is nowhere:
  "That file could not be found."
  *Checked:* the same test; scene `thread-change-failed`, the card 0.25. The engine's words are
  drawn as it says them, its "could not be found" included.
- [x] C9. The frame's corners give the change the whole workspace: the pane takes the card, the
  conversation is hidden and only the reply box stays under it (`artifactMode = 'focus'`, the
  change keeping "the pane's own full screen", `fullScreenTakes` false). Its head moves up into
  the strip; the corners then read "Exit Focus" with the title "Move it to the side · Esc closes
  it" and turn inward; a press on them, on close or Escape puts it back beside the conversation.
  *Checked:* `the_corners_give_it_the_whole_workspace_and_escape_puts_it_back_beside`,
  `the_card_stands_over_the_marks_where_it_covers_them`,
  `the_marks_over_the_whole_workspace_sleep_until_the_pointer_moves`,
  `a_change_a_narrow_window_leaves_the_card_to_has_no_marks`; scenes `thread-change-full` (0.39
  and 0.36) and `thread-change-full-marks`.
- [x] C10. Escape with the change open beside the conversation puts it down (a change is always
  opened by a press); with it over the whole workspace, Escape puts it back beside first.
  *Checked:* `its_keys_are_its_own_j_and_k_do_nothing_and_escape_puts_it_down` and the corners'
  test.
- [x] C11. Leaving the thread takes the change with it, and coming back with nothing new puts it
  back, as for any file in the pane.
  *Checked:* the pane's own tests; coming back reads the change again, as the web's pane mounts
  again.

## D. The files, as a tree, and the narrow pane's picker

- [x] D1. Down the change's left, 232 points wide, with a line on its right (`.code-tree`).
  *Checked:* the tree 0.27 to 0.32 at 1600 by 1000.
- [x] D2. Its head: "11 files +36 −88" (`changeSummary`: "1 file" for one, the figures without
  thousands separators and with both sides always), then "not committed" when the run's work
  reached no commit and is still loose in the folder (`standingLabel`), then "in 6 minutes" when
  the run's length is known (`tookLabel`: "under a minute", "1 minute", "N minutes", "1 hour",
  "N hours"). 11.5 points, the faint ink, the count in the dim ink; wrapping between the three,
  never inside one; four over, twelve right, ten under, eighteen left.
  *Checked:* `the_words_on_the_change`; the tree's pictures.
- [x] D3. The rows: folders before files at every level, each sorted by name; a chain of folders
  with one child each is one row (`renderer/src`); a folder row names how many files are under
  it (`treeRows`). The files are drawn in the column in the tree's order (`filesInTreeOrder`).
  *Checked:* `the_changed_files_are_a_tree`, `a_narrow_pane_has_the_picker_and_a_wide_one_the_tree`.
- [x] D4. A row is 12.5 points, five over and under, twelve right, `12 + depth × 13` from the
  left, eight between its parts, its name cut short with dots. A folder in the faint ink with a
  chevron (13 points) turned down while open and right while shut, and its count in mono 10.5
  faint; a file in the dim ink with its counts in mono 10.5, six apart, green and red, each only
  when not zero; a file's title is its whole path.
  *Checked:* the tree's pictures.
- [x] D5. The file at the top of the column (E3) is lit: the strong wash, the text's ink and a
  bar two points wide down its left edge in `--select-bar` (`.code-file.is-at`).
  *Checked:* scene `thread-change-at` at 1600 by 1000, the tree 0.31.
- [x] D6. A file folded shut (F5) has its name and counts at 0.45 (`.code-file.is-folded`).
  *Checked:* scene `thread-change-folded`, the tree 0.28.
- [x] D7. A file with unsaved words (J) shows a green dot, mono 14, where its counts were
  (`.code-unsaved`), with the title "Not saved yet. Press ⌘S."
  *Checked:* scene `thread-change-typed`, the tree 0.32.
- [x] D8. Under the pointer a row washes (`--wash`), at once both ways; the pointing hand.
  *Checked:* the hand by `MERCURY_FEEL`; the wash read in `tree_row` (no scene rests the pointer
  on a row).
- [x] D9. A press on a folder shuts or opens it, hiding everything under it and nothing beside it
  (`visibleRows`); it moves only the tree, never the code (`at` indexes the files).
  *Checked:* `a_press_on_a_tree_row_jumps_and_the_arrows_then_walk_the_files`; scene
  `thread-change-shut`, the tree 0.27.
- [x] D10. A press on a file jumps the column to it (E4), every press, the current file included.
  *Checked:* the same test.
- [x] D11. The tree's rows scroll on their own (eight under the last), and the lit row is kept in
  view whether it was reached by a key, a press or the column's scroll.
  *Checked:* read: the tree scrolls, and the lit row is kept on screen from each of the three.
- [x] D12. No key legend anywhere, here or as a hover, menu or "?".
  *Checked:* none is drawn.
- [x] D13. A pane 800 points wide or less (the pane, not the window) has no tree: a picker stands
  over the column instead, eight over and under and twelve in, the whole width, mono 12 on lines
  of 1.5, the dim ink, a line round it with the app's corners on `--code-ground`, eight in and 34
  on the right where its own chevron stands (13 points, faint, twelve in from its edge, centred),
  each file its path and " · Unsaved" when it has unsaved words. Choosing one jumps to it (E4).
  It shows the file at the top of the column. (`.code-file-pick`, `@container artifact`.)
  *Checked:* `a_narrow_pane_has_the_picker_and_a_wide_one_the_tree`; the picker 0.41, 0.54 and
  0.24 at 900 by 760; its arrow pointer by `MERCURY_FEEL`.

## E. The column: one running column of every file

- [x] E1. Each file is a block: its head, then its hunks, ten under the block.
  *Checked:* the column 0.41 at 1280 by 760, 0.71 at 1600 by 1000.
- [x] E2. The head stays at the top of the column while its file is in view and is pushed off by
  the next file's head (`position: sticky`): `--code-ground`, a line under it, nine over and
  under, fourteen right, sixteen left, ten between its parts. A chevron (13 points, faint, four
  points left of its place) turned a quarter back while the file is folded, turning over 120 ms
  (`transition: transform 120ms ease`); the file's name, 12.5 weight 600, dim (the text's ink
  for the file at the top, E3); its folder, 11.5 faint, cut short with dots; its counts, mono
  10.5; "Unsaved" with a dot before it, 11 points, the text's ink, at the right end while it
  has unsaved words. Nothing else on the row. A folded file's head has no line under it.
  *Checked:* scenes `thread-change-at` (a stuck head), `-folded`, `-typed`; the turn read as a
  120 ms tween on the chevron.
- [x] E3. The file at the top of the column is the last whose block has come within twelve points
  of the top, or the first before any has (`fileOnScreen`); it lights its tree row (D5), names
  itself in the picker (D13), lights its head's name and is what the open mark opens (C6). It
  follows the scroll, except while a jump is landing.
  *Checked:* `the_file_at_the_top_of_the_column`; `a_large_change_builds_only_the_rows_near_the_view`.
- [x] E4. A jump (a tree row, the picker, a chip opening the change at a file) puts the file's
  block at the top of the column, held there while the rows under it settle their heights, and
  let go by the person's own wheel, press or key.
  *Checked:* `a_chip_opens_the_change_at_its_file_every_press`, the tree's test, scene
  `thread-change-at`; the hold let go read in the wheel, press and key handlers.
- [x] E5. The head folds its file: a press anywhere on it shuts the file's code away, another
  opens it again; its title says "Fold this file" or "Show this file"; the pointing hand. Folding
  the file whose head is stuck at the top keeps its head where it was. Folds are kept for the
  change while the app runs, and forgotten on a restart (`foldsFor`).
  *Checked:* `a_file_folds_from_its_head_and_stays_folded_while_the_window_runs`.
- [x] E6. A hunk's block: eight over it; its rows two in over and under.
  *Checked:* the column's pictures.
- [x] E7. Between two hunks whose lines git numbered and that do not touch, one quiet line:
  "N lines not shown", mono 11 faint, a hairline 44 wide before it and one to the end after it,
  three over and under, fourteen right, two around (`linesSkipped`, `.code-skip`). Under the
  pointer its words take the dim ink and its hairlines the strong line, at once; the pointing
  hand; the title "Show these lines".
  *Checked:* `the_gap_counts_the_new_files_lines_not_drawn`; scene `thread-change-open`; the hand
  by `MERCURY_FEEL`.
- [x] E8. A press on it reads the file as it is on disk now and draws the lines it stood for in
  place, numbered from where the gap starts, read only, uncoloured by a bed (`hiddenLines`).
  Refused with a line when the file has moved on since ("This file has changed since, so the lines
  in between could not be placed. Open it in your editor to read them."), when the change carries
  no numbers ("This change does not say which lines it skipped.") or when the file cannot be read
  (the engine's reason: not part of the change, not on this machine, too big to edit here, no
  longer there). Opened gaps stay open while the change stays open, folding the file included.
  *Checked:* `a_gap_opens_onto_the_files_own_lines`, `the_hidden_lines_open_in_place`; scene
  `thread-change-gap`, the column 0.61.
- [x] E9. A row: its number (44 wide, ten before the mark, right aligned, tabular figures, the
  faint ink), its mark (22 wide, centred: `+` green, `−` red, nothing on an unchanged line), and
  its line (the rest, fourteen short of the right edge), all mono 12 on lines of 1.55.
  *Checked:* the column's pictures.
- [x] E10. A removed line is numbered in the old file, every other line in the new one; a change
  read from the conversation rather than git has no numbers and draws the gutter empty.
  *Checked:* `a_change_is_read_off_the_engines_answer`; the pictures.
- [x] E11. An added line lies on a green bed (`--code-plus-bed`), a removed one on a red bed.
  *Checked:* the pictures.
- [x] E12. Inside a removed line and the added line that replaced it, the words that changed lie
  on a stronger bed of the same colour (32% of it), rounded to two (`wordChanges`, `marked`): a
  run of removed rows paired in order with the run of added rows right after it; between their
  common start and common end; nothing when the two lines are the same or more than six tenths of
  either differs; nothing across an unchanged row; nothing on a line the person has edited.
  *Checked:* `the_words_that_changed_are_marked`, `the_colours_and_the_changed_words_are_byte_ranges_of_the_line`.
- [x] E13. Syntax colours, one line at a time (`tokenize`): comments (italic), strings, keywords,
  numbers, types and capitalised names, functions run or defined, names being made, constants in
  capitals; a `/** */` block's inner lines are comment; everything else the window's ink.
  *Checked:* the same test and the script colours' own (`src/rules/code.rs`).
- [x] E14. A long line wraps rather than running out of reach, keeping its leading spaces, and a
  word longer than the line breaks anywhere (`pre-wrap`, `overflow-wrap: anywhere`).
  *Checked:* `a_press_lands_where_the_line_wraps`; the invented large change's two-thousand
  character lines.
- [x] E15. Every row of a hunk is drawn, however long; nothing is hidden behind a button.
  *Checked:* `a_selection_from_the_first_file_to_the_last_copies_every_file_between` reads every
  row of forty files.
- [x] E16. The agent's sentence before an edit is not drawn over the change.
  *Checked:* none is drawn.
- [x] E17. A new file is all green rows numbered in the new file; a deleted file all red rows
  numbered in the old one, filed under its old name; a renamed file under its new name. A binary
  file and a rename with nothing changed are not in the change (the engine leaves them out).
  *Checked:* the scenes' deleted note and new file; what the engine leaves out never reaches the
  window.
- [x] E18. A huge change opens on its first screenful at once and the rest comes as it is needed,
  never freezing the window (`firstBatch`, `fillNext`, `content-visibility` per slice of 48
  rows).
  *Checked:* `a_large_change_builds_only_the_rows_near_the_view`; in a release build 20,000 rows
  open in 14.7 ms and scroll at 0.82 ms a frame (README).
- [x] E19. The column scrolls on its own.
  *Checked:* `its_keys_are_its_own_j_and_k_do_nothing_and_escape_puts_it_down`.

## F. The keys while a change is open

- [x] F1. The keys the change answers are kept from the app's own: ↑ ↓ ← →, Page Up, Page Down,
  Home, End, J, K, ] and [ (`CHANGE_OWNS`). So ↑ and ↓ do not walk the agent's options, and J and
  K do not go to another thread: they do nothing at all. Escape, E, R, Z, Enter, the numbers and
  every other key stay the app's.
  *Checked:* `the_keys_a_change_answers`, `its_keys_are_its_own_j_and_k_do_nothing_and_escape_puts_it_down`.
- [x] F2. ] and [ go to the start of the next change below the top of the column, or the one
  before, landing its rows just under the file's stuck head; a change already at the top is
  skipped (four points of slack); at either end nothing moves. A file not drawn yet counts as one
  change; a folded file has none (`nextChange`).
  *Checked:* the same window test and the rules' cases.
- [x] F3. Otherwise ↓ and ↑ move the column a row (19 points), Page Down and Page Up a screenful
  less two rows, Home and End and ⌘↑ ⌘↓ to its ends, never past them (`codeScroll`).
  *Checked:* `the_arrows_scroll_the_code`.
- [x] F4. After a press on a tree row, the arrows, Home and End walk the files (`stepInTree`,
  `walkFiles`): down and up one, stopping at both ends; Home and End the first and last; a file
  hidden in a shut folder steps to the nearest shown one.
  *Checked:* `the_arrows_walk_the_files_and_stop_at_the_ends` and the tree's window test.
- [x] F5. ⌘S saves every file with unsaved words (J); it is heard from inside a line being typed
  in too.
  *Checked:* `a_line_takes_typing_keeps_it_and_says_unsaved`, the save tests.
- [x] F6. Escape inside a line being typed in lets go of the line, keeping its words; the next
  Escape is the pane's (C10).
  *Checked:* `a_line_takes_typing_keeps_it_and_says_unsaved`.

## G. Selecting and copying

- [x] G1. The code's words can be selected, removed and opened-gap lines too; the numbers and the
  marks cannot (`user-select: text` on `.code-line`, none on the gutter).
  *Checked:* `a_copy_across_lines_leaves_the_removed_line_out`,
  `a_drag_from_the_margin_takes_the_lines_from_their_front` (the gutter never copied).
- [x] G2. A drag selects across lines, files and hunks; a press in the margin, on a number or a
  mark, starts the selection at the front of that line.
  *Checked:* the same two tests, and
  `a_selection_from_the_first_file_to_the_last_copies_every_file_between`. A drag that starts
  inside the line holding the caret stays in it (G3).
- [ ] G3. A shifted press extends the selection from where the last plain press landed, across
  lines.
  *Partly:* from a press on a removed line, a gap's line, a line not yet holding the caret or the
  margin, it does (`a_shifted_press_extends_the_selection_across_lines`, and across far files in
  the test above). From the line holding the caret, a shifted press on another line lets go of
  the line and selects nothing, and a drag from inside it stays in it. *Missing in Rune:* a way
  for a window to set a selection that runs across two texts (`Tx::select_text` takes one); the
  line holding the caret is a field, outside the texts a selection runs through. *Closest:* the
  press before the shifted one made anywhere but the line holding the caret.
- [ ] G4. Shift with an arrow, Home or End extends a selection from a line across into the lines
  above and below, keeping its column, off the end of a line onto the next and back, stopping at
  both ends (`extendHead`).
  *Partly:* inside the line holding the caret, as the field extends it. `extend_head` is ported
  and held to the web's cases, and not called. *Missing in Rune:* the same: no selection set by
  the window across two texts. *Closest:* a drag or a shifted press crosses lines.
- [ ] G5. A double press takes a word, a triple press the line.
  *Partly:* on a removed line and a gap's line, at once; on a line holding the caret, at once
  (`a_double_press_takes_a_word_and_a_triple_press_the_line`). On a line that takes typing and
  does not hold the caret yet, the first press makes it a field and Rune counts the field's
  presses from its own first, so the word comes with the third press and the line with the
  fourth. *Missing in Rune:* a press run carried into the element that replaces the one pressed;
  a press on a field reaches no handler of the window's to count it. *Closest:* one press more.
- [x] G6. ⌘C copies the lines joined by newlines, the first and last clipped where the selection
  starts and ends, with no gutter mark and no number (`copiedText`). The removed lines are left
  out when any other line is selected; when only removed lines are, they are what is copied
  (`linesToCopy`). A selection inside one line is copied as it is.
  *Checked:* `copying_code_out_of_a_change`, the window's copy tests; the lines between two far
  ends are read from the change, as the web window has drawn every row by then.
- [x] G7. The selection is drawn in the app's own selection colour.
  *Checked:* read: the column's texts take the app's selection colour.

## J. Typing into a change

- [x] J1. Every line that is in the file now (added and unchanged ones) takes typing, as an editor
  does; a removed line and an opened gap's lines do not. It looks like the rest until it has the
  caret: then the text cursor, a caret in the added green and a faint bed under the line
  (`--code-live-bed`).
  *Checked:* `a_line_takes_typing_keeps_it_and_says_unsaved`, `editable_rows` in the rules;
  scene `thread-change-typed`; the I-beam by `MERCURY_FEEL`.
- [x] J2. A press on such a line puts the caret where it landed.
  *Checked:* the same test, `a_press_lands_where_the_line_wraps`. Where a wide character stands
  before it, see the README's "What still differs".
- [x] J3. Return is refused (a row is one line of the file); a paste comes in as plain text with
  its line breaks turned into spaces.
  *Checked:* the typing test (Return); the paste read in the field's paste handler.
- [x] J4. ↑ and ↓ move the caret to the line above or below at the same column; ← at a line's front
  goes to the end of the line above, → at its end to the front of the line below; at the first or
  last line it stays. Removed lines are skipped.
  *Checked:* `the_caret_crosses_lines` and the typing test.
- [x] J5. While a selection covers more than one line, a keystroke that would change the code
  (a letter, Backspace, Delete, Return, a paste, a cut, a drop) is refused and the selection stays
  for ⌘C (`editsAcrossLines`, `inputChangesText`).
  *Checked:* `an_edit_across_lines_is_refused`; the field lets go when a selection takes more than
  one line.
- [x] J6. The words typed are kept while the app runs, the pane closed and opened again included,
  never on disk unsaved; folding and opening a file shows them. A file with them is marked
  Unsaved on its head (E2), in the tree (D7) and in the picker (D13).
  *Checked:* the typing test; scene `thread-change-typed` at 1280, 1600 and 900.
- [x] J7. ⌘S, for each file with unsaved words: the file is read as it is now (`zero:code-file`),
  each hunk's lines found in it exactly as the run left them and swapped for the typed ones
  (`applySplices`: the first place a repeated block sits; applied from the foot of the file up;
  its last line break kept), and written (`zero:save-code-file`, with the time it was read).
  Then "Saved name.ts, 2 lines." for each file in one line, and the drawn rows take the saved
  words. Nothing is written when nothing changed.
  *Checked:* `typing_into_a_change`, `save_puts_the_typed_lines_back_where_the_run_left_them_or_says_why_not`,
  and against a running scratch engine, `a_save_goes_through_a_running_engine`.
- [x] J8. Refused, with its reason in a line and nothing written: the run removed those lines
  ("Those lines were removed by the run, so there is nothing in the file to save them onto."),
  the file moved on ("The file has changed since this run, so those lines are not in it any more.
  Nothing was saved."), two edits overlap ("Two of those edits overlap in the file, so nothing was
  saved."), the file cannot be opened, or the engine refuses the write (outside the run's folders,
  changed while typing, gone).
  *Checked:* the same tests; the scratch engine refused a file outside the run's folders and one
  moved while typing in its own words.

## H. Opening the change at a file, from a run's step

- [x] H1. In a run's opened fold, a step that edited or wrote a file (Edit, Write, MultiEdit,
  NotebookEdit) whose file the thread's change holds wears that file as a chip
  (`fileInChange`: the whole path or the change's key as its end at a folder boundary, the longest
  match, `/private/tmp` read as `/tmp`). A file the change does not hold is no chip.
  *Checked:* `a_steps_file_is_a_chip_only_when_the_change_holds_it` (window and rules).
- [x] H2. The chip: a page mark nine by eleven in the faint ink, seven before the path the line
  showed, the dim ink; one point over and under, six in, a transparent edge; under the pointer the
  line's edge, the wash, the text's ink, the mark dim; the title "Open <file> in the code". The
  line's own subject moves into the chip; its note of how many lines came back follows it.
  *Checked:* scene `thread-change-chip`; its pointer by `MERCURY_FEEL`.
- [x] H3. A press on the chip opens the change beside the conversation at that file (E4); pressing
  the same chip again after walking away lands again (`atFrom`). The step's own press still folds
  it open.
  *Checked:* `a_chip_opens_the_change_at_its_file_every_press`.

## K. The command bar and the thread's menu

- [x] K1. ⌘K offers "View code changes" on the open thread while its page draws the button
  (keywords: code changes diff files what changed); it opens the change (never closes it).
  *Checked:* `the_command_bar_offers_view_code_changes_on_a_thread_with_one`,
  `a_thread_with_a_change_has_its_code_row_after_later_and_found_by_diff`.
- [x] K2. The thread's menu's "View code changes" row is drawn only where the corner has no
  button, which is never while the button is drawn: there is nothing to draw.
  *Checked:* read: `change: !!change && !codeButton` in `ThreadMenu.tsx`.

## L. What the web window's view does not have

- [x] L1. No split (side by side) layout and no switch for one: one running column.
- [x] L2. No notes on a line, and no box to write one: a change takes no notes sidebar.
- [x] L3. No "Edited … ago", no versions and no Restore.
- [x] L4. Nothing says that rows were left out of a file past 20 000 (`leftOut` is not drawn).

  *Checked:* none of these is drawn here either.

## Checked how

- Tests: `cargo test` in `desktop/`, the window's in `src/change.rs` and the rules' in
  `src/rules/change.rs` (the web's own cases), the bar's row in `src/rules/palette.rs`.
- Pictures: each `thread-change*` scene photographed in the web window and drawn here, at 1280 by
  760, 1600 by 1000 and 900 by 760; the measures are in the README.
- The pointer: `MERCURY_FEEL` over every scene, and `tools/hover.py` on the two that rest it.
  The pointer tool runs no poll, so the scenes that open at a file were walked again with the
  jump landed.
- Writes: only on a scratch engine (`a_save_goes_through_a_running_engine`).
- Speed: `a_big_change_opens_in`, in a release build.
