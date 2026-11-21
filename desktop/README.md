# Mercury's desktop window, native

Mercury's window drawn with [Rune](https://github.com/anomalyco/rune) (Rust, GPU, no browser
inside) instead of a web page in Electron. The engine stays what it is: this window only draws,
and asks the engine for everything through a local door.

Phase 1, where this is now: it reads. Home, the Inbox and one conversation, from the same snapshot
the web window gets, kept current while the engine works. It does not write yet.

The Inbox is its head strip (the title, New thread, the cloud button on a Mac set up for the cloud
or offered it, the display button), its tabs (Needs you, In progress, Later while something is in
it, Done) and the open tab's threads as a table. A row is its title with what follows it (an aside,
"not started", a pin), the Status word with its mark, the project, the priority and a time: when it
was sent, when it comes back under Later, when it was finished under Done.

## Run it

```sh
cd desktop
cargo run                      # the window, on the running app's inbox (see "Live" below)
cargo run -- --fixture shots/home-fixture.json --snap shots/home.png --size 1280x760 --scale 2
cargo run -- --fixture shots/inbox-fixture.json --view inbox --snap shots/inbox.png --size 1280x760 --scale 2
cargo run -- --fixture shots/inbox-later-kinds-fixture.json --view inbox --tab later --snap shots/inbox-later-kinds.png --size 1280x760 --scale 2
cargo run -- --fixture shots/thread-fixture.json --thread --snap shots/thread.png --size 1280x760 --scale 2
cargo run -- --fixture shots/home-fixture.json --tree shots/home-tree.json --size 1280x760 --scale 2
```

`--snap` draws one picture with no window; `--tree` writes every element's box, text and colour.
`--tab needs|progress|later|done` opens the Inbox on that tab.
The first build fetches Rune and takes about a minute. `rust-toolchain.toml` names the toolchain
Rune pins, and rustup installs it by itself.

## Live: the engine's door

The window reads the engine over HTTP on 127.0.0.1, with a key, on the same channels the web
window calls (`POST /api/<channel>`, and `/events` for pushes). Two engines open such a door:

- **The desktop app itself** (`main/local-door.mts`), when `zero.config.json` says
  `"localDoor": true`. It leaves a note with its port and key in its own folder
  (`~/Library/Application Support/Mercury/local-door.json`), which this window finds by itself.
  This is the inbox the app is running, not a copy. It also hands over what the web window counts
  as delivered and as read, so both windows show the same Inbox and the same unread threads.
- **The engine with no window** (`node bin/mercury.mjs --no-open`), which prints its address:
  `cargo run -- --door http://127.0.0.1:<port> --token <key>`.

`cargo run -- --watch-door 20` follows the engine for twenty seconds with no window and prints a
line each time its conversations change: a check that the live path works.

The app opens a new door, on another port and with another key, each time it starts. The window
reads the note again when the door it has stops answering, so it carries on by itself after the
app restarts, and one opened before the app finds the door once there is one.

The app's door does not push yet, so the window asks every two seconds. Measured on a 99-thread
inbox, that costs the app about 2% of one core and the reader next to nothing.

## Compare a screen with the real one

```sh
arch -arm64 node desktop/tools/reference.mjs <checkout with renderer/dist> desktop/shots dark 1280 760 home
python3 desktop/tools/compare.py desktop/shots/home-real.png desktop/shots/home.png desktop/shots home
```

