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

## A. The line at a conversation's foot (`Live.tsx`)

### A1. Which state holds (`liveLine`), in this order

- [ ] A1.1 An imported terminal session (`item.agent`) draws no line.
- [ ] A1.2 A walkthrough practice thread (`first-run` label) draws "working" while the app's own
  answer is on its way (`answering`), and nothing otherwise.
- [ ] A1.3 A finished thread (`status === 'done'`) draws nothing.
- [ ] A1.4 `asking` (an approval in the spool names the thread, or an open sign-in waits on it)
  comes before the session: the agent waits for a yes.
- [ ] A1.5 A session on the thread: working.
- [ ] A1.6 Signed out (`supervisor.signInNeeded[id]`, and not shipping), in or out of In progress.
- [ ] A1.7 Stopped (`stalled`) draws no line: the stalled bar says it.
- [ ] A1.8 Put off to a moment still ahead (`scheduledUntil > now`) draws no line.
- [ ] A1.9 Nothing came back (`supervisor.silent[id]`, and nothing on the row to read: no result
  and no checkpoint other than `(withdrawn)`), in or out of In progress.
- [ ] A1.10 Out of In progress (`belongsInProgress` false), nothing more is said.
- [ ] A1.11 Shipping (`supervisor.shipping`).
- [ ] A1.12 Working elsewhere: claimed, a claim that has not run out, and not held between
  Remote Control turns (`remoteHeld`).
- [ ] A1.13 At its limit (`supervisor.atLimit[id]`).
- [ ] A1.14 Cannot start: queued with a `cannotStart` reason the app has a sentence for.
- [ ] A1.15 Up next: queued and pushed with Run now (`runNow`).
- [ ] A1.16 Queued.
- [ ] A1.17 Paused (`supervisor.paused`).
- [ ] A1.18 Just sent: sent or replied to under 30 s ago, nothing claimed or written by an agent
  since, reads Queued.
- [ ] A1.19 Otherwise idle: nothing running.

### A2. The facts it is read from (`factsFor`, what `App.tsx` and `Focus.tsx` add)

- [ ] A2.1 The session is the snapshot's running entry for the thread.
- [ ] A2.2 `queued`, `runNow`, `remoteHeld`, `shipping`, `signInNeeded`, `atLimit`,
  `cannotStart`, `silent`, `paused`, `whyNotCloud` off `supervisor`.
- [ ] A2.3 `asking`: an approval names the thread, or an open sign-in waits on it.
- [ ] A2.4 `stalled` on the conversation: on `supervisor.stalled`, not asking who should take
  it (Auto), and not signed out.
- [ ] A2.5 `inProgress`: the In progress rule (`belongsInProgress`) on the thread's due moment,
  its session or a start on its way, the ship owed, stopped.
- [ ] A2.6 `scheduledUntil`: when the thread comes back (`returnAt`), when that is ahead.
- [ ] A2.7 `signInBar`: the Inbox's sign-in bar is up. This window draws no such bar, so the
  sentence never points at one (see A3.6).
- [ ] A2.8 A thread just sent counts as queued while an answer held for its undo is on its way
  (`queuedWithHeld`).

### A3. The words of each state

- [ ] A3.1 Working: the label of the newest call running, or "Thinking"; with subagents out
  the word is "N Subagents Working" / "1 Subagent Working" only where `shortWord` is used.
