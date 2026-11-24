# The ship question, what a close asks first, and the threads a thread filed

The checklist of three things the web window puts in a conversation that this window did not
draw (`docs/parity-asks.md`, part B, B4 to B6), read off the web window's code top to bottom
before any of it was ported, and whether this window now does the same. A line is ticked only
once it is ported and checked; how it was checked is said on the line. A line left open says
exactly why.

The spec, by file (`R` is `renderer/src`, `S` is `shared`):

- B4: `R/ship-offer.ts` (all of it); its uses in `R/components/Focus.tsx` (the change's
  summary read for its figures, its standing and its files; `shipAsk`, `stripShown`,
  `options`, `ask`/`askAll`, `onShown`, the one-at-a-time page's `.one-choices` strip with
  `bare={!shipAsk?.headed}`, `ChoicesShow`); `R/components/OptionsOffer.tsx` (the head, "The
  question, in full", the rows); `offerNow`, `pickOption` and the keys in `R/App.tsx`;
  `needsYourWords`, `discardsTheChange`, `offerIsLive` in `R/format.ts`; `askIsClipped` in
  `R/ask-line.ts`; `.opt-head`, `.opt-head-ask`, `.opt-peek` in `R/styles.css`, `.one-choices`
  in `R/one-thing-at-a-time.css`; `wakesItselfAt` in `S/answers.mts`, `saysARunGotNowhere` in
  `S/spawn-trouble.mts`, `referencedFiles` in `S/referenced-files.mts`. The engine's side:
  `zero:code-change` with `summary: true` (`main/code-change.mts`, the standing written by
  `main/git-change.mts`).
- B5: `R/close-asks-first.ts`, `R/discard-asks-first.ts`; in `R/App.tsx` `askingToClose`,
  `closeAskOn`, `closeAskWhy`, `closeAskRaised`, `markDone`, `answerCloseAsk`,
  `keepThreadOpen`, the key handler's `closeAskOwnsKey` branch; in `R/components/Focus.tsx`
  `closeAskStrip`, `askAtTask`, `closeAskRef`; in `R/components/ItemThread.tsx` `askTurn`,
  `askHere`, the unfolding effect; `askTurnOf` in `R/filed-in-thread.ts`; `bringIntoView` in
  `R/thread-bottom.ts`; `notShipped`, `shipItAsk` in `S/not-shipped.mts`; `.opt-named*`,
  `.close-ask-at-task` in `R/styles.css`.
- B6: `R/components/ThreadsMade.tsx`, `R/components/threads-made.css`, `R/threads-made.ts`,
  `R/filed-in-thread.ts`; where `R/components/ItemThread.tsx` places them (`made`,
  `madeList`, `tail`, `made.atFoot`) and `R/components/Thread.tsx` (`tail?.(ends)`); the
  handlers in `R/components/Focus.tsx` (`openFiled`, `approveFiled`, …) and `R/App.tsx`
  (`filed=`, `resolve`, `doItHere`, `markDone` with `stay`/`force`, `answerAllFiled`,
  `stateOfMine`/`tabState`).

Each line: **[x]** done, **[ ]** not done. "Tests" are `cargo test` in `desktop/`: the rules'
own cases in `src/rules/ship_offer.rs`, `src/rules/close_asks.rs`, `src/rules/threads_made.rs`,
`src/rules/filed_in_thread.rs` and `src/rules/shipping.rs` (the web's own test cases, from the
`tests/*.test.mts` files each module names), the window's in `src/ship.rs`, `src/closing.rs` and
`src/made.rs`. "Pictures" are the scenes `tools/scenes/thread-ship*.mjs`,
`thread-close-*.mjs` and `thread-filed*.mjs`, photographed from the web renderer and drawn here
from the same fixture; the rectangles compared are in part D.

## B4. The app's own question after a run that left a change (`ship-offer.ts`)

### B4.1 The rules, function by function (`src/rules/ship_offer.rs`)

- [x] `changedFiles`. Test `the_changed_files_and_the_ones_the_thread_names`. The window reads
  the files off the summary `change.rs` already parsed, so this one is kept for its cases.
- [x] `onlyNamedFiles`: every case of `a-file-the-thread-shows-is-not-offered-as-code-to-ship`.
- [x] `standingFrom` and `UNREAD`: test `the_standing_is_read_off_a_saved_change_and_nothing_else_is_either`.
  The window reads the same three facts off the parsed summary (`Change`, which now keeps
  `knowsYou`).
- [x] `shipOfferFor`, its five outcomes and `held`: tests `the_offer_comes_off_where_the_folder_stands`,
  `a_file_the_thread_shows_is_asked_about_in_plain_words`,
  `a_git_that_knows_nobody_asks_for_a_name_and_an_email`,
  `ship_it_is_not_recommended_over_the_persons_own_words`, `the_offers_words_are_the_webs`.
- [x] `saidNotToShip`: the same test, the web's seven sentences.
- [x] `agentSaidNotReady`: test `the_agent_saying_it_is_not_ready_holds_the_recommendation`, the
  web's twenty-five sentences.
- [x] `folderMayStillMove`, `FOLDER_SETTLES_MS`: test `the_folder_is_read_again_for_two_minutes_after_the_result`.
- [x] `shipOfferLive`, every condition: tests
  `a_finished_run_that_left_a_change_is_asked_about_until_something_answers_it`,
  `a_pick_taken_back_is_offered_again`, `an_answer_that_says_nothing_is_left_asks_nothing`,
  `a_run_the_app_says_got_nowhere_is_not_offered_for_shipping`,
  `the_agents_own_question_wins_over_the_apps`.

### B4.2 What is read

- [x] The change's summary is the read the strip's code button already makes (`change.rs`):
  asked when the thread opens, as its row changes, every four seconds while a run is up and for
  two minutes after one ended. It now keeps `knowsYou`. Pictures (the fixture answers it).
- [x] `change` is whether the summary has figures; `running` a run up on the thread or the
  thread being one of the person's own agent sessions (`ship::offer`). Tests in `ship.rs`.
- [x] The files the thread's words name: its result, note, body and answer, read against the
  project's `dir` (`referenced_files`). Test `files_the_thread_shows_are_asked_about_by_name`;
  picture `thread-ship-files`.
- [x] `held`: the body and answer saying not to ship, or the result saying it is not ready.

### B4.3 What is drawn

- [x] The offer stands where an agent's options do, in the strip over the reply box, numbered,
  "recommended" beside it, "↵ send" on the lit one, washed under the pointer, its little arrow
  folding it. Pictures `thread-ship`, `thread-ship-hover` (the row's wash and ink).
- [x] Bare for "Ship this change?"; the other three heads inside the strip's frame, 12.5
  points, 600, two lines at most. Pictures `thread-ship-loose`, `-name`, `-files`; tests
  `a_finished_run_that_left_a_commit_is_offered_for_shipping_bare`,
  `work_nobody_committed_says_so_over_its_options`.
- [x] A head cut short at two lines shows whole in a card over the strip while the pointer is
  on it ("THE QUESTION, IN FULL"). Test `a_head_cut_short_shows_whole_over_the_strip_while_the_pointer_is_on_it`.
  Not photographed: no head is cut short at the window sizes the scenes use.
- [x] Folded, the reply box's sign counts the offer's options; not while a question stands in
  their place (`!closeAsk`, which the discard question's sign was missing too, and now has).
- [x] Not while an answer is on its way, nor while a close or a discard question stands.
  Test `throwing_it_away_asks_first_then_sends_the_pick`.
- [x] The agent's own offer wins. Test `nothing_is_offered_over_a_thrown_away_change_or_an_agents_own_question`.

### B4.4 What its options do

- [x] The numbers, the arrows and Enter answer it, and a press does (`offer_now` read by the
  keys, `pick_option` and the typing rule). Tests in `ship.rs`.
- [x] "Ship it" (and "Commit it and ship", "Add it to the project"): "Option 1: …" as any pick.
  Test `a_finished_run_that_left_a_commit_is_offered_for_shipping_bare` (`zero:answer`).
- [x] "Discard the change", "Throw it away": the question before a discard, then the pick.
  Test `throwing_it_away_asks_first_then_sends_the_pick`.
- [x] "Commit it and ship, as the name and email below": opens the box with its words, sends
  nothing. Test `a_git_that_knows_nobody_opens_the_box_for_a_name_and_an_email`.

## B5. What a close asks first (`close-asks-first.ts`)

### B5.1 The rules (`src/rules/close_asks.rs`, `src/rules/shipping.rs`, `src/rules/filed_in_thread.rs`)

- [x] `closeAsk(n)`: tests `with_no_task_waiting_the_close_goes_through`,
  `one_task_waiting_offers_to_start_it_close_both_or_keep_the_thread`,
  `several_say_how_many_and_recommend_deciding_each_one` (one, two, three, twelve; no option
  says approve or drop).
- [x] `unshippedAsk()`: test `work_that_never_reached_main_recommends_shipping_it`.
- [x] `discardAsk()`, now the same strip's: test `the_discard_question_keeps_the_change_on_two`.
- [x] `closeAskOwnsKey`: test `it_takes_only_the_keys_that_answer_it`.
- [x] `askTurnOf`: test `the_close_question_stands_under_the_newest_turn_with_a_task_waiting`.
- [x] `notShipped`, `shipItAsk` (with the branch a cloud run set the work aside on): test
  `the_agent_is_asked_to_ship_it_in_words_that_name_where_the_work_is`.

### B5.2 When it is asked

- [x] Every close goes through it before the close is held (`close_now`): E on the thread and
  on a row of Needs you or In progress, the Done button, ⌘K's Close, an agent's own "Close this
  task" and "Keep it on the branch". The once-ever warning about a waiting agent still comes
  first, except for an option picked. Tests in `closing.rs`.
- [x] Work that never reached main asks unless the close says it is meant. Test
  `a_close_on_work_that_never_reached_main_asks_first`; pictures `thread-close-unshipped`.
- [x] Filed tasks still waiting ask unless the close is forced. Test
  `a_close_with_a_filed_task_waiting_asks_what_happens_to_it`; picture `thread-close-filed`.
- [x] A close from a list opens the thread and asks there; leaving the thread drops the
  question; asking again raises it again. Tests `a_close_from_the_list_opens_the_thread_and_asks_there`,
  `a_press_answers_it_and_leaving_the_thread_drops_it`.
- [x] Nothing is lit when it comes up. Same tests.
- [x] One question at a time: raising a close's puts the discard question away and the other
  way round.

### B5.3 Where it stands and how it looks

- [x] The unshipped question at the conversation's foot, where the discard question stands,
  22 over it and 6 under, over the line that says what is happening (the discard question
  stood under that line here; both now stand over it, as `closeAskStrip` does). Pictures
  `thread-close-unshipped`, `-unshipped-lit`.
- [x] The filed question under the list of the tasks it is about, 12 under it; at the foot
  when they belong to the newest turn. Pictures `thread-close-filed`, `-filed-two`,
  `-filed-turn`.
- [x] Raising the filed question unfolds the conversation and its every run, once the thread
  has landed (landing on a thread puts back only what was unfolded when it was last left), and
  brings the question onto the screen: centred at its task, at the nearest edge at the foot.
  Picture `thread-close-filed-turn`; test `a_close_with_a_filed_task_waiting_asks_what_happens_to_it`.
- [x] While it stands, the agent's options and the app's offer are not drawn over the reply box,
  nor the folded box's sign.
- [ ] Full screen (a file over the whole card): the question, with the tasks named, each a dot
  and its title that opens it (`.opt-named`), is drawn over the reply box in a frame of its own,
  where the web window makes it the reply card's lid (`.dock-card > .opt-strip`), and the
  question before a discard now stands there too. Test
  `under_a_file_over_the_whole_card_the_question_stands_over_the_reply_box`; not photographed,
  and the lid is not drawn as the web draws it.

### B5.4 What its answers do (`answer_close`)

- [x] Keep this thread open, and "decide each one": the question goes, nothing is written.
  Tests `a_close_on_work_that_never_reached_main_asks_first`, `two_waiting_recommend_keeping_the_thread_to_decide_each_one`.
- [x] Have the agent ship it: a reply in the words of `shipItAsk`, held three seconds, with no
  priority and no mode, the reply box left as it was. Test
  `have_the_agent_ship_it_sends_the_ask_as_a_reply_with_nothing_else`.
- [x] Close it anyway: the close again with `closeUnshipped: true`. Test
  `close_it_anyway_closes_it_meant`.
- [x] Start: each waiting task "Approved. Run it.", then the thread closed, each press writing
  the one before it. Test `starting_the_task_approves_it_then_closes_the_thread`.
- [x] Close both: each task closed, then the thread. Test `closing_both_closes_the_task_then_the_thread`.
- [x] The keys, as the discard question's, now one rule for both (`ask_key`). Tests
  `a_close_with_a_filed_task_waiting_asks_what_happens_to_it`, `close_it_anyway_closes_it_meant`,
  `an_option_that_throws_the_change_away_asks_first` (unchanged and green).
- [x] A press on a row answers it. Test `a_press_answers_it_and_leaving_the_thread_drops_it`.

## B6. The threads this thread filed (`ThreadsMade.tsx`)

### B6.1 The rules (`src/rules/threads_made.rs`, `src/rules/filed_in_thread.rs`)

- [x] `threadsMade`: test `a_thread_lists_what_it_filed_in_its_own_project_oldest_first`.
- [x] `madeShown`: test `what_waits_is_listed_first_and_alone_in_a_demand`.
- [x] `stateWord`: test `a_row_says_the_tab_it_sits_in`. The tab is read off this window's own
  lists, Needs you first, then In progress, Later and Done (`made::state_of`).
- [x] `waitingCounts`, `toApproveWords`: test `the_inbox_counts_the_tasks_waiting_under_each_thread`;
  the Inbox's row now says its count through `to_approve_words`.
- [x] `approvableFiled`: test `an_open_task_an_agent_filed_waits_on_a_press_while_its_tab_says_so`.
- [x] `doItHereWords`, `doItHereWrites`: tests `doing_it_here_names_the_proposal_and_carries_what_it_said`,
  `doing_it_here_writes_the_parent_first_and_wakes_it_as_a_reply_does`.
- [x] `answerAllWrites`: test `start_all_and_dismiss_all_write_once_per_proposal_and_take_it_all_back`.
- [x] `filedOnTurn`: tests `they_hang_off_the_last_message_of_the_turn_that_filed_them`,
  `the_newest_turns_stay_at_the_foot`, `a_turns_rows_are_oldest_first_and_nothing_filed_is_nothing`.
- [x] `waitingWords`: test `the_count_line_counts_what_the_dots_count`.

### B6.2 Where the list stands

- [x] On the turn that filed them, under its last message and over the line for a missing
  middle, 12 under it. Picture `thread-filed-turn`.
- [x] The newest turn's at the foot, 22 over and 6 under; on the page that leads with the
  newest answer alone, there too. Pictures `thread-filed`, `-many`, `-quiet`.
- [x] Not on a chat being got ready, which draws only the filed question.

### B6.3 What it says

- [x] The label in the mono capitals: "N PROPOSALS WAITING ON YOU", else "FILED FROM THIS
  THREAD". Tests `a_thread_lists_what_it_filed_with_a_proposals_answers_beside_it`,
  `with_nothing_waiting_it_is_filed_from_this_thread_with_each_ones_tab`; pictures.
- [x] A proposal first, its dot and its answers; any other row its tab's word, faint when Done
  or Later, and faint whole while proposals wait. Pictures `thread-filed`, `-quiet`, `-many`.
- [x] A long title ends in an ellipsis before the answers. Pictures (the web shortens the same
  titles the same way, `rowTitle`).
- [x] "DISMISS ALL" and "START ALL" where two or more wait. Picture `thread-filed-many`.

### B6.4 What its controls do

- [x] A row's title opens that thread. Test `a_rows_title_opens_that_thread`.
- [x] New thread: "Approved. Run it." held three seconds, "Running → {title}", the person stays.
  Test `new_thread_approves_it_as_its_own_thread_and_the_person_stays`.
- [x] Do here: parent first, held, "Doing it in this thread: {title}" naming the parent; Z after
  it reopens the proposal and says "Back as its own thread: {title}. The message already went.";
  Z inside the three seconds leaves "Undid doing it here". Test
  `do_here_hands_it_back_parent_first_and_z_after_reopens_it`.
- [x] Dismiss: closed as its own row closes it, with its undo. Test
  `dismiss_closes_it_as_its_own_row_closes_and_the_person_stays`.
- [x] Start all and Dismiss all, the held thing written first, "Started N threads · press Z to
  undo" / "Dismissed N proposals · press Z to undo", one Z for the lot that stops their agents
  and leaves its line on each proposal. Test `start_all_and_dismiss_all_answer_the_lot_at_once_and_one_z_takes_it_back`.
- [x] Under the pointer: the row's wash, a quiet title going dim, New thread's accent wash,
  the strong wash and the text's ink on the others; the pointing hand on every press; nothing
  fades. Pictures `thread-filed-hover-approve`, `-hover-quiet`, `-hover-all`.
- [ ] Tab does not walk the list's presses, nor the strip's (as A3.12 in `parity-asks.md`).

## C. How it was checked

`cargo test` in `desktop/`: 785 pass (727 before), 58 of them new. The web's own test cases of
the five rules files were carried over case for case; the window's tests press and type as a
person does and hold every write to the channel and payload the web window sends.

Every scene was photographed from the web renderer built from this commit and drawn here from
the same fixture at 1280 by 760, 2x, dark, and the part compared rectangle against rectangle
(the web dump's element by class, this window's by key), then looked at side by side, cropped.
The whole window is not compared: the web window's palette changed upstream (README).

## D. The rectangles

This window's conversation column stands 15.6 points further right than the web window's for
now (README); every `x` below differs by that and nothing else.

| Scene | Part | Web `[x, y, w, h]` | Here |
|---|---|---|---|
| `thread-ship` | the strip (`.opt-strip`, `options-strip`) | 212, 598, 760, 63 | 227.6, 598, 760, 63 |
| `thread-ship-loose`, `-name`, `-files`, `-hover` | the strip | 212, 562.5, 760, 98.5 | 227.6, 562.5, 760, 98.5 |
| the same | its head (`.opt-head`, `ask-head`) | 213, 563.5, 758, 35.5 | 228.6, 563.5, 758, 35.5 |
| `thread-filed`, `-quiet` | the list (`.made`) | 212, 319.54, 760, 105 | 227.6, 319.54, 760, 105 |
| `thread-filed-many` | the list | 212, 319.54, 760, 187 | 227.6, 319.54, 760, 187 |
| `thread-filed-turn` | the list, on its turn | 212, 205.79, 760, 105 | 227.6, 205.79, 760, 105 |
| `thread-filed-hover-approve` | New thread | 872.4, 349.54, 90.6, 26 | 888, 349.54, 90.6, 26 |
| `thread-filed-hover-quiet` | the quiet row | 213, 382.54, 758, 41 | 228.6, 382.54, 758, 41 |
| `thread-filed-hover-all` | Start all | 879.66, 313.54, 83.34, 26 | 895.26, 313.54, 83.34, 26 |
| `thread-close-unshipped`, `-lit` | the question | 212, 319.54, 760, 147.84 | 227.6, 319.54, 760, 147.85 |
| `thread-close-filed`, `-two` | the question | 212, 477.27, 760, 147.84 | 227.6, 477.27, 760, 147.85 |
| `thread-close-filed-turn` | the question, centred | 212, 313.79, 760, 147.84 | 227.6, 313.58, 760, 147.85 |
| `thread-discard-ask` (before, unchanged) | the question | 212, 293.95, 760, 112.39 | 227.6, 293.95, 760, 112.4 |

The question's strip is a hundredth of a point taller here, as the discard question's already
was: its rows' heights are Chromium's to a thousandth and add up a hundredth apart. Centred at
its task, it lands 0.21 points higher: the browser scrolls to a whole point (`Math.round`),
this window to the exact middle.