`reference.mjs` photographs the built web renderer with an invented inbox and writes, for a scene,
`<scene>-real.png`, `<scene>-dump.json` (every element's measurements) and `<scene>-fixture.json`
(the snapshot, the clock and the page's own facts the picture was taken with). A screen here is
written from those numbers, not by eye, and drawn from that same fixture. A scene is a file in
`tools/scenes/` that takes the page from Home to the screen to photograph: `inbox`, its three other
tabs (`inbox-progress`, `inbox-later`, `inbox-done`), `thread` (a whole conversation),
`thread-folded` (as it first opens) and `thread-more` (the rest of the markdown).

The invented inbox has almost nothing switched on: nothing in Later, no cloud, no team. So some
scenes add to it. A scene of Home dresses the inbox before the page opens (`dress`): `home-team`
is Home for someone with teammates, a meeting recorder, a reading of usage and an account signed
in, and `home-update` is Home while a new version waits for a restart. A scene of the Inbox hands
the changed snapshot on in the fixture: `inbox-later-kinds` is the Later tab with one thread of
each kind it holds, and `inbox-cloud`, `-off`, `-count`, `-named`, `-again` and `-adding` are the
states of the cloud button.

A real inbox has all of that at once. To photograph the web window on one, hand it a snapshot and
what the real window keeps for itself, both of which `rows.mjs --door` leaves in its folder:

```sh
MERCURY_SNAPSHOT=$PWD/shots/live/snapshot.json MERCURY_KEEPS=$PWD/shots/live/keeps.json \
  arch -arm64 node tools/reference.mjs <checkout> shots/live dark 1280 760 home
```

`compare.py` needs `pillow` and `numpy`. It prints the difference whole and per region
(`tools/scenes/<scene>.regions.json`, or those of the scene it is a kind of: `inbox`'s for
`inbox-later`) and writes a side-by-side and a difference map.

At 1280 by 760, 2x, dark, against the web renderer on the same data:

| Screen | Mean difference, of 255 | Similarity |
|---|---|---|
| Home | 0.85 | 0.996 |
| Inbox | 0.24 | 0.998 |
| Inbox, the Later tab (`inbox-later-kinds`) | 0.27 | 0.998 |
| Inbox, each state of the cloud button | 0.24 | 0.998 |
| A conversation | 0.41 | 0.997 |

Text lands where the browser puts it to a hundredth of a point.

One size proves one size: Home was right at 1280 by 760 and wrong in a taller window until it
was compared there. Home and a conversation are compared at 900 by 760, 1600 by 1000 and 1920 by
1200 as well, and stay between 0.25 and 1.0 of 255.

The Inbox holds at other window sizes too, where its table gives up columns (Priority at a table
of 915 points, Project at 785). The first tab and `inbox-later-kinds`:

| Window | First tab | Later |
|---|---|---|
| 1600 by 1000 | 0.18 | 0.20 |
| 900 by 760 | 0.29 | 0.33 |
| 800 by 760 | 0.31 | 0.35 |

And on one person's real inbox, read through the door, with teammates, a cloud, a recorder and a
sign-in all on: Home 0.86, the Inbox's four tabs 0.14 to 0.25, a conversation 0.47.

## The same rows as the web window

Which threads are in which list, in which order and with which words is decided by rules ported
from the web window's own (`src/rules/`, each function naming the TypeScript it follows;
`docs/rules-spec.md` is the map). `tools/rows.mjs` holds the two against each other: it reads the
rows the web window drew out of the dumps and compares them with `mercury --rows`, row by row and
field by field, on Home and under each of the Inbox's four tabs. Under a tab it also holds what
stands around the rows: which tabs the strip draws and with which numbers, what the time column is
called, and what the cloud button wears. Any other picture of the Inbox left in the folder (a
cloud scene, `inbox-later-kinds`) is held the same way.

```sh
node desktop/tools/rows.mjs desktop/shots --shoot <checkout>            # the invented inbox
node desktop/tools/rows.mjs desktop/shots/real --shoot <checkout> --real   # the person's real threads
node desktop/tools/rows.mjs desktop/shots/live --shoot <checkout> --door   # the running app's inbox, this minute
```

`--door` asks the running app for its snapshot and for what its window keeps, through the local
door, so the fleet, the cloud and the team are the real ones too.

The real threads are read and never written, their words are printed as lengths only, and nothing
of them is ever committed. `cargo test` runs the rules' own tests on invented threads.

## What still differs from the web app, and why

- **Glass is less saturated.** The web app's glass is `blur(20px) saturate(1.5)`; Rune's
  `backdrop_blur` has no saturation. Up to 6 of 255 warmer in the real one, inside the composer.
- **Four blurred surfaces a frame.** Rune's `MAX_BACKDROP_BLURS` is 4 and Home has five, so the two
  small doors go unblurred. With the constant raised to 8 in a local copy all five blur. The code
  here turns their blur on by itself when Rune allows it.
- **A cut-off title loses its last space.** The browser keeps the space before the ellipsis. An
  aside cut short does the same. And the selected row's underline runs under the three dots of a
  title cut short, where the browser's stops before them.
- **Large text sits a fraction of a pixel differently**, which shows only in a difference map.
- **A long conversation can sit a pixel higher**, in a tall window: the heights of its parts add
  up a sliver apart.
- **A file named in a message is its path.** The web window draws a picture in place and makes a
  page, a document or a film a link; here they are the words the agent wrote.
- **The team's card has no rows.** With teammates, Home's second card is the team's week; it says
  so when nobody else is on the team and is bare otherwise, until the Team page is drawn.
- **The composer's chips are drawn, not chosen.** The model always reads Opus 5.5, and To always
  reads Agent.
- **The rail does not widen under the pointer**, and a person's photo is their two letters.
- **What the web window remembers only for itself is not known here**: an update card the person
  closed still shows its Restart row, and the Sign in row of someone who skipped signing in is
  not drawn.
- **Later lists threads only.** The web window also lists repeating tasks there, above the threads,
  and counts them in the tab's number. They are rules on a channel of their own that this window
  does not ask, so on a Mac with repeating tasks its Later says a smaller number and has fewer rows.
  A terminal agent put off, which the web window lists when the setting for such agents is on, is
  not there either.
- **No "try again" mark.** After the Status word of a thread nothing will start (Stopped, Nothing
  running, At its limit) the web window draws a small button that starts it again. This window
  only reads, and leaves it out.
- **The running marks stand still.** The half-dot and the small cloud before "Working" breathe in
  the web window.
- **The cloud button is a picture of the cloud's state**, with no menu behind it. Live, it goes on
  saying "Sign in again" for as long as an account is stopped: the web window stops saying it once
  its menu has been opened on that account, which it remembers for itself
  (`zero.cloudBrokenSeen`) and the door does not hand over. A fixture carries it.
- **When a thread comes back is said in English**, on a twelve-hour clock ("Sun 5:24 PM"), as the
  other dates are. The web window asks the system's language.

## What is where

- `src/main.rs`: the state, the three screens' routing, the command line.
- `src/source.rs`: fixtures and the door.
- `src/data.rs`: the snapshot's shapes and the lists the screens draw. `src/rules/`: the rules
  behind those lists, behind the Inbox's tabs and its cloud button (`page.rs`, `cloud.rs`), and
  behind what else depends on whose Mercury it is: `window.rs` (the team, the account, the rail's
  foot, a project's colour) and `place.rs` (where a conversation runs).
- `src/chrome.rs`: the page, the card, the rail. `src/theme.rs`: colours and type.
- `src/home.rs`, `src/inbox.rs`, `src/thread.rs`: the screens. `src/talk.rs` reads a thread's
  ledger into a conversation (`docs/thread-data.md` says what the engine is asked), and
  `src/markdown.rs` draws an agent's words.
- `tools/install-mac.sh`: builds the app and puts it in `~/Applications` as "Mercury Native".
- `src/sky.rs`: the dithered sky, ported from `renderer/src/home/sky.ts` with the same seed.
- `src/icons.rs`: the icons, with the web app's own paths and strokes.
- `assets/fonts`: Geist and Geist Mono (SIL Open Font License), the files the web app ships.
