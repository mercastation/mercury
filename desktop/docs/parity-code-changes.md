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

## A. The button in the head strip

- [ ] A1. Drawn only when the thread's run left a change with something in it: the change file
  `runs/<thread id>/the-change-it-made.change` reads, holds one file or more, and adds or removes
  at least one line (`figuresFrom`). A thread that is one of the person's own agent sessions
  (`item.agent`) is never asked and never has one.
- [ ] A2. Where: in the strip's group after the Files button and before where it runs, Done or
  Reopen, and the dots (`ts-more-wrap`, ten points apart).
- [ ] A3. What it says: the code mark (two chevrons, `m8.5 7-5 5 5 5`, `m15.5 7 5 5-5 5`, 16
  points, stroke 1.4) and the figures, `+36` in the added green and `−88` in the removed red,
  six points apart, in the mono face at 12.5 points, weight 560, tracking −0.015 em
  (`.ts-code-figures`). Thousands are separated (`+2,199`); a side that is zero is left out
  (`−0` is never drawn); the minus is U+2212 (`signed`, `figure`). The count of files is not on
  the button.
- [ ] A4. Its frame is Done's: 32 tall, nine in on the left and twelve on the right, eight
  between the mark and the figures, the strong line, the tag's corners, the dim ink
  (`.ts-done.ts-code`).
- [ ] A5. Under the pointer: the text's ink and the strong wash, at once both ways
  (`.ts-done:hover`, no transition). The pointing hand.
- [ ] A6. Pressed look while the change is the file open beside the thread (`aria-pressed`): the
  text's ink and the strong wash, pointer or not.
- [ ] A7. A press opens the change beside the conversation; pressed while it is open, it puts the
  change away (`openChange`: `changeOpen ? onCloseDoc() : onOpenDoc(changePathFor(item.id))`).
  Opened from the button it opens at its top, never at a file.
- [ ] A8. It carries no key hint and no title; its name for assistive technology is "View code
  changes".
- [ ] A9. The thread's name in the strip gives way to it, cut short with dots, as it does to the
  Files button.

## B. What is read, and when it is read again

- [ ] B1. The button's figures: `zero:code-change` with `summary: true` (the change without its
  rows), asked when the conversation opens, again whenever the thread's row changes
  (`item.updatedAt`) and when a run starts on it (`session.startedAt`).
- [ ] B2. While a run is up on the thread, asked again every four seconds; once it has ended,
  every four seconds for two minutes after its answer was written (`folderMayStillMove`,
  `FOLDER_SETTLES_MS` = 120 000), and then no more. A thread with neither is asked once.
- [ ] B3. The chips on a run's steps (H below) read the same summary on the conversation's own
  beat: when it opens, on every change the engine pushes and every eight seconds while a run is
  up (`ItemThread.tsx`).
