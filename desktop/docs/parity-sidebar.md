# The sidebar and the arrows: the web window's, line by line

Read off `renderer/src/components/WorkspaceNavigation.tsx`, `renderer/src/sidebar.css`, the
rules it still takes from `renderer/src/team/rooms/rooms.css` and
`renderer/src/workspace-navigation.css`, `renderer/src/recent-threads.ts`,
`renderer/src/back-and-forward.ts` with `renderer/src/components/BackForward.tsx`, and how
`renderer/src/App.tsx` feeds them, as they stood on 2026-11-24. Each line says what this window
does of it: **same**, **differs** (and how), or **left out** (and why). The native side is
`desktop/src/chrome.rs` (`rail`, `card`, `backdrop`, `back_forward`, `trail_arrow`),
`desktop/src/walk.rs`, `desktop/src/usage.rs`, and the rules `desktop/src/rules/recent.rs` and
`desktop/src/rules/trail.rs`.

## The frame

| The web window | Here |
|---|---|
| `body::before`: `--bg` #0b0b0c and a dot of 1.15 points at 5% of white every 32 (`--skin-image` under a dark sky, `home.css`) | same (`backdrop`); the warm glow of the older look is gone |
| `.workspace-layout::before`: the card 40, 22, 22 and `--workspace-left` from the window's edges; `--skin-pane` #131315, whole; corner `--pane-corner` 10; `inset 0 0 0 1px` of `--skin-pane-line` #363639 and `--shadow-card` | same (`card`, `theme::PANE_CORNER`) |
| `--workspace-left`: 58 folded, 246 open, eased with the sidebar's width on one curve | same (`card_left`) |
| `.home-sky`, its ring of 7% white on the card's corner; the Inbox's sky behind its head | same |
| Every ground and line by its variable (`--bg`, `--surface`, `--bg-raised`, `--bg-hover`, `--bg-selected`, `--pop`, `--line`, `--line-strong`, `--code-ground`) | same, by name in `theme.rs` |
| The plain Dark look, with no sky (the warm Ash), the other dark skies, and the light looks | left out: this window draws the dark orange sky and no other (already in what still differs) |

## The sidebar (`aside.workspace-navigation.gt-rail-nav`)

