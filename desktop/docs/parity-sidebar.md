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
| 14 points from the window's left, 50 from its top, 22 from its foot; 28 wide folded, 220 open (`--sidebar-w`) | same |
| Opens 140 ms after the pointer moves onto it (`hoverAt`, `pointerMoved`), folds as it leaves; 0.24 s open, 0.2 s shut; words in after 0.08 s over 0.14 s, out over 0.06 s; under Reduce motion, whole | same (`Rail`) |
| Kept open while Settings' "Shrink the sidebar to icons" is off | same |
| Type 14 at 500 on a line of 20, in `--text-dim` | same |
| A row: 30 tall, 6 in, its mark in a 16-point column drawn at 18, its word 8 past; `cursor: default` | same (`rail_row`) |
| Under the pointer, and on the page on screen: `--sidebar-fill` (the text at 6%), the word in `--text`, the mark from `--text-faint` to `--text-dim` | same |
| Marks drawn heavier (`stroke-width: 2.07px`; 2.33 inside Search's group at 20 points) | same (`icons::RAIL_*`) |
| `.th-team`: 8 under the top, 28 tall, the mark in a 20-point tile, the name 14 at 600 with -0.18 tracking, cut with dots; its title is the name while open | same |
| The team's logo (`th-mark-logo`) or its initial (`th-mark`) for a team with a name | **differs**: always the app's own mark, as before. This window signs into no team of its own |
| `.ws-new`: the pencil at the row's end, 28 square, 16-point mark; "New thread (N)"; opens the New thread card; not drawn folded | same |
| `.workspace-find`: 12 under the team's row; open, a field on `--surface` inside `--line-strong`, its glass at 20, "Search", ⌘K at 12 points; folded, the glass alone; title "Search · ⌘K"; opens the command bar; under the pointer its word a step brighter | same |
| Home (`data-tab="home"`), its title while folded | same |
| Inbox: lit on every tab of it and on a thread; its count in small mono figures (11.5 on 16, +0.23) at the row's end while open; folded, a 5-point dot at 18, 4 on its mark; title "Inbox · N threads waiting" | same |
| The row's name, `inboxLabel`: "Inbox" whether or not rooms are live (its default, and what `App.tsx` passes) | same |
| Demands (`data-tab="demands"`) | left out: no Demands page here |
| "Workspace" (`SectionHead`): 16 under the tabs, 24 tall, 12 at 500 in `--text-faint`, a 7-point arrow at 75%; a press folds its rows and the next brings them back, kept (`agentbox.sidebarFolds`); folded sidebar: not shown, no press, keeps its height | same (`section_head`, `rules/recent.rs`, kept in `native-picks.json`) |
| The arrow turns over 0.12 s as the section folds | **differs**: it turns at once |
| Chats (`groups`), Team (`onTasks`), Connections | left out: no such pages here |
| Files | same |
| "Recent" (`.ws-recent`): shown only while open and only with something in it; the six threads last opened, newest first (`recentThreads`), not the chats with one person; each its mark, its name cut with dots, a 6-point orange dot while it waits on the person and is not the one open; its title is its name; a press opens it and reads it | same (`recent_row`, `rules/recent.rs`) |
| A Recent row's mark for a teammate's thread (`TeammateMark`) or one a person sent (`Face`) | **differs**: always the harness's mark; this window draws no teammates |
| Recent gives way first in a short window (`flex: 0 1 auto; overflow: hidden`) | same |
| The foot (`.workspace-bottom`), 16 over its first row | same |
| A new version waiting, folded: the Restart row (`.sb-update-row`, 36 tall, its mark 10 in, 13 points) in a slot 45 tall | same (`update_row`) |
| The same, open: the card risen over the row ("New version ready", "Restarting takes about a minute.", Restart to update, and × to close it for this version) | **differs**: the Restart row with its words (already in what still differs) |
| A copy that cannot update itself (`SidebarMove`), a recording or what stopped one (`SidebarMeeting`), Record meeting | left out: no such cards here |
| Sign in (`onSignIn`), Feedback, Instructions | left out: no sign-in, no feedback card, no instructions page here |
| Settings: the last row while nobody is signed in, hidden once the account row holds it (`.ws-in-menu`) | same |
| The person's own row (`.th-me-btn`): 2 under the rows, 32 tall, their face at 20 points (9-point mono letters), their name at 500; the email under it no longer drawn | same |
| A press on it opens their account menu: what they are up to, Invite people, Settings, Sign out | **differs**: opens Settings straight away; no account menu here |
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