- [ ] B4. The view itself: `zero:code-change` without `summary`, once, when the change opens
  beside the conversation (`DocPane`'s effect on `doc.src`). The engine's answer carries where
  the folder stands now (`withStandsNow`): "not committed" is read off that answer.
- [ ] B5. While the view is open it is not read again: an agent still writing shows its new lines
  only when the change is opened again. The button's figures do move (B1, B2).
- [ ] B6. A file's lines between two hunks are read when that gap is pressed (`zero:code-file`),
  each time, never kept on disk.
- [ ] B7. Nothing is written until ⌘S (J below), and then only through `zero:code-file` (read)
  and `zero:save-code-file` (write).

## C. The pane around a change

- [ ] C1. A change opens in the pane beside the conversation, which narrows as it does for any
  file there (the split, the divider, its double press, the card under a thousand points taking
  the whole card); the pane's rules are shared with every other file.
- [ ] C2. Flush, not on the glass card: no air, no border, no corners; the code's own ground
  (`--code-ground`, #0e0f12 in the dark) under the head and the change; the divider's hairline in
  `--line` (`.doc-pane.doc-code`).
- [ ] C3. The head: `--code-ground`, a line under it, twelve in on the right, eighteen on the
  left, twelve over and eleven under (`.doc-pane.doc-code .doc-head`).
- [ ] C4. The path says "The change" for a run's change (`runs/w-<hex>/the-change-it-made.change`);
  any other `.change` its own crumb.
- [ ] C5. Its marks: the frame's corners ("Give this document the full workspace"), the open mark
  ("Open this file in your editor") and close ("Close this document (esc)"). No globe (a change
  is not shared, `canShareFile`) and no notes' mark (`takesNotesSidebar`).
- [ ] C6. The open mark opens the file of the change at the top of the column now (`fileAt`, from
  the scroll, E3), not the change file; before any file is current, the change file itself. Found
  as the engine finds it (`resolveArtifact`), handed to the app that owns it; a refusal is said in
  a line.
- [ ] C7. While the change is read: the skeleton that holds a document's place (`PreviewLoading`:
  a title bar, two lines, a block, two lines, breathing over 2.4 s, still under Reduce motion).
- [ ] C8. When the change cannot be read: the engine's reason, or "That change could not be read.",
  in the faint ink, 13.5 points, forty in (`.doc-missing`). When the change file is nowhere:
  "That file could not be found."
- [ ] C9. The frame's corners give the change the whole workspace: the pane takes the card, the
  conversation is hidden and only the reply box stays under it (`artifactMode = 'focus'`, the
  change keeping "the pane's own full screen", `fullScreenTakes` false). Its head moves up into
  the strip; the corners then read "Exit Focus" with the title "Move it to the side · Esc closes
  it" and turn inward; a press on them, on close or Escape puts it back beside the conversation.
- [ ] C10. Escape with the change open beside the conversation puts it down (a change is always
  opened by a press); with it over the whole workspace, Escape puts it back beside first.
- [ ] C11. Leaving the thread takes the change with it, and coming back with nothing new puts it
  back, as for any file in the pane.

## D. The files, as a tree, and the narrow pane's picker

- [ ] D1. Down the change's left, 232 points wide, with a line on its right (`.code-tree`).
- [ ] D2. Its head: "11 files +36 −88" (`changeSummary`: "1 file" for one, the figures without
  thousands separators and with both sides always), then "not committed" when the run's work
  reached no commit and is still loose in the folder (`standingLabel`), then "in 6 minutes" when
  the run's length is known (`tookLabel`: "under a minute", "1 minute", "N minutes", "1 hour",
  "N hours"). 11.5 points, the faint ink, the count in the dim ink; wrapping between the three,
  never inside one; four over, twelve right, ten under, eighteen left.
- [ ] D3. The rows: folders before files at every level, each sorted by name; a chain of folders
  with one child each is one row (`renderer/src`); a folder row names how many files are under
  it (`treeRows`). The files are drawn in the column in the tree's order (`filesInTreeOrder`).
- [ ] D4. A row is 12.5 points, five over and under, twelve right, `12 + depth × 13` from the
  left, eight between its parts, its name cut short with dots. A folder in the faint ink with a
  chevron (13 points) turned down while open and right while shut, and its count in mono 10.5
  faint; a file in the dim ink with its counts in mono 10.5, six apart, green and red, each only
  when not zero; a file's title is its whole path.
- [ ] D5. The file at the top of the column (E3) is lit: the strong wash, the text's ink and a
  bar two points wide down its left edge in `--select-bar` (`.code-file.is-at`).
- [ ] D6. A file folded shut (F5) has its name and counts at 0.45 (`.code-file.is-folded`).
- [ ] D7. A file with unsaved words (J) shows a green dot, mono 14, where its counts were
  (`.code-unsaved`), with the title "Not saved yet. Press ⌘S."
- [ ] D8. Under the pointer a row washes (`--wash`), at once both ways; the pointing hand.
- [ ] D9. A press on a folder shuts or opens it, hiding everything under it and nothing beside it
  (`visibleRows`); it moves only the tree, never the code (`at` indexes the files).
