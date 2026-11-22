# Mercury's desktop window, native

Mercury's window drawn with [Rune](https://github.com/anomalyco/rune) (Rust, GPU, no browser
inside) instead of a web page in Electron. The engine stays what it is: this window only draws,
and asks the engine for everything through a local door.

Where this is now: it reads, and it has begun to write. Home, the Inbox and one conversation come
from the same snapshot the web window gets and are kept current while the engine works. A
conversation can be answered, by typing or by picking one of its agent's options, marked done and
opened again, and the keys walk the lists (see "What the keys do, and what it writes"). A new
thread is started from Home's box or from the New thread card N opens over any other page, on the
project, model, effort and level picked in its menus ("Starting a thread", below), with the files
dropped on it, picked with its paper clip or pasted into it ("Files in a new thread").

What can be chosen in a conversation is chosen as in the web window ("Choosing in a
conversation", below): the model and level its next reply runs on, from the reply box's chip or
the model named on the folded box; where it runs, this Mac or the cloud, from the place button's
panel; and from the dots, Later and its priority. The strip's tab word and its arrows go where the
web window's go.

A file an agent names in a conversation is not left as its path: a picture is drawn in place, any
other file is a mention that opens it, and a folder on a line of its own shows its newest pictures
("The files a message names", below). A thread that made or named files carries the Files button
in its head strip, with how many, which opens the list of them ("The files a thread made"). A
press on any of them opens the file big over the window, its versions and the files beside it a
key away ("A file opened big"), and the rail's Files is the place of every file the threads made
("The Files place").

⌘K opens the command bar from anywhere: it finds threads by their words, as the web window's bar
does, and runs what this window can do to the thread under the keys and the places it can go
("The command bar", below).

Under the pointer it answers as the web window does: the rail opens while the pointer rests on
it, every control takes the web window's pointer and look under it, a key hint comes after the
pointer has waited, the strips move the window, words can be selected where the web window's
can, and a right press on a link or a picture opens the web window's menu for it ("How it answers
the pointer and the hands", below).

A conversation shows what each of its agent's runs did: what the run said, between the messages,
and what it ran folded to one line under that, which a press opens. A thread an agent is on says
at its foot which step it is on and for how long, and is read again as the run goes on ("What a
run did", below).

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
cargo run -- --fixture shots/thread-pictures-fixture.json --thread --snap shots/thread-pictures.png --size 1280x760 --scale 2
cargo run -- --fixture shots/palette-typed-fixture.json --view inbox --snap shots/palette-typed.png --size 1280x760 --scale 2
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
line each time its conversations change: a check that the live path works. It also reads the
newest thread the way an open conversation with a run up is read, on every change and every
eight seconds, and with `MERCURY_READS=1` says what each read asked and how many bytes came back.

The app opens a new door, on another port and with another key, each time it starts. The window
reads the note again when the door it has stops answering, so it carries on by itself after the
app restarts, and one opened before the app finds the door once there is one.

The app tells the window when its inbox changed, on the door's event stream, with the one word
it tells its own window (`zero:changed`, never what changed: `tellTheDoorToo` in
`main/local-door.mts`), and the window then reads the snapshot again at once. It still asks
unprompted every five seconds, because the engine does not say everything: its store learns of
a change from a watch on its folders, which misses some, and which threads were read in the web
window is never pushed. An app from before this (or any engine that keeps the stream open and
says nothing on it) is asked every two seconds, as it always was; measured on a 99-thread
inbox, that costs the app about 2% of one core and the reader next to nothing.

### The banner, the island and this window

The desktop app puts up a banner, and its island in the notch (the corner tag elsewhere), only
while the person is somewhere other than the app, and it judged that by its own window's focus.
With the day's work done here, its window had no focus, so the banner spoke and the island came
up over this very window, and pressing either brought the web window forward. So the two tell each
other two things more, on the desktop app's own door alone (`src/front.rs`, and
`main/native-window.mts` in the app):

- **This window says where it is** (`zero:native-window`, `{ "focused": true }`) each time it gains
  or loses focus, and again every five seconds, in front or not. The app counts the person as at
  the app while this window is in front, as it does for its own window. It believes a word for
  three beats: each beat is asked of this window's drawing thread, so a window that hangs falls
  silent and stops counting, and one that quits or crashes hangs up the event stream and is
  forgotten at once.
- **A press comes here** (`zero:open-in-native` on the event stream, `{ "id", "product" }`). While
  the app has heard this window within three beats, a press on the banner, on the island or the
  corner tag, or on a line of either, is sent here instead of bringing the web window forward: the
  window comes forward and opens the thread, as a press on its row would. The tray knows a
  thread's id and not its project, which is taken from the snapshot. A press naming no thread (the
  island itself), or a thread this window does not have, brings the window forward as it is.

An approval's banner still brings the web window, which alone draws the card that answers it, and
so does Settings in the tray's menu. An approval's line in the tray asks for the app as it is, as
a press on the island itself does, so it brings this window, whose foot says the yes is answered
in Mercury's own window. With no window beating at the door, a press does what it always did.

The other way, this window asks the app for two things of its own (`ask_the_app` in
`src/front.rs`). Its rail's Settings, and the person's own row under it, bring the web window
forward on its Settings (`zero:open-settings-here`, a channel only the desktop app's door answers,
which takes nothing from the caller and opens Settings and nothing else); and Restart to update,
while a new version waits, is the web window's own button (`zero:update-install`): the app quits
onto the new version and comes back, and this window finds its new door. When it works nothing
here changes; a refusal (no app, or its window gone) is said in a line.

Checked headless on 2026-11-22 against a small door on loopback that answers the snapshot with an
invented inbox, records what this window says and pushes `zero:open-in-native` at set times; a
script's `{"t": 0.5, "focus": true}` stands for the window gaining focus:

```sh
cargo run -- --door http://127.0.0.1:<port> --token <key> --script focus.json --realtime \
  --snap /tmp/after.png --size 1280x760 --scale 1
```

Each change of focus reached the door within 10 ms, a window never focused said where it was at
its first beat and every five seconds after, and a push opened its thread within 10 ms (the
`zero:phone-seen` that opening writes, and the picture after). A push naming no thread, a thread
that has gone, or one under another project left the page as it was. A headless run drops the
request to come forward, so this window coming to the front over another app has not been seen;
on a Mac it is winit's `focus_window` (`activateIgnoringOtherApps`, then `makeKeyAndOrderFront`).

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

