# The command bar, row by row: the web window's and this one's

Every row the web window's ⌘K bar can offer, and every behaviour of the bar itself, read from
its code: `renderer/src/components/Palette.tsx` (the rows it makes itself and `PaletteBox`),
`renderer/src/palette-rows.ts`, `palette-pick.ts`, `palette-keys.ts`, `palette-glyphs.ts`,
`thread-commands.ts`, `thread-model-commands.ts`, `priority.ts` (`priorityCommands`),
`run-now.ts`, `try-again.ts`, `cloud-menu.ts` (`cloudCommands`), `update-row.ts`, and
`renderer/src/App.tsx`, which hands the bar its props (`<Palette` and `itemCommands`).

Each line says what the web's row is called, the drawing at its head (by its name in `ICONS`),
the key drawn at its end or else its hint, when it is there, and what pressing it does; then
whether this window offers it.

- **done**: offered here, under the same words, drawing, key and hint, in the same place, doing
  what the web's row does (the write named is the one checked on a scratch engine).
- **out**: not offered here, and why. A row whose doing needs a screen this window does not
  have stays out, and so does one that would do nothing here.
- **another task**: offered by another task's change, which ports the control it belongs to.

A row's place among the others is the web's: the order below is the order it is put together
in, which is the order drawn before anything is typed.

## The bar itself