| The web window | Here |
|---|---|
| 14 points from the window's left, 50 from its top, 22 from its foot; 28 wide folded, 210 open (`--sidebar-w`, 220 before the second pass) | same |
| Opens 140 ms after a pointer that came onto it from outside last stopped travelling (`hoverIntent`, `crossedIn`, `stillTravelling` at 0.15 of a point a millisecond, `pointerTrail` fed by every move in the window, `pointerMoved`) | same (`rules/sidebar.rs` with the web's cases; `Rail::moved_on`; the root's observer in `main.rs`) |
| Folds 300 ms after the pointer leaves; a move back onto it calls the fold off (`leaveAt`) | same (`Rail::left`, `rail_moves`) |
| Open, the 22 points past its right edge are still its own (`.workspace-navigation::after`) | same (`BRIDGE`) |
| 0.24 s open, 0.2 s shut; words in after 0.08 s over 0.14 s, out over 0.06 s; under Reduce motion, whole | same (`Rail::point`) |
| Kept open while Settings' "Shrink the sidebar to icons" is off; ⌘\ flips it from anywhere but Settings (`toggleSidebarShrinks`) | same (`keep_or_shrink`, `keys.rs`) |
| Kept open, the skies are painted for the card as it stands, stretched to it until it holds still (`folds`, `reach`) | same (`sky_box`, `painted_card_size`) |
| Type 14 at 500 on a line of 20, in `--text-dim` | same |
| A row: 30 tall, 6 in, its mark in a 16-point column drawn at 18, its word 8 past; `cursor: default` | same (`rail_row`) |
| Under the pointer, and on the page on screen: `--sidebar-fill` (the text at 6%), the word and its mark in `--text` (`color: inherit` on the mark) | same |
| Marks at a line of 1.2 on the screen (`stroke-width: 1.6px` on a 24 grid at 18; 1.8 inside Search's group, scaled to .8) | same (`icons::RAIL_*`) |
| `.th-team`: 8 under the top, 28 tall, the mark in a 20-point tile, the name 14 at 600 with -0.18 tracking, cut with dots; its title is the name while open | same |
| The team's logo (`th-mark-logo`) or its initial (`th-mark`) for a team with a name | **differs**: always the app's own mark, as before. This window signs into no team of its own |
| `.ws-new`: the pencil, 28 square, 16-point mark, 166 to 194 across while open; "New thread (N)"; opens the New thread card; not drawn folded | same |
| `.ws-keep`: two points past the pencil, 28 square, a 16-point window with its sidebar marked off, faint; lit and in `--text` while the sidebar is kept open (`aria-pressed`); labelled "Keep sidebar open" or "Shrink sidebar to icons"; no title; its plate (`keep-sidebar`, `shrink-sidebar`: ⌘\ and what a press does now) hung over it (`data-hint-side="above"`); not drawn folded | same (`feel::hinted_above`) |
| `.workspace-find`: 12 under the team's row; open, a field on `--surface` inside `--line-strong`, its glass at 18, "Search", ⌘K at 12 points; folded, the glass alone, the field going and coming at once; title "Search · ⌘K"; opens the command bar; under the pointer its glass, word and keys a step brighter | same |
| Home (`data-tab="home"`), its title while folded | same |
| Inbox: lit on every tab of it and on a thread; its count in small mono figures (11.5 on 16, +0.23) at the row's end while open; folded, a 5-point dot at 18, 4 on its mark; title "Inbox · N threads waiting" | same |
| The row's name, `inboxLabel`: "Inbox" whether or not rooms are live (its default, and what `App.tsx` passes) | same |
| Demands (`data-tab="demands"`) | left out: no Demands page here |
| "Workspace" (`SectionHead`): 12 under the tabs, 20 tall, 12 at 500 in `--text-faint`, a 7-point arrow at 75%; a press folds its rows and the next brings them back, kept (`agentbox.sidebarFolds`); folded sidebar: its name and arrow fade, a line 16 by 1 of `--line-strong` stands in its place 6 in and half way down (crossing in 0.06 s), no press, no light | same (`section_head`, `Rail::line`, `rules/recent.rs`, kept in `native-picks.json`) |
| The arrow turns over 0.12 s as the section folds | **differs**: it turns at once |
| Chats (`groups`), Team (`onTasks`), Connections | left out: no such pages here |
| Files | same |
| "Recent" (`.ws-recent`): shown only while open and only with something in it; the six threads last opened, newest first (`recentThreads`), not the chats with one person; each its mark, its name cut with dots, a 6-point orange dot while it waits on the person and is not the one open; its title is its name; a press opens it and reads it | same (`recent_row`, `rules/recent.rs`) |
| A Recent row's mark: the harness's, bare, 20 points across in the 16-point column, in the harness's own colour (`.ws-recent-mark > .harness-mark`); the agent's face at 18 for a harness with no mark | same (`recent_mark`) |
| A Recent row's mark for a teammate's thread (`TeammateMark`) or one a person sent (`Face`) | **differs**: always the harness's mark; this window draws no teammates |
| Recent gives way first in a short window (`flex: 0 1 auto; overflow: hidden`) | same |
| The foot (`.workspace-bottom`), 16 over its first row | same |
| A new version waiting (`announcesUpdate`: ready or being built), folded: the restart row as one more of the foot's marks, 30 tall, its mark at 18 in the dim ink; its title what a press does, how long it takes and what changed (`tip`); a press is the restart (`zero:update-install`) | same (`foot_says`, `foot_row`, `rules/update.rs` with the web's cases) |
| The same, open (`RailCard`): the row keeps its place and is the card's button (30 tall, 12 in from either side, the white wash inside `--line-strong`, "Restart to update" at 13); the card stands on the row's slot (42 tall, 6 over the next row) and rises over what is above: 12 in, its mark at 18 in the dim ink and "New version ready" at 13.5, then how long a restart takes in 12-point faint type on lines of 17.4, or why the last did not finish and that a press tries again; its title is what changed, when the app says | same (`foot_card`) |
| Its ×: 22 square, 6 in from the card's top and right, a 13-point mark in the dim ink, `--text` under the pointer; shown while the pointer is on the card or its slot; titled "Hide until the next version"; hides card and row for that version, kept (`zero.updateClosed`), never while it rebuilds; a version the app could not name is closed for this run only | same (kept in `native-picks.json`, `update_closed`) |
| While it rebuilds: no card; the row is one of the foot's rows, says "Updating", its mark turning once in 1.4 s, takes no press; titled "Updating. Mercury restarts by itself in about a minute." | same (`LoopSpec::rotation`) |
| A copy that cannot update itself (`SidebarMove`, `announcesMove`): the same card where the update card would be and never beside it, "Updates are stuck", where it runs from in half the words or why the last move failed, Move to Applications (`zero:update-move`); its × titled "Hide it · Settings still offers it", kept by place (`zero.moveClosed`); folded, its mark | same (`Foot::Stuck`, `State::move_to_applications`) |
| A recording or what stopped one (`SidebarMeeting`), Record meeting | left out: no such cards here |
| Sign in (`onSignIn`), Feedback, Instructions | left out: no sign-in, no feedback card, no instructions page here |
| Settings: the last row while nobody is signed in, hidden once the account row holds it (`.ws-in-menu`) | same |
| The person's own row (`.th-me-btn`): 2 under the rows, 32 tall, their face at 20 points (9-point mono letters), their name at 500; the email under it no longer drawn | same |
| A press on it opens their account menu: what they are up to, Invite people, Settings, Sign out | same, but for what they are up to (left out): see "The account menu" below |
| "Sign in to your team" (`.th-me-signin`) where a team is set up and nobody is signed in; going invisible (`.th-me-invis`) | left out |
| `.workspace-usage`: a bar 32 by 4 at the end of the last row (30 tall, 32 when it is the person's), 6 from its end; its track the text at 13%, its fill `--text-dim`, and `--text` once 80% is spent; as full as the headline window is spent; its label says every limit; not drawn folded; the person's row gives it 48 points | same |
| Resting on the bar 120 ms opens the card of limits (`.usage-panel`) 6 over the row from the sidebar's left edge; it lingers 220 ms; a press opens that agent's page in Settings | same (`usage.rs`); the card's lines are 20 tall and at 500, as the sidebar's type now makes them |
| In a window under 701 by 521 the pencil, the section names and Recent are not drawn, and the phone's bar takes over | left out: this window never draws the phone's layout |

## Back and Forward

| The web window | Here |
|---|---|
| The trail: the list and each thread once, where the person was in it last (`arrive`, `land`, `settle`, `stepBack`, `stepForward`, 50 places) | same (`rules/trail.rs`, with the web's cases) |
| A place counts once it has stayed 300 ms (`settledPlaces`); a step on its way is not a new visit | same (`walk::tend`) |
| A teammate's thread and a chat opened from Needs you are places too | left out: this window opens neither |
| The arrows are drawn on the inbox page alone: before the Inbox's title on any tab, and in a thread's strip; none on Home, Files, Settings | same (`back_forward`) |
| Each dim at 45% with nowhere to go; lit, its title "Back to <place> (⌘[)" or "Forward to <place> (⌘])", the list named by its tab and a thread by its name | same (`trail_arrow`) |
| ⌘[ and ⌘] held on every page, stepping only on the inbox page; not inside a code editor, which indents with them | **differs**: here they are taken before a change's editor sees them, as before |
| A mouse's side buttons step back and forward | left out: Rune hands this window no such buttons |
| Back to the list leaves the thread as Escape does, the keys where they were; another tab opens from its first row; a thread opens over the inbox | same (`go_to_stop`) |
| The last thread stood on is washed in the list (`lastThread`) | the rule is kept (`State::last_thread`); the row's wash is drawn with the Inbox's rows |
| The Inbox button before the arrows, only on a thread | same |

## The account menu

Read off `AccountMenu` and `AccountMenuList` in `renderer/src/team/account-menu.tsx`, `GoInvisible`
in `renderer/src/team/rooms/GoInvisible.tsx`, the corner in `WorkspaceNavigation.tsx`, the doors
`renderer/src/App.tsx` hands it, the rules `.th-me-wrap`, `.th-me-btn`, `.th-acct-*`, `.th-me-pop`
(`team/team.css`), `.rm-ipop` and the rail's word fades (`team/rooms/rooms.css`), `.th-me small`
(`threads/pages.css`), `.workspace-navigation button`/`svg` (`workspace-navigation.css`), the
artifact corner (`soft-corners.css`), and the web's tests
`your-own-row-opens-your-account-menu-with-sign-out`,
`with-multiplayer-off-a-signed-in-person-sees-the-single-person-app`,
`a-status-runs-until-any-time-you-choose` and `team-a-status-you-write-yourself`, as they stood
on 2026-11-24. The native side is `desktop/src/account_menu.rs` and its rules
`desktop/src/rules/account_menu.rs`; the row it hangs from is `chrome::rail`'s, keyed
`account_menu::ROW`. Sizes were measured against the web's dump in the `home-team-account*`
scenes (`tools/scenes/account.mjs`).

60 lines: 40 same, 5 differ (7, 11, 17, 33, 45), 15 left out (4, 13, 34, 39, 46–56). Of those
left out, the status (13, 46, 48–56) is one decision: its box cannot be drawn the same here.

### What it holds, and when (TSX, `App.tsx`)

| The web window | Here |
|---|---|
| 1. The corner is the account menu's row whenever someone is signed in (`me && onAccount`; `App.tsx` always hands `onAccount`); nothing at all with nobody signed in | same (`rules::doors` is None; no row, no menu) |
| 2. `.th-me-btn`: one button, `aria-label="Your account"` | same (`label`) |
| 3. Its title "Your account" only while the sidebar is folded | same |
| 4. `aria-haspopup="menu"`, `aria-expanded` | **left out**: Rune hands assistive technology no expanded state |
| 5. A press opens the menu; a second press closes it (`open ? close() : setOpen(true)`) | same (`toggle`) |
| 6. `close()` puts away the menu, the status box and Go invisible's card together | same (`close`) |
| 7. A `pointerdown` anywhere outside the row and what is open over it closes it, on the way down, and the press goes on to what it was aimed at | **differs**: it closes on the way down, and a layer under the menu takes the press, as `thread::files_list` does; a second press reaches the page |
| 8. Escape closes it (heard on `document`, before the page's own keys) and goes no further; every other key goes on to the page | same (`account_menu::key`, before the page's keys in `keys::pressed`) |
| 9. A press on a row closes the menu first, then does what the row does (`go`) | same |
| 10. Who they are (`.th-acct-who`): opens Settings on the team's page (`onAccount`: `setSettingsPane('team')`) | same: with a team (`team.configured`) the web window comes forward on its Team page (`settings::bridge`, `zero:open-settings-here` `{ "page": "team" }`); with none, Settings has no Team page and `paneFrom` lands on General, which this window draws (`settings::open`) |
| 11. `onAccount` also closes the web window's card, search and open thread (`setOpenCard(null)`, `closeSearch()`, `setFocused(null)`) | **differs**: the door's `zero:open-settings` handler only opens Settings, so whatever the web window had open stays under its Settings; this window's own page is untouched |
| 12. `disabled={!onAccount}` | same (never disabled: the door is always handed) |
| 13. What they are up to (`onStatus`, only with `statuses={multiplayer}`): "Set a status" with a smiley, or what they said and how long it holds, which opens the status box in the menu's place | **left out**: the box cannot be drawn the same (lines 51–56); its rules are ported and held to the web's cases (`rules::said`, `holds_until`, `status_row`, `end_of`, `until_words`, `hold_ends`, `read_until`, `status_payload`). Its channel, `zero:team-status` `{ text, hold, until? }`, is one the door already passes |
| 14. Go invisible, or Come back while invisible, only while Rooms (alpha) is on, live, and knows the person (`roomsUi.invisible`: `invisible_until` ahead of now) | same (`rules::doors`) |
| 15. Come back: `zero:rooms` `{ "op": "comeBack", "args": [] }` | same |
| 16. Go invisible: its card takes the menu's place (`going`) | same |
| 17. Invite people, only with Multiplayer on (`onInvite={multiplayer ? …}`): Settings on the team's page with the caret in its email box (`setInviteFocus(true)`) | **differs**: it lands on the same page as who they are (line 10); no channel of the door puts the caret in the email box |
| 18. Settings (`onSettings`): Settings on General, a new visit | same (`settings::open(General)`) |
| 19. Sign out (`onSignOut`): `api.teamSignOut()` (`zero:team-sign-out`, no payload), nothing asked first, the lists read again after; nothing said of how it went | same (`Value::Null`; the door reads the lists again after every write); with no app behind the window, "Mercury is not running, so this was not sent." |
| 20. Sign out only with its door, behind a hairline of its own | same |
| 21. With Multiplayer off: who they are, Settings and Sign out | same |
| 22. Under Settings › General, Sign out too, while someone is signed in | same (already there, `settings/pages.rs`) |

### Go invisible's card (`GoInvisible`, `.rm-ipop.rm-pop`)

| The web window | Here |
|---|---|
| 23. 340 wide with its edge, 16 in at the top and sides and 8 at the foot; the pop's grey, `--line-strong` edge, `--shadow-lg`, the artifact corner (10); six over the row from its left (`.th-me-wrap .rm-ipop`) | same (measured: 340 by 261 at 14, 439 in both) |
| 24. Its title: the ghost at 18 in `--text-dim`, nine before "Go invisible" at 15, 500, in `--text` | same |
| 25. Three facts, 7 apart, 12 under the title and 10 over the rule: each mark at 18 in `--text-faint` a point down, ten before its words at 13 on 18 in `--text-dim`, wrapped | same (the second wraps to two lines in both) |
| 26. A hairline of `--rm-line` across the whole card, 2 over and 6 under | same |
| 27. For an hour (60 minutes), Until tomorrow (`minutesToMidnight`), Until I turn it off (null): 32 tall, eight in on a row eight wider than the words' column, 13 at 500 in `--text-dim`; under the pointer `--wash-strong` and `--text` | same |
| 28. A pick closes everything and asks `zero:rooms` `{ "op": "goInvisible", "args": [minutes] }` | same |
| 29. `role="menu"`, `aria-label="Go invisible"`; no keys of its own, so Escape closes all of it | same |

### Its look (`team.css` with the sheets that cascade into it)

| The web window | Here |
|---|---|
| 30. `.th-acct-menu`: left and right of the row's, its foot six over the row (`bottom: calc(100% + 6px)`) | same, hung from the row's laid-out rect, so it follows the sidebar's width (measured 14, 547 to 700 in both; 210 wide there, 220 here until the rail is 210) |
| 31. Four in, a point between rows, `--pop`, a point of `--line-strong`, `--shadow-lg`, the artifact corner (10) | same |
| 32. `z-index: 60` inside the sidebar: over the page, under the toast and the dialogs | same (`main::view`: drawn after the usage card, before the toast) |
| 33. Comes in with `prio-in`, 0.13 s on `cubic-bezier(.2, .8, .3, 1)`: from 4 points lower, 98% and clear; goes at once | **differs**: a spring stands for the curve, as `chips.rs` has it |
| 34. Light looks: on `--surface` | **left out**: this window draws the dark look only |
| 35. Folded sidebar: `right: auto; width: 240px` | same |
| 36. Folded sidebar: every `button > span` in it, these words with them, faded to nothing over 0.06 s and back over 0.14 s after 0.08 s (`rooms.css`): the folded menu shows faces and marks alone | same (the rail's own fade, `Rail::words`); measured 0.05/255 apart |
| 37. A row: as wide as the menu, 7 over and under and 9 in, ten between mark and words, 13 at 500 on 20 in `--text-dim`, left, corner 6 (`--radius`), the pointing hand; `overflow: hidden` | same (34 tall; words 56, 625 for Settings in both, to a pen step of 0.13) |
| 38. Under the pointer: `--wash-strong`, the words in `--text`, the mark from `--text-faint` to `--text-dim` | same (measured in `home-team-account-hover`) |
| 39. `:focus-visible` washes a row reached by the keys | **left out**: not checked; Rune's own focus ring and Tab order on these rows are whatever it gives a pressable row |
| 40. Marks at 18 (`.workspace-navigation svg`) in `--text-faint`, each with its own stroke (1.4; the settings glyph 1.25) | same (`icons::ACCOUNT_*`) |
| 41. A row's words cut with dots on one line | same |
| 42. Who they are: 8 over, 9 under and in, nine between; the face at 20 (the sidebar's `.th-me .tm-av` wins over `team.css`'s 28) with its two mono letters; the name 13 at 500 in `--text`, the email 12 (`.th-me small` in `pages.css` wins over 11.5) at 500 in `--text-faint`, both on 20 and cut with dots; the pointing hand; `--wash-strong` under the pointer | same (57 tall; the name at 57, 560 in both) |
| 43. `.th-acct-rule`: a point of `--line`, 3 over and under, 5 in | same |
| 44. The row lit while the menu is open (`[aria-expanded='true'] { background: var(--wash) }`) | same at rest (15/255 in both) |
| 45. The row lit and under the pointer: the web's computed style says the sidebar's `--sidebar-fill` (6% of the text), and its picture shows 18/255 | **differs**: 24/255 here, the rail's own fill under the pointer, which is what its other rows show in both windows; the cause of the web's darker picture was not found |
| 46. A status said (`.said`): its words in `--text` on a line of 17.55, the mark a point down, the row from its top; two lines at most by `team.css`, but the rail's `.gt-rail-nav button > span { white-space: nowrap }` keeps it to one line cut with dots (33 tall) | **left out** with the status row (line 13) |
| 47. `.th-me-btn` with `.th-me-invis`: 40 tall, "Invisible" with an eye under the name | **left out**: part of the row (`chrome::rail`), already in the sidebar's list above |

### The status box (`.th-me-pop`, `StatusComposer`, `UntilMenu`): left out

| The web window | Here |
|---|---|
| 48. In the menu's place: 360 wide, 14 16 12 in, the pop's grey, edge, shadow and corner, six over the row | **left out** |
| 49. One sentence at 15 on 24 in `--text-faint`: "I'm … until … ." | **left out** |
| 50. What they type is a run of that sentence (`contentEditable`), in `--text`, wrapping with the words around it, the caret put at its end on open; cut to 80 characters, white space to one | **left out**: Rune has no field that is part of a sentence and wraps with it; a `text_input` is a box of its own |
| 51. It is underlined with dots of `--line-strong`, 4 under the line, on every line it wraps to | **left out**: a Rune span's underline is solid, with no offset |
| 52. "the end of today" (or the end already chosen, `endOf`): a word in `--text-dim` underlined with dots, 3 under, which opens the When menu over it (presets, and a box that reads "friday 5pm") | **left out** |
| 53. Enter saves, Escape closes all, nothing said clears; a refusal is said under it in the accent (`tm-status-error`) | **left out** (the write is `zero:team-status`, `rules::status_payload`) |
| 54. "↵ save", "esc cancel" under a hairline, and Clear status while something is said | **left out** |
| 55. The When menu's Escape and Enter are its own | **left out** |
| 56. Scenes `home-team-account-status`, `-status-typed`, `-status-until` photograph it, for whoever ports it | the native pictures show it missing |

### The web's tests

| The web window | Here |
|---|---|
| 57. The menu holds who you are, the status, Invite people, Settings, then Sign out, Sign out behind a hairline of its own, each a `menuitem` | same, but for the status (line 13): `account_menu::tests` |
| 58. A row whose door was not handed in is not drawn | same: the doors are read off the snapshot as `App.tsx` reads them (`rules::account_menu::tests`) |
| 59. A status set shows with how long it holds; one that ran out asks again; `holdsUntil`, `holdEnds`, `endOf`, `untilWords`, `readUntil`, the payload | same, as rules (their cases ported); no row draws them yet |
| 60. Every press into Settings lands on the page it asked for | same |