The same conversation is photographed with the logs its runs left, invented in the supervisor's
own shape (`runs` in `thread.mjs`'s `open`): `thread-steps-folded` (as it first opens),
`thread-steps` (unfolded, each run's work folded), `thread-steps-open` (one fold pressed open),
`thread-steps-failed` (a run turned away at its usage limit, and the fold that holds a failed test
pressed open) and `thread-working` (opened from In progress while a run is up, its last call
still running). A fixture says which folds were pressed (`runsOpen`) and which thread was open
when it is not the first in Needs you (`thread`).

A conversation that names files needs files. `tools/pictures.mjs` plants an invented project
folder in the system's temporary folder (flat pictures of several shapes, a folder of rounds, a
page, a note, a sheet, a second version of a picture and of the note, and a picture and a report
in `attachments/`, where the app saves what a message carries), a scene moves its thread's
project into it (`files` in `thread.mjs`'s `open`), and the page's pictures are answered from it. The native window reads the same files off the disk. Each planting starts the
folder again, so these scenes are photographed one after another, never two at once; and a task
that photographs scenes while another does on the same Mac plants its own folder
(`MERCURY_PLANTED=<folder>`), or each starts the other's again under it. The scenes:
`thread-pictures` (a picture named bare in a sentence, as a markdown image, as a link),
`thread-pictures-ways` (in a code span, by its full path, found beside the code),
`thread-pictures-row` (three in one sentence), `thread-pictures-large` (wider than the column,
taller than the window), `thread-pictures-missing`, `thread-pictures-arriving` (the place held
before one comes), `thread-pictures-said` (inside an unfolded conversation), `thread-links` (one
file of each kind that is not a picture), `thread-links-beside` (a design page and a note on a
line of its own), `thread-links-media` (a film and a sound, which the web window plays and this
one names), `thread-folder` and `thread-folder-quiet` (a folder on a line of its own),
`thread-pictures-many` (ten screenshots, for the memory measurement), and `thread-files` and
`thread-files-open` (a thread whose files come in versions, its Files button and the list it
opens; "The files a thread made", below), and `thread-reply-attached` (a reply that carries a
pasted picture and a report, drawn as the person's own words).

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

A conversation that names files, whole window, at the three sizes. Its head strip carries the
Files button with its count, as the web window's does (all but `thread-folder`, which names only
a folder): the strip alone is 0.55 at 1280 by 760, as a strip with no files is (`thread`), where it
was 0.86 while this window drew no button. At 900 by 760 the thread's name in the strip used to
run on over the arrows instead of being cut short: the strips with the button are 0.68 there,
where they were 1.79, and `thread-folder`'s 0.71, where it was 0.98.

| Scene | 1280 by 760 | 1600 by 1000 | 900 by 760 |
|---|---|---|---|
| `thread-pictures` | 0.32 | 0.30 | 0.42 |
| `thread-pictures-ways` | 0.35 | 0.32 | 0.46 |
| `thread-pictures-row` | 0.33 | 0.27 | 0.43 |
| `thread-pictures-large` | 0.23 | 0.17 | 0.29 |
| `thread-pictures-missing` | 0.44 | 0.34 | 0.59 |
| `thread-pictures-arriving` | 0.39 | 0.39 | 0.51 |
| `thread-pictures-said` | 0.23 | 0.37 | 0.29 |
| `thread-links` | 0.66 | 0.61 | 0.89 |
| `thread-folder` | 0.36 | 0.36 | 0.47 |
| `thread-folder-quiet` | 0.32 | 0.27 | 0.41 |

Inside the answer alone, at 1280 by 760: 0.07 where it is only pictures (`thread-pictures-large`),
0.29 to 0.67 where sentences name them, and 1.23 for `thread-links`, whose ten mentions each
carry an underline two points higher than the web window's (see what still differs).

The command bar is photographed open over the Inbox: `palette` (its box empty), `palette-typed`
("check": a command, then the threads the word found) and `palette-none` (words nothing
answers). The web window's bar offers far more commands than this one can do, so the two are
only alike when this window draws the same commands. Each of these scenes reads the rows its
page drew and hands them on in the fixture (`paletteRows`: a heading, a command with its mark,
name, hint and key, or only the place of a thread), with the words typed (`palette`). Given
them, this window draws the web's commands as they were, pressing them does nothing, and where
the web listed a thread it lists the next one its own search found for the same words, so the
threads section measures the search as well as the drawing. Without `paletteRows` it draws its
own rows, as it does live. Each scene names its parts (`tools/scenes/palette*.regions.json`).

| Part, at 1280 by 760 | `palette` | `palette-typed` | `palette-none` |
|---|---|---|---|
| Whole window | 0.15 | 0.14 | 0.13 |
| The box, its frame included | 0.22 | 0.28 | 0.39 |
| Search row | 0.67 | 0.45 | 0.67 |
| Esc | 0.03 | 0.03 | 0.03 |
| The lit row | 0.32 | 0.56 | |
| The rows under it, the threads found, the line saying nothing matches | 0.12 | 0.18 | 0.30 |
| Foot | 0.15 | 0.15 | 0.15 |
| The backdrop around the box | 0.09 to 0.14 | 0.08 to 0.13 | 0.07 to 0.12 |

The empty bar is 0.17 at 900 by 760 and 0.11 at 1600 by 1000, its parts as at 1280. What is
left is the caret (see what still differs) and glyphs, the placeholder's among them, landing
within half a device pixel of the browser's.

A conversation with its runs' logs, at 1280 by 760, whole window and the area the runs changed
(each scene's own `.regions.json`). The text-dense areas of the earlier scenes sit between 1.1 and
1.9 the same way, from how the two draw a glyph's edge:

| Scene | Whole window | Where the runs are drawn |
|---|---|---|
| `thread-steps-folded` | 0.39 | 1.65, the title and its "Show 8 earlier messages" |
| `thread-steps` | 0.50 | 1.22, a run's sentences and its two folds |
| `thread-steps-open` | 0.44 | 0.92, a run and its fold opened |
| `thread-steps-failed` | 0.47 | 0.96 the run turned away, 0.86 the fold with the failed test |
| `thread-working` | 0.41 | 1.39 what the run said, 0.57 the line at the foot |

`thread-steps-folded` is 0.53 at 900 by 760 and 0.33 at 1600 by 1000.

## What a press does, frame by frame

A still picture says nothing about what happens between two of them. Rune animates a press by
default: what two pages share springs from its old place to its new one, the rest fades across,
and colours blend over a few frames. The web window's pages, tabs, rows and rail have no
transition, so the window's root lands every change at once (`instant_changes` and
`Spring::NONE`, in `src/main.rs`). To see a press as it plays, script it and tile the frames:

```sh
echo '[{"t": 0.30, "type": "tap", "x": 292, "y": 133}]' > /tmp/tab.json   # the second tab
cargo run -- --fixture shots/inbox-fixture.json --view inbox --script /tmp/tab.json \
  --strip shots/tab-strip.png --start 0.28 --frames 12 --fps 30 --cols 2 --crop 82,104,1176,130 \
  --size 1280x760 --scale 1
```

The frame after the press should be the new page whole, and every frame after it the same.
The command bar was checked so at 60 frames a second: opening it, moving its lit row and closing
it each land in the next frame, and Rune counts no frame as animating.
Pressing the fold of what a run ran in `thread-steps`, filmed at 60 frames a second, draws the
closed fold until the tap lets go and the opened one whole from the next frame on. So does the
Files button's list (`thread-files`), opened and closed every way it closes. So do a tab and a row
of the Inbox pressed with the pointer resting on them, and a place pressed on the rail while it
is open. The one thing that eases is the rail opening and folding under the pointer, below.

## How it answers the pointer and the hands

A picture of a screen at rest says nothing of what the pointer does to it. The web window's
stylesheet says what each control does while the pointer is on it, which pointer it shows, which
words can be selected and where a press moves the window, and its scripts say what waits on a
pointer resting. This window does the same, read off the web window the way its screens were.
`src/feel.rs` holds what several screens share; each screen draws its own controls' looks.

- **The rail opens.** 140 ms after the pointer moves onto the rail it opens from 38 points to 202,
  the whole sidebar with its words, and leaving it folds it at once (`hoverAt` in
  `WorkspaceNavigation.tsx`): a pointer passing over it on its way somewhere is gone before then,
  and a pointer that did not move opens nothing, so a rail uncovered under a pointer standing
  still (by the command bar closing, say) stays folded until it moves (`pointerMoved`). It
  eases open over 0.24 s (`cubic-bezier(.22,.61,.36,1)`) and shut over 0.2 s
  (`cubic-bezier(.4,0,.2,1)`); the words come in 0.08 s later over 0.14 s and go over 0.06 s, and
  Search's line and Inbox's dot cross over 0.14 s and 0.1 s (`team/rooms/rooms.css`), each frame
  drawn while it moves. The card moves over with it, 44 points past its edge, rather than being
  covered, and the skies stay painted for the folded card and pinned to its right edge, so the rail
  uncovers them rather than painting them again (`sky-follows-the-card.ts`). Open, it says what
  the web window's says: each place's word, Inbox's count in its accent where the dot was, the
  team's name, Search with ⌘K, the foot's rows, the usage row naming the agent read last and its
  headline figure, and the person's name over their email. Its places wash under the pointer.
  Under the system's Reduce motion it opens and folds whole, as the same stylesheet's
  `prefers-reduced-motion` block takes every one of those transitions away.
- **Under the pointer.** Every control these screens draw takes the web window's look while the
  pointer is on it and gives it up the moment the pointer leaves. The web window's `:hover` rules
  have no transition unless one says so, and almost none does, so the root lands every hover at
  once both ways (`hover_motion_within` in `src/main.rs`; Rune's own hover fades out over 0.1 s).
  The two that ease, a mention and Search's line, land at once here too. On Home: the
  two doors, the rows (a Needs you row under the pointer becomes the row the keys act on), the
  cards' links, the box's chips, paper clip, Send and its caret. On the Inbox: New thread, the
  cloud and the display button give up their dark glass for the wash, back and forward brighten
  while they have somewhere to go, the tabs, and the rows. Over a conversation: the strip's arrows,
  the crumb, the thread's name, Done or Reopen, the Files button, the place button and ⋯; the line
  that shows the earlier messages; a run's fold, whose mark comes in; the options; the folded reply
  box and its model; the opened box's chips, clip, Send and caret; mentions; pictures, whose edge
  brightens; a folder's name, its pictures and its "more" line. On the New thread card: the head's
  words, whose dotted line brightens, and the bar. And in either box, a file's chip and a
  picture's ×. A control the web window does something with that this window does not do yet
  still looks and points as it does there (`feel::answers`).
- **The pointer's shape.** The pointing hand over every control, the magnifier over a picture and a
  folder's pictures, the I-beam over words a hand can select and over the whole of a box to write
  in, a press anywhere in which puts the caret after the words (on Home, on the New thread card and
  in the Later picker), and the arrow everywhere else. The Inbox's row under the pointer is the
  one Enter, E and L act on, its title faintly underlined, until J, K, the arrows, Escape or Tab
  take the list back (`pointed` in `renderer/src/App.tsx`).
- **Key hints.** 700 ms after the pointer moves onto New thread, a tab, a row or the cloud on the
  Inbox, or onto a conversation's Inbox arrow, its crumb, J and K's arrows, Done, the Files button,
  the place button or ⋯, a plate under it names its keys and what they do, in the web window's words
  (`HINTS` in `renderer/src/hint-plate.ts`): seven points under the control, or over it near the
  window's foot, lined up with its left, its right or its middle as the web window lines each up,
  kept eight points inside the window, on the window's glass with a cap per key. It goes the moment
  the pointer leaves or a key is pressed, and comes back after the pointer's next move; none comes
  while the command bar, the New thread card, the Later picker or the reply box has the keys
  (`hintsOn`). A pointer that moved, as the web window listens for `mousemove` and not
  `mouseover`: a row that comes under a pointer standing still, as the next one does when E
  closes the one under it, claims no plate.
- **Titles.** What the web window says in a `title` this one says with Rune's `help`: Back's "Back
  to Home (⌘[)" and the rest, New thread's "New thread (N)", the cloud's question, "View and
  filters", each rail place, the box's chips, clip, Send and caret, a conversation's whole name, a
  mention's address, a picture's or a folder's path, and a file in a box: its name, Remove, or why
  it cannot go. A Mac window hands them to AppKit, which keeps its own wait and look, as Electron's
  are the system's. ⌘[ goes back, as Back's title says.
- **Moving the window.** As the web window's `-webkit-app-region: drag`: the band along the window's
  top, 40 points, over everything but the command bar while it is open
  (`.workspace-layout::after`), and the card's top strip on the Inbox and over a conversation,
  whose buttons, crumb and name keep their presses (`no-drag`). Home's strip is hidden there and
  is no band here, and the rail never is. A double press on a band zooms the window or puts it
  away, as the system's setting says.
- **Words to select.** What the web window lets a hand select, this one does: what a conversation's
  reading column holds (`.focus-body`), its title, the line under it, every message's head and its
  words, code and tables. The rest of the window, as the web window's `body { user-select: none }`,
  is not.
- **A right press.** On a link in a message, the web window's menu (the `context-menu` handler in
  `main/main.mts`): Copy File first when the link finds a picture or a film in the project's
  folders, then Open Link and Copy Link Address, the address as Chromium parses it (a site named
  bare ends in its slash, a file is a `file://` url to where it was found). On a picture, Copy File
  when the message named it by its whole path, as the web window's picture carries it, and Copy
  Image over the drawing itself, read whole from its file rather than the small copy drawn here.
  A folder's pictures carry their whole paths. On the words around them, Rune's own Copy, after
  selecting the word under the pointer, as Chromium on a Mac does; in a box, Rune's Cut, Copy and
  Paste. The menus are the system's own, as Electron's are.
- **Keys.** Tab on Home does nothing to be seen: the web window takes it for the Inbox's tabs and
  lets its focus go (`onTheTabs`), where Rune would walk the focus into the box. Neither window
  ever draws a focus ring (`*:focus, *:focus-visible { outline: none }`).

### How it is measured

A scene can leave the pointer resting on something (`tools/scenes/pointer.mjs`): moved onto it in
two steps, as a hand moves, so the page sees a pointer that moved, then held still for `rested`
milliseconds before the shutter (`settle`). The reference picture's dump now says, for the middle
of every element on screen, the pointer's shape there (`pointers`: the computed `cursor`, with
`auto` taken as Chromium takes it, the I-beam only over words a hand can select), and each
element's `user-select`, app region and transitions. The scenes: `*-hover-*` (a control under the
pointer, before its hint's 700 ms), `*-hint-*` (past them), and `*-rail-open`.

```sh
arch -arm64 node tools/reference.mjs <checkout> shots dark 1280 760 inbox-hover-tab
python3 tools/hover.py shots inbox-hover-tab=inbox                 # the tab, and the tab at rest
MERCURY_FEEL=shots/thread cargo test pointer -- --ignored --nocapture # every shape over a scene
```

`tools/hover.py` moves this window's pointer the same way (a `--script`), draws it that long after
(`--at`), and measures the control under the pointer, of 255, beside the same box at rest when the
resting scene is named, which is as close as the two draw that control with no pointer on it; it
measures a hint's plate too. The ignored test walks the pointer over every point of a scene's dump
and holds each shape to the web window's (`rt.cursor()`, the shape Rune hands the system).

Under the pointer, the control, at 1280 by 760, 2x; at rest, the same box with no pointer:

| Control | Under the pointer | At rest |
|---|---|---|
| Home: a door, a card's link | 6.04, 5.92 | 5.44, 5.35 |
| Home: a row, the box's chip, clip, Send's caret | 1.11, 2.94, 2.78, 1.61 | 1.39, 2.74, 2.24, 1.34 |
| Home: a file's chip, a picture's × | 0.43, 7.34 | 0.30, 5.08 |
| Inbox: back, display, New thread, a tab, a row | 0.01, 0.39, 1.63, 2.23, 0.33 | 0.00, 0.25, 1.46, 1.67, 0.33 |
| Conversation: Inbox, back, crumb, name | 0.32, 0.42, 1.01, 2.74 | 0.01, 0.12, 1.15, 2.15 |
| Conversation: J's arrow, Done, Reopen, Files, ⋯ | 0.69, 1.63, 2.80, 1.59, 0.75 | 0.26, 0.93, 1.93, 0.54, 0.06 |
| Conversation: earlier messages, a run's fold, an option | 3.93, 2.05, 0.76 | 2.41, 1.62, 0.45 |
| Conversation: a mention | 14.96 | 7.18 |
| Conversation: a picture, a folder's picture, a folder's name | 0.15, 0.33, 2.76 | 0.14, 0.33, 1.92 |
| The folded reply box, its model | 0.70, 4.14 | 0.70, 3.28 |
| The opened reply box: a chip, clip, Send, caret | 1.63, 1.62, 1.25, 0.37 | 1.22, 1.09, 1.20, 0.27 |
| The New thread card: its model, a chip, Send | 0.73, 1.41, 2.89 | 0.73, 1.23, 2.10 |

The mention is what still differs most: its mark does not brighten with it (see what still
differs). The plates, with a point around each: New thread 2.67, a tab 2.51, a row 2.17, Done 1.89,
J's arrow 1.36, the Inbox arrow 1.42, the crumb 4.89 (the edge of its glass over the strip's) and ⋯
1.09. Each is drawn between 0.70 and 0.74 s after the pointer came onto its control, as the web
window's comes 700 ms after (`HINT_WAIT`). New thread
was measured before its title was: since, a headless picture draws Rune's own stand-in for the
title from 0.5 s, over the plate (36.89), where a Mac window leaves the title to AppKit and its wait.

Measured before the rail lost the buttons nothing here did (see what still differs), so against a
rail that still had Demands, Connections and Files under the Inbox:
the rail open, the pointer on Demands: the whole window 0.90 at 1280 by 760, 0.94 at 900 by 760
and 0.71 at 1600 by 1000, the rail itself 0.21, 0.22 and 0.17, and Demands 0.57, 0.58 and 0.55.
Over the Inbox 0.26 and a conversation 0.41. With teammates, a recorder and a sign-in (`home-team-
rail-open`) 0.99, the rail 0.55; while an update waits (`home-update-rail-open`) 1.21, the rail
1.97, as the update's card is not drawn open (see what still differs).

The pointer's shape, points that match the web window's of all the points in each scene:

| Scene | Arrow | Pointing hand | I-beam | Magnifier |
|---|---|---|---|---|
| `home`, `home-attached` | 50 of 50, 56 of 56 | 78 of 78, 76 of 76 | 2 of 2, 1 of 1 | |
| `inbox` | 29 of 29 | 171 of 172 | | |
| `compose-card`, `palette` | 186 of 186, 204 of 204 | 17 of 17, 65 of 65 | 19 of 19, 2 of 2 | |
| `thread`, `thread-reply` | 33 of 41, 34 of 38 | 35 of 35, 47 of 47 | 15 of 15, 10 of 10 | |
| `thread-options`, `thread-steps` | 37 of 41, 28 of 34 | 62 of 72, 42 of 42 | 24 of 24, 23 of 23 | |
| `thread-pictures`, `thread-folder` | 35 of 37, 29 of 30 | 46 of 50, 41 of 41 | 11 of 11, 11 of 11 | 9 of 9, 25 of 25 |
| `thread-links` | 27 of 39 | 63 of 84 | 11 of 11 | |
| `thread-files-open` | 40 of 40 | 23 of 71 | 0 of 10 | 0 of 3 |

What is left: the I-beam over the end of a line past its last word, every mention's mark, the
sorting headers of a table in a message, the Inbox's box that ticks every row (not drawn), and the
page under the open list of files, which shows the arrow. Each is in what still differs. Where the
46 `*-hover-*` and `*-hint-*` scenes rest the pointer, its shape is the web window's at every one:
the pointing hand, the I-beam over a conversation's name and the folded reply box, the magnifier
over a picture and a folder's, and the arrow over Send with nothing to send.

The screens at rest did not move: home, inbox, the Inbox's other tabs and `inbox-later-kinds`,
thread, `thread-folded`, `thread-later`, `thread-options`, `thread-steps`, `thread-reply`,
`thread-files`, `thread-files-open`, `thread-pictures`, `thread-folder`, `thread-links`,
`home-team`, `home-typed`, `home-attached`, `home-attached-picture`, `compose-card`,
`compose-card-attached`, `palette` and `palette-typed` are drawn to the same pixel as before.

Tests hold the rest to the web window on an invented inbox (`src/feel.rs`): the rail opens 140 ms
after the pointer moves onto it and folds when it leaves, whole under Reduce motion, and stays
folded under the command bar and when the bar closes over a pointer standing still; a hint comes
700 ms after the pointer moved onto its control, hangs where the web window's does, goes with the
pointer or a key, comes back on the next move, and is claimed by no row that slides under a still
pointer; the row under the pointer takes Enter until J walks the list; ⌘[ goes where Back's title
says; the bands move the window and their buttons keep their presses; and a right press on an
address, a picture's name, a picture named short and one named whole opens the web window's menu
for it, each item putting on the clipboard what the web window's does.

### What only a screen can show

Everything above was checked headless, and some of it is the system's to do, which a headless run
does not reach:

- **The pointer itself.** The tests read the shape Rune hands the system (`rt.cursor()`); the shell
  shows it as the matching `NSCursor` (the magnifier as AppKit's zoom-in cursor). No pointer was
  seen on a screen.
- **Moving the window.** The tests see Rune's titlebar action (a drag, a double press), which the
  shell hands to AppKit (`drag_window`, the double press's zoom). No window was dragged.
- **Titles and menus.** In a Mac window AppKit draws the titles, after its own wait, and the right
  press's menus. Neither was seen; a headless picture draws Rune's stand-in titles.
- **Scrolling** with a trackpad, and the rail's easing at a display's own rate.

### Where Rune, as pinned, stops

- `El::cursor` shows only over something that takes the pointer, so a control this window does not
  do anything with yet takes it with a handler that does nothing (`feel::answers`).
- An element has one `on_hover`, and a key hint's wait is a layer of its own around its control.
  Rune's `hover_delays` waits on a pointer however it came to be there, so the rail's wait and a
  hint's start from the pointer's own moves (`on_pointer_move_local`) and are kept in the state.
- Rune's hover fades out over 0.1 s unless the root says otherwise, and almost none of the web
  window's `:hover` rules fades, so the root lands every hover at once. There is no transition per
  property, so the rail's easing is drawn frame by frame from the poll (`chrome::Rail`), and the
  two hovers that ease in the web window do not here.
- Selectable words show the I-beam over their whole box, where Chromium shows it over the words'
  lines only.
- A link covers words and not a mark set in among them, so a mention's mark neither points nor
  brightens with its words, and a link's look under the pointer lands at once.
- A right press's menu opens as the button comes up, where a Mac app's opens as it goes down; a
  handler cannot read the words selected, so a link's menu has no Copy above it; a box's menu is
  Rune's own.

## What the keys do, and what it writes

The keys are the web window's (`src/keys.rs`, from the keydown handler in `renderer/src/App.tsx`):

| Where | Key | Does |
|---|---|---|
| A list | J, K, ↓, ↑ | Walks the rows |
| A list | Enter | Opens the row, which reads it |
| The Inbox | Tab, Shift Tab | Turns its tabs |
| The Inbox | E | Marks the row done (under Needs you and In progress); under Later, brings it back |
| The Inbox, a conversation | L | Opens the Later picker on the thread |
| The Later picker | ↓, ↑, Enter, Esc | Walks its moments and picks the one lit, or what is typed; Esc closes it |
| A conversation | Esc | Folds the reply box, then goes back, onto the tab that holds the thread now |
| A conversation | J, K | Goes to the next thread that needs the person, or the one before |
| A conversation | ↓, ↑, Enter | Walks the agent's options and sends the one lit |
| A conversation | 1 to 9 | Sends that option |
| A conversation | R | Opens the reply box; ⌘Return sends what is in it |
| A conversation | E | Marks it done and goes on to the next thread |
| A conversation's list of files | Esc | Closes it; every other key goes on to the page under it |
| A conversation's menu (the dots) | ↓, ↑, Home, End, Enter, Space | Walks its rows around both ends and presses the one lit; every other key goes on to the page |
| A conversation's menu, its place panel | Esc | Closes it; Tab closes the menu too |
| The reply box's model menu | ↓, ↑, Enter, Esc | Walks its rows around both ends and picks the one lit; Esc closes it and puts the caret back in the words; any other key goes on to the page |
| Home | N, or a letter none of the list's keys | Puts the caret in the box on the page (the letter typed into it) |
| The Inbox, a conversation | N | Opens the New thread card over the page |
| The box, the card | ⌘Return | Sends the thread; Return is a new line |
| The box, the card | Esc | Closes the open menu; then lets go of the box, or closes the card, keeping the words |
| A chip of the box | ↓, ↑ | Opens its menu (after Esc left the keyboard on it) |
| A menu of the box | ↓, ↑, Enter | Walks its rows around both ends and picks the one lit |
| Anywhere | Z | Takes back the last thing done |
| Anywhere | ⌘K | Opens the command bar, and closes it again |
| The command bar | ↓, ↑, Enter, Esc | Walks its rows and runs the one lit; Esc closes it |

Everything written is one call on a channel the web window calls too, with the web window's own
words in it (`src/rules/writes.rs`): a reply, a picked option ("Option 2: …") and a close go to
`zero:answer`, Reopen to `zero:reopen`, and a thread opened is told to the web window, which
alone keeps what has been read, the way a phone tells it (`zero:phone-seen`). An option that
closes ("Close this task"), leaves the work on its branch or needs the person's own words
("Change something first") does that instead of being sent, by the web window's rules. A reply
typed in the box carries the model, level and agent picked in it, and what it may do, as the web
window's box sends them; a level picked in a conversation's menu and where it runs are written
on their own channels ("Choosing in a conversation", below).

What is done waits three seconds before it is written, as in the web window (`deferCommit`): it
shows at once, a line says so with Z and Undo beside it, and Z inside that time takes it back
whole, because nothing was written. After a close is written, Z opens the thread again as it
was, blocked if it was blocked, so no run starts. If the engine refuses a write, its own reason
is said in one line and what was refused is put back.

Later is the one thing not held back, as in the web window (`snoozeUntil`, `unsnooze`): L opens
the picker (`src/later.rs`), which offers the web window's moments and reads a time typed in
words by its grammar (`src/rules/later.rs`, with the web's own test cases: "3h", "tomorrow at
noon", "fri 3pm", "oct 10"). Typed words decide, and words that are not a time are refused
beside the box rather than falling through to the lit row. The pick is written at once to
`zero:snooze`, and only when the engine has it does the line say so and the thread leave the
list; Z then writes the moment it had before and opens the thread. A thread already put off
offers "Now" first, which brings it back, as E does on its row under Later: the reminder comes
off, a run set for later is let go now (`zero:schedule`), and a thread written down and never
begun is started (`zero:thread-edit`). Z puts all of that back.

To try any of this without touching a real inbox, run an engine with no window and no agents on
an empty store, and point the window at it:

```sh
env -i HOME=/tmp/scratch/home PATH="$PATH" MERCURY_HOME=/tmp/scratch/store ZERO_NO_SUPERVISOR=1 \
  AGENTBOX_NO_SYSTEM_WINDOWS=1 node bin/mercury.mjs --no-open --dir /tmp/scratch/proj   # prints its address
cargo run -- --door http://127.0.0.1:<port> --token <key> --thread <id> \
  --script keys.json --realtime --strip shots/keys.png --frames 6 --fps 1 --size 1280x760 --scale 1
```

`ZERO_NO_SUPERVISOR` keeps the supervisor's clock stopped (`bin/mercury.mjs`), and that is all:
a thread composed on that engine (`zero:compose`, even with `"start": false`) still started a run
on 2026-11-22. Under the empty `HOME` it ended at once, refused as not logged in, and the
supervisor wrote its own note on the thread. To try reading a run's log, plant one under the
thread's `sessions/` folder in the store rather than write anything that could start a run.

A script is a list of events (`{"t": 0.5, "type": "key", "name": "e"}`, `"type": "type"` with
`"text"`, `"type": "tap"` with `x` and `y`); `--realtime` lets the three seconds pass. The scenes
`thread-options`, `thread-option-picked`, `thread-reply`, `thread-reply-typed`,
`thread-options-reply`, `thread-done-held`, `inbox-done-held` and the four `thread-later*` are the
web window in each of these states; a fixture says what its picture had open (`replying`,
`option`, `closed`, `later`, `laterRefused`).

## Choosing in a conversation

What a conversation offers to choose is the web window's, ported where the person uses it: the
model and level of the next reply (`src/reply.rs`, its rules `src/rules/reply_pick.rs`), where the
thread runs and the thread's own menu (`src/strip.rs`, the place's rules `src/rules/place.rs`),
and the strip's other presses.

**The model and how hard it thinks.** The reply box's model chip opens the New thread box's own
model menu (`compose.rs`), raised out of the chip eight points over it, its left edge on the chip's
(`ReplyModel.tsx`, which draws the card's `ModelMenu`): the models used lately, each with its agent
where this Mac offers more than one, All models, and the level strip at its foot. It opens with the
keys on the model the reply is on; the arrows walk it around both ends, Enter picks, Escape closes
it and puts the caret back in the words. The model named at the folded box's right end opens the
box with the menu already open. Then, as in the web window, whose box takes the focus back as it
mounts (asked of the web page itself: the caret in the words, no row lit, ↓ moving the caret), no
row is lit until the pointer moves onto one, which then takes the keys. The chip says the model,
with its level where one will be sent ("Sonnet 5.5 · High", `effortSaid`). A model of another
agent moves the thread to that agent, and a line under the words says it starts fresh
(`freshNote`).

A pick is not written. It is kept beside the box's words until the next reply, which carries it
to `zero:answer`, and the store writes the agent, model and level before the words, so the run
that reply starts reads them (`pickToSend`, `answerItem` in `main/store.mts`): Claude Code's model
always named, the other agents' defaults by naming none, the agent only where the pick moves the
thread. A pick with nothing typed writes nothing, a picked option of the agent's carries none, and
the folded box goes on naming the thread's own model until a reply carries the pick. Z inside the
three seconds puts the words and the pick back; a refused reply comes back by its words alone, as
in the web window (`restoreFailedDraft`). The reply also says what it may do, the thread's own mode
or none, which the engine keeps beside the ledger rather than on it (`permissionMode`, as
`DockComposer` sends it).

**Where it runs.** On a Mac set up for the cloud, the button beside Done, a cloud or a Mac, opens
the panel under it, 440 points wide, its right edge on the button's (`PlaceButton.tsx`): Runs on,
Cloud and This Mac with the thread's setting picked, a note saying why in the words of
`renderer/src/where-why.ts`, and for a thread the cloud handed back, what it needs. A place that
cannot be picked is faded. A pick is written at once: keeping the thread here or letting the cloud
have it to `zero:set-thread-on-mac`, carrying its run there now to `zero:move-to-cloud`, and with
the whole cloud off, the workspace back to what it ran when on (`zero:settings-set-workspace`) and
the thread to the cloud (`placeThread` in `renderer/src/App.tsx`). The panel stays open on the
pick, and the button with it, until the engine says something new about the thread or four
seconds pass (`afterPick`). Nothing is said, unless the cloud takes none of a move: "The cloud
can’t take this thread right now". Escape closes it, and so does a press outside it.

**The thread's menu.** The dots open the web window's menu under them, 264 points wide, its right
edge on theirs (`ThreadMenuList` in `renderer/src/threads/ThreadMenu.tsx`), with the rows this
window does, in the web's order: Later (L), which opens the Later picker on the thread, and
Priority with the level it runs at, which turns the menu to the four levels, the one it is at
ticked. A level picked is written at once (`zero:answer` with the level and no words, which starts
nothing: `retag`), and "Priority: high" is said once the engine has it. Done and Reopen stay
buttons of their own beside the dots, as the web window's menu never lists them. The first row is
lit as it opens; the pointer and the arrows move it, Home and End go to the ends, Enter and Space
press it, Escape and Tab close it, and any other key goes on to the page, as the web window's
capture listener lets it through (E still marks the thread done). On a finished thread the web
window's menu holds only the terminal and renaming, so the dots are drawn and a press says where
those are.

One of the three is open at a time, on one thread: leaving the thread, the command bar, the Later
picker and the New thread card close it. A press outside the strip's group of buttons closes the
menu or the panel on its way down; a press on one of those buttons (the Files button, the place
button, Done, the dots) goes on to it.

**The rest of the strip.** The tab's word before the thread's name is the way back, as Escape is:
to where the person came from, and when that is the Inbox and the thread has moved tabs while it
was read, onto the tab the strip names, with the keys on the thread's row (`closeThreadOntoItsTab`).
The arrows go to the thread before and after it in Needs you (`QueueNav`), and the list button at
the strip's left to the Inbox's first tab, from its first row (`openView('inbox')`).

The scenes: `thread-actions` (the menu open over the waiting thread), `thread-actions-priority`
(turned to the four levels), `thread-place` (the panel, on the inbox with the cloud on) and
`thread-reply-model` (the model's name on the folded box pressed). The web's rows this window does
not do are handed on in the fixture (`actionRows`) and drawn as they were, a press on them doing
nothing, as the command bar's scenes do with its commands. A fixture says what was open
(`actions`). At 2x, dark, each scene's own regions:

| Scene | Window | Whole window | The menu or panel | Its button |
|---|---|---|---|---|
| `thread-actions` | 1280 by 760 | 0.69 | 0.25 | 0.64 |
| | 900 by 760 | 0.82 | 0.25 | 0.66 |
| | 1600 by 1000 | 0.73 | 0.25 | 0.66 |
| `thread-actions-priority` | 1280 by 760 | 0.68 | 0.08 | 0.61 |
| `thread-place` | 1280 by 760 | 0.67 | 0.24 | 0.68 |
| | 900 by 760 | 0.84 | 0.24 | 0.55 |
| | 1600 by 1000 | 0.73 | 0.25 | 0.66 |
| `thread-reply-model` | 1280 by 760 | 0.64 | 0.22 | the chip 0.79 |

The page under them is the waiting thread's, 0.68 with nothing open (`thread-options`); its
options and reply bar are 0.58. The screens before this did not move, to the thousandth: Home
0.844, the Inbox 0.240, a conversation 0.409 and as first opened 0.389, the Later picker 0.216,
the card 0.221, the command bar 0.146, the waiting thread 0.679. The reply box opened with R
(`thread-reply`) went from 0.435 to 0.432, its three chips' words now landing where Chromium lands
them.

Filmed at 60 frames a second, the menu, the four levels, the panel and the reply box's menu each
land whole on the frame after the press lets go or the key, close the same way, and every frame
after is the same to the pixel. Rune counts no frame of the menu's film as animating, and one each
where the reply box's menu opens and a place is picked: a colour landing at once (the chip's edge,
the picked place's fill), with no source named and nothing in motion. A press on the dots or the
place button acts on its way up, as the web window's click does.

Against a scratch engine (above), each pick written by this window beside the web window's own
payload for the same pick, sent by hand to a second thread, wrote the same lines, in order:

- Sonnet 5.5 and High picked in the reply box, then a reply sent: `{"model":"sonnet"}`,
  `{"effort":"high"}`, then the words. Picked and folded with nothing typed: no line at all.
- Codex's own picked on a Claude Code thread: `{"engine":"codex"}`, `{"model":""}`,
  `{"effort":""}`, then the words, the box having said "Codex starts fresh with this thread."
- High picked in the menu: `{"priority":7}`, and "Priority: high" in the strip.
- Later in the menu, then Enter: `{"snoozedUntil":…}`, thirty minutes on, as L does.
- Z inside the hold after a reply with a pick: no line, and the words and the pick back in the box.

The engine there has no cloud set up, so it draws no place button and the panel was not pressed
against it. Its three writes were sent by hand, as `place::writes` makes them: keeping the thread
here answered `{"ok":true}` and the supervisor kept it (`threadOnMac`), letting the cloud have it
answered the same and moved it to `threadToCloud`, and a move with no run to move answered
`{"moving":0}`, on which the panel says the cloud cannot take it. Picks in the panel were pressed
only on pictures, where nothing is sent.

## Starting a thread

There is one box to start a thread in, drawn in one of two places, as the web window draws its one
composer either in Home's slot or in a dialog (`ThreadComposer.tsx`, `drawnIn` in `App.tsx`):

- **On Home** it is open under the greeting (`src/compose.rs`, `inline`). N, a press on it, or a
  letter that is none of the list's own keys puts the caret in it.
- **Anywhere else** N, and New thread on the Inbox's strip, open it as the **New thread card**
  over the page: the page veiled and blurred a little, the card 680 points wide a tenth above the
  middle, its head saying who the thread is to (where someone besides the agent could get it) and
  on which model, then the words, then its bar. While the card is up Home's slot is empty, as in
  the web window. The empty Inbox has no box of its own here, so N opens the card there too.

Both hold the same words and picks: whatever was typed on Home is in the card, and the other way
round. Escape, or a press outside the card, closes it and keeps every word.

Under the words: the model (a chip on the page, the head's Model on the card), the project and the
level, the paper clip, and Send with the caret beside it. Each opens the web window's menu, built
from its measurements: the models used lately, each with its agent where there are two, then
every model in a column for each agent, and how hard it thinks at the foot; the projects in the
person's order with New project last; Agent Decides and the four levels; and under the caret, Add
it to Later. On the page every menu drops under its chip; on the card the model's drops from its
word and the others rise out of the bar. A menu is kept inside the window as the web window keeps
it (`fitMenu`), opens with the keys on what is picked, and is walked by the pointer and by ↓, ↑
and Enter around both ends. Escape closes it and leaves the keyboard on its chip, where an arrow
opens it again and a letter goes to the words. A press outside it closes it on the way down and
goes on to what it was aimed at in the box: another chip opens its menu, the words take the caret,
Send sends.

Send, or ⌘Return, writes the thread at once, one `zero:compose` as the web window's
(`api.compose`): the project, the title and body split out of the words by the web window's rule
(`splitMessage`), `kind: directive`, the level as a number (nought for Agent Decides), the Claude
model always (another agent's only when moved off its default), the agent where this Mac offers
more than one, the effort only when one was picked, and `start: later` when Later was. The box
empties at once and the card closes; a line says where it is going, then names the thread with O
to open it and Z to take it back, which closes it and stops its agent unless the agent has
already answered. Z hands the words back and opens the card on them. A send the engine refuses
puts every word and pick back and opens the card on them, unless something else is over the page
by then, when the line says where the words are. Alone on the Inbox the tab moves to where the
thread lands.

What was picked is kept: the project, and for each agent its model and effort, in memory and in
one small file beside the door's note (`~/Library/Application Support/Mercury/native-picks.json`,
`src/picks.rs`), written when a pick changes and read when the window opens. A fixture's picture
never reads or writes it (its picks are the fixture's `picks`, or none), nor does a window given a
door by hand (`--door`), which may be a scratch engine's, unless `--picks <file>` says where.

The scenes: `home-typed` (words in Home's box), `home-menu-project`, `home-menu-priority`,
`home-menu-model`, `home-menu-models` (every model), the same two with two agents
(`home-menu-model-engines`, `home-menu-models-engines`, whose settings the scene answers with
invented Codex models), `home-sending` (between Send and the answer), `home-sent` (the line that
names the new thread), and the card N opens on the Inbox: `compose-card`, with its menus
`compose-card-model` and `compose-card-priority`, and `compose-card-team` (To over Model). The
card scenes are drawn with `--view inbox`. Each has its regions (`tools/scenes/*.regions.json`).
At 1280 by 760, 2x:

| Scene | Whole window | Where it changed |
|---|---|---|
| `home-typed` | 0.90 | the box 2.48 (Home's own box is 2.15: see the glass below) |
| `home-menu-project` | 0.77 | the menu 0.27 |
| `home-menu-priority` | 0.76 | the menu 0.29 |
| `home-menu-model` | 0.72 | the menu 0.23 |
| `home-menu-models` | 0.70 | the menu 0.16 |
| `home-menu-model-engines` | 0.72 | the menu 0.29 |
| `home-menu-models-engines` | 0.63 | the menu 0.15 |
| `home-sending` | 0.85 | the line 0.86 |
| `home-sent` | 0.84 | the line 0.35 |
| `compose-card` | 0.22 | the card 0.29 |
| `compose-card-model` | 0.20 | the card 0.15, the menu 0.22 |
| `compose-card-priority` | 0.22 | the card 0.27, the menu 0.30 |
| `compose-card-team` | 0.20 | the card 0.16 |

At 900 by 760 the menus are 0.23 to 0.29 and the card 0.27 (whole windows 0.84 to 0.91 for Home
with a menu, 0.23 to 0.25 for the card); at 1600 by 1000 the menus are 0.23 to 0.29 and the card
0.29 (whole windows 0.56 to 0.61, and 0.15 to 0.16).

Filmed at 60 frames a second, a menu, the card and a send each land whole on the frame after the
press and stay still. Against a scratch engine (above), a thread typed on Home and sent wrote
its title, body, level and model to the ledger; one sent to Later wrote `start: later`, and Z
then wrote its close and opened the card on its words; one sent from the card on the Inbox, at
High, wrote level 7.

### Files in a new thread

Files go with a new thread as they go from the web window's composer (`ThreadComposer.tsx`,
`renderer/src/attachments.ts`, `components/AttachRow.tsx`). The rules are `src/rules/attach.rs`,
with the web's own test cases; the box's side of it is `src/attach.rs`.

- **What attaches**: files dropped anywhere on Home's box or on the New thread card; the file
  the paper clip's picker picks (the system's own, over the window: Rune's `native-file-dialog`,
  which builds `rfd`); and ⌘V in the words, where files copied in Finder attach as themselves
  and a picture on the clipboard, a screenshot, attaches as `pasted-<number>.png`, the number
  the clock's, as the web window names it. A Finder copy also carries the file's icon as a
  picture, which is not what was meant and is left. A folder goes as the files inside it, which
  the engine reads.
- **How they stand**: in a row between the words and the bar (`.attach-row`, twelve points under
  the words on Home, ten over the bar on the card). A picture is 84 points tall, as wide as its
  shape makes it up to 220 and cropped to fill, with a × in its corner; any other file is a chip
  with its name, its size and a ×. A press on the ×, or anywhere on a chip, takes the file out
  and lets go of the keyboard, as the web window's focus goes with the button it was on. A press
  on a picture opens it with the system. The box reads a picture's shape off its header as it
  comes in, so the row stands where it will from the first frame; the picture itself is read and
  shrunk to its place in device pixels on a worker, a still picture on the spot.
- **What cannot go**: a file over 1 GB, or one that cannot be read, lights its chip with no size
  in it, a line under the row says why in the web window's words ("launch-film.mov is 1.2 GB,
  over the 1 GB a message can send."), and Send stays off. A TIFF or a HEIC, a photo from a
  phone, is handed to the engine to be turned into a PNG or a JPEG as it comes in
  (`zero:stage-attachment`, which uses the Mac's `sips`), as in the web window, and goes as that
  picture; until the engine answers it is a chip with its name, and Send waits for it. With no
  engine to ask, or one that cannot turn it, it goes as the file it is.
- **What Send writes**: first `zero:save-attachments`, with the project and each file: a file on
  this Mac by its path (`srcPath`), which the engine copies into the project's `attachments/`
  folder (a clone on APFS, so no second copy of the disk), a pasted picture by its bytes in
  base64, read and encoded on the worker. Then `zero:compose` as before, its body the words with
  the engine's links under them after a blank line, one to a line: `![name](attachments/…)` for
  a picture, `[name](attachments/…)` for anything else (`attachmentLinks`). With files and no
  words the thread is called after the first of them, "Attached: shot.png" (`threadMessage`).
  Either write refused puts the words and the files back and opens the card on them, with the
  engine's own reason; Z takes the thread back as before, and hands back its files with its words.
- **Home makes room**: Home's column is at least 660 points and gives way when the box grows, as
  `home.css` lays it out: the room above the greeting first, a thousand times more readily than
  the cards, which give down to 240 points (CSS flexbox's shrinking, ported in `home.rs`). So the
  greeting rises and the cards shorten as in the web window. The box's own share, a sliver of a
  point, is taken from its slot and not from the box.
- **A conversation draws them**: a person's message is drawn as typed, but the lines the app wrote
  into it for its files are lifted out and drawn as the picture or the link they are, in the
  person's own type, heavier and on shorter lines than an agent's (`renderer/src/typed-message.ts`,
  ported as `src/rules/typed.rs` with its tests). So a thread sent with files shows them when it
  is opened, and so does a reply that carried them.

The scenes: `home-attached` (words, and a PDF picked through the paper clip: the scene sets it on
the page's own file input, `DOM.setFileInputFiles`), `home-attached-picture` (a picture pasted
with no words, a clipboard event carrying `image.png`), `home-attached-too-big` (a film of 1.2 GB,
invented and empty, of which the disk keeps nothing), `compose-card-attached` (the card on the
Inbox with a picture and a note) and `thread-reply-attached` (a whole conversation whose reply
carries a picture and a file). A fixture says what the box held: `attached` (files by their
path, attached as a drop attaches them) and `pasted` (each picture's PNG and the name the page
gave it). Each scene plants the invented folder, so photograph one, draw it, then the next: one
planted later than the film's scene has no film in it, and the box then says the file was not
found.

At 1280 by 760, 2x: the whole window; the box on Home or the card; the attachment row
(`.attach-row`, which on Home is mostly the box's glass, two of 255 apart everywhere, see the
glass below); and the files alone.

| Scene | Whole window | Box or card | Attachment row | The files |
|---|---|---|---|---|
| `home-attached` | 0.88 | 2.42 | 1.84 | 0.30 |
| `home-attached-picture` | 0.84 | 1.99 | 1.57 | 0.44 |
| `home-attached-too-big` | 0.93 | 2.62 | 1.88 | 1.25, and 2.35 the line saying why |
| `compose-card-attached` | 0.23 | 0.25 | 0.12 | 0.38 |

At 900 by 760 the three on Home are 0.93 to 1.05 whole (Home with words and no file is 1.08
there), their rows 1.25 to 1.44 and their files 0.29 to 0.44, the lit chip 1.23; at 1600 by
1000, 0.72 to 0.76 whole (0.71 with no file), rows 1.66 to 1.73, files 0.30 to 0.45, the lit
chip 1.25. The card is 0.26 and 0.17 whole, its row 0.12 and its files 0.38 at both. The web window's caret was between blinks when
`home-attached-picture` was photographed, and this window draws its caret.

`thread-reply-attached` is 0.83 whole. Its reply's parts stand where the web window's do to a
hundredth of a point, measured from the reply's head; the whole conversation sits half a point
lower, because it lands at its foot and Chromium's scroll height is a whole number of points
where Rune's is not. With that half point taken back, the reply is 0.42, its picture and name
0.43, and the file's link 6.3, which is a mention's underline (see what still differs).

Filmed at 60 frames a second, a file dropped, a picture pasted and a file taken out (on the
release of the press) each land whole on the next frame, the column and the box's row with
them, and every frame after is the same; Rune counts no frame as animating. A large picture's
copy can land after its place: a full screenshot takes about two seconds to read in a debug
build, and its place stands empty in its edge until then, as an image not yet decoded does in
the web window.

Against a scratch engine: a thread typed on Home, with a PDF dropped and a picture pasted, put in
Later and sent, wrote "Check the receipt layout against these" with the body
`[week-32.pdf](attachments/…-week-32.pdf)` and `![pasted-89459.png](attachments/…-pasted-89459.png)`
under it, and both files were in the project's `attachments/` folder, the same bytes as dropped
and pasted. Opened, it drew the PDF as a mention and the picture. A file dropped and taken off
the disk before Send was refused in the engine's words (`Not sent: ENOENT: no such file or
directory, stat '…'`), and the card opened on the words and the file. A picture sent alone, to
Later, as "Attached: pasted-1708.png", was taken back with Z: the ledger has its close, and the
card opened on the picture. A TIFF dropped on the box came in as a PNG and went as
`![receipt-scan.png](attachments/…)`. A script that types words lets each letter land 35 ms
after the last (`"every"`), so a press scripted while it is still typing opens a menu that then
takes the letters after it, as a menu should.

## The command bar

⌘K opens it over whatever is on screen, and ⌘K again closes it; Control K does the same, as in
the web window (`src/palette.rs`, from `PaletteBox` in `renderer/src/components/Palette.tsx`).
It opens empty, the caret in its box and its first row lit, its middle three eighths of the way
down the window as the web window's is. The arrows move the lit row and bring it into view
inside the list, which scrolls past 320 points; the pointer lights the row it moves over, never
one the list moved under a pointer standing still (`pointerMoved`). Enter runs the lit row and a
press the row pressed, the bar closing first; Escape, the Esc button or a press outside closes it
and runs nothing. It stands over the page alone: opening it closes the Later picker and folds the
reply box, keeping its words.

A row is there only when this window does what it says, through the code its own key or button
already runs, under the web row's own name, mark and key:

- On the thread under the keys (the open conversation, else the Inbox's picked row; Home has
  none), in the web's order: Mark done (E), or Reopen on a finished thread; Reply (R), which
  opens the thread with the caret in its reply box; Back to Inbox (E) on a thread put off, "Let
  it run now" when its agent set the moment; Later (L), which opens the picker; and Open (↵)
  from the Inbox.
- Then Go to Home, Go to Inbox and each of the Inbox's other tabs.

Nothing stands for Z: the web window's bar has no Undo row either.

Words typed keep the commands every word of which lands in the name or its keywords, ranked as
the web ranks them ("rem" is Later), and find threads by their words (`src/rules/search.rs`, from
`renderer/src/search.ts`): every word at the start of a word, the typed phrase ahead of the same
words scattered, a title ahead of prose, finished threads too. The best five are listed under
Threads, each its title over the sentence it was found in with the words in ink, its project and
age beside them, dimmed once finished. A command named by the words goes above them, one the
words only touch by a keyword after, under Commands (`paletteOrder`). With nothing found, one
line says what was searched for. The rules are `src/rules/palette.rs`, tested on the web's own
cases.

New thread is drawn first, with N beside it (the web has it after New project, which this bar
does not offer): it opens the New thread card, Home included, with the caret in its words
(`starts_a_thread` in `src/palette.rs`). The bar and the card stand over the page one at a
time: ⌘K over the card closes the card, keeping its words.

## The files a message names

The web window does not leave a file an agent names as text, and neither does this one. A
message is read in the web window's three steps, each a rule ported from its TypeScript
(`src/rules/paths.rs`, `mentions.rs`, `roots.rs`; `src/markdown.rs` runs them):

1. Every path typed into a sentence becomes a link or a picture: a short path, a quoted name,
   a full path into the project's folder or the thread's own checkout, a folder on a line of its
   own, `localhost:3000` (`renderer/src/artifact-paths.ts`).
2. What draws moves under the sentence that names it, which keeps its name: one picture large,
   several in a row 240 points tall (`renderer/src/media-row.ts`).
3. Each link is drawn as the web window's mention, the icon of its kind and the name it goes
   by (`renderer/src/mentions.ts`), and a code span that is a source file or a path is one too.

A picture is looked for where the web window looks (`pictureRoots`): the folders the message
names, the thread's designs folder, the worker's checkout, the project's folder with its designs
and attachments, the code folder. It is drawn at the web window's size: its own up to 760 points
wide and the window's height less 320, with its name under it. One that is nowhere says
`missing image:` and its path. A folder on a line of its own is its name, how many pictures it
holds, its six newest three across and a line for the rest, or only its name in a conversation
that draws pictures of its own.

Pressing a picture, a folder's picture or a mention of a file the big view draws opens it big
over the window ("A file opened big", below); with ⌘ held, as a browser opens a link in a new
tab, it opens with the system instead (`asksForANewTab`), and so does any other file and a
folder's name (`open`, through `files::open_with_system`). A file macOS would run is shown in
Finder instead, as the web app does (`main/opens-a-program.mts`).

### How a picture is read

Off the disk, by this window, never through the engine's door and never on the frame
(`src/pictures.rs`). The page asks for a picture by the places it may be in and the size it will
be drawn at, and draws a place for it. A worker thread finds the file, decodes it, shrinks it to
that size in device pixels and wakes the window, which draws the page again with the picture in
it. A still picture (`--snap`) cannot wait, so there the asking does the reading.

- **A copy is the size it is drawn at.** A 2880 by 1800 screenshot is 20 MB decoded and 4.7 MB as
  the copy a 1280 by 760 window draws (1402 by 876 pixels on a 2x screen). The web window keeps
  its copies the column's width whatever height they are drawn at.
- **All copies together stay under 64 MiB** (`BUDGET`): eleven screenshots or more at the
  column's width, which is more than a long answer holds. What has been asked for and not read
  yet counts too, so a page of thirty screenshots reads the first eleven and then only what
  comes on screen. Past the budget, the copies that left the screen longest ago are let go and
  read again when they come back; what is on screen is always held. A still picture reads
  everything its one frame asks for.
- **Leaving a conversation lets go of every copy.** Each picture's size is remembered (of the
  last few thousand), so a conversation opened again holds every picture's exact place from the
  first frame. A copy made for a smaller window is made again when the picture is drawn a tenth
  larger; one made for a larger window is kept until the conversation closes.
- **A file that changes on disk is read again**, and a missing one is looked for again: the
  worker looks at each file's size and date every two seconds while the conversation is open.

Before a picture has ever been read its size is not known, and the page holds what the web
window holds: four to three across the column, no taller than the window allows. When the
picture lands, the words under it move once to their place, as they do in the web window.
Filmed frame by frame (`--script` with a key that opens the thread, `--strip`), a conversation
with three small pictures draws that held place for one frame and is whole and still from the
second; one that opens at its foot keeps its last words where they are throughout. A folder
draws three held places until it has been listed, then its pictures as each is read.

Measured, each a headless still at 1280 by 760, 2x, in the debug build
(`/usr/bin/time -l ./target/debug/mercury --fixture shots/<scene>-fixture.json --thread --snap …`).
Photograph `thread-pictures-many` last before measuring: any other scene that plants the folder
starts it again without the ten screenshots, and the measurement then reads none.

| Conversation | Most memory resident | Peak footprint |
|---|---|---|
| `thread`, no pictures | 135 MB | 312 MB |
| `thread-pictures`, three small ones | 135 MB | 313 MB |
| `thread-pictures-many`, ten 2880 by 1800 screenshots | 238 MB | 423 MB |

So ten screenshots cost about 100 MB at their worst moment: 47 MB of copies, what the ones on
screen take again in Rune's own atlas of what it draws, and the one screenshot being decoded and
shrunk, which is let go before the next is read. The 135 MB under them is the headless run
itself. A still reads all ten at once; a window was not measured.
Reading one such screenshot takes 49 ms in the release build and 2.2 s in the debug build
(`cargo test --release -- --ignored --nocapture a_screenshot_is_read_in`), so a debug window
fills in a page of screenshots slowly. `MERCURY_PICTURES=1` prints what is held each time it
changes.

### The files a thread made

A thread whose agent made or named files carries the Files button in its head strip, first of
what can be done to it: Done's frame around a mark of two pages and how many files there are
(`ThreadFilesButton` in `renderer/src/threads/ThreadFiles.tsx`, `src/thread.rs` here). Which
files is the web window's rule (`filesMadeHere`), ported with the web's test cases in
`src/rules/thread_files.rs`, read off the thread's ledger as the engine hands it over
(`zero:item-history`):

- Every file a line names that the big view reviews (a picture, a film, a PDF, a note, a deck),
  as a link, in a code span or bare in a sentence; and every sheet (`.csv`, `.tsv`) named as a
  link or in a code span, or by an agent with a folder in its path or alone on a line. A
  person's own words count only what they attached, a link; a fenced block of code names
  nothing; a page or a script named in passing is not something the thread made.
- Each file once, at its newest version: `plan.md` and `plan-v2.md` are one file, a bare name
  and the whole path to it are one, the same name in another folder is its next version, and an
  edit saved in the app is the version it saved. Names that number themselves say the order.
- Newest first: when a version of it was last named.

Until the engine has answered with the ledger the strip has no button, as the web window's has
none until its answer comes; the ledger is read again only when it or the notes change.

A press on the button opens the list under it, 360 points wide, its right edge on the button's:
each file its kind in a small tile (four letters of its extension at most), its name cut short
with dots, and how many notes are on any of its versions (`item.artifacts.notes`). Past 440
points, or seven tenths of the window, the files scroll inside it. The row under the pointer is
washed, at once both ways, as the web window's has no transition. A press on a file closes the
list and opens the file big, as a mention opens it (`files::open`); a file the big view does not
draw opens with the system, and one found nowhere opens nothing. Escape closes the list, and so
does a press anywhere outside it, on its way down; a
press on the button closes it on its way up, as the web window's click does. Every other key goes
on to the page, as in the web window, whose list takes only Escape: J and K go on to another
thread, which closes the list as leaving the thread does, and coming back finds it closed.

The thread's name in the strip gives way to what stands beside it, cut short with dots, as the
web window cuts it. It used to run on over the arrows wherever it did not fit, which the button
made happen in more windows: a conversation without files at 900 by 760 is 0.54 now (`thread`;
0.56 before), `thread-folded` 0.51 (0.53) and `thread-later` 0.28 (0.28).

The scenes: `thread-files` (the strip: a picture and a note each named in two versions, two
sheets and a report, five files), `thread-files-open` (its list open) and `thread-files-many`
(fourteen files, two of them with notes and one with a long name, the list open and full).
Every box of the button and the list lands where the browser puts it, to a hundredth of a point
(`--tree` against the dump). What is left is glyphs: the strip puts the button on a fraction of
a point, and Rune lands a glyph's pen on half a device pixel rounded down where Chromium rounds to
the nearest quarter, so the count and the names sit a quarter of a pixel to the left.

At 2x, dark; the strip is the card's head, the button and the list the page's own boxes:

| Scene | Window | Whole window | The strip | The button | The list |
|---|---|---|---|---|---|
| `thread-files` | 1280 by 760 | 0.44 | 0.55 | 0.54 | |
| | 900 by 760 | 0.76 | 0.68 | 0.41 | |
| | 1600 by 1000 | 0.40 | 0.47 | 0.47 | |
| `thread-files-open` | 1280 by 760 | 0.44 | 0.61 | 0.67 | 0.49 |
| | 900 by 760 | 0.72 | 0.75 | 0.60 | 0.49 |
| | 1600 by 1000 | 0.42 | 0.51 | 0.59 | 0.49 |
| `thread-files-many` | 1280 by 760 | 0.75 | 0.57 | 0.76 | 0.61 |

A strip with no files is 0.55 at 1280 by 760 (`thread`). Most of what `thread-files-many` differs
by is its answer's fourteen mentions, whose underlines sit two points higher (see what still
differs).

Filmed at 60 frames a second, the list opens on the frame the press lets go, closes on the frame
of Escape or of a press outside it (on the press's way up for the button itself), and every
frame after is the same to the pixel; Rune counts no frame as animating. A press on a file in a
script says what a window would have opened (`a press opens …/notes/rounding-v2.md`).

## A file opened big

A file opens over the whole window, as the web window's big view does (`ArtifactView.tsx`, its
kinds in `ArtifactStage.tsx`; `src/viewer.rs` here, its rules `src/rules/viewer.rs`): the paper
the file lies on, the bar over it and, from Files, the line of places it was sent in under it.

**What opens it.** In a conversation: a picture, a folder's picture, a mention of a file of a
kind the view draws (a picture, a film, a PDF, a deck, a page, a document, a table, a script or
any text), and a row of the Files button's list. In the Files place: a tile. The web window
opens what is looked at (a picture, a film, a PDF, a page, a deck) over the window and what is
read (a document, a table, a script) in a pane beside the conversation, which this window does
not have, so both open big here (`whereAFileOpens`, ported with its cases). With ⌘ held a press
opens the file with the system, as there; an address goes to the browser; any other file opens
with the system as before. A picture in the box a thread is started in still opens with the
system (the web window has a viewer of its own for those).

**What it holds.** The files the lines it was opened on name (`filesInLines`, the pages, tables
and scripts beside them only when one of them is the file opened), each file's versions in the
order the lines name them or their names number them (`threadFiles`), and the file pressed,
even when no line names it (`withFilesAsked`). From Files the lines are the file's own, one to a
version at the path it opens from (`zero:files-place-open`). A short path is found where a
picture in the conversation is looked for (`pictureRoots`).

**The bar** (`.av-top`, 44 points, its band moves the window): back; the file's name, a file with
versions by its first version's name without the version (`familyName`); what the thread called
it, the words a message put beside it (`labelSaidFor`, ported with the web's test cases: "Before",
"Option 2: names beside faces"); with versions, ‹ v2 of 3 › and, on an older one, Show v3; then
the mark that opens it outside, and the close mark. The web window's comments, Approve and Share
are not drawn (see what still differs), nor the mark that puts the file beside the conversation.

**The arrows** at the stage's sides step to the file before and after: through the pictures of
the part of the thread the pressed picture was in (`picturesFor`, here `turn_pictures`: one of
the person's turns and the agent's answers to it), each picture once at its newest version
(`stepsThrough`); opened any other way, through the agent's latest pictures when the file is one
(`latestPicturesSaid`), else every file in turn (`nextStep`). Versions are the bar's to step,
never the arrows'. One with nowhere to go is not drawn.

**What each kind draws:**

- A picture: fitted to the stage with 52 points either side and 24 over and under, never larger
  than its own pixels, on rounded corners over a hairline and a soft shadow; a tall one fitted to
  its width and read by scrolling, unless it is a phone screen that reads whole (`pictureFit`).
  The pill under it zooms out and in through the web window's stops (1, 1.5, 2, 3, 4, 6, 8) and
  says the size it is drawn at, a press on that fitting it again; + and − do the same, 0 fits it,
  1 shows its own pixels, and a pinch or ⌘ with the wheel zooms freely, keeping the middle of
  what is in view in the middle. Zoomed in, the copy is read again at the size it is drawn, up to
  the picture's own, and the stage scrolls both ways. ↑ and ↓ move it 80 points, the page keys
  nine tenths of the stage.
- A document (`DocStage`): a page to read, as `.doc-prose` lays it out (`markdown::blocks_doc`):
  16 points on lines of 1.72, headings at weight 620, lists, quotes, code and rules each with the
  margins of its own ems, links as underlined words (an address to the browser, a file beside the
  document to the app that owns it), and a picture it names beside itself drawn from its folder.
  Over it who shared it and when ("Shared by your agent · today at 1:19pm", `sharedByWords`). Its
  sheet is the text's 820 points between margins (`.doc-sheet`), and with three headings or more
  its contents stand in the right margin, or as a column of dashes where the margin is narrow
  (`DocOutline`); a press on one scrolls to its heading.
- A script or any text: its lines numbered and coloured as an editor colours them
  (`tokenize` in `code-artifact.ts`, ported as `src/rules/code.rs` with the web's test cases) on
  its sheet, at most 980 points and as wide as its widest line, the stage scrolling both ways.
  A table (`.csv`, `.tsv`) is drawn the same way, its lines uncoloured.
- A page, a film, a PDF or a deck: what the web window draws it in is a frame or a player this
  window does not have, so the stage says what the file is and offers Open outside, which opens
  it in the browser or the app that owns it, as the bar's mark does.

**The keys** (`rest` in `ArtifactView.tsx`): every key is the view's and none reaches the page
under it: Escape goes back to where it was opened from, ← and → step the files, and a picture's
keys are above. A chord is the system's, and ⌘K still opens the command bar over it.

The scenes: `thread-file-picture` (a picture of a conversation pressed: the newest of its two
versions), `thread-file-older` (the same, its older version shown), `thread-file-steps` (the
middle of three pictures in one sentence, the arrows at the sides), and from the Files place
`file-picture`, `file-doc`, `file-code`, `file-table`, `file-page` and `file-pdf`. A scene of a
conversation answers the page's finder and draws its big pictures from the planted folder, and
hands the native window the path pressed and its part's pictures (`pictureOpen`); one of Files
hands it the list and each file's lines (`filesPlace`, `filesPlaceViews`) and the tile pressed
(`fileOpen`).

At 1280 by 760, 2x. The web window draws its notes down the right, 340 points, which this one
leaves out, so its stage is that much narrower and the file stands that much further left: a
picture in the middle of the stage 170 points, a document's column 140, a script's sheet 98. Each
part is measured against its own place in the web window's picture:

| Scene | The bar | The file | Its pill or line under |
|---|---|---|---|
| `thread-file-picture` | 0.45 to the version line, 1.11 with what the thread called it; the close mark 0.01 | the picture 0.28 | the pill 0.47 |
| `thread-file-older` | 0.49 | 0.27 | 0.47 |
| `thread-file-steps` | 0.45 | the arrows 0.37 and 0.51 | |
| `file-picture` | 0.17 | 0.28 | the pill 0.47, Sent in 0.50 |
| `file-doc` | 0.16 | the byline 0.49, the title 1.50, the text 0.76 | 0.38 |
| `file-code` | 0.12 | the lines 0.19, their numbers 2.16 | 0.38 |
| `file-table`, `file-pdf` | 0.13, 0.10 | (see below) | 0.38, 0.30 |

Whole windows differ by what the notes and the narrower stage move: 2.3 for a picture, 3.1 for a
script, 6.4 for a document. A picture 800 points square opens fitted to its width here and
scrolls, where the web window's narrower stage fits it whole at 84% (`thread-file-steps`, whole
35.6): the rule is the web window's, on a wider stage. A page is the web window's white frame
with the page in it, and a card here (`file-page`, 107.6 whole).

Tests hold the rest on invented files (`src/feel.rs`): a picture pressed in a conversation opens
big, takes every key (E closes nothing under it, + and 0 zoom), and Escape and the close mark go
back to the conversation; a mention of a document opens it big, and with ⌘ held does not; the
rail's Files opens the Files place, a tile opens big from the list and fixture of a Files place,
the place under it opens its thread, and Escape walks back from the thread to Files and from Files
to where it was opened from. No test, and no headless run, hands a file to the system.

## The Files place

The rail's Files, under the Inbox, opens the Files place (`FilesPlace.tsx`, its look
`files-place.css`, its rules `files-shown.ts`; here `src/files_page.rs` and
`src/rules/files_shown.rs`, ported with the web's test cases): every file the threads made, one
tile each however many threads it went to, from the list the app answers (`zero:files-place`,
read when the page opens and again as the store changes, at most every two seconds).

- **The strip**: back and forward, and Files over the Inbox's sky, as the web window draws it.
- **The bar**: the way back (All files, then the project and its folders, each a press away) and
  the box that searches (`/` puts the caret in it; Escape takes back what is typed, then the
  keys). Every word must start a word of a file's name, its project or its folder, and the files
  named for a word come before, and instead of, those that only sit in a place of that word.
- **The tools**: All and each kind the files here are (Documents, Pictures, Films, Pages, Tables,
  Code…), Needs review while a file has comments nobody approved, what the files take on this
  Mac, Recent and Name (Name reads figures as numbers and capitals and accents as their letters),
  and Grid or List.
- **The cards**: on All files a card for each project, with how many files it holds, which opens
  its page; on a project's page, its folders.
- **The tiles**: as many columns of at least 232 points as fit, 22 apart; each tile's face sixteen
  by ten, its edge brighter under the pointer, then the name a person reads (a name nobody chose,
  a page shot's, is its thread's title) and its line: the version, who made it, how long ago and
  its size. A picture's face is its top, filling the tile, or the whole of it in the middle where
  filling would cut away more than four tenths (`coverFit`); a document's its title, headings and
  first lines fading out (`docBlocks`); a script's or a table's its first twelve lines; a film's
  its play mark; a page's, a PDF's and a deck's their kind's word. Sixty at once, the next sixty
  as the foot comes near (`TILES_AT_ONCE`). As a list, each file one line with a small
  thumbnail. A press opens the file big ("A file opened big"), with the places it was sent in
  under it, each of which opens its thread; Back from there comes back to Files.
- **The keys**: `/` and Escape, which leaves the place for where it was opened from; N still
  starts a thread.

The scenes: `files` (All files), `files-project` (a project's page) and `files-list` (the list),
over an invented list built on the planted folder; `MERCURY_FILES=<a saved list>` photographs the
web window on another list. At 1280 by 760, 2x, whole windows and each part:

| Scene | Whole | The strip | The bar | The tools | The cards | The tiles |
|---|---|---|---|---|---|---|
| `files` | 0.56 | 0.12 | 0.47 | 1.55 | 0.28 | 0.56 |
| `files-list` | 0.63 | 0.12 | 0.47 | 1.55 | 0.28 | 0.70 |
| `files-project` | 3.13 | | 0.60 | 1.56 | | 1.17, and 51.9 the page's tile |

At 900 by 760 the two are 0.99 and 1.10 whole; at 1600 by 1000 the list is 0.45 and the grid 6.49,
which is the page's tile in view: the web window draws the page in it, white, and this window its
word. Every part stands where the browser puts it; what is left is glyphs, and the rail, whose
other places this window does not draw.

On the person's own files, read through the running app's door and never written, the web
window photographed on the same list (`MERCURY_FILES`) and this window drawn on the door
(`--view files`): the same three project cards, the same kinds, and the first sixty tiles in the
same order with the same name and the same line, the web window's harness having no team to tell
"Your agent" from another's.

## What a run did

Each run that picks a thread up leaves a log (`zero:session-trace`; `docs/thread-data.md` says
what is in one and how it is asked for). The page reads them as the web window does
(`itemThread` in `renderer/src/item-thread.ts`, ported in `src/talk.rs`):

- Each run hangs under the pickup that started it: the latest run that had already begun when the
  claim was written, since a worker claims its thread as its first action, sometimes an hour
  later when the Mac slept first. Its sentences become the agent's messages, each at the moment it
  was said, and the tools it ran the quiet lines between them, in plain words where a command can
  be read ("ran the tests", "read …/src/checkout/tax.ts", "pushed the branch main") and as the
  command otherwise (`src/rules/trace.rs`, `src/rules/work.rs`). The store's own bookkeeping is
  left out, and a run that stopped on the tool's own refusal says so in the app's words ("This
  run stopped because the account hit its usage limit.").
- A run's closing message that types again the answer it wrote to the thread is drawn once, as
  the thread's; a run still talking after its own line above wears no second name.
- Each stretch of what a run ran is folded to one line ("Changed files, ran the tests, saved the
  changes"), with how many of them failed in the stopped colour. A press opens it at once onto
  each thing it ran, with a mark where there is more behind it and how many lines came back.
- A long conversation keeps its opening and its last forty messages, with a line for the middle
  that asks for it whole, as the web window does.
- While an agent is on the thread, the work since anybody last said anything leaves the
  conversation, and the line at its foot says which step is running in the agent's own words,
  how many more run beside it, how long it has worked, and where it runs when that is news. The
  sentence the agent is typing this second (`saying`) is placed where it began, once.
- The head strip names the tab that holds the thread now (In progress while an agent is on it),
  and draws the arrows to the threads before and after only for one in Needs you.

`src/rules/notes.rs` is `renderer/src/notes.ts` whole, with its tests: the diary of a run's
sentences and its one-line summaries, which no screen of the web window draws today either.

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
- **Under the pointer, where it still differs** (see "How it answers the pointer and the hands"):
  - The I-beam shows over the whole box of a run of words, past the last word of a short line too,
    where the web window's shows it over the words' lines only: Rune finds selectable words by
    their box. Over a title or a heading shorter than the column, the same.
  - A mention's mark takes the I-beam, not the pointing hand, and stays muted when the words
    brighten (`a.mention:hover .mention-icon`): Rune's link covers words and not a mark set among
    them. And a mention's look lands at once, where the web window's eases in over 0.12 s and out
    over 0.2 s (`components/mention.css`).
  - Search's line in the open rail brightens at once under the pointer, where the web window's
    eases over 0.14 s.
  - The open rail's update card, which the web window draws while a new version waits
    (`home-update-rail-open`, the rail 1.97 of 255), is its Restart row here, and the usage row
    opens no panel of its figures under the pointer.
  - A table in a message does not sort: its headers are words, where the web window's take the
    pointing hand. The Inbox's head has no box that ticks every row.
  - With the list of a thread's files or a box's menu open, the page under it shows the arrow: the
    layer that catches a press outside them covers the page.
  - A key hint is not taken away when the page scrolls under the pointer, as the web window's is.
  - A right press's menu opens as the button comes up, where the web window's, as any Mac app's,
    opens as it goes down, so pressing, moving onto an item and letting go picks nothing here.
    Words selected before a right press on a link are not offered as Copy above the link's items,
    as Rune does not tell a handler what is selected. A box's menu is Rune's: Cut, Copy and Paste,
    the first two greyed with nothing selected, and no Select All (⌘A still selects all). A film
    alone on its line is its name here, a link, so it offers Open Link and Copy Link Address
    where the web window's player name offers only its file. Copy File takes the projects' own
    folders and code folders as the app's; the web window takes the account's folder, which holds
    the projects', and the code folders.
  - Tab inside a conversation walks Rune's focus where the web window's walks the browser's, both
    drawing nothing.
- **A file named in a message, where it still differs:**
  - A film or a sound is its name, which opens it. The web window plays it in place.
  - A document, a table or a script opens big over the window. The web window opens it in a pane
    beside the conversation, and its rules open a design page or a note on a line of its own
    there by themselves (`message-artifacts.ts`), neither of which this window has. The
    reference page has no engine to find a file with, so that pane was never photographed
    (`thread-links-beside` draws the same in both).
  - A press outside that list only closes it, where the web window's also lands on what it was
    aimed at (another button of the strip, a mention). And the wheel over the page moves nothing
    while the list is open, where the web window's page scrolls under it: the layer that catches
    the press stands over the page.
  - Escape closes whatever stands over the list first (the command bar, the Later picker, the
    New thread card), where the web window's list takes the Escape before them, its listener
    running before every other. Escape in the reply box, opened with R while the list is open,
    folds the box before it closes the list.
  - A mention's underline sits where the font puts it, two points higher than the web window's
    (`text-underline-offset: 4px`), and runs under the space before a line break. Rune's span
    has an underline and no offset. The same was already true of a link.
  - A picture not found under any root is missing. The web window then asks the app, which
    also hunts the project's folders for a file of that name.
  - A picture on the web (`https://…`) or pasted as `data:` is a mention, not drawn: this window
    reads pictures from the disk only. An `.svg` and an `.avif` are not decoded and say missing;
    an animated picture shows its first frame.
  - The place held for a picture on its way, and a missing picture's mark, do not change under
    the pointer, and the place does not shimmer.
  - A conversation opened unfolded lands on its foot. The web window lands on the top of the
    last answer when that answer is taller than the window, which pictures make it.
  - What only a conversation between people has is not drawn: tiles under a message
    (`MessageFiles`), how a file reached a teammate, a voice note. Nor a thread or a connection
    named as a link, which stays a link.
  - With more pictures than the budget holds, one less than half on screen may hold its place
    until more of it shows: Rune reports an element on screen from half of it.
- **A file opened big, where it still differs:**
  - No comments down the right, no box to write one, no Approve and no Share. On the person's own
    Mercury no file has ever had a comment, a reply or an approval (none on any of the files in
    Files, read through the door), so the stage takes the whole width, and everything on it
    stands further right than in the web window (see the table above). A picture is therefore
    fitted to a wider stage, and one that a narrower stage would fit whole can open fitted to its
    width (the web's own rule, `pictureFit`).
  - Versions are counted from the lines the view was opened on. The web window also reads the
    file's line from Files (`zero:file-line`), which numbers a version made in another thread;
    a file sent to more than one thread can count one version more there.
  - A page, a film, a PDF and a deck are a card with Open outside: the web window draws the page
    in a frame, plays the film with its marks, and draws the PDF's pages and the deck's slides.
  - A table is its lines, as a script's are; the web window draws it as a table to sort, search
    and edit. A line longer than four thousand characters is cut there, and only the first 4 MB
    of a file is read.
  - The pointer does not pan a zoomed picture by dragging it (the wheel and the trackpad scroll
    it), and a picture shows the arrow where the web window's crosshair offers a pin.
  - The document's outline does not light the section being read, and its column of dashes
    does not open into words under the pointer.
  - A document cannot be edited and a version cannot be restored (the web window's editor and
    Restore this version), and a file is not renamed by pressing its name.
  - A document's inline code and table take the message's look, a little smaller than the web
    window's reader draws them.
- **The Files place, where it still differs:**
  - Nothing is written from it: no New (a document, a table, a folder, files from the computer),
    no menu on a tile (rename, move to a folder, a new folder, Move to the Trash), no dragging a
    tile onto a folder, no choosing files for the Trash, and the total is its words, without the
    menu that frees the space or keeps files in the cloud.
  - A page's tile is its kind's word: the web window draws the page itself, small. A PDF's, a
    deck's and a film's tiles have no first page, slide or frame, and a film's no length.
  - The order does not hold still while the pointer moves to a tile and a file arrives
    (`HOLD_STILL_MS`), and the fold of the check pictures agents took marks itself with a
    character rather than the web window's chevron.
  - The search box is drawn with Rune's caret.
- **The team's card has no rows.** With teammates, Home's second card is the team's week; it says
  so when nobody else is on the team and is bare otherwise, until the Team page is drawn.
- **Choosing in a conversation, where it still differs.** Ported is what is used day to day: the
  model and level of the next reply, Later, and where a thread runs. Left out is what this window
  cannot do and what goes unused, a thread's name, project, pin, waits, demand and shipped mark,
  and what only teammates or approvals need:
  - Of the reply box's chips only the model is chosen. The level (`PriorityPicker`, its own list
    and not the box's menu) rides on the next reply in the web window and goes unused, so it is
    drawn, and the thread's menu changes the level instead, at once. The second chip is
    not who takes the thread but what this message may do (`ModePicker`, Claude Code's
    permission modes, "Bypass Permissions" on a project run with full access): it now says the
    thread's own mode in the web's words, and is not chosen. Nothing can be attached to a reply
    and nothing sent later (the caret beside Send).
  - The thread's menu leaves out every row this window cannot do, which are also the ones never
    used: Open terminal (⌘J; this window has no terminal), Rename, Move to another project, Pin
    thread, Share…, Add people and Ask or hand off… (teammates), Waits on… and Blocked by…, Mark as
    shipped, Stop repeating, Close task, Run now, View code changes and Demand. On a finished
    thread, where the web's menu holds only the first two, the dots say where they are.
  - The model picked in the reply box lasts as long as the window. The web window keeps it in
    its storage beside the draft (`saveDraftPick`), so it survives a restart.
  - Codex's own model is held as no model, and the web window holds it as an empty one: the reply
    sends `null` where the web's sends `""`, and the store writes both as `""`.
  - One row of a menu is lit, by the arrows or the pointer. The web window can light two, the
    row under the pointer and the one with the focus.
  - The place button pressed while the thread's menu is open closes the menu. The web window's
    leaves both open: its menu holds every button of the strip inside it.
  - A press outside the menu or the panel closes it and lands nowhere else, but on the strip's own
    buttons. The web window's press also lands on what it was aimed at (an option, a row).
  - The panel's two places do not brighten under the pointer. Its part for a thread the cloud
    handed back (Needs) is drawn from the stylesheet: no scene hands a thread back, so it was
    never photographed.
  - The panel was pressed only on pictures: the scratch engine has no cloud and draws no place
    button. What a pick writes was sent to it by hand (above).
  - The command bar still offers none of these: its rows for a thread's model, its priority and
    where it runs stay the web window's.
- **Starting a thread, where it still differs:**
  - Out of reach here, and drawn where the web window draws them: To (it says Agent and opens no
    menu), @-names in the words, recording, a demand, the branch a thread starts from, who sees
    it, and the tutorial's suggestions.
  - The caret's menu offers Add it to Later and nothing with a clock in it: no Tomorrow morning,
    no Monday morning, no time typed in words, no repeating task, which the web window's menu
    lists under it.
  - New project and Reorder are drawn, and both are the web window's to do (its New project
    card, Settings): New project says so in a line.
  - The words last as long as the window. The web window writes its card's draft down on every
    keystroke and has it back after a restart.
  - The picks are this window's own. The web window keeps its own in its storage
    (`zero.lastProduct`, `zero.lastModel`, `zero.lastEffort`), which the door does not hand
    over, so each window opens on its own last project, model and effort.
  - A menu lands at once. The web window's fades in and rises three points over 0.13 s (`tc-in`).
  - On the card, Escape out of a menu leaves the caret in the words rather than on the chip:
    Rune's dialog hands the keyboard back to its one field. A second Escape still closes the card
    and a letter still goes to the words, as in the web window; ↓ there moves the caret instead of
    opening the menu again.
  - A press outside an open menu goes on only to the box's own controls (another chip, the words,
    Send). Anywhere else on the page it only closes the menu, where the web window's also lands
    where it was aimed (a row opens).
  - Z on a new thread is refused when its agent has answered by the window's last reading of the
    thread; the web window reads the engine afresh first.
  - The caret is half a point wider and a fifth shorter, as Rune draws every caret, and the large
    words on Home sit up to half a point from the browser's, glyph by glyph.
- **Files in a new thread, where they still differ:**
  - The paper clip's picker picks one file at a time: Rune's file dialog (`Tx::open_file`) has
    no choosing of several, where the web window's file input takes several at once. A drop and
    a paste take any number.
  - A drop is taken anywhere on the box or the card. The web window takes it on its words, which
    are most of the box; a drop on the rest of it does nothing there.
  - A picture in the box opens with the system. The web window opens it large in a viewer of its
    own that steps through the box's pictures. A pasted picture is no file yet and opens nothing.
  - Nothing is read in pieces: a file on this Mac goes as its path, and a pasted picture is held
    as its bytes until Send. So nothing is ever on its way in but a photo being turned, and the
    bar the web window runs along a chip's foot as the pieces of a big file land is never drawn.
    A pasted picture goes across the door in one request, and the door takes 32 MB of it, about
    a 24 MB picture; Rune's clipboard hands over 32 MB of PNG at most. Between the two a send is
    refused with the door's reason and the words and files come back. The web window sends such
    a picture in pieces (`stageAttachmentPart`).
  - The web window writes a pasted picture to disk as it is pasted, so its draft survives a
    restart. Here the files last as long as the window, as the words do.
  - A picture whose header does not read is a chip, where the web window draws a broken picture.
    It still goes, as the file it is.
  - A file that cannot be read when it is attached says so at once, in the web window's words for
    a file that is not there or else the system's own. The web window finds out at Send, and
    says the engine's reason then.
  - A link the app wrote into a person's message is a mention whose underline sits a point or two
    above the web window's, as every mention's does here.
- **A reply shows in the conversation once it is written.** The web window draws it at once,
  marked as sending. Here the box folds, the line says Sent, and the words appear three seconds
  later with the engine's next answer.
- **Z after a reply or a pick is written takes nothing back.** The web window withdraws it and
  stops the run it started. Here Z works for the three seconds before, and after only for a close.
- **No card to allow or deny what an agent asks to run.** The web window draws one over the
  page, answered with ⌘1 and ⌘2. Here a line at the card's foot says an agent is waiting for a
  yes and where to give it. Where every project runs on full access no agent ever asks.
- **No question before throwing work away.** An option that discards the change asks first in
  the web window. Here it is not sent, and the line says to pick it there.
- **No warning before closing a thread whose agent waits on an answer.** The web window gives it
  once ever, on the first such E, and remembers for itself that it did.
- **Several rows cannot be ticked**, so nothing is done to many threads at once.
- **The rail has only what works here**: Search, Home, the Inbox, Files, Restart to update while
  a new version waits, Settings, the usage row and the person's own row. The web window's also
  has Demands, Chats (with teammates), Connections, Record meeting and Feedback. Drawn here they
  were buttons that went nowhere, so they are left out until this window has their pages, and
  Files stands under the Inbox where the web window has Demands and Connections before it.
  Settings and the person's row bring the web window
  forward on its Settings, as there is no Settings page here; the usage row only reads, where the
  web window's opens the limits under it.
- **The command bar offers only what this window can do.**
  - Not listed, because nothing here does them yet: New project, Record meeting, Filter,
    Projects and ranking a project first, the cloud's
    rows, Groups and each group, Files, Connections, teammates, the theme rows, updates and
    restarts, Pause agents, Resume interrupted agents, General agent instructions, Usage and
    limits, Settings, Keyboard shortcuts, Show as a board, Import chats and agents, the tutorial,
    setting Mercury up again, a demo copy and keyboard shortcut hints. On a thread: Remote
    control, Open terminal, Approve, Run now, Close task, Try again and Resume this agent, Mark
    read and Mark unread, the thread's own menu (who sees it, pin, rename, where it runs and the
    rest), its model, moving it to another project, Stop, Send back to agent, Restart the agent
    and its priority. With rows ticked the web's bar acts on all of them; here none can be ticked.
  - It finds threads only, the best five, with no "See all" row, since this window has no search
    page. The web's bar also finds people and groups, repeating tasks, teammates' threads, and
    lines, files, comments and decisions inside things, which its main process searches.
  - A thread a day old or more says its date in English. The web window asks the system's
    language.
  - With a team's tabs on, the web's bar goes to two pages, Later and Done being columns of In
    progress there. This window draws no team tabs, and its bar goes to all four.
  - The caret is Rune's own, 1.5 points wide and four fifths of the line, where Chromium's is 1
    point and the whole line; Rune has no setting for it.
  - Control K is answered as the web window answers it, but only ⌘K has been tried, by script.
- **The Later picker takes a moment, not a rhythm.** In the web window "every monday 9am" typed
  there makes the thread a repeating task, and its box says so in its hint. This window keeps
  no repeating tasks, so the phrase is "not a time" and the hint names only what is read. A
  terminal agent's row cannot be put off here, and the one warning before putting off a thread
  whose agent waits on an answer is not given, as before closing one.
- **A person's photo is their two letters.**
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
- **What a run did, where it still differs:**
  - A line inside an opened fold does not open. The web window opens it onto the whole command
    and the first six lines of what came back, and all of them on a second press; here it shows
    its mark and how many lines came back. A command the column cuts gets no mark here, where the
    browser tells the web window it was cut and that gives it one.
  - A file a run changed is not a chip into the code. The web window makes it one when the
    thread's change holds that file (`zero:code-change`, not read yet).
  - Escape does not fold back what was opened on this visit, and an opened fold is not
    remembered when the thread is left and opened again (`useEscapePutsAway`, `keepPlace`).
  - The line for a long conversation's missing middle is drawn from the stylesheet: no scene has
    forty messages, so it was never photographed.
  - The line at a working thread's foot does not open. The web window opens it onto the steps
    since the agent last said something, five at most, and the calls running now. Only its
    working state is drawn: queued, up next, signed out, at its limit, paused, nothing came
    back, working elsewhere, shipping and waiting on a yes are not.
  - Its mark and word stand still: the lattice at the web window's own opacity for no motion,
    the word in full ink without the band that travels through it. A photograph of the web
    window catches both mid-animation.
  - In the opened reply box of a thread In progress, the web window puts Stop where Send is
    until something is typed. This window does not stop agents, so Send stays.
  - A conversation drawn from the answer its agent is working from lands at its foot, as an
    unfolded one does; the web window lands on the top of the last message when that message is
    taller than the window.
- **The cloud button is a picture of the cloud's state**, with no menu behind it. Live, it goes on
  saying "Sign in again" for as long as an account is stopped: the web window stops saying it once
  its menu has been opened on that account, which it remembers for itself
  (`zero.cloudBrokenSeen`) and the door does not hand over. A fixture carries it.
- **When a thread comes back is said in English**, on a twelve-hour clock ("Sun 5:24 PM"), as the
  other dates are. The web window asks the system's language.

## What is where

- `src/main.rs`: the state, the three screens' routing, the command line, and what a reply, a
  pick, a close and Z do to that state.
- `src/keys.rs`: what each key does. `src/write.rs`: sending, off the main thread, and what
  waits to be sent.
- `src/source.rs`: fixtures and the door. `src/front.rs`: whether this window is in front, said
  to the desktop app, and the app's word to come forward at a thread.
- `src/data.rs`: the snapshot's shapes and the lists the screens draw. `src/rules/`: the rules
  behind those lists, behind the Inbox's tabs and its cloud button (`page.rs`, `cloud.rs`), and
  behind what else depends on whose Mercury it is: `window.rs` (the team, the account, the rail's
  foot, a project's colour) and `place.rs` (where a conversation runs, what its panel says and
  what a pick there writes).
- `src/chrome.rs`: the page, the card, the rail and how it opens, the bands that move the window.
  `src/theme.rs`: colours and type.
- `src/feel.rs`: what the screens share of answering the pointer: the key hints and their plate,
  the row under the pointer, the strip's framed buttons, the washes; and the tests of how the
  window feels. `tools/scenes/pointer.mjs` rests the pointer in a reference picture, and
  `tools/hover.py` measures the control under it against this window's.
- `src/home.rs`, `src/inbox.rs`, `src/thread.rs`: the screens. `src/talk.rs` reads a thread's
  ledger and its runs' logs into a conversation (`docs/thread-data.md` says what the engine is
  asked), `src/reads.rs` asks for them and says when, and `src/markdown.rs` draws an agent's words.
- `src/rules/notes.rs`, `trace.rs`, `work.rs`: a run's log read as the web window reads it, its
  sentences, the tools it ran in plain words and what came back. `src/rules/live.rs`: the line at
  a working thread's foot and the line under its title.
- `src/later.rs`: the Later picker. `src/rules/later.rs` reads the time typed into it, on the
  calendar `src/rules/calendar.rs` keeps.
- `src/reply.rs`: the model and level a conversation's reply box is on, and its model menu (the
  box's own, from `compose.rs`). `src/rules/reply_pick.rs`: what the box opens on, what a pick
  keeps and what the next reply writes. `src/strip.rs`: what the strip over a conversation
  opens, the thread's menu and the place panel, and what a pick there writes.
- `src/compose.rs`: the box a thread is started in, on Home and as the New thread card, its
  menus and what a send does. `src/rules/compose.rs`: what a new thread is made of and the
  menus' rules. `src/picks.rs`: the picks kept from one window to the next.
- `src/attach.rs`: the files in that box, how they come in and how they are drawn.
  `src/rules/attach.rs`: which can go, what the box says of one that cannot, and how they are
  saved and linked. `src/rules/typed.rs`: the lines the app wrote into a person's message for
  its files, which a conversation draws.
- `src/palette.rs`: the command bar ⌘K opens. `src/rules/palette.rs` decides what it offers and
  in which order; `src/rules/search.rs` finds a thread by its words.
- `src/files.rs`: the files those words name, drawn, and the one function that opens one with the
  system. `src/pictures.rs`: the pictures, read off the disk on a worker and kept small.
- `src/viewer.rs`: a file opened big, over the window. `src/rules/viewer.rs`: which files it holds
  and steps through, where a press lands, what the thread called the file, who shared it, and a
  picture's fit and zoom. `src/rules/code.rs`: a script's colours.
- `src/files_page.rs`: the Files place. `src/rules/files_shown.rs`: what it shows, in what order,
  and what a tile says.
  `src/rules/paths.rs`, `mentions.rs`, `roots.rs`: which words name a file, what kind it is and
  what it is called, and where it is looked for. `src/rules/thread_files.rs`: which files a
  thread made, each once at its newest version, for the Files button (`src/thread.rs` draws it
  and its list).
- `tools/pictures.mjs`: the invented project folder the scenes that name files are drawn from.
- `tools/install-mac.sh`: builds the app and puts it in `~/Applications` as "Mercury Native".
- `src/sky.rs`: the dithered sky, ported from `renderer/src/home/sky.ts` with the same seed.
- `src/icons.rs`: the icons, with the web app's own paths and strokes.
- `assets/fonts`: Geist and Geist Mono (SIL Open Font License), the files the web app ships.
