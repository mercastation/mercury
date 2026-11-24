# Around the Inbox: the row of tasks that could not run, the plan bar, a new project

The checklist of three things the web window puts in front of the person outside a
conversation's inside, read off its code top to bottom before any of it was ported, and whether
this window does the same. A line is ticked only once it is ported and checked; how it was
checked is said on the line. A line left open says exactly why.

A sweep of the web window (2026-11-23) found all three missing or dead here: the row was drawn
but opened onto "This conversation is not here any more." and took no E; the bar was never
drawn, and a signed-out row's line was written as if it never would be; and every New project
said only that a project is made in the other window.

The spec, by file (`R` is `renderer/src`, `S` is `shared`, `M` is `main`):

- Part A: `R/trouble-row.ts` (all of it), its uses in `R/App.tsx` (`troubleClosed`,
  `setTroubleClosedAt`, `troubleItem`, `closeTroubleRow`, `markDone`'s and `resolve`'s branches,
  `openSnooze`'s refusal, the guards on every write), `R/components/Focus.tsx` (`trouble`,
  `made`), `R/components/List.tsx` (`trouble-row`, `trouble-rule`), `R/list-rules.ts`
  (`rowSummary`), and the sentences it reads, `S/spawn-trouble.mts` (`strandedTitle`,
  `causeWhat`, `causeNext`, `causeBreakdown`). Web tests: `tests/the-row-that-says-nothing-is-running.test.mts`.
- Part B: `R/plan-setup.ts` (`COPY.bar*`, `planName`, `needsPlan`, `sureNoPlan`, `BarWords`,
  `planBar`), `PlanBar` in `R/components/PlanSetup.tsx`, `.ps-bar` in `R/styles.css`, in
  `R/App.tsx` THE FALLBACK BAR (`checkPlan`, its `focus` listener) and where the bar is drawn,
  `engineSetup` in `R/api.ts`, `signInBar` in `R/live-line.ts`. Web tests:
  `tests/the-plan-bar-offers-the-agent-that-is-on-the-computer.test.mts`,
  `tests/the-plan-bar-only-says-agents-cannot-start-when-that-is-true.test.mts`.
- Part C: `R/components/NewProject.tsx`, its call in `R/App.tsx` (`newProject`, `onCreate`,
  `onRefused`, `whyNotMade`, `shortFolder`), `R/project-folder.ts`, `nameFromFolder` and `COPY`
  in `R/onboarding.ts`, `S/name-words.mts`, `S/project-folder-check.mts`, the handlers of
  `zero:choose-folder` and `zero:create-product` in `M/ipc.mts`, the row `new-project` in
  `R/components/Palette.tsx`. Web tests: `tests/a-path-ending-in-a-dot-is-the-folder-it-names.test.mts`,
  `tests/the-setup-refuses-your-whole-mac.test.mts`, `tests/the-folder-chooser-opens-where-the-projects-are.test.mts`.

Each line: **[x]** done, **[ ]** not done. "Tests" are `cargo test` in `desktop/`; "pictures" are
scenes in `tools/scenes/` photographed from the web renderer and drawn here from the same
fixture, a part's rectangle in the web's dump held against the native tree's. The two windows
wear different palettes and rails at the time of writing (README, "What still differs"), so a
part is compared by its size and its place against its neighbours, and its crops looked at side
by side, not by whole-window numbers.

## A. The row that says tasks have not been able to run

### A1. The row

- [x] A1.1 Built from the supervisor's `spawnTrouble` as a thread with the one id `trouble`, in
  no project, open, at the middle level, both its times when the pile began (`troubleRow`).
  Tests `rules::trouble_row::tests::it_is_one_row_with_one_id_no_urgent_mark_and_no_project`,
  `it_is_stamped_with_when_this_started_and_opens_with_the_line_the_row_prints`.
- [x] A1.2 Its title, "N tasks have not been able to run", one task "has" (`troubleTitle`,
  `strandedTitle`). Test `the_row_says_how_many_in_the_words_the_app_uses_everywhere_else`.
- [x] A1.3 Its second line, one short clause per cause in the supervisor's order, "back at" the
  hour read out of the cause's sentence (`troubleSummary`, `shortNext`, all fourteen causes); the
  row prints the body's first line only (`rowSummary`). Tests
  `its_second_line_leads_with_what_happens_next_and_what_needs_her_hands_first`,
  `a_supervisor_from_before_the_breakdown_still_draws_its_own_two_sentences`.
- [x] A1.4 First in Needs you, above everything, kept whatever the View and filters' filters
  (already here before this port: `rules.rs`, `rules/display.rs`).