- [ ] A3.2 Working: "+N" for the other calls running beside it, when more than one runs.
- [ ] A3.3 Working: the time, `shortSpan` of now less `runningSince` (the reply a cloud run
  took, else the run's start): "just now" under a minute, "4m", "2h 7m", floored.
- [ ] A3.4 Working: where it runs, only where it is news: "in the cloud"; "on this Mac" while
  the cloud takes everything (`whyNotCloud.at === 'mac'`).
- [ ] A3.5 Working: "on this Mac" carries the why card on hover (`Running on this Mac`, and
  `whyWords`'s body), opening upward, with the help pointer.
- [ ] A3.6 Signed out: the whole sentence in the word's place ("Claude Code is signed out.
  Open a terminal, run claude, and type /login, and this starts on its own."; Codex's own).
- [ ] A3.7 At its limit: the whole sentence ("… is at its usage limit, so this starts on its own
  when it resets on Thu at 3pm."), the day only when not today, the clock only when known.
- [ ] A3.8 Cannot start: the whole sentence for its reason (setup, asleep, cloud-carried,
  engine, login by tool, memory, slow-folder, error, brake).
- [ ] A3.9 Asking: "Asking", and the time of the run behind it.
- [ ] A3.10 Elsewhere: "Working".
- [ ] A3.11 Shipping, Up next, Queued, Paused: "Shipping", "Up next", "Queued", "Paused".
- [ ] A3.12 Nothing came back: "Nothing came back", and how long ago the run ended.
- [ ] A3.13 Idle: "Nothing running".
- [ ] A3.14 What is read aloud (`role="status"`): the word while it works, else the whole
  sentence (`line`), including the subagent sentence and "4 minutes in".

### A4. The mark (`LiveMark`)

- [ ] A4.1 Turning (the orbit) while working and while working elsewhere.
- [ ] A4.2 Still everywhere else: every cell at 0.35, the middle at 0.1 (`.is-still`).
- [ ] A4.3 Under Reduce motion every cell at 0.65, the middle at 0.1.
- [ ] A4.4 Turning only while on screen (this window's own, see README).

### A5. The look

- [ ] A5.1 The line 14 points under what is above it, 2 under it, gap 7, 20-point line,
  13.5-point type (`.live-brief`).
- [ ] A5.2 A still word: dim ink, weight 500, cut short with dots rather than wrapping.
- [ ] A5.3 The time: faint, tabular figures, after the word, 7 points on.
- [ ] A5.4 Working: the word shimmers (`.is-shimmering`); none of the other states does.

### A6. Try again on the line

- [ ] A6.1 Offered where `offersTryAgain` says (A7), and not while stopped (the bar has it).
- [ ] A6.2 Drawn after the word and its time: a quiet button, 24 high, 9 in, 4 more to its
  left, a hairline edge, 12.5-point dim words "Try again".
- [ ] A6.3 Under the pointer: the text's ink, the strong edge, the strong wash; the pointing hand.
- [ ] A6.4 Out of In progress with no line, but Try again offered (the app's note that a run got
  nowhere): the still mark, "Nothing running", and Try again.
- [ ] A6.5 Pressed: Try again (A8).

### A7. When Try again is offered (`offersTryAgain`)

- [ ] A7.1 Never on an imported session, a finished thread, one with a session, the practice
  project, a walkthrough thread.
- [ ] A7.2 Never before the fleet has been read (`queued` not a list).
- [ ] A7.3 Never while asking, shipping or held between Remote Control turns, or put off to a
  moment ahead.
- [ ] A7.4 Stopped and not signed out: offered.
- [ ] A7.5 Otherwise offered on idle, nothing came back and at its limit.
- [ ] A7.6 With no line: open, not queued, the agent's newest result the app's own note that a
  run got nowhere, and not the note that the model is for the other agent.

### A8. Pressing Try again (`tryAgain` in `App.tsx`)

- [ ] A8.1 Not on an imported session, the app's trouble row or its update row.
- [ ] A8.2 Writes `zero:resume-items` with `{ ids: [id] }` (Resume on that one thread).
- [ ] A8.3 Says what came back, always (`tryAgainWords`): "Trying again now.", "Queued.",
  "Already running.", "It could not start: <why>.", "This thread is gone from this computer.",
  "It cannot start on this computer: another computer runs it.", and "Try again did not reach
  the app." with no app to ask. A door that fails is `failed` with its reason.
- [ ] A8.4 Reads the lists again at once (`refresh`). It stays where it is.

### A9. The clock

- [ ] A9.1 While a run is up, or while a silent row counts how long ago its run ended, the line
  is drawn again every 15 s so a minute's turn is never late by more than that.

## B. The line under the title while it works (`workingContext`)

- [ ] B1 Only while an agent works (a session), and only when the thread carries no summary
  (`problem`).
- [ ] B2 What was asked (`askLine`): the first paragraph not a lead-in ending in a colon, a link
  read as its words, "You asked: …" with a full stop added; nothing when the title says it.
- [ ] B3 A cause row's ask is not said.
- [ ] B4 A repeating run adds its schedule: "Repeats daily at 9:00am." (`repeatsLine` of the
  thread's rule, read from `zero:repeats`).

## C. The bar over a stopped conversation (`.stalled-bar`)

- [ ] C1 Drawn when the thread is stopped (A2.4), at the top of the reading column, over the
  title: 4 points above, 18 under, 10 by 14 inside, the raised ground, the card's corner.
- [ ] C2 Its words: "The agent stopped without finishing this." and, with an answer of the
  person's still waiting (not withdrawn), "… Your answer is still waiting."; at its end
  "try again →" in the accent, 13.5 points, semibold.
- [ ] C3 Under the pointer: its edge in the faint ink; the pointing hand over all of it.
- [ ] C4 Pressed anywhere: Try again (A8).

## D. The same on the lists (`ThreadCells`, `RowCells`, `statusCell`)

- [ ] D1 Every Status word of those states, with its mark: Asking (accent ring), Working (half
  dot or cloud, breathing), Working locally and Moving to cloud, Shipping, Up next and Queued
  (ring), Not started, Signed out, At its limit, Paused, Nothing came back, Nothing running,
  Stopped (accent ring).
- [ ] D2 The Status word reads the same `liveLine` the conversation does (one rule).
- [ ] D3 The cell's hover: the whole sentence (`title`), or, where the cloud says why, its why
  card instead: 320 wide, 10 under the cell, the head in the text's ink and the body dim, the
  help pointer.
- [ ] D4 The try again mark after the word on a stuck row (`offersTryAgain`, with the row's tab
  as In progress): 18 square, 2.5 after the word, the refresh arrow 11 points in dim ink, always
  drawn; under the pointer the text's ink on the strong wash, the pointing hand, its title "Try
  again".
- [ ] D5 Pressing the mark tries again and does not open the row.
- [ ] D6 Not on a row out with a person you asked (team only), nor on a cause's row or a row
  behind one.
- [ ] D7 `tools/rows.mjs` holds the word, the mark, the mark of try again and the hover sentence
  against the web window's, on every case (`--cases`), with 0 differences.

## E. The command bar

- [ ] E1 "Try again" on the thread under the keys where `offersTryAgain` says (its tab rule
  `stoppableNow`), after Reply, found by "resume restart retry start stuck run again", drawn
  with the refresh mark; it runs A8.

## F. A run's steps (`RunLine`, `WorkLine`)

- [ ] F1 A fold's head carries the run's whole summary as its title (`runSummary`).
- [ ] F2 A line inside an opened fold has three states: shut, open (the whole command and the
  first six lines of what came back), all (every line).
- [ ] F3 A line opens when something came back, when its words were shortened (`full`), or when
  the column cut its command; otherwise it does not answer the pointer (no hand, no hover).
- [ ] F4 A command the column cuts gets the mark and opens onto itself whole.
- [ ] F5 Open: the whole command in a block (`.did-full`): 11.5-point code face, 1.55 line, dim,
  on the work chip's wash, 7 by 10 inside, 6 above, the card's corner, wrapping anywhere.
- [ ] F6 Open: what came back in a block (`.did-out`): 11.5-point code face, 1.55, faint, on
  the work fill under a hairline, 9 by 12 inside, 6 above, the card's corner, kept as written.
- [ ] F7 Open with more than six lines: "show all N lines", 12 points, faint, 5 above; under the
  pointer dim; pressed, all of them.
- [ ] F8 All: under the lines, what the log did not keep (`outputCut`), when it cut any.
- [ ] F9 A run past its cap says how many more it ran (`runOverflow`), in any state.
- [ ] F10 Shut: "· N lines" after the command when it opens and more than one line came back.
- [ ] F11 The mark turns down while open; under the pointer the verb in the text's ink and the
  mark dim.
- [ ] F12 A press on an open line shuts it, from open or from all.

## G. Escape folds back what was opened on this visit (`useEscapePutsAway`)

- [ ] G1 The folds and lines opened since the thread was opened are one step, at the moment of
  the newest opening; the working line's steps are another, at the moment they opened.
- [ ] G2 One Escape puts away the newest step: every fold and line opened on this visit at
  once, or the working line's steps; the next Escape the next step; then the thread goes.
- [ ] G3 Folding a thing by hand takes it off the step; when nothing of it is left open the step
  is gone.
- [ ] G4 With a file the person opened beside the conversation, only what was opened after the
  file is put away first; then the file closes; then the older steps.
- [ ] G5 A fold kept from an earlier visit is not a step of this one.

## H. An opened fold is remembered (`keepPlace`)

- [ ] H1 Where the person was in a thread is kept while the window is open: the earlier messages
  shown, the middle asked back, the folds opened and the lines opened and how far.
- [ ] H2 Coming back to the thread puts all of it back, only while its newest answer
  (`arrivedAt`) is the one it had when it was left; else it opens as new and forgets it.
- [ ] H3 How far the person had scrolled it themselves is kept too, and coming back reaches for
  it while the page settles (2.5 s, until the person scrolls).
- [ ] H4 The newest fifty threads are kept.

## I. The line for a long conversation's missing middle

- [ ] I1 Where the middle is cut, "N earlier messages" (`conversationGap`), 13 points, faint,
  18 above.
- [ ] I2 Under the pointer: dim and underlined; the pointing hand.
- [ ] I3 Pressed: every run's log is asked for whole, and the place being read stays where it
  is on the screen as the middle comes in above it.
- [ ] I4 Photographed against the web window's on a conversation longer than forty messages.

## J. The working line's steps, read again while open

- [ ] J1 Opening the line asks for the run's log at once.
- [ ] J2 While open it is asked again on every change the engine pushes and every eight seconds.
- [ ] J3 A read that fails says "Couldn’t read the command history." under the steps, until a
  read lands.
- [ ] J4 With no steps and no call running: "No commands yet."
- [ ] J5 The newest five finished steps since the run last said something, oldest first, the
  rest counted ("3 earlier steps since it last said something"), then each call running now.
- [ ] J6 Folds when another thread or another run opens; a step opens onto its command.
- [ ] J7 The command is set in the system's monospace (`ui-monospace`), 12 points on a 1.6 line.

## K. Where a conversation lands when opened (`holdAtBottom`)

- [ ] K1 A conversation drawn as a conversation (unfolded, working, or short enough to be drawn
  whole) lands 20 points above the top of its last message, or at its foot when that and
  everything under it fit (`landingTop`).
- [ ] K2 While the page settles it keeps landing there, for 2.5 s or until the person scrolls.
- [ ] K3 Landed at its foot it sticks there as it grows; landed above it, it does not.
- [ ] K4 A page that leads with the newest answer alone opens at its top.
- [ ] K5 A place kept from an earlier visit (H3) wins over the landing.
