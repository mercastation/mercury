# Answering a thread and stopping its agent: what the web window does

Every behaviour of the web window when a person answers a thread or stops its agent, read off
the web window's own code, so the native window can do each of them the same way. `R` is
`renderer/src`, `A` is `R/App.tsx`, `F` is `R/components/Focus.tsx`, `M` is `main/`.

Each line is ticked once the native window does it, with how that was checked. A line left open
says why: Rune cannot do it (what is missing, and the closest it comes), or it needs a screen
this window does not have.

## S. Stopping an agent

- [ ] S1. Where it is offered. In the opened reply box (`DockComposer`'s `stop` prop) of a thread
  that `stoppable` holds (`R/list-rules.ts`: not done, and `belongsInProgress` with the list's own
  facts: its moment, a live run, the supervisor's stalled list), never on an agent row, the
  trouble row, the update row or a conversation with a person (`stoppableNow` in A, `stoppable`
  in F). The folded box never carries it (F, "NOTHING RIDES ON THE FOLDED BOX").
- [ ] S2. It takes the Send slot while the box holds no draft: no words (trimmed) and no files.
  The first character typed or the first file attached turns it back into Send, and emptying the
  box turns it into Stop again (`stop && !hasDraft`). Send's caret goes with Send; the paper clip
  stays.
- [ ] S3. Its look (`.dock-stop`, `.compose-sentence.dock-bar .dock-stop`, `soft-corners.css`): a
  filled square of 9 points, then "Stop", 13 points at weight 500, 7 points apart; the dim ink; a
  1 point border in the strong line; the small corner; 12 points each side; 32 points tall; right
  after the paper clip. Under the pointer the text's own ink, the wash, the border in the faint
  ink; pressed, 0.88 as bright. In a box under 460 points the word goes and it is 32 points wide
  (`@container reply`).
- [ ] S4. Its title and name: "Stop this agent. It goes back to your inbox." while a run is up,
  else "Stop this task. It goes back to your inbox." (`aria-label` "Stop this agent" / "Stop this
  task").
- [ ] S5. A press puts the caret back in the box (the span's `requestAnimationFrame` focus) and
  stops (`stopAgent` in A).
- [ ] S6. What is written: first whatever is held for its three seconds (`flushPending`, so a
  reply still held lands before the stop and does not reopen the row after it), then
  `zero:stop-session` with `{ product, id }`, which kills the session and writes the row
  `blocked` (`M/ipc.mts`).
- [ ] S7. What is said, once the engine has it: "Agent stopped. Back in your inbox. Reply to
  redirect it." when a run was on the thread at the press (`working`: in `supervisor.running`),
  else "Stopped before it started. Back in your inbox. Reply to redirect it." Nothing is put on
  the undo pile: Z after a stop reaches what was done before it.
- [ ] S8. The thread stays open, and the person is following it (`setFollowing`): leaving it
  goes to the Inbox's Needs you with its row picked, where a stopped thread now sits (`wayOut`
  in `R/stay-with-a-command.ts`, the effect in A). The box stays as it was; with the row no
  longer In progress the slot is Send again.
- [ ] S9. The command bar's row (A, `paletteCommands`): on a thread `stoppableNow` holds, "Stop
  Agent (Moves to Inbox)" while a run is up, else "Stop Task (Moves to Inbox)" with the hint
  "nothing has started on it yet"; id `stop`; the bar closes and the stop runs. It stands after
  the Move rows and before Send back to agent.
- [ ] S10. No other place stops a run: no key, not the thread's menu (`threadMenuRows` has only
  Stop repeating), not the working line.

## P. A reply shown at once

- [ ] P1. At Send the box empties and folds (`setModal(null)`), and the reply is drawn at once as
  the person's message at the moment of the press (`sending`, `PendingSaid`, placed by
  `itemThread` in `R/item-thread.ts`), its head saying "Sending… press Z to undo" where the time
  stands, in the dim ink (`.msg-sending`, `headFacts` in `R/components/Thread.tsx`).
- [ ] P2. A picked option is drawn at once as the line its written copy will be ("You picked
  option 2", the option's words after), not as a message.
- [ ] P3. One written copy cancels one drawn copy, oldest first, and only a copy written at or
  after the press, compared through the same normalising (`said`); so "yes" sent twice shows
  twice. The drawn copy is let go `LANDED_MS` (10 s) after the write, by when the written one is
  what is read.
- [ ] P4. A thread with no run: the reply is held three seconds (`deferCommit`), the row reads
  In progress meanwhile (`holdRow`, `heldRowStarts`), and the corner card says "Sent → <project>"
  with Z, naming the thread for O. The head keeps "Sending… press Z to undo" until the written
  copy arrives, as Z still takes it back after it is written (U3).
- [ ] P5. A thread with a run on it (`snap.supervisor.running`): held the same three seconds with
  no corner card at all; then handed over and its head says "Sending…"; on Claude Code with a run
  up, "Waiting for its current step" and a "Send now" button after it (`onSendNow`,
  `cutsARunningStep`), whose press says "Sending now…" and writes `zero:send-now`
  `{ product, id }`; the button goes once the message is taken.
- [ ] P6. What is written for a reply, in order: `zero:answer` `{ product, id, answer, status?,
  priority?, permissionMode?, model?, effort?, engine? }` (status `open` on a blocked or finished
  thread, none on a running one); then, if a reply was waiting on the thread for later,
  `zero:schedule` `{ runAt: 0 }`; then, if the thread was put off (`snoozedUntil` ahead),
  `zero:snooze` `{ snoozedUntil: 0 }`.
- [ ] P7. Refused (the write throws, or the engine says no): the drawn copy goes, the words and
  files go back in the box after whatever it holds now, a blank line between
  (`restoreFailedDraft`), and the box opens if the person is still on the thread, with the
  engine's words or "Could not send. Your message is back in the reply box."; elsewhere the line
  is "A message you sent did not reach its agent. Click to open it, your words are back in the
  reply box." naming the thread.
- [ ] P8. Where the person goes: a reply moves on to the next thread waiting when the thread was
  opened from Needs you on the Inbox (`replyMovesOn` in `R/advance.ts`: not from Home, not
  through search, not a thread come back to with O; and the row was in the list), else stays
  (`stay`). A pick moves on unless the option promises more (ends with "?", "ask me", "I'll
  explain" and the like) or a run is up (`pickStaysPut`). Staying, the person follows the thread
  (S8's way out).
- [ ] P9. Moved on, the line is said above the next thread's reply box instead of in the corner
  (`saysWhereItWent`, `SentLine`): a dot, "Sent to <title>" (a pick: the same), O Back and Z
  Undo as keys, and a ring that runs out over five seconds and closes it, with × to close it now.
- [ ] P10. A pick's corner line is "Sent “<option>” to <title>" (cut at 48 letters), not the
  project (`pickSentWords`).
- [ ] P11. A reply the window never saw land comes back after a restart: the words, the body and
  the files are written down at Send (`markReplyInFlight`) and let go once written, refused or
  taken back; at the next launch each still written down is looked for in its thread's ledger
  three times (at once, 30 s, 90 s), and one never found goes back in its box with "Your reply
  didn't send. It's back in the box." (or "N replies didn't send. They're back in the box.")
  naming the thread (`bringBackWhenSure`).
- [ ] P12. What is held is written before the window goes (`pagehide`, `onWriteHeld`): a reply
  pressed a second before quitting still lands.

## U. Z after a reply or a pick is written

- [ ] U1. Z inside the three seconds takes it back whole: nothing written, the drawn copy goes,
  the words, files and model back in the box and the box open (a pick gives nothing back), the
  thread opened again if the person had moved on, and "Undone. Your message is back in the reply
  box." or "Undone. Nothing changed." (`graceUndoneLine`).
- [ ] U2. What is done goes on a pile, each with its moment (`pushUndo`). Z takes the newest:
  done at once inside 30 s of it, and after that Z asks "Press Z again to <what it undoes>." and
  only a second Z inside 8 s does it; any other key but a modifier on its own means no
  (`nextUndo`, `undoAsk`, `askStillStands`, `HOLDS_A_KEY` in `R/undo-window.ts`). A Z that
  fails says "Could not undo that. <why>" and keeps its place on the pile. Nothing left says
  "Nothing to undo yet."
- [ ] U3. A reply written to a thread with no run: "take back that reply and stop the agent".
  Writes, in order: the waiting reply's moment back (`zero:schedule` with the old `runAt`) when
  one was taken with it, the person's own moment back when the store lifted it
  (`laterHerWordLifts`), the reminder back (`zero:snooze` with the old moment) when it was put
  off, `zero:stop-session` `{ product, id, withdraw: true }`, then `zero:answer`
  `{ product, id, answer: "(withdrawn)", status: <the status the reply found> }`
  (`withdrawReply`); the person stops following it. Then its words, files and model go back in
  the box unless the box holds something newer (`restoreDraft`), the thread opens, and the line
  says "Reply withdrawn, back in your inbox · your message is back in the reply box" (without the
  tail when nothing came back).
- [ ] U4. A pick written: "take back option N". Writes `zero:stop-session` `{ product, id }` then
  `zero:answer` `{ answer: "(withdrawn)", status }`; says "Option N withdrawn, back in your
  inbox"; opens the thread when the person is on a thread (`brings`, `shownAfterUndo`).
- [ ] U5. A reply handed to a running agent cannot be taken back: Z then says "Your message is
  already with the agent. Send another one to add to it." and opens that conversation; nothing
  is written. A hand-over that fails takes that entry off the pile again.
- [ ] U6. Each Z on a thread leaves one line in it, as the person's own action ("Undid your
  reply", "Undid picking" with the option after it, "Undid closing it"), kept on this Mac (the
  newest 300) and drawn among the conversation; the ledger lines the Z itself wrote fold under it
  (`R/undo-marks.ts`, `itemThread`).

## Q. The question before throwing work away

- [ ] Q1. An option that discards the change (`discardsTheChange`), picked by a press, its number
  or Enter, is not sent: the thread is opened if it was not, no option is lit, and the question
  stands where the options stood, at the conversation's foot: "Discard the change?" over "1
  Discard it" and "2 Keep it" (recommended), drawn as the options are (`discardAsk` in
  `R/discard-asks-first.ts`, `closeAskStrip` in F). The options strip over the box goes while it
  is up.
- [ ] Q2. Its keys while up (`closeAskOwnsKey`): 1 to 9 answer, ↓ and ↑ walk it (↓ from none
  lights the first, ↑ from the first lights none), Enter answers the lit one and does nothing
  with none lit, Escape keeps the thread as it was. Every other key keeps its meaning.
- [ ] Q3. "Discard it" sends the option as any pick is sent (P2, P4, U4), confirmed; "Keep it"
  takes the question away and leaves the offer standing. Leaving the thread takes it away too.

## W. The warning before closing a thread whose agent waits on an answer

- [ ] W1. A thread whose agent is waiting on the person: its offer is live and not every option
  closes the task (`stoppedWaiting` in `R/stopped-agent.ts`).
- [ ] W2. The first E ever on such a thread (or Done, or the bar's Mark done; not the agent's own
  "Close this task" picked) does not close it: the corner card says "This agent is waiting on
  your answer, and marking it done leaves it stuck. Press again to go ahead." naming the thread,
  and the press after it closes as any close does.
- [ ] W3. The same once-ever marker answers L: the first L on such a thread opens the Later picker
  with "This agent is waiting on your answer, and putting it off leaves it stuck." inside it, and
  nothing is held.
- [ ] W4. Remembered for good (`zero.stoppedWarned`), and an unreadable store counts as already
  told.

## D. Drafts

- [ ] D1. The words of each thread's reply box are kept as they are typed, the files beside them,
  and the model, level and agent picked in its menu beside those (`saveDraft`,
  `saveDraftAttachments`, `saveDraftPick`), across leaving the thread and across a restart. The
  priority and mode picks are not kept: a box opened anew starts on the thread's own.
- [ ] D2. Pasted pictures are kept as their bytes within a budget of 3 MB; past it the newest
  come off first, and the box says "One pasted image is too big to save. Closing the box loses
  it." (or "N pasted images are too big to save. Closing the box loses them."). A file named by
  its path costs nothing and always stays.
- [ ] D3. Opening a thread whose box holds a draft (words or files) opens the box on it
  (`hasDraft`).
- [ ] D4. Sending takes the draft away whole (words, files, pick); Z and a refusal give it back
  (U1, U3, P7); a Z declines to overwrite a newer draft.
- [ ] D5. A letter typed on an open thread with the box folded goes into the box, after its words,
  and opens it with the caret at the end (`keyOnAnOpenThread`, `typeIntoTheBox`): any printable
  key that is not one of the thread's own (J K R N Z O G S, a number while options stand). E and
  L right after the screen changed wait 350 ms for a letter after them, which then types both;
  the app having just moved the person here sends them to the box. Keys go to the box for a
  second after the last one typed into it that way.
- [ ] D6. R held down on an empty box does not type R into it (`heldKeyThatOpenedTheBox`).

## B. The rest of the reply box and of picking an option

- [ ] B1. A press anywhere outside the box (`.dock-card`) folds it when it holds nothing.
- [ ] B2. A press outside one of the box's open menus closes the menu and lands where it was aimed,
  anywhere on the page (an option, a row, a button), not only on the box's own controls.
- [ ] B3. One row lit in a menu of the box: the web lights the row under the pointer and, apart,
  the row with the keys.
- [ ] B4. A number pressed within 0.8 s of the app moving the person onto a thread does not answer
  it: "That number was for the thread you just left. Press it again to answer this one."
  (`pickLandsHere`).
- [ ] B5. A press on an option after the strip moved under a still pointer (by 4 points or more)
  within 0.8 s picks nothing and says "The options moved under your pointer. Click the one you
  want again." (`stripUnderThePointer`, `clickLandsOnTheRowYouSaw`).
- [ ] B6. An option cut short in the strip shows whole in a card over the strip while the pointer
  is on it (`.opt-peek`, in over 0.13 s).
</content>
</invoke>