- [x] A1.5 Gone while `zero.troubleClosed` is the pile's `since`; back by itself the moment a new
  pile begins. Test `trouble::tests::it_comes_back_by_itself_when_a_new_pile_begins`.
- [x] A1.6 The row's look in the web window's newer Inbox: a rule at its left edge in place of
  the select box, and the second line under the title, the row drawn as its other rows are, with
  "Answered", as the web's is (`docs/parity-inbox-rows.md` A7 and B14). Picture `inbox-trouble`:
  the rule at 78, 213, 2 by 41 in both; test
  `inbox::tests::words_after_the_title_that_the_second_line_says_leave_the_first_and_the_app_s_own_row_wears_its_rule`.
- [ ] A1.7 Its byline, "nothing has started for {age}" (`Byline.tsx`): the opened row's band in
  the web window draws no byline on it in the scene (`thread-trouble`), and this window draws
  none either; not checked on another page that might.

### A2. Opening it (`made` in `Focus.tsx`)

- [x] A2.1 Opened by Enter, a press, ⌘K's Open, or a line naming it, it is a conversation's page:
  the strip with "Inbox / 3 TASKS HAVE NOT BEEN ABLE TO RUN", the arrows to the threads before
  and after it in Needs you, Done and the dots. Picture `thread-trouble`: Done at 1143.78, 63,
  31.22 by 17 in both.
- [x] A2.2 Under the strip, bare, no title, no label, no conversation: what `troubleBody` says,
  as markdown in the column's measure, 28 points into the page and 28 over the text
  (`.outcome`). Picture `thread-trouble`: every line the same size and height, to the hundredth
  (the summary 760 by 25.59 at 160; the first cause's paragraph at 193.59; the items 738 wide at
  227.19, 255.78, 322.97; the second cause at 289.38); crops looked at side by side.
- [x] A2.3 `troubleBody`: the summary, then each cause's "**what.** next" and its tasks by title
  with "(project)", a task no longer here as "A task that is no longer in your inbox", the rows no
  cause accounts for as "**N more.** Mercury keeps trying on its own.", ids only from an old
  supervisor. Test `opening_it_names_every_cause_what_happens_next_and_the_tasks`, with the web
  test's own night (four signed out, seven at 6pm).
- [x] A2.4 No reply box, and R and a letter open none. Test
  `trouble::tests::opening_the_row_says_which_tasks_are_stopped_and_what_each_waits_on_with_no_reply_box`.
- [x] A2.5 L, and Later in its menu, say "Nothing to put off here. Press E to close it until
  something else stops." Same test.
- [x] A2.6 The dots open the thread's menu over it. Picture `thread-trouble-menu`: Later at
  1012.13, 142.75 and Priority at 177.75, where the web's stand (1012, 142.75 and 177.75).
- [ ] A2.7 That menu's rows Open terminal, Rename, Move to another project and Pin thread: this
  window has none of them on any thread (README, "Choosing in a conversation").
- [ ] A2.8 A level picked in that menu: the web window writes it (`zero:answer` with product ""
  and id `trouble`), which no ledger can take, and nothing is said. This window closes the menu
  and writes nothing. For the owner.

### A3. Putting it away (`closeTroubleRow`)