- [ ] D10. A press on a file jumps the column to it (E4), every press, the current file included.
- [ ] D11. The tree's rows scroll on their own (eight under the last), and the lit row is kept in
  view whether it was reached by a key, a press or the column's scroll.
- [ ] D12. No key legend anywhere, here or as a hover, menu or "?".
- [ ] D13. A pane 800 points wide or less (the pane, not the window) has no tree: a picker stands
  over the column instead, eight over and under and twelve in, the whole width, mono 12 on lines
  of 1.5, the dim ink, a line round it with the app's corners on `--code-ground`, eight in and 34
  on the right where its own chevron stands (13 points, faint, twelve in from its edge, centred),
  each file its path and " · Unsaved" when it has unsaved words. Choosing one jumps to it (E4).
  It shows the file at the top of the column. (`.code-file-pick`, `@container artifact`.)

## E. The column: one running column of every file

- [ ] E1. Each file is a block: its head, then its hunks, ten under the block.
- [ ] E2. The head stays at the top of the column while its file is in view and is pushed off by
  the next file's head (`position: sticky`): `--code-ground`, a line under it, nine over and
  under, fourteen right, sixteen left, ten between its parts. A chevron (13 points, faint, four
  points left of its place) turned a quarter back while the file is folded, turning over 120 ms
  (`transition: transform 120ms ease`); the file's name, 12.5 weight 600, dim (the text's ink
  for the file at the top, E3); its folder, 11.5 faint, cut short with dots; its counts, mono
  10.5; "Unsaved" with a dot before it, 11 points, the text's ink, at the right end while it
  has unsaved words. Nothing else on the row. A folded file's head has no line under it.
- [ ] E3. The file at the top of the column is the last whose block has come within twelve points
  of the top, or the first before any has (`fileOnScreen`); it lights its tree row (D5), names
  itself in the picker (D13), lights its head's name and is what the open mark opens (C6). It
  follows the scroll, except while a jump is landing.
- [ ] E4. A jump (a tree row, the picker, a chip opening the change at a file) puts the file's
  block at the top of the column, held there while the rows under it settle their heights, and
  let go by the person's own wheel, press or key.
- [ ] E5. The head folds its file: a press anywhere on it shuts the file's code away, another
  opens it again; its title says "Fold this file" or "Show this file"; the pointing hand. Folding
  the file whose head is stuck at the top keeps its head where it was. Folds are kept for the
  change while the app runs, and forgotten on a restart (`foldsFor`).
- [ ] E6. A hunk's block: eight over it; its rows two in over and under.
- [ ] E7. Between two hunks whose lines git numbered and that do not touch, one quiet line:
  "N lines not shown", mono 11 faint, a hairline 44 wide before it and one to the end after it,
  three over and under, fourteen right, two around (`linesSkipped`, `.code-skip`). Under the
  pointer its words take the dim ink and its hairlines the strong line, at once; the pointing
  hand; the title "Show these lines".
