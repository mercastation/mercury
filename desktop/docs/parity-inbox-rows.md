# The Inbox's rows, and E and Z: the web window's behaviour, line by line

The checklist of everything the web window's Inbox table draws for a row since its look of
2026-11-23 (two lines a row, the face on the title), and of the rule that E and Z always say what
they did, read off its code and tests, and whether this window does the same. A line is ticked
only once it is ported and checked; how it was checked is said on the line. A line left open says
exactly why.

The spec, by file (`R` is `renderer/src`):

- `R/threads/Pages.tsx`: `RowCells`, `RowLine`, `lineFor`, `ThreadCells`, `HarnessMark`'s use.
- `R/threads/pages.css`: the block headed "TWO LINES A THREAD" at its end, and `.th-grid`,
  `.th-cell-*`, `.th-aside*`, `.th-pin`, `.th-slot`, `.th-table .row.unread::before`.
- `R/styles.css`: `.row`, `.row + .row::after`, `.row.recent`, `.row .subject`, `.mark`,
  `.trouble-rule`, `.toast-warns`. `R/team/rooms/rooms.css`: `.gt-inset` (`--gutter`, `--row-r`,
  the unread dot's left). `R/components/harness-mark.css`, `R/components/agent-face.css`.
- `R/components/List.tsx`: every kind of row a tab draws (`ruleRows`, a teammate's card, a
  thread, the app's own rows, `data-hint`).
- `R/list-rules.ts`: `rowSummary`, `asideBesides`, `rowTitle`.
- `R/keys.ts` (`eOnThisList`), `R/stopped-agent.ts` (`pressAgainGoesAhead`), `R/stray-letters.ts`
  (the held Z), their uses in `R/App.tsx`, `R/home/home.css` (the menus' text shadow).
- The commits: 328d95c35, 61ee22c0c, 285456e9d, cb09a114f and 53a3ceb72 (the rows); 2f405fb36 and
  5eaf5ba28 (E and Z); a0718cb07 (the menus over the sky).
- Web tests: `tests/an-inbox-row-has-two-lines-and-its-face-sits-on-the-title.test.mts`,
  `tests/every-inbox-row-reads-the-same-way.test.mts`,
  `tests/an-inbox-row-says-only-what-its-tab-does-not.test.mts`,
  `tests/a-repeating-task-is-a-row-like-every-other.test.mts`,
  `tests/e-and-z-always-say-what-they-did.test.mts`.

Each line: **[x]** done, **[ ]** not done. "Tests" are `cargo test` in `desktop/` (rules in
`src/rules/list.rs` and `src/rules/e_and_z.rs`, the window's in `src/inbox.rs`, `src/answer.rs`,
`src/feel.rs`). "Pictures" are scenes photographed from the web renderer and drawn here from the
same fixture, their rects compared to a hundredth of a point: `inbox`, `inbox-two-lines` (new: two
unread rows, a pinned row, a row whose aside its line says, a row with a link's comment, a row
with no line), `inbox-kinds-needs` (new: a cause's row among the others), `inbox-done-rows`
(new: Done with rows, one without a line), `inbox-later-kinds`, `inbox-progress-stuck*`,
`inbox-trouble`.

## A. The second line

- [x] A1 A row has a second line when `lineFor` gives one: what came back (`rowSummary` read for
  the tab the row is under), unless `asideBesides` says it only repeats the title; with no words
  and no one leading it there is none (`said = line && (line.text || line.who)`). Rule
  `rules::list::line_for`, tests `the_second_line_is_what_came_back_and_never_the_title_again`;
  window test `a_row_is_its_title_and_what_came_back_and_one_with_nothing_more_to_say_stays_one_line`;
  pictures (the row whose answer is its title has none in both).
- [x] A2 `rowSummary(item, tab)`: every row of Done reads its result (`view === 'done'`), as a
  finished row does; the search reads it as `done` or `inbox` by the thread's status.
  `row_summary` takes the view; test `the_line_is_whichever_of_ask_and_answer_was_written_last`.
- [x] A3 The line under the title: 13 points on an 18-point line in `--text-faint`, 3 points under
  the title (`row-gap`), 35 points in and 32 short of the column's end, on one line, cut with three
  dots. Pictures: at 102, 234, 526 by 18 in both, its words at 137.13; the cut "Merge i…" in both.
- [ ] A4 Led by who said it (`.th-line-who`, 6 points before the words, in `--text-dim`) when the
  row is a person's, an AI teammate's or a task between people. This window has no team and no
  teammates (left out by agreement), so no row of its own has anyone to lead the line with. The
  rule takes the name (`line_for(.., who)`, test above) for when one does.
- [x] A5 The faint words after the title that the line says again are not drawn on the first
  line (`.aside-moved`); the cell keeps its shape of boxes. `rules::list::moved_to_the_line`, test
  `what_the_second_line_says_leaves_the_first`; window test
  `words_after_the_title_that_the_second_line_says_leave_the_first_and_the_app_s_own_row_wears_its_rule`;
  picture `inbox-two-lines` ("Snooze is lost on restart").
- [ ] A6 Who it is from, after the title, leaves when the line starts with them (`.name-moved`).
  Ported as a rule (same test); no row of this window carries a name after its title (no team).
- [x] A7 On every kind of row this window draws: a thread of the person's own, a cause's row
  (`inbox-kinds-needs`, "1 task to appr…" and its line), a thread behind a cause and the rest of
  Later (`inbox-later-kinds`), In progress (`inbox-progress-stuck`: "Ship the cart badge" with its
  line, the rest one line), Done (`inbox-done-rows`), the app's row of stopped tasks
  (`inbox-trouble`: "2 need a login · 1 back at 6pm").
- [ ] A8 A repeating task's row, whose line says how its last run went (`repeatRow(rule).last`).
  This window lists no repeating tasks (README, "Later lists threads only").
- [ ] A9 A teammate's card's row, whose line is its problem led by their name. No team here.

## B. The row's shape

- [x] B1 17 points above and below the lines (`.th-table .row:not(.th-head)`): 75 points a row
  with a line, 54 without. Window test above (75 and 54); pictures (rows at 194, 269, … and the
  one-line row 54 tall at 794 in `inbox-two-lines` at 1100 tall).
- [x] B2 The title at 14.5 points, its letters 1% closer (-0.145), 35 points into its cell and 32
  short of its end, 20 points a line; unread, 530. Pictures: the cell at 102, 211, 526 by 20, its
  words at 137.13 in both.
- [x] B3 The title's cell as a line of text, cut from its end with the pin first, then "not
  started", then the words; with an aside, a row of boxes in which the aside gives way first.
  Pictures: the aside's box at 341.61 by 254.39 against the web's 341.62 by 254.38; the pin at
  405.44, 216 in both. One narrow case differs: B3a below.
- [ ] B3a At 800 points wide, Later's first row ("Rewrite the onboarding emails", not started,
  pinned): the browser drops "not started" and puts the dots after the title, though "not
  started" and the dots fit with 0.44 points to spare; this window draws "not started…". The rule
  Chromium follows for a box that is not text next to the dots was not found; the cell's other
  cuts land where the browser's do.
- [x] B4 One face, 24 points, its top on the top of the title's capitals: 5 points into the
  title's line, 0.36 higher, moved without being snapped to a pixel (`translateY`). Pictures: at
  102, 215.64 in both; its ring row by row within 2 of 255.
- [x] B5 The harness's mark on a tile of the card's own ground (`--surface`), a 7-point corner, a
  soft 8% edge (`--face-ring`), the mark 19.68 points across. Pictures: tile and mark rects equal.
  The starburst's ink lands about 0.6 of a device pixel lower than the browser's (Rune places an
  icon on whole device pixels; `pixel_snap(false)` does not reach it); listed in the README.
- [x] B6 A harness with no mark: the agent's eyes, 16 points, in a round 24-point ring of 12%
  (`.agent-face`, which neither the corner nor the softer edge reaches). Read off the CSS; no
  scene of the invented inbox runs on a harness without a mark, so it is not measured.
- [ ] B7 A person's or a teammate's face (`.tm-av`, `.tq`, `.th-mate`, `i.mate-face`). No team.
- [x] B8 The sidebar's Recent rows keep the 18-point mark (`inbox::harness_mark`), unchanged.
- [x] B9 Status, Project, Priority and the time centred on the two lines: 10.5 points down, the
  first three lifted 0.6 and the time let down 0.5. Pictures: 220.9 and 222 in both; window test.
  A row without a line keeps them on its one line (17 down): window test, pictures.
- [x] B10 The unread dot, 6 points of the accent, 25 in (`.gt-inset`), 30.64 down, level with the
  face's middle. Window test `an_unread_rows_dot_is_level_with_the_middle_of_its_face`; picture
  `inbox-two-lines` (25, 30.64 in the web's computed style and here).
- [x] B11 The hairline above every row but the first, 20 in and 32 short, `--line` at .375. As
  before; pictures.
- [x] B12 `--row-r: 28px` in the two-line block. The built page's rows still end 32 short of the
  card (`.gt-inset .list { --row-r: 32px }` wins), which is what this window draws. Pictures: the
  grid at 102 by 1124 in both.
- [x] B13 The selected row's title underlined in the accent at .6, the pointed row's faintly, 19
  points into the title's line and a point thick on a 2x screen. As before; pictures.
- [x] B14 The app's own row of stopped tasks wears a faint 2-point rule at its left edge where a
  thread's select box would be, 20 in, from 19 points down to 15 short of its foot, round, in
  `--text-faint` (`.trouble-rule`). Window test above; picture `inbox-trouble` (78, 213, 2 by 41 in
  both). This ticks A1.6 of `docs/parity-inbox-bars.md`.
- [ ] B15 The select box at the row's left (`.mark`, now 18 down). This window ticks no rows
  (README, "Several rows cannot be ticked").
- [x] B16 A status card under the pointer (`.th-cell-state.has-card:hover { z-index: 30 }`): in
  the built page each row is its own layer (`content-visibility: auto`), so the card is still cut
  at its row's foot by the next row, as here. Picture `inbox-progress-stuck-hover-why`.
- [ ] B17 The last thread you were in, washed (`.row.recent`), its hairlines from the card's
  edge. This window keeps no trail of threads stood on (README: "⌘] and the mouse's back and
  forward").
- [ ] B18 The demand after the title (`dm-on-desk`), the lock and the people marks, Move to cloud
  over the time. Demands, the team and the cloud row action are not in this window (README).

## C. Everything that assumed a row is 54 points

- [x] C1 The row's height is its content's, not a number: 75 or 54 as above.
- [x] C2 The pointer's row and Enter on it: `feel::tests::the_row_the_pointer_moves_onto_takes_enter_until_the_keys_walk_the_list`
  walks 75 points a row.
- [x] C3 The key plate hangs from the row by its laid-out place (`feel::plate`), so it follows the
  row's height; picture `inbox-hint-row`.
- [x] C4 A status card hangs from its cell by its laid-out place and is cut at its row's laid-out
  foot (`foot::with_card`); pictures `inbox-progress-stuck-hover-*`.
- [x] C5 Rows built only near the screen: the web window skips a far row's painting
  (`contain-intrinsic-size: auto 75px`); this window's Inbox builds every row of the tab, as it did
  (`virtual_rows` is a long conversation's), so nothing holds a 54 there.
- [x] C6 Bringing the picked row into view under the keys: the web window's list does not scroll
  to the row J and K land on (only the board does), and neither does this window.

## D. The row's key plate

- [x] D1 Only under Needs you, where its keys mean what it says (`tab === 'inbox' && !isImportRow`):
  window test `the_row_hint_is_only_where_its_keys_mean_what_it_says`.

## E. E and Z always say what they did

- [x] E1 E under Needs you closes the row the keys are on. Window test
  `e_closes_in_needs_you_and_says_why_it_does_not_in_progress_and_done`.
- [x] E2 E under In progress closes nothing and says "Open this thread to close it."; under Done,
  "This thread is already done." (`eOnThisList`). This window closed an In progress row with E.
  Rule test `e_on_a_list_where_it_closes_nothing_says_why_on_in_progress_and_on_done`; window test
  above (nothing held, nothing written, the row still there).
- [x] E3 E under Later brings the thread back (`unsnooze`). As before.
- [x] E4 With no row under the keys, nothing is said (`else if (pointed)`). As before.
- [ ] E5 E on ticked rows closes them all (`batchDone`). No rows are ticked here.
- [x] E6 The waiting agent's warning is learned once a press goes ahead on the thread it warned
  of, not when it is shown; E on another such thread in between is held and warned of too
  (`pressAgainGoesAhead`); L's warning is learned as it is said. Rule test
  `the_press_after_a_stuck_agent_warning_goes_ahead_only_on_the_thread_it_warned_about`; window
  tests `the_first_close_ever_of_a_waiting_agent_is_held_and_says_what_it_costs`,
  `the_press_after_the_warning_goes_ahead_only_on_the_thread_it_warned_about`.
- [ ] E7 The warning's line wraps rather than being cut (`.toast-warns .toast-line`). The toast is
  drawn in `src/chrome.rs`, outside this port; its line about a named thread is cut with dots.
- [x] E8 Two quick Zs are one Z: a Z within 250 ms of the last that went through does nothing
  more, so it neither undoes twice nor answers "Press Z again" (`stray-letters.ts`). Rule test
  `two_quick_zs_reach_the_app_as_one_z`; window test
  `z_past_half_a_minute_asks_first_and_any_other_key_says_no`.
- [ ] E9 A Z that starts a word runs nothing, and a Z waits 250 ms before it acts. This window has
  no stray-letter guard at all (a letter with no box under it is a shortcut at once); the two-Z
  rule above is the part of it this port carries.

## F. The rest of the Inbox page

- [x] F1 A menu opened from the strip wears none of the strip's dark glow on its words
  (a0718cb07): the View and filters menu, the cloud's menu and its buttons. Pictures
  `inbox-display*`, `inbox-cloud-menu*`.
- [ ] F2 The View and filters menu's "Terminal sessions" line (Always, When they need you,
  Never). This window lists no terminal sessions (the terminal is left out by agreement); a line
  that changed what only other windows show is left out with it.
- [ ] F3 The toast's ground: the web's is `--pop` (31, 31, 33) under a `--line` edge; this
  window's is still the warm (54, 52, 49), in `src/chrome.rs`, outside this port.
- [ ] F4 The app's own row of stopped tasks, E pressed on it: the web window keeps the row in the
  list for its three seconds (picture `inbox-trouble-done`), where this window takes it out at
  once (`trouble::put_the_trouble_away`, `docs/parity-inbox-bars.md` A3.2). Left as it is for
  whoever owns that row's behaviour to decide.
