# Where a run stands and what it did: the web window's behaviour, line by line

The checklist of everything the web window shows about an agent's run, read off its code, and
whether this window does the same. A line is ticked only once it is ported and checked; how it
was checked is said on the line. A line left open says exactly why.

The spec, by file (`R` is `renderer/src`, `S` is `shared`):

- `R/live-line.ts` (`liveLine`, `shortWord`, `shortSpan`, `livePlace`, `cannotStartLine`,
  `cameBackEmpty`), drawn by `R/components/Live.tsx` and `R/components/LiveMark.tsx`.
- `R/try-again.ts` (`offersTryAgain`, `tryAgainWords`), the `tryAgain` callback and the
  `try-again` row of the command bar in `R/App.tsx`, `TryAgainButton` in `R/threads/Pages.tsx`.
- `R/status-word.ts` (`statusCell`, `factsFor`), `R/where-why.ts` (`whyWords`), `RowCells` and
  `ThreadCells` in `R/threads/Pages.tsx`.
- The stalled bar in `R/components/Focus.tsx` (`.stalled-bar`).
- `R/one-thing-at-a-time.ts` (`workingContext`, `askLine`), its use in
  `R/components/ItemThread.tsx` (`QueueSummary`'s `context`).
- `R/components/Thread.tsx` (`RunLine`, `WorkLine`, the gap line, the fold state and `kept`),
  `R/escape-steps.ts`, `R/thread-place.ts`, `R/thread-bottom.ts` (`holdAtBottom`, `landingTop`,
  `lastMessageIn`), and where `App.tsx` leaves and finds a thread's place.
- Their stylesheets: `R/styles.css` (`.live-*`, `.stalled-bar`, `.did*`, `.thread-gap`) and
  `R/threads/pages.css` (`.th-cell-state`, `.th-retry`, `.th-why-card`).

Each line: **[x]** done, **[ ]** not done. Under it, how it was checked, or why it is not done.
"Tests" are `cargo test` in `desktop/` (the rule's own cases in `src/rules/live.rs`, the window's
in `src/foot.rs` and `src/thread.rs`); "pictures" are the scenes in `tools/scenes/` photographed
from the web renderer and drawn here from the same fixture, with the difference in the README.

## A. The line at a conversation's foot (`Live.tsx`)

### A1. Which state holds (`liveLine`), in this order

- [x] A1.1 An imported terminal session (`item.agent`) draws no line.
  Test `it_refuses_to_say_twice_what_a_bar_says…`.
- [x] A1.2 A walkthrough practice thread (`first-run` label) draws "working" while the app's own
  answer is on its way (`answering`), and nothing otherwise.
  Test `a_thread_just_sent_reads_queued_and_a_practice_thread_works_while_it_is_answered`. This
  window has no walkthrough, so `answering` is never true here and the line draws nothing.
- [x] A1.3 A finished thread (`status === 'done'`) draws nothing. Tests.
- [x] A1.4 `asking` comes before the session: the agent waits for a yes.
  Test `a_yes_waited_on_outranks_the_run_behind_it`; picture `thread-line-asking`.
- [x] A1.5 A session on the thread: working. Tests; pictures `thread-working`, `thread-line-here`.
- [x] A1.6 Signed out, and not shipping, in or out of In progress.
  Tests; picture `thread-line-signedout`.
- [x] A1.7 Stopped draws no line: the stalled bar says it. Tests; picture `thread-line-stalled`.
- [x] A1.8 Put off to a moment still ahead draws no line. Tests.
- [x] A1.9 Nothing came back, in or out of In progress, never over something to read.
  Tests (with `cameBackEmpty`'s cases); picture `thread-line-silent`.
- [x] A1.10 Out of In progress, nothing more is said. Tests.
- [x] A1.11 Shipping. Tests; picture `thread-line-shipping`.
- [x] A1.12 Working elsewhere: a live claim not held between Remote Control turns.
  Tests; picture `thread-line-elsewhere`.
- [x] A1.13 At its limit. Tests; picture `thread-line-limit`.
- [x] A1.14 Cannot start, for a reason with a sentence; any other reason reads as queued.
  Tests; picture `thread-line-cannot`.
- [x] A1.15 Up next. Tests; picture `thread-line-next`.
- [x] A1.16 Queued. Tests; picture `thread-line-queued`.
- [x] A1.17 Paused. Tests; picture `thread-line-paused`.
- [x] A1.18 Just sent reads Queued. Tests.
- [x] A1.19 Otherwise idle: nothing running. Tests; picture `thread-line-idle`.

### A2. The facts it is read from

- [x] A2.1 The session is the snapshot's running entry for the thread, by its id, the newest
  winning, as the fleet's map keeps it. Test `the_facts_are_read_off_the_supervisor_by_the_threads_id`.
- [x] A2.2 The supervisor's lists, by the thread's id. Same test.
- [x] A2.3 `asking`: an approval names it, or an open sign-in waits on it. Picture
  `thread-line-asking`; test on the sign-in sentence.
- [x] A2.4 The conversation's `stalled` leaves out a thread asking who should take it and one
  signed out. By code (`foot::facts`); picture `thread-line-signedout` (signed out, no bar).
- [x] A2.5 `inProgress` is the In progress rule with the thread's due moment, its run or a start
  on its way, the ship owed, stopped. By code (`foot::facts`, `list::belongs_in_progress`);
  pictures of the In progress states.
- [x] A2.6 `scheduledUntil`: when it comes back, when that is ahead. By code; the rule's tests.
- [ ] A2.7 `signInBar`. Not done: it says the Inbox's sign-in bar is up, a screen this window
  does not have, so it is always down here and the signed-out sentence says to sign in from a
  terminal, as the web window's does with its bar down.
- [x] A2.8 A reply held for its undo counts as queued. Test `a_reply_held_for_its_undo_reads_as_queued`.

### A3. The words of each state

- [x] A3.1 Working: the newest call's label, or "Thinking"; subagents counted inside the word
  where `shortWord` is used. Tests (with the web's subagent cases).
- [x] A3.2 "+N" for the other calls running beside it. Test.
- [x] A3.3 The time, `shortSpan` of now less `runningSince`. Tests (with the web's cases).
- [x] A3.4 "in the cloud"; "on this Mac" while the cloud takes everything. Tests; picture
  `thread-line-here`.
- [x] A3.5 "on this Mac" carries the why card over it while the pointer rests there.
  Picture `thread-line-hover-place` (card 0.75 of 255).
- [ ] A3.5b … with the help pointer (`cursor: help`). Not done: Rune's pointers have no help
  shape, and Chromium's is a picture of its own rather than one the system has; the arrow shows.
- [x] A3.6 Signed out: the whole sentence, by tool. Tests; picture.
- [x] A3.7 At its limit: the whole sentence, day only when not today, clock only when known.
  Tests; picture.
- [x] A3.8 Cannot start: the sentence for each reason. Tests; picture.
- [x] A3.9 Asking: "Asking" and the run's time. Test; picture.
- [x] A3.10 Elsewhere: "Working". Test; picture.
- [x] A3.11 Shipping, Up next, Queued, Paused. Tests; pictures.
- [x] A3.12 Nothing came back and how long ago. Tests; picture.
- [x] A3.13 Nothing running. Tests; picture.
- [x] A3.14 What is read aloud: the word while it works, else the whole sentence. By code (the
  line's label and polite live region); not heard with a screen reader.

### A4. The mark

- [x] A4.1 Turning while working and while working elsewhere. Picture `thread-line-elsewhere`
  (loops on the cells, as the working mark's, tested already).
- [x] A4.2 Still elsewhere: every cell at 0.35, the middle too (read off the dump: the hole's
  0.1 gives way to `.is-still`'s 0.35). Pictures; cell colours equal to the web's to the byte.
- [x] A4.3 Under Reduce motion every cell at 0.65, the middle at 0.1, for a turning mark; a
  still one stays at 0.35. By code; the working case tested already.
- [x] A4.4 Turning only while on screen. This window's own (README, "What moves on its own").

### A5. The look

- [x] A5.1 14 over, 2 under, gap 7, a 20-point line, 24 with Try again. Pictures.
- [x] A5.2 A still word: dim, 500, wrapping where longer than the column (the web's `.live-word`
  wraps; only the working one is cut short). Pictures.
- [x] A5.3 The time: faint, tabular figures. Pictures `thread-line-silent`, `-asking`.
- [x] A5.4 Only the working word shimmers. Pictures; loops counted in tests.

### A6. Try again on the line

- [x] A6.1 Offered where `offersTryAgain` says, not while stopped. Tests.
- [x] A6.2 The button's size and look. Pictures `thread-line-idle`, `-limit`, `-silent`.
- [x] A6.3 Under the pointer. Picture `thread-line-hover-retry` (2.04, 1.31 at rest).
- [x] A6.4 Out of In progress, the app's note: still mark, "Nothing running", Try again. Test.
- [x] A6.5 Pressed: Try again. Tests; headless against a recording door.

### A7. When Try again is offered (`offersTryAgain`)

- [x] A7.1–A7.6 Every case of `tests/a-stuck-thread-offers-try-again.test.mts`, and the model for
  the other agent. Test `try_again_is_offered_where_nothing_will_start_the_thread`; rows.mjs.

### A8. Pressing Try again

- [x] A8.1 Not on an imported session, the trouble row or the update row. Test.
- [x] A8.2 Writes `zero:resume-items` with `{ ids: [id] }`. Headless against a small recording
  door on loopback: the press wrote `{"ids":["w-readme0"]}`. The channel itself was called on a
  scratch engine.
- [x] A8.3 Says what came back, always, read as JavaScript reads the engine's answer (its
  `queued` is the supervisor's list over the count, so anything that did not start says Queued).
  Test with the answers a scratch engine gave; the door check said "Trying again now.". No app:
  "Try again did not reach the app." (picture `thread-line-tried`).
- [x] A8.4 The lists are read again at once; the person stays where they are. By code
  (`door.ask_soon`); the window test stays in the Inbox.

### A9. The clock

- [x] A9.1 Drawn again every 15 s while it counts. By code (`foot::tend`); `counting` tested.

## B. The line under the title while it works (`workingContext`)

- [x] B1 Only while an agent works, and only with no summary. By code; pictures `thread-working`.
- [x] B2 `askLine`, with the web's cases. Tests.
- [x] B3 A cause's ask is not said. Test `asked_of`.
- [x] B4 A repeating run's schedule, from `zero:repeats`. Tests (`repeats_line`, and the window
  with a fixture's rules).

## C. The bar over a stopped conversation (`.stalled-bar`)

- [x] C1 Where and how large. Picture `thread-line-stalled` (the bar 0.73).
- [x] C2 Its words, with the waiting answer. Test; picture.
- [x] C3 Under the pointer, its edge. Picture `thread-line-hover-stalled` (0.79, 0.79 at rest).
- [x] C4 Pressed: Try again. Test `a_stopped_thread_draws_its_bar…`.

## D. The same on the lists

- [x] D1 Every Status word of those states with its mark. rows.mjs, 0 differences: the invented
  inbox, the cases, `inbox-progress-stuck` and the running app's inbox through `--door`.
- [x] D2 The Status word reads the same `liveLine`. By code (`rules::status` reads
  `rules::live`); its tests and rows.mjs.
- [x] D3 The hover sentence, or the why card in its place; the rows after it paint over the card,
  as in the web window. rows.mjs holds the sentence and the card's presence; picture
  `inbox-progress-stuck-hover-why`.
- [ ] D3b … with the help pointer. Not done: as A3.5b; the row's pointing hand shows.
- [x] D4 The mark after the word. rows.mjs holds it (5 under Needs you, 16 under In progress in
  the cases); pictures `inbox-progress-stuck`, `-hover-retry`.
- [x] D5 A press tries again without opening the row. Test
  `a_stuck_rows_mark_tries_again_without_opening_the_row…`.
- [x] D6 Not on a cause's row or one behind a cause. By code; rows.mjs. (A row out with a person
  asked for help is a team's, which this window does not draw.)
- [x] D7 rows.mjs holds the mark and the sentence, 0 differences everywhere.

## E. The command bar

- [x] E1 "Try again" after Reply where offered, found by its words, with the refresh mark, and
  run. Tests (the rule's order and words; ⌘K over a stuck row lists it).

## F. A run's steps

- [x] F1 A fold's title is its tally (`runSummary`, with the web's cases). Tests.
- [x] F2 Three states. Pictures `thread-steps-line` (0.80), `-line-all` (0.88); test.
- [x] F3 Opens only on output, shortened words or a cut command; no press otherwise. Pictures;
  by code.
- [x] F4 A cut command gets the mark and opens onto itself. Pictures `thread-steps-cut-shut`
  (1.11), `thread-steps-cut` (1.21); a frame after first layout (README).
- [x] F5 The whole command's block. Pictures.
- [x] F6 What came back, six lines. Pictures; test.
- [x] F7 "show all N lines" and its press. Picture `-line-all`; test.
- [x] F8 What was not kept, under all of it (`outputCut`, the web's cases). Tests.
- [x] F9 A run past its cap, always. By code (as before).
- [x] F10 "· N lines" when shut. Pictures.
- [x] F11 The mark turns; under the pointer the verb and mark brighten. Pictures; by code.
- [x] F12 A press shuts it from either. Test (Escape test opens and shuts); by code.

## G. Escape folds back what was opened on this visit

- [x] G1 Folds and lines one step at the newest opening; the working line's steps another. Tests.
- [x] G2 Newest first, one per press, then the thread goes. Tests (`escape_folds_back…`, and the
  working line's).
- [x] G3 Shut by hand comes off the step. By code (`note_fold`).
- [x] G4 With a file beside it, what was opened after the file first. By code (`Doc.opened`,
  `escape_in_the_conversation`); not exercised by a test.
- [x] G5 A fold kept from an earlier visit is no step of this one. Test.

## H. An opened fold is remembered

- [x] H1 Earlier messages, the middle, folds and lines kept. Test `a_fold_opened_is_there_again…`.
- [x] H2 Only while the newest answer is unchanged. Same test.
- [x] H3 How far the person scrolled it, reached for while the page settles. Test (the wheel
  scrolls, the thread is left and opened, and lands where it was). The hand is the wheel only;
  the web window also counts a key or a press (README).
- [x] H4 The newest fifty, and across a restart, with the picks. Test in `src/picks.rs`.

## I. The line for a long conversation's missing middle

- [x] I1 Its words, size, ink and room. Picture `thread-long` (the line 0.29).
- [x] I2 Under the pointer: dim and underlined. Picture `thread-long-hover-gap` (0.58).
- [x] I3 Pressed: the middle asked whole, the block under it held in place. By code
  (`keep_on_screen` on the block under the line; `scroll_anchor` for what comes after).
- [x] I4 Photographed on a conversation of forty-five checkpoints. Picture.

## J. The working line's steps, read again while open

- [x] J1 Opening asks for the log at once. By code (`ask_for_the_open_thread(true)`).
- [x] J2 On every push and every eight seconds. By code (as the conversation is read while a run
  is up; README says how that differs).
- [x] J3 "Couldn’t read the command history." Picture `thread-working-steps-unread` (0.38).
- [x] J4 "No commands yet." By code; height read off the web's dump (20.8).
- [x] J5 The newest five, the rest counted, then what runs now. Tests.
- [x] J6 Folds for another thread or run; a step opens onto its command. Tests; picture.
- [x] J7 The system's monospace, 12 on 1.6. Picture `thread-working-steps-open` (the command 3.43,
  4.17 in Geist Mono).

## K. Where a conversation lands when opened

- [x] K1 20 points over its last message, or its foot. Test `a_conversation_lands_on_the_top…`.
- [x] K2 Holds for 2.5 s or until the person scrolls. Same test. It holds on the window's turns
  (README).
- [x] K3 Landed at its foot it sticks; above it, not. By code (`stick_bottom`); test of the short one.
- [x] K4 A page that leads with the answer opens at its top. Pictures unchanged.
- [x] K5 A kept place wins over the landing. Test.