- [ ] E8. A press on it reads the file as it is on disk now and draws the lines it stood for in
  place, numbered from where the gap starts, read only, uncoloured by a bed (`hiddenLines`).
  Refused with a line when the file has moved on since ("This file has changed since, so the lines
  in between could not be placed. Open it in your editor to read them."), when the change carries
  no numbers ("This change does not say which lines it skipped.") or when the file cannot be read
  (the engine's reason: not part of the change, not on this machine, too big to edit here, no
  longer there). Opened gaps stay open while the change stays open, folding the file included.
- [ ] E9. A row: its number (44 wide, ten before the mark, right aligned, tabular figures, the
  faint ink), its mark (22 wide, centred: `+` green, `−` red, nothing on an unchanged line), and
  its line (the rest, fourteen short of the right edge), all mono 12 on lines of 1.55.
- [ ] E10. A removed line is numbered in the old file, every other line in the new one; a change
  read from the conversation rather than git has no numbers and draws the gutter empty.
- [ ] E11. An added line lies on a green bed (`--code-plus-bed`), a removed one on a red bed.
- [ ] E12. Inside a removed line and the added line that replaced it, the words that changed lie
  on a stronger bed of the same colour (32% of it), rounded to two (`wordChanges`, `marked`): a
  run of removed rows paired in order with the run of added rows right after it; between their
  common start and common end; nothing when the two lines are the same or more than six tenths of
  either differs; nothing across an unchanged row; nothing on a line the person has edited.
- [ ] E13. Syntax colours, one line at a time (`tokenize`): comments (italic), strings, keywords,
  numbers, types and capitalised names, functions run or defined, names being made, constants in
  capitals; a `/** */` block's inner lines are comment; everything else the window's ink.
- [ ] E14. A long line wraps rather than running out of reach, keeping its leading spaces, and a
  word longer than the line breaks anywhere (`pre-wrap`, `overflow-wrap: anywhere`).
- [ ] E15. Every row of a hunk is drawn, however long; nothing is hidden behind a button.
- [ ] E16. The agent's sentence before an edit is not drawn over the change.
- [ ] E17. A new file is all green rows numbered in the new file; a deleted file all red rows
  numbered in the old one, filed under its old name; a renamed file under its new name. A binary
  file and a rename with nothing changed are not in the change (the engine leaves them out).
- [ ] E18. A huge change opens on its first screenful at once and the rest comes as it is needed,
  never freezing the window (`firstBatch`, `fillNext`, `content-visibility` per slice of 48
  rows).
- [ ] E19. The column scrolls on its own.

## F. The keys while a change is open

- [ ] F1. The keys the change answers are kept from the app's own: ↑ ↓ ← →, Page Up, Page Down,
  Home, End, J, K, ] and [ (`CHANGE_OWNS`). So ↑ and ↓ do not walk the agent's options, and J and
  K do not go to another thread: they do nothing at all. Escape, E, R, Z, Enter, the numbers and
  every other key stay the app's.
- [ ] F2. ] and [ go to the start of the next change below the top of the column, or the one
  before, landing its rows just under the file's stuck head; a change already at the top is
  skipped (four points of slack); at either end nothing moves. A file not drawn yet counts as one
  change; a folded file has none (`nextChange`).
- [ ] F3. Otherwise ↓ and ↑ move the column a row (19 points), Page Down and Page Up a screenful
  less two rows, Home and End and ⌘↑ ⌘↓ to its ends, never past them (`codeScroll`).
- [ ] F4. After a press on a tree row, the arrows, Home and End walk the files (`stepInTree`,
  `walkFiles`): down and up one, stopping at both ends; Home and End the first and last; a file
  hidden in a shut folder steps to the nearest shown one.
- [ ] F5. ⌘S saves every file with unsaved words (J); it is heard from inside a line being typed
  in too.
- [ ] F6. Escape inside a line being typed in lets go of the line, keeping its words; the next
  Escape is the pane's (C10).

## G. Selecting and copying

- [ ] G1. The code's words can be selected, removed and opened-gap lines too; the numbers and the
  marks cannot (`user-select: text` on `.code-line`, none on the gutter).
- [ ] G2. A drag selects across lines, files and hunks; a press in the margin, on a number or a
  mark, starts the selection at the front of that line.
- [ ] G3. A shifted press extends the selection from where the last plain press landed, across
  lines.
- [ ] G4. Shift with an arrow, Home or End extends a selection from a line across into the lines
  above and below, keeping its column, off the end of a line onto the next and back, stopping at
  both ends (`extendHead`).
- [ ] G5. A double press takes a word, a triple press the line.
- [ ] G6. ⌘C copies the lines joined by newlines, the first and last clipped where the selection
  starts and ends, with no gutter mark and no number (`copiedText`). The removed lines are left
  out when any other line is selected; when only removed lines are, they are what is copied
  (`linesToCopy`). A selection inside one line is copied as it is.
- [ ] G7. The selection is drawn in the app's own selection colour.

## J. Typing into a change

- [ ] J1. Every line that is in the file now (added and unchanged ones) takes typing, as an editor
  does; a removed line and an opened gap's lines do not. It looks like the rest until it has the
  caret: then the text cursor, a caret in the added green and a faint bed under the line
  (`--code-live-bed`).
- [ ] J2. A press on such a line puts the caret where it landed.
- [ ] J3. Return is refused (a row is one line of the file); a paste comes in as plain text with
  its line breaks turned into spaces.
- [ ] J4. ↑ and ↓ move the caret to the line above or below at the same column; ← at a line's front
  goes to the end of the line above, → at its end to the front of the line below; at the first or
  last line it stays. Removed lines are skipped.
- [ ] J5. While a selection covers more than one line, a keystroke that would change the code
  (a letter, Backspace, Delete, Return, a paste, a cut, a drop) is refused and the selection stays
  for ⌘C (`editsAcrossLines`, `inputChangesText`).
- [ ] J6. The words typed are kept while the app runs, the pane closed and opened again included,
  never on disk unsaved; folding and opening a file shows them. A file with them is marked
  Unsaved on its head (E2), in the tree (D7) and in the picker (D13).
- [ ] J7. ⌘S, for each file with unsaved words: the file is read as it is now (`zero:code-file`),
  each hunk's lines found in it exactly as the run left them and swapped for the typed ones
  (`applySplices`: the first place a repeated block sits; applied from the foot of the file up;
  its last line break kept), and written (`zero:save-code-file`, with the time it was read).
  Then "Saved name.ts, 2 lines." for each file in one line, and the drawn rows take the saved
  words. Nothing is written when nothing changed.
- [ ] J8. Refused, with its reason in a line and nothing written: the run removed those lines
  ("Those lines were removed by the run, so there is nothing in the file to save them onto."),
  the file moved on ("The file has changed since this run, so those lines are not in it any more.
  Nothing was saved."), two edits overlap ("Two of those edits overlap in the file, so nothing was
  saved."), the file cannot be opened, or the engine refuses the write (outside the run's folders,
  changed while typing, gone).

## H. Opening the change at a file, from a run's step

- [ ] H1. In a run's opened fold, a step that edited or wrote a file (Edit, Write, MultiEdit,
  NotebookEdit) whose file the thread's change holds wears that file as a chip
  (`fileInChange`: the whole path or the change's key as its end at a folder boundary, the longest
  match, `/private/tmp` read as `/tmp`). A file the change does not hold is no chip.