- [x] A3.1 E on its row in Needs you, E inside it, Done in its strip, and ⌘K's Mark done and
  Approve and run, all put it away (`markDone`'s branch, `resolve` handing it to `markDone`).
  Tests `e_puts_it_away_writes_it_to_the_web_window_three_seconds_later_and_z_puts_it_back`,
  `done_in_its_strip_puts_it_away_and_goes_on_to_the_next_thread`, and the command bar's rows in
  `rules::palette::tests::the_whole_list_is_the_threads_rows_then_the_webs_other_rows_in_its_order`.
- [x] A3.2 "Done: what is not running" with Z and Undo for three seconds; nothing written until
  they pass, and the row stays in the list until they have: the put-away is a fact kept beside
  the lists (`troubleClosed`), set only when the held write goes, and the hold hides no made
  row. (This window took it out at once until 2026-11-24, which the web window's picture of
  `inbox-trouble-done` showed to be wrong.) Pictures `inbox-trouble-done` (the line
  158.12 by 17.55 at 700.45 in both) and `thread-trouble-done` (at 800.57 and 800.44, 62.23, the
  same size; the strip then on the next thread, "Checkout fails…", with Reopen).
- [x] A3.3 From inside it, on to the next thread in Needs you (`leaveResolved`). Same test and
  picture.
- [x] A3.4 Z inside the three seconds: back, nothing written, "Undone. Still open: what is not
  running" (`graceUndoneLine`). Test above.
- [x] A3.5 Written after three seconds, or as soon as something else is done, where the web
  window keeps it: `zero:trouble-closed-here` with `{ "since": <the pile's since> }`, a channel of
  the desktop app's door that sets `zero.troubleClosed` in the web window by the event its own
  page listens for (`TROUBLE_CLOSED` in `R/trouble-row.ts`), as E there sets it. Tests above (the
  payload asserted), and `tests/the-local-door-opens-only-with-its-key.test.mts` ("the native
  window's E and Z on the row of tasks that could not run").
- [x] A3.6 Read back from there: the door's `zero:window-keeps` now hands over
  `troubleClosed` with the three it had, and this window holds what it wrote until that record
  says the same. Test above (`keep`), and the door's test of the keeps script (four keys).
- [x] A3.7 Once written, Z takes it back: "Back in your inbox: what is not running", asked first
  past half a minute ("put the stopped tasks back in your inbox"), writing what it was before
  (`{ "since": <was> }`). Test above.
- [x] A3.8 Stops nothing and starts nothing: no other write. Tests (only the one channel tried).
- [ ] A3.9 A put-away the web window did not take (an app from before the channel, or no window
  open): the row comes back and a line says "Not done: restart Mercury first, so it can take
  that from this window." or "Not done: <why>". The web window's own E cannot fail, so these
  words are this window's (the same shape as Mark unread's). For the owner. Test
  `done_in_its_strip_puts_it_away_and_goes_on_to_the_next_thread`.
- [ ] A3.10 Z's entry goes on the pile once the web window has the put-away, where the web
  window's goes on at the three seconds: a refused write leaves nothing to take back.

### A4. The command bar on it

- [x] A4.1 Over its row or with it open: Approve and run, Mark done (E), Later (L), and Open (↵)
  from the Inbox, in the web's order and words. Picture `palette-trouble` (each row where the
  web's stands, 379.13 against 379, at 244.5, 282.5, 434.5, 662.5); test above.
- [ ] A4.2 The web's other rows on it: Reply (it has no box), Resume this agent, Mark unread
  (the door's channel takes only a project's thread), the three models, Move to each project and
  the four levels. Each does nothing to this row in the web window or writes what no ledger can
  take; none is offered here. For the owner.

## B. The bar that says agents cannot start until a plan is signed in

### B1. Whether it is up

- [x] B1.1 Asked as the window opens: `zero:engine-setup` with `{ "action": "ready", "engine":
  "claude" }` and the same for `codex`, read as `engineSetup` reads them: not `ok`, or no answer,
  is sure of nothing. Test `rules::plan_setup::tests::a_reading_is_the_engines_answer_and_a_check_that_did_not_run_is_sure_of_nothing`.
- [x] B1.2 Up only on a certain no from both (`sureNoPlan`). Test
  `the_bar_says_agents_cannot_start_only_on_a_certain_no`, the web test's cases.
- [x] B1.3 Asked again each time the window comes to the front while it is up, and not while it
  is down. Test `plan_bar::tests::coming_back_to_the_window_reads_the_computer_again_only_while_it_is_up`.
- [ ] B1.4 Not asked while the first run is up (`!fresh`): this window has no first run.

### B2. Where it stands

- [x] B2.1 First on the page, across the card under its strip, the page laid out 56 points lower
  under it: on the Inbox, Home and Files. Pictures `inbox-plan-bar` (the bar 56 high at 104 in
  both; the web's 1200 wide on a card 1200 wide, this one 1176 on a card 1176 wide) and
  `home-plan-bar`; test `the_bar_stands_over_the_inbox_says_why_and_lays_the_page_out_under_it`.
  Files not photographed.
- [x] B2.2 Not over an open conversation, and not while anything runs. Test
  `it_stands_over_no_conversation_and_not_while_anything_runs`; picture `thread-plan-bar-line`.
- [x] B2.3 Not over Settings (the web window hides the page under Settings).

### B3. What it says

- [x] B3.1 "Agents cannot start until they have a plan to run on.", "… until you sign in to your
  Claude plan.", "… your ChatGPT plan.", "… your Claude or ChatGPT plan.", then "Takes a couple of
  clicks." in the dim ink. Pictures `inbox-plan-bar`, `-codex`, `-both`; tests
  `the_bar_leads_with_the_agent_that_is_on_the_computer`.
- [x] B3.2 Two buttons, the agent on the computer leading: "Set up with Claude" or "Sign in to
  Claude", then the other; the first on the selected ground in the ink, the second quiet. Pictures:
  every button the web's size and place to the hundredth (142.83 by 31 at 929.17, 116; 154 at
  1086; 142.61 at 940.56; 131.45 at 951.95; 142.61 at 1097.39), its words at the same offset.
- [x] B3.3 Under the pointer a button takes the hover ground and the ink. Picture
  `inbox-plan-bar-hover`, crops side by side.
- [x] B3.4 While it is up, whether drawn or not, a signed-out thread's line says "Claude Code is
  signed out. Sign in to Claude at the top of your inbox, and this starts on its own." (`signInBar`).
  Picture `thread-plan-bar-line` (the same words, at 311.54, 20 high; 596.63 wide against the
  web's 596.52); test `while_it_is_up_a_signed_out_thread_points_at_it`.

### B4. Its buttons

- [x] B4.1 A press hands over to the web window: `zero:open-plan-setup-here` with `{ "plan":
  "claude" | "codex" }`, a channel of the desktop app's door that brings its window forward and
  opens its setup card on that plan (`setSetupPlan`), as its own bar's press does. Test
  `its_buttons_lead_with_the_agent_on_the_computer_and_hand_over_to_the_web_windows_setup_card`;
  the door's in `tests/the-local-door-opens-only-with-its-key.test.mts` ("the native window's
  plan bar").
- [ ] B4.2 The setup card itself (`PlanSetupCard`): the first run's, agreed with the owner to
  open in the web window. A deviation by agreement.
- [ ] B4.3 With no app to hand over to, a line says "A plan is set up in Mercury's own window.":
  this window's words, as the cloud's setup has its own. For the owner.
- [x] B4.4 After the card, the bar reads the computer again when this window comes back to the
  front (B1.3). The web window reads it again as its card closes, which this window cannot hear.

## C. New project

- [x] C1 Every door this window has: the project menu of the box a thread is started in (New
  project, last), the command bar's first row "New project…" (the plus, no key, no hint), and
  New project on Settings › Projects. Tests `new_project::tests::the_project_menu_of_the_box_opens_the_folder_window_and_makes_the_project`,
  `the_command_bars_first_row_opens_the_folder_window_and_the_folder_picked_is_made_a_project`,
  `settings::tests::new_project_on_the_projects_page_opens_the_folder_window_over_settings`;
  picture `palette-trouble` (the row at 379.13, 168.5, 88.89 by 19, the web's at 379, 168.5).
- [x] C2 The Mac's own folder window, at once, over this window: Rune's `Tx::open_folder`, a
  sheet of this window. The app's `zero:choose-folder` hangs its window on the web window
  (`dialog.showOpenDialog(window, …)`), which would bring that window forward over this one.
- [x] C3 It opens where the app's would: the first folder of `chooserStartsIn` (beside the
  projects already connected, commonest parent first, then the folders people keep code in, the
  home folder last) that is there. Test `the_folder_window_opens_where_the_projects_are`.
- [x] C4 Cancel makes nothing and says nothing. Test
  `a_cancel_makes_nothing_and_a_folder_that_is_not_a_project_is_turned_down_in_the_apps_words`.
- [x] C5 The folder picked is checked as the app checks it before handing it back
  (`folderFromChooser`): a `.git` is its repository; the home folder, the guarded seven, the
  system's own folders, a disk and a `.git` inside are turned down, and a folder not there, each
  in the app's sentence, said in the line (`onRefused`). Tests `a_path_is_the_folder_it_names`,
  `the_folders_that_are_not_projects_are_turned_down_with_what_to_pick_instead` and the window's.
- [x] C6 Named after its folder (`projectFromFolder`, `nameFromFolder`, `titleFromFileName`:
  `greet-js` is "Greet JS"), and made: `zero:create-product` with `{ name, repoPath }`. Tests.
- [x] C7 Made: "New project: X · its code is in ~/…" (`shortFolder`), and the next thread is
  addressed to it (`rememberProject`, `setPickProject`). Picture `palette-new-project` (the line
  326 by 35.1 at 907.13, 682.9 against the web's 907, 682.91, the same words); test.
- [x] C8 Turned down by the store: the walk's own words (`whyNotMade`: "You already have a
  project called X. Give this one another name.", a name with no letters, "Mercury could not
  make the project. <why>."). Picture `palette-new-project-taken`; tests.
- [ ] C9 The folder window's own words: the app's asks Electron for "Pick a project folder" (a
  title macOS does not show) and a button "Use this folder"; Rune's names neither, so its button
  is the system's Open. For the owner.
- [ ] C10 In a browser tab the web window draws its own folder picker (`browse`): this window is
  never a tab.
- [ ] C11 The two other doors, the agent import and a teammate's page, are not in this window.