| # | Behaviour (web) | Status |
|---|---|---|
| B1 | ⌘K opens it over whatever is on screen and closes it again; Control K the same (`(e.metaKey \|\| e.ctrlKey) && key 'k'`). | done (as before) |
| B2 | Not while Settings is up, nor the new project card: those own the keys (`if (settingsOpen) return`). | done (as before: Settings takes every key here too; there is no new project card) |
| B3 | It is one `modal` with the reply box, the Later picker, the New thread card and the cloud's question: opening it closes them; a row that opens one of them replaces the bar. | done (as before) |
| B4 | Opens empty, the caret in the box, the first row lit, the list at its top (the box is mounted anew each time). Opened from the thread menu's Move row it opens with "Move to " typed (`paletteTyped`). | done; the typed prefill has no way in here, as this window's thread menu has no Move row (see the rows) |
| B5 | The placeholder is "Search threads or type a command…" (threads are searched); "Type a command…" without a search; "Search" on a phone. | done (as before) |
| B6 | Nothing typed: every row that is not `unlisted`, in the order put together. | done |
| B7 | Words typed keep a row when every word lands somewhere in its label or its keywords, as a piece of a word or a whole one (`matchesQuery`). | done (as before) |
| B8 | A row with `onlyWith` is kept only when one typed word of two letters or more starts that word, or the word starts it, even when every word matched; with nothing typed it is listed unless `unlisted`. | done (the cloud's toggle carries it; the gate was missing here) |
| B9 | The rows kept are ranked: Later (`snooze`) first, then a row whose label starts with what was typed, then the rest, each group in the order put together (`rankMatches`). | done (as before) |
| B10 | The threads found: the best five of the list's own search, the one titled by the words first (`threadRows`, `titleFirst`). | done (as before) |
| B11 | More than five found: a sixth row, "See all N threads", which opens the thread search page with the words in it. | out: this window has no search page |
| B12 | People and groups found (`findChats`) lead the threads when the words name one of them better than any thread title, else follow them, under "People and groups" (`peopleLead`). | out: no rooms, teammates or people here |
| B13 | Order: commands named by the words (every word starts a word of the label or of a name it had) go first; then the threads under "Threads"; then the commands the words only touch by keyword, under "Commands". A command that carries a found thread's whole title stays below it unless its label starts with the words (`paletteOrder`). | done (as before) |
| B14 | "Set Mercury up again" is kept for typed words only when they name it (`ASKED_FOR_BY_NAME`). | out with its row |
| B15 | What was found inside things comes 90 ms after the words, from the app (`findInside`, `zero:find-in-threads`), after everything else, under "Lines", "Files", "Comments" and "Decisions"; the last answer stays up until the next lands. The app is asked to read the threads ahead as the bar opens (`zero:warm-find-in-threads`). | see "Found inside things" below |
| B16 | Each row has a key of its own, unique even when two rows share an id (`commandKeys`). | done (as before) |
| B17 | The lit row is a row, not a place: a list that sorts again or grows under it keeps it lit (`keptPick`); new words light the first row again. | done (as before) |
| B18 | ↓ and ↑ move the lit row and stop at either end; only a key brings it into view (`scrollIntoView({ block: 'nearest' })`). | done (as before) |
| B19 | The pointer lights the row it moves over, never one the list moved under a pointer standing still (`pointerMoved`). | done (as before) |
| B20 | Enter runs the lit row; a press runs the row pressed. A row closes the bar itself where it does (`setModal(null)` in its `run`), so the one that does nothing leaves it open. | done |
| B21 | Escape closes it from the window's capture phase, before anything under it (`escapeClosesThePalette`); so does the esc button, and a press on the backdrop on its way down (`closesOnPress`). | done (as before) |
| B22 | Nothing matches: one line where the first row would be, "Nothing matches “…”.", the words cut at a word past 32 letters (`emptyLine`). | done (as before) |
| B23 | A row: its drawing (16 points, faint, the text's ink when lit), its label, and at its end its key's cap or else its hint; a thread its title over the sentence it was found in with the words in ink, its project and age beside them, dimmed when finished; the lit row a wash. | done (as before) |
| B24 | The Files place's list is read as the bar opens, for its projects' rows (`useFilesList(modal === 'palette')`). | done (as before) |
| B25 | Its first close finishes the tutorial's lesson about it (`finishLesson(tour, 'palette')`). | out: no tutorial here |
| B26 | With rows ticked every command acts on all of them (`batch`). | out: no row can be ticked here |
| B27 | On a phone: full screen, threads first, a Cancel button, no keys. | out: not a phone |
| B28 | Sub-lists: no row of the web's bar opens a list of its own inside the bar. A row either acts, or opens something else (the Later picker, the reply box, the New thread card, Settings, ⌘C's question), which replaces the bar. | done (nothing to port) |

## The rows before anything is typed

In the order `Palette.tsx` puts them together. The thread's rows go where the third line says.

| Id | Words | Drawing | Key, or hint | When | Does | Status |
|---|---|---|---|---|---|---|
| `new-project` | New project… | plus | | always | Opens the new project card | out: the new project card is a screen this window does not have |
| `new-thread` | New thread | compose | N (hint "ask an agent" not drawn) | always | Opens the New thread card (`setModal('compose')`) | done (as before) |
| `record-meeting` | Record meeting | record | "into <project>" | the recorder can record | Starts recording a meeting | out: no recording here |
| `stop-recording` | Stop recording | stop | | a meeting is being recorded | Stops it | out: no recording here |
| (the thread's rows) | | | | a thread under the keys | see the next section | |
| `filter` | Filter… | filter | "project, priority or agent" | always | `setModal('filter')`: closes the bar and opens nothing, as the menu it opened is gone from the web window | out: in the web window it does nothing but close the bar; a dead row is not ported (for the owner: View and filters is the nearest) |
| `clear-filter` | Clear filter | clear | | the box filter is on | Clears the box filter (project, priority, agent) | out: that filter is only ever set by the tutorial, which this window does not run; View and filters' own filters are another state, which the web's row does not touch |
| `end-tutorial` | End the tutorial | clear | | the tutorial is on | Ends it | out: no tutorial here |
| `projects-page` | Projects… | folder | "order, names and rules" | always | Settings, on Projects | to port |
| `rank-<slug>` | Rank <project> first | top | "now #N of M" when the order names it, else "made for you" for the workspace made for the person, else "not ranked yet" | each project that is not a conversation, in the running order | The project goes first in the order (`zero:set-product-order`), and "<project> is now first" | to port |
| `cloud-toggle` | Turn the cloud on, or Turn the cloud off | cloud | ⌘C | this Mac is set up for the cloud; off needs a word naming "cloud" (`onlyWith`) | On: `cloudRuns` to what on means. Off: ⌘C's question | to port |
| `cloud-full` | Cloud on: when this Mac is full | cloud | "On now" while it is | set up for the cloud | `cloudRuns` `full` | to port |
| `cloud-all` | Cloud on: everything | cloud | "On now" while it is | set up for the cloud | `cloudRuns` `all` | to port |
| `view-home` | Go to Home | home | | Home is on | Home | done (as before) |
| `view-inbox`, `view-progress`, `view-later`, `view-done` | Go to Inbox, Go to In progress, Go to Later, Go to Done | inbox, progress, later, done | | always (a team's tabs: Inbox and In progress only) | That tab of the Inbox | done (as before) |
| `view-groups` | Go to Groups | hash | "people you work with" | Groups is on | The Groups page | out: no Groups page here |
| `group-<key>` | <group> | hash | | Groups is on | That group | out: no Groups page here |
| `team-list-<id>` | What shipped this week, and each list of decisions | shipped, flag | | on a team | That list | out: no team here |
| `files-all` | Go to Files | file | "every file you can see" | always | The Files place | done (as before) |
| `files-project-<name>` | Files in <project> | file | | each project the Files place lists | That project's page in it | done (as before) |
| `view-connections` | Go to Connections | link | "tools your agents reach" | always | The Connections page | out: no Connections page here |
| `view-teammates`, `new-teammate`, `teammate-<id>`, `teammate-pause-<id>` | Go to Teammates, New teammate, each teammate, Pause or Resume each | team, add-person, person, pause or play | | teammates are on | Their pages, and pausing one | out: no teammates here |
| `theme-light`, `theme-dark`, `theme-match` | Theme: Light, Theme: Dark, Theme: Match system (the two not worn) | theme | "theme" | always | The web window's theme | out: this window draws the dark look only, and the theme is the web window's own setting |
| `theme-sky-<id>` | Theme: <sky> (each not worn) | theme | "theme" | Home is on | A sky theme | out: this window draws one sky |
| `update` | Restart to update Mercury | download | "version N is ready" | a new version is ready or installing, and names its version | `zero:update-install` | to port |
| `update-move` | Move to Applications | move | "cannot update from here" | this copy cannot update itself from where it is | `zero:update-move`, and its error said when it did not move | to port |
| `update-check` | Check for updates | refresh | "look for one now" | no update waits | "Looking for a new version.", then `zero:update-check` and what it found, in Settings' words | to port |
| `stale` | Restart Mercury to pick up main-process changes | refresh | "N files changed" | the app's main process changed on disk | Nothing: the bar stays open | to port |
| `pause` | Pause agents, or Unpause agents | pause, or play | "no new sessions spawn", or "sessions will spawn again" | always | `zero:supervisor-pause` | to port |
| `resume-stopped` | Resume interrupted agents | play | "after limits or crashes" | no rows ticked | `zero:resume-agents`, and how many respawn | to port |
| `standing` | General agent instructions… | instructions | "every agent, every task" | always | Opens the instructions card over the app (`Standing`) | out: the instructions card is a screen this window does not have (for the owner: Settings → Instructions brings the web window forward on the same files) |
| `usage` | Usage and limits | gauge | "what is left" | always | Settings, on Claude Code | to port |
| `settings` | Settings… | settings | "workspace and every project" | always | Settings, on General | to port |
| `shortcuts` | Keyboard shortcuts | keyboard | "every key Mercury has" | always | Settings, on Shortcuts | to port (Shortcuts is the web window's page: as this window's Settings does, the web window comes forward on it) |
| `flip-view` | Show as a board, or Show as a list | board | B | not in a full-screen file, a search or a team's page | Turns the Inbox into the board | out: the board is a screen this window does not have (B says so in a line) |
| `import-agents` | Import chats and agents | download | "Claude Code or Codex" | always | Opens the import card | out: the import card is a screen this window does not have |
| `tutorial` | Take the tutorial | help | "nothing is kept" | always | Starts the tutorial | out: no tutorial here |
| `first-run` | Set Mercury up again | refresh | "makes a project" | always | Walks the welcome screens again | out: no welcome screens here |
| `demo` | Open a demo copy | spark | "an invented team · yours keeps running" | only once "demo" is typed (`onlyWith`, `unlisted`) | Opens a second copy on an invented team | out: a copy of the web app, nothing of this window's |
| `keyhints` | Turn off keyboard shortcut hints, or Turn on… | keyboard | "shown when you point" | always | The key hints switch | to port |

## The rows on the thread under the keys

The thread under the keys is the open conversation, else the Inbox row under the pointer, else
its picked row (`focused ?? pointed`); Home and anything covering the list have none. In
`App.tsx`'s order:

| Id | Words | Drawing | Key, or hint | When | Does | Status |
|---|---|---|---|---|---|---|
| `remote-control` | Remote control | swap | | a conversation is open | Its remote control panel | out: no remote control panel here |
| `open-terminal` | Open terminal | terminal | | a conversation is open | Its terminal | out: no terminal here |
| `approve` | Approve recommended, Approve as proposed, or Approve and run | check | | not an outside agent's, no answer yet, and a review, a question with options, or an open proposal the person did not file | Held three seconds, then the recommended option, "Approved. Proceed as you proposed." or "Approved. Run it." (`resolve`) | to port |
| `run-now` | Run now | run | | no run on it, not pushed already, and queued or belonging In progress (`offersRunNow`) | `zero:run-now`, and what came back | to port |
| `done` | Mark done (Done on an outside agent's row) | done | E | not finished, or an outside agent's | The close, held three seconds | done (the outside agent's word fixed) |
| `reopen` | Reopen | reopen | | finished, and not an outside agent's | `zero:answer` with `status: open`, "Reopened: …", and Z marks it done again (`reopenDone`) | to port (it wrote the Reopen button's `zero:reopen` here) |
| `close-task` | Close task | closed | | a task a teammate gave | Closes it for both | out: no teammates here |
| `reply` | Reply | reply | R | always | Opens it with the caret in its reply box | done (as before) |
| `try-again` | Try again | refresh | | nothing will start it without a person (`offersTryAgain`) | `zero:resume-items` on it | another task (Try again) |
| `resume-any` | Resume this agent, or Resume this agent (reopen and carry on) when finished | refresh | | not an outside agent's, no run on it, not stoppable, and Try again not offered | `zero:resume-items` on it, what happened said, and the next thread when it started | to port |
| `read` | Mark read | read | | unread | Counted as read now (`markSeen`) | to port: `zero:phone-seen`, the way this window tells the web window a thread was read |
| `unread` | Mark unread | unread | | read | The web window's own record of it set back to unread | out: only the web window keeps what has been read, and the app has no channel that sets a thread back to unread (the phone's only moves it forward) |
| `unsnooze` | Back to Inbox, or Let it run now when its agent set the moment | inbox | E | put off until later | Brings it back | done (as before) |
| `snooze` | Later | later | L | always | Opens the Later picker | done (as before) |
| `route-*`, `answer-*`, `ask-*`, `give-*`, `take-back` | the strip a teammate's task opens with, answering their ask, Ask … for a review or for help, Hand over to …, Take it back | give, reopen | | on a team | Teammates' moves | out: no teammates here |
| `wait-<id>`, `wait-stop` | Waits on <thread>, Stop waiting | wait | | on a team | What it waits on | out: no teammates here (the choices are a team's) |
| `decide`, `shipped` | Save its answer as a decision, Mark as shipped / Not shipped after all | flag, shipped | | on a team | The team's records | out: no team here |
| `see-team`, `see-people`, `see-private` | Share with the team, Share with chosen people…, Make private | team, add-person, lock | | on a team | Who sees it | out: no team here |
| `pin` | Pin thread, or Unpin thread | pin | | not finished, not an outside agent's | `zero:set-thread-pins`, unpinning forgets a dragged place (`zero:set-thread-places`), "Pinned to the top of your list" or "Unpinned" | to port |
| `rename` | Rename thread… | rename | | a thread of work, or a conversation | Opens its title for typing, on its page | out: this window has no box to rename a thread in |
| `code` | View code changes | code | | the open conversation shows a change | Opens the change beside it | another task (View code changes) |
| `place` | Run on this Mac, Keep on this Mac, Run on the cloud, or Move to the cloud | cloud | "in the cloud now", "cloud allowed", "this Mac only", "on this Mac now", "needs this Mac" | not finished, not an outside agent's, and the cloud could run it (`threadPlace` not `off`) | What the place button's panel writes, and its line said | to port |
| `end-repeat` | Stop repeating | stop-repeat | | it is a repeating task's run | Ends the rule | out: this window does not read repeating tasks |
| `add-people` | Add people… | add-person | | someone could be added | Adds them | out: no teammates here |
| `model-<engine>:<model>` | Model: <name>, up to three | spark | the agent's name | not an outside agent's; the models used lately, the other agent's first where this Mac offers it, never the one it is on | `zero:answer` with the agent and model and no words, and "<name> picks this up next" | to port |
| `move-refused` | Move to another project | move | why it cannot move | it cannot move (finished, an outside agent's, or its work is not on main) | Says why | to port |
| `move-<slug>` | Move to <project> | move | | it can move: each other project a thread can be filed in | `zero:move-item`, "Moved to <project> · press Z to undo", Z closes the copy and opens the original, the copy opened if the original was | to port |
| `stop` | Stop Agent (Moves to Inbox), or Stop Task (Moves to Inbox) | stop | "nothing has started on it yet" for a task | stoppable now | Stops it | another task (stopping an agent) |
| `send-back` | Send back to agent | reopen | | blocked | `zero:reopen`, "Back in the queue → <project>", and the next thread | to port |
| `redeliver` | Restart the agent (it stopped) | refresh | | stalled, and Try again not offered | `zero:redeliver`, and what came back | to port |
| `open` | Open | open | ↵ | it is a row, not the open conversation | Opens it | done (as before) |
| `priority-urgent`, `-high`, `-medium`, `-low` | Priority: Urgent … Priority: Low, "(current)" after the one it shows | priority-urgent … priority-low | | always | `zero:answer` with the level and no words, then "Priority: high" | to port |

A teammate's thread opened from Team offers only the answers to their ask (`answer-*`): out, no
teammates here. Rows ticked turn the thread's rows into Done, Later, Back to Inbox, Mark read,
Mark unread, Resume agents and the four levels "(N selected)": out, no row can be ticked here.

## What words typed find

| What | Words, drawing, hint | Does | Status |
|---|---|---|---|
| Threads (`thread-<product>-<id>`) | the title over the sentence found, thread, "<project> · <age>", dimmed when finished | Opens it | done (as before) |
| Repeating tasks (`thread-repeat-…`) | the rule's title, thread, "Repeats …" | Opens the rule | out: this window does not read repeating tasks |
| Teammates' threads (`card-…`) | the title, thread, "<name>'s thread" | Opens their card | out: no teammates here |
| See all N threads (`all-threads`) | search | The thread search page | out: no search page here |
| People and groups (`chat-…`) | person, hash | Their chat | out: no people or groups here |
| Found inside things: Lines (`line-…`) | the line, text, who · where · when | Opens the thread at that message | see below |
| Found inside things: Files (`file-…`) | the file's name (or the line, after the name), file | Opens the thread, the file beside it or big | see below |
| Found inside things: Comments (`comment-…`) | the comment, after the file's name, comment | Opens the file big at the comment | see below |
| Found inside things: Decisions (`decision-…`) | the decision, flag | Its group | out: no Groups here |

### Found inside things

90 ms after the words stop changing, the web window asks the app (`zero:find-in-threads`, with
`{ query, limit: { lines: 6, files: 5, comments: 4 } }`) and keeps three lines, three files and
two comments of what comes back that sit in a thread it lists (`insideGroups`, newest first but
files in the app's order), each group under its heading after every other row. A line opens its
thread and brings that message to the middle of the page, marked for a moment (`revealLine`); a
file opens its thread with the file beside it, or big when it is not one that opens beside; a
comment opens the file big at the comment. The app is asked to read the threads ahead as the
bar opens (`zero:warm-find-in-threads`).

Status: to port.