- [ ] H2. The chip: a page mark nine by eleven in the faint ink, seven before the path the line
  showed, the dim ink; one point over and under, six in, a transparent edge; under the pointer the
  line's edge, the wash, the text's ink, the mark dim; the title "Open <file> in the code". The
  line's own subject moves into the chip; its note of how many lines came back follows it.
- [ ] H3. A press on the chip opens the change beside the conversation at that file (E4); pressing
  the same chip again after walking away lands again (`atFrom`). The step's own press still folds
  it open.

## K. The command bar and the thread's menu

- [ ] K1. ⌘K offers "View code changes" on the open thread while its page draws the button
  (keywords: code changes diff files what changed); it opens the change (never closes it).
- [ ] K2. The thread's menu's "View code changes" row is drawn only where the corner has no
  button, which is never while the button is drawn: there is nothing to draw.

## L. What the web window's view does not have

- [ ] L1. No split (side by side) layout and no switch for one: one running column.
- [ ] L2. No notes on a line, and no box to write one: a change takes no notes sidebar.
- [ ] L3. No "Edited … ago", no versions and no Restore.
- [ ] L4. Nothing says that rows were left out of a file past 20 000 (`leftOut` is not drawn).

## Checked how

(Filled in as each part is ported: the test that holds it, the picture that measures it, or the
script that presses it.)
