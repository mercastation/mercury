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
dropped on it, picked with its paper clip or pasted into it ("Files in a new thread"). A reply
carries pictures and files the same three ways ("Files in a reply").

Answering a thread and stopping its agent are the web window's whole ("Answering a thread and
stopping its agent", below): Stop stands where Send is while a thread is under way and its box
is empty; a reply or a picked option is on the page from the press, saying where it is; Z takes
it back whole inside three seconds, and after that withdraws it and stops the run it started,
asking first once it is half a minute old; an option that throws the change away asks first; the
first close of a thread whose agent waits on an answer says what it costs; and each box keeps its
words, files and model across a restart, as the web window's does.

What can be chosen in a conversation is chosen as in the web window ("Choosing in a
conversation", below): the model and level its next reply runs on, from the reply box's chip or
the model named on the folded box; the priority the next reply sets and what it may do, from the
box's other two chips, and Shift+Tab; where it runs, this Mac or the cloud, from the place
button's panel; and from the dots, Later and its priority. The strip's tab word and its arrows go
where the web window's go, and the little arrow over an agent's choices folds them away.

The rail's usage row names the agent used last and what it has spent, and resting on it opens the
card of every agent's limits over it, as the web window's sidebar does ("The agents' limits",
below).

A file an agent names in a conversation is not left as its path: a picture is drawn in place, any
other file is a mention that opens it, and a folder on a line of its own shows its newest pictures
("The files a message names", below). A thread that made or named files carries the Files button
in its head strip, with how many, which opens the list of them ("The files a thread made"). A
press on a document, a table or a script opens it in a pane beside the conversation, which
narrows to make room, as the web window's does ("A file opened beside the conversation"); a
picture, a film, a PDF, a page or a deck opens big over the window, its versions and the files
beside it a key away ("A file opened big"), and the rail's Files is the place of every file the
threads made ("The Files place").

A thread whose run changed code carries the code button in its head strip, with the lines added
and removed, which opens the change beside the conversation: its files as a tree, every file's
hunks in one running column, the lines between two hunks a press away, its own keys, lines to
select and copy, and the lines now in the file to type into and save with ⌘S. A file a run's
step edited is a chip that opens the change at that file ("The code a thread changed", below).

⌘K opens the command bar from anywhere: it finds threads by their words, as the web window's bar
does, and offers every row of the web's whose doing this window has, under its words, in its
order, writing what the web's row writes ("The command bar", below; `docs/parity-command-bar.md`
lists each row of the web's with what this window does of it).

The rail's Settings opens Settings here, over the whole window, as the web window draws it: its
menu with the search, and the pages that are used, General, Phone, Alpha features, Claude Code,
Codex, Antigravity, Running, Projects and a project's own page, Files, and Plan and Billing. The
other pages, and what goes on to a sign-in or to Stripe, bring the web window forward on that page
("Settings", below).

Under the pointer it answers as the web window does: the rail opens while the pointer rests on
it, every control takes the web window's pointer and look under it, a key hint comes after the
pointer has waited, the strips move the window, words can be selected where the web window's
can, and a right press on a link or a picture opens the web window's menu for it ("How it answers
the pointer and the hands", below).

A conversation shows what each of its agent's runs did: what the run said, between the messages,
and what it ran folded to one line under that, which a press opens, and each line in it opens
onto the whole command and what came back. A thread an agent is on says at its foot which step it
is on and for how long, and is read again as the run goes on; a thread no agent of this window is
on says there why, in the web window's words (queued, up next, signed out, at its limit, paused,
nothing came back, working elsewhere, shipping, waiting on a yes, cannot start, nothing running),
with Try again where nothing will start it without the person, and a stopped one wears the bar
that tries again over its title ("What a run did", below). Escape folds back what was opened on
this visit, a conversation left and opened again is as it was left, and one opens where a person
starts reading, its last message. The working line moves as the web window's does, its mark
lighting in orbit and its word shimmering, and a press opens it onto the steps since the agent
last said anything ("What moves on its own", below).

The Inbox is its head strip (the title, New thread, the cloud button on a Mac set up for the cloud
or offered it, the display button), its tabs (Needs you, In progress, Later while something is in
it, Done) and the open tab's threads as a table. A row is the mark of the agent it runs on, its
title with what follows it (an aside, "not started", a pin), the Status word with its mark, Try
again after it on a thread nothing will start, and under the pointer its whole sentence or the
card saying why the cloud has not got the thread, the
project, the priority, Medium included, and a time: when it was sent, when it comes back under
Later, when it was finished under Done. The display button opens
View and filters, whose sort and filters every list follows and which this window keeps for the
next; the cloud button opens the cloud's menu, which turns it on or off, says what on means, moves
runs there and says what it used; and ⌘C turns the cloud on or off from anywhere ("The Inbox's
View and filters, and the cloud's menu", below).

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
`src/front.rs`). A Settings page this window does not draw brings the web window forward on that
page (`zero:open-settings-here`, a channel only the desktop app's door answers, which takes one
word from the caller, `{ "page": <id> }`, opens Settings on it if it is one of Settings' own pages
and on General if it is anything else, and does nothing more); and Restart to update,
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
pasted picture and a report, drawn as the person's own words). The reply box holding files has
scenes of its own (`thread-reply-files*`, "Files in a reply"), and so do the working line opened
onto its steps (`thread-working-steps*`), a chat being got ready (`thread-getting-ready`) and the
model menu coming in, for a film (`thread-reply-model-opening`).

The invented inbox has almost nothing switched on: nothing in Later, no cloud, no team. So some
scenes add to it. A scene of Home dresses the inbox before the page opens (`dress`): `home-team`
is Home for someone with teammates, a meeting recorder, a reading of usage and an account signed
in, `home-update` is Home while a new version waits for a restart, `home-few` is Home with two
threads that need the person and none in progress, and `home-working` is Home while an agent works
on a thread. A scene of the Inbox hands
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
| Home | 0.86 | 0.995 |
| Inbox | 0.27 | 0.998 |
| Inbox, the Later tab (`inbox-later-kinds`) | 0.30 | 0.997 |
| Inbox, each state of the cloud button | 0.27 | 0.998 |
| A conversation | 0.41 | 0.996 |

Text lands where the browser puts it to a hundredth of a point. The Inbox and a conversation are
measured against the web window's newer look: the rows' marks of their harness, every level in
the Priority column, a conversation's head strip of 64 points with buttons of 32. Before this
window followed it they were 0.47 and 0.92 against it; most of what is left is the rail, whose
places differ (0.52 of 255 inside it).

One size proves one size: Home was right at 1280 by 760 and wrong in a taller window until it
was compared there. Home and a conversation are compared at 900 by 760, 1600 by 1000 and 1920 by
1200 as well, and stay between 0.25 and 1.0 of 255, but for a conversation at 1600 by 1000
(`thread`, 1.12; 0.55 at 900 by 760), whose foot rests half a point lower than the browser's
(see what still differs).

Home was measured again against the web window's newer look, which gave its rows the agent's
face, its tags the line's own words and its box a shorter bar (`home-few`: three rows or fewer,
the two cards only as tall as their rows; `home-working`: a thread in progress, its face with
the green dot):

| Scene | 1280 by 760 | 900 by 760 | 1600 by 1000 |
|---|---|---|---|
| `home` | 0.86 | 1.00 | 0.69 |
| `home-typed` | 0.91 | 1.08 | 0.73 |
| `home-few` | 0.82 | 1.13 | 0.58 |
| `home-working` | 0.87 | | |

At 900 by 760 `home` is 1.10 on the card and 0.50 on the rail, which has fewer places and a
shorter foot than the web window's (see what still differs).

The Inbox holds at other window sizes too, where its table gives up columns (Priority at a table
of 915 points, Project at 785). The first tab and `inbox-later-kinds`:

| Window | First tab | Later |
|---|---|---|
| 1600 by 1000 | 0.20 | 0.22 |
| 900 by 760 | 0.32 | 0.36 |
| 800 by 760 | 0.34 | 0.38 |

And on one person's real inbox, read through the door, with teammates, a cloud, a recorder and a
sign-in all on, before the web window's newer look: Home 0.86, the Inbox's four tabs 0.14 to
0.25, a conversation 0.47.

A conversation that names files, whole window, at the three sizes. Its head strip carries the
Files button with its count, as the web window's does (all but `thread-folder`, which names only
a folder): the strip alone is 0.50 at 1280 by 760, as a strip with no files is (`thread`, 0.49),
where it was 0.86 while this window drew no button. At 900 by 760 the thread's name in the strip
used to run on over the arrows instead of being cut short: the strips with the button are 0.57
and 0.63 there, where they were 1.79, and `thread-folder`'s 0.63, where it was 0.98.

| Scene | 1280 by 760 | 1600 by 1000 | 900 by 760 |
|---|---|---|---|
| `thread-pictures` | 0.35 | 0.32 | 0.45 |
| `thread-pictures-ways` | 0.38 | 0.33 | 0.49 |
| `thread-pictures-row` | 0.35 | 0.29 | 0.46 |
| `thread-pictures-large` | 0.25 | 0.18 | 0.32 |
| `thread-pictures-missing` | 0.46 | 0.36 | 0.61 |
| `thread-pictures-arriving` | 0.40 | 0.40 | 0.53 |
| `thread-pictures-said` | 0.25 | 0.38 | 0.32 |
| `thread-links` | 0.68 | 0.62 | 0.92 |
| `thread-folder` | 0.37 | 0.38 | 0.49 |
| `thread-folder-quiet` | 0.34 | 0.28 | 0.44 |

Inside the answer alone, at 1280 by 760: 0.08 where it is only pictures (`thread-pictures-large`),
0.37 to 0.66 where sentences name them, and 1.23 for `thread-links`, whose ten mentions each
carry an underline two points higher than the web window's (see what still differs).

The command bar is photographed open over the Inbox: `palette` (its box empty), `palette-typed`
("check": a command, then the threads the word found) and `palette-none` (words nothing
answers); over a conversation, waiting (`palette-thread`, and `-model` with "model" typed) and
finished (`palette-thread-done`); with words of each kind typed over the Inbox
(`palette-words` "prio", `-move` "move to", `-settings`, `-agents`, `-keys` "keyboard", `-rank`);
and on an inbox dressed for the rows that only show then: the cloud on with "cloud" typed
(`palette-cloud`), an update waiting, a build to restart, the agents paused and a project made
for the person (`palette-update`), a queued thread under In progress on a Mac with the cloud
(`palette-progress`, drawn with `--tab progress`), an agent's question (`palette-approve`) and a
stopped agent (`palette-stalled`). Each of these scenes reads the rows its
page drew and hands them on in the fixture (`paletteRows`: a heading, a command with the name of
its drawing, its name, hint and key, or only the place of a thread), with the words typed
(`palette`). The page draws a row's drawing from its paths, so the scene names it by matching
them against the web's own table (`ICONS` in `renderer/src/palette-glyphs.ts`, read from the
checkout photographed). Given them, this window draws the web's rows in the web's places: a
command it has too, by its words and drawing, is its own row and does what it says, one it does
not have is drawn as it was and does nothing, and where the web listed a thread it lists the
next one its own search found for the same words, so the threads section measures the search as
well as the drawing. Without `paletteRows` it draws its own rows, as it does live. Each scene
names its parts (`tools/scenes/palette*.regions.json`).

The rows themselves are held to the web's: `cargo test the_rows_are_the_webs -- --ignored
--nocapture`, with `MERCURY_PALETTE="<fixture> --view inbox; <fixture> --thread; …"`, lines this
window's own rows for each picture's inbox and words up against the rows the web drew, row by
row. In all seventeen scenes every row this window draws is the web's, with its words, drawing
and key or hint, in the web's order, and the only rows the web has beside them are the ones
`docs/parity-command-bar.md` marks out or another task's.

Measured against the web window's newer bar, whose rows lead with line drawings, whose lit row
is a wash with no accent down its edge, whose Esc is a key's cap and which has no foot:

| Part, at 1280 by 760 | `palette` | `palette-typed` | `palette-none` |
|---|---|---|---|
| Whole window | 0.20 | 0.20 | 0.19 |
| The box, its frame included | 0.21 | 0.29 | 0.48 |
| Search row | 0.70 | 0.62 | 0.70 |
| Esc | 0.44 | 6.39 | 0.44 |
| The lit row | 0.18 | 0.27 | |
| The rows under it, the threads found, the line saying nothing matches | 0.12 | 0.18 | 0.30 |
| The backdrop around the box | 0.12 to 0.26 | 0.11 to 0.26 | 0.11 to 0.25 |

With every row of the web's this window has drawn as its own, of 255 at 1280 by 760, 2x, the
whole window, the box (read off the web window's dump), its search row and the rows under it:

| Scene | Whole | The box | Search row | Rows |
|---|---|---|---|---|
| `palette`, `-typed`, `-none` | 0.15, 0.14, 0.14 | 0.21, 0.29, 0.47 | 0.69, 0.62, 0.69 | 0.13, 0.19, 0.23 |
| `palette-thread`, `-model`, `-done` | 0.16, 0.18, 0.14 | 0.20, 0.28, 0.21 | 0.68, 0.45, 0.68 | 0.12, 0.21, 0.13 |
| `palette-words` (prio), `-move`, `-settings` | 0.13, 0.14, 0.13 | 0.20, 0.51, 0.34 | 0.31, 0.51, 0.54 | 0.17, 0.52, 0.22 |
| `palette-words-agents`, `-keys`, `-rank` | 0.16, 0.14, 0.14 | 0.36, 0.46, 0.27 | 0.53, 0.69, 0.39 | 0.31, 0.31, 0.23 |
| `palette-cloud`, `-update`, `-progress` | 0.14, 0.19, 0.12 | 0.29, 0.21, 0.19 | 0.44, 0.69, 0.69 | 0.24, 0.13, 0.10 |
| `palette-approve`, `-stalled` | 0.15, 0.15 | 0.21, 0.21 | 0.69, 0.69 | 0.13, 0.13 |

What is left is glyphs' edges, the caret, and the box's height on a quarter point.

The backdrop is the Inbox under the scrim, whose rows have changed in the web window too (their
harness marks); the box itself is where it was. The empty bar is 0.20 at 900 by 760 and 0.14 at
1600 by 1000, its parts as at 1280. What is left is the caret (see what still differs), glyphs,
the placeholder's among them, landing within half a device pixel of the browser's, and Esc in
`palette-typed`, whose box stands on a quarter point: its word lands a device pixel lower there.

A conversation with its runs' logs, at 1280 by 760, whole window and the area the runs changed
(each scene's own `.regions.json`). The text-dense areas of the earlier scenes sit between 1.1 and
1.9 the same way, from how the two draw a glyph's edge:

| Scene | Whole window | Where the runs are drawn |
|---|---|---|
| `thread-steps-folded` | 0.41 | 1.65, the title and its "Show 8 earlier messages" |
| `thread-steps` | 0.50 | 1.20, a run's sentences and its two folds |
| `thread-steps-open` | 0.45 | 0.91, a run and its fold opened |
| `thread-steps-failed` | 0.49 | 0.92 the run turned away, 0.85 the fold with the failed test |
| `thread-working` | 0.43 | 1.38 what the run said, 0.58 the line at the foot |

`thread-steps-folded` is 0.54 at 900 by 760 and 0.34 at 1600 by 1000.

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
is open. What eases is what eases in the web window: the rail opening and folding under the
pointer, below, and a box's menu coming in; and what moves on its own moves as the web window's
does ("What moves on its own").

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
  Inbox's dot crosses over 0.1 s (`team/rooms/rooms.css`), each frame drawn while it moves.
  Search has no line under it any more, so nothing of it crosses (`.workspace-find`). The card
  moves over with it, 44 points past its edge, rather than being covered, and the skies stay
  painted for the folded card and pinned to its right edge, so the rail uncovers them rather than
  painting them again (`sky-follows-the-card.ts`). Open, it says what the web window's says: each
  place's word, Inbox's count in the dim ink where the dot was, the team's name, Search with ⌘K,
  the foot's rows, the usage row naming the agent used last and its headline figure (resting on
  it opens the card of the agents' limits, "The agents' limits"), and the person's name over
  their email. Its places, Search among them, wash under the pointer; Search
  keeps its faint ink as it does, where the places' marks and words come up to the text's
  (`home-rail-search`: Search under the pointer 0.52).
  Under the system's Reduce motion it opens and folds whole, as the same stylesheet's
  `prefers-reduced-motion` block takes every one of those transitions away.
- **Under the pointer.** Every control these screens draw takes the web window's look while the
  pointer is on it and gives it up the moment the pointer leaves. The web window's `:hover` rules
  have no transition unless one says so, and almost none does, so the root lands every hover at
  once both ways (`hover_motion_within` in `src/main.rs`; Rune's own hover fades out over 0.1 s).
  The two that eased there, a mention and Search's line, no longer do, and Search's line is
  gone. On Home: the
  two doors, the rows (a Needs you row under the pointer becomes the row the keys act on), the
  cards' links, the box's chips, paper clip, Send and its caret, and on a team's Home To's face and
  a level nobody picked, which washes as a square. On the Inbox: New thread, the
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
  closes the one under it, claims no plate. A press on a control takes its plate away, and one on
  its way, and the control says nothing more until the pointer has left it: a plate under a tab
  just pressed only said what had just been done (`quietOnceClicked`, the hint listener's
  `pointerdown`). A control whose menu, list or panel is open says nothing at all, as its plate
  would sit on the menu's first row (`aria-expanded`): the cloud, the Files button, the place
  button and ⋯.
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
| Home: a door, a card's link | 6.00, 6.15 | 5.40, 5.58 |
| Home: a row, the box's chip, clip, Send's caret | 1.21, 3.34, 2.82, 1.41 | 1.42, 3.14, 2.21, 1.38 |
| Home: a file's chip, a picture's × | 0.43, 7.34 | 0.30, 5.08 |
| Inbox: back, display, New thread, a tab, a row | 0.01, 0.39, 1.63, 2.23, 0.35 | 0.00, 0.25, 1.46, 1.67, 0.35 |
| Conversation: Inbox, back, crumb, name | 0.32, 0.42, 1.01, 2.74 | 0.02, 0.12, 1.16, 2.15 |
| Conversation: J's arrow, Done, Reopen, Files, ⋯ | 0.76, 1.61, 2.71, 1.56, 0.80 | 0.23, 0.90, 1.84, 0.49, 0.07 |
| Conversation: earlier messages, a run's fold, an option | 3.93, 2.05, 0.76 | 2.41, 1.62, 0.45 |
| Conversation: a mention | 14.97 | 7.19 |
| Conversation: a picture, a folder's picture, a folder's name | 0.15, 0.33, 2.76 | 0.14, 0.32, 1.92 |
| The folded reply box, its model | 0.70, 4.14 | 0.70, 3.28 |
| The opened reply box: its first chip, clip, Send, caret | 0.47, 1.81, 1.25, 0.37 | 0.34, 1.20, 1.20, 0.27 |
| The New thread card: its model, a chip, Send | 0.73, 1.41, 3.16 | 0.73, 1.23, 2.35 |

The Inbox's and a conversation's rows were measured again on the web window's newer look: the
strip's buttons now 32 points with the strong line, the reply bar's clip 28. The mention is what
still differs most: its mark does not brighten with it (see what still differs). The plates, with
a point around each: New thread 2.67, a tab 2.51, a row 2.41, Done 1.79, J's arrow 1.41, the Inbox
arrow 1.54, the crumb 4.01 (the edge of its glass over the strip's) and ⋯ 1.35. Each is drawn
between 0.70 and 0.74 s after the pointer came onto its control, as the web window's comes 700 ms
after (`HINT_WAIT`). New thread
was measured before its title was: since, a headless picture draws Rune's own stand-in for the
title from 0.5 s, over the plate (36.89), where a Mac window leaves the title to AppKit and its wait.

The rail open, the pointer resting where the web window's rail has Demands (Files here): its
places under the Inbox and its foot are the web window's own and more (see what still differs),
so the rail is held to the web window's where the two draw the same, the team's name, Search,
Home and the Inbox with its count. Against the newer look, whose Search has no line under it and
whose count is in the dim ink, that part is 0.44 at 1280 by 760, 0.47 at 900 by 760 and 0.43 at
1600 by 1000 (0.62, 0.65 and 0.60 before it followed), the same over the Inbox and a
conversation; the whole rail is 1.01, 1.03 and 0.81, and Home with the rail open 0.99, 1.06 and
0.76. With teammates, a recorder and a sign-in (`home-team-rail-open`) the whole window is 1.12,
the rail 1.61 (the web window's has Chats and Record meeting); while an update waits
(`home-update-rail-open`) 1.32, the rail 3.39, as the update's card is not drawn open (see what
still differs).

The pointer's shape, points that match the web window's of all the points in each scene:

| Scene | Arrow | Pointing hand | I-beam | Magnifier |
|---|---|---|---|---|
| `home`, `home-attached` | 51 of 51, 51 of 51 | 80 of 88, 83 of 91 | 2 of 2, 1 of 1 | |
| `inbox` | 29 of 29 | 170 of 179 | | |
| `compose-card`, `palette` | 193 of 193, 215 of 215 | 19 of 19, 73 of 73 | 19 of 19, 3 of 3 | |
| `thread`, `thread-reply` | 33 of 41, 34 of 38 | 27 of 35, 43 of 51 | 15 of 15, 10 of 10 | |
| `thread-options`, `thread-steps` | 34 of 37, 29 of 35 | 60 of 79, 34 of 42 | 21 of 21, 22 of 22 | |
| `thread-pictures`, `thread-folder` | 35 of 37, 29 of 30 | 38 of 50, 33 of 41 | 11 of 11, 14 of 14 | 9 of 9, 22 of 22 |
| `thread-links` | 27 of 39 | 55 of 84 | 11 of 11 | |
| `thread-files-open` | 37 of 37 | 26 of 74 | 0 of 10 | 0 of 3 |

What is left: the I-beam over the end of a line past its last word, every mention's mark, the
sorting headers of a table in a message, the Inbox's box that ticks every row (not drawn), the
page under the open list of files, which shows the arrow, and the rail's places and foot rows the
web window has and this one leaves out: on the web window's newer look those are Demands,
Connections and Feedback, and eight of the pointing hands each scene misses are theirs. On the
New thread card the marks To and the model hang beside their words take the pointer, as part of
their buttons; only the corners of To's round face differ, which Chromium finds as a circle. Each
is in what still differs. Where the
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
  one hover that eases in the web window, Search's line, does not here.
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
| A conversation | Esc | Folds back what was opened on this visit, newest first (the folds and lines of what a run ran together, the working line's steps), then the reply box, then goes back, onto the tab that holds the thread now; with a file beside it, what was opened after the file, then the file |
| The reply box | Esc | Folds it when it is empty; holding words or files, lets go of it, and the next Esc folds it |
| A conversation | J, K | Goes to the next thread that needs the person, or the one before |
| A conversation | ↓, ↑, Enter | Walks the agent's options and sends the one lit |
| A conversation | 1 to 9 | Sends that option |
| A conversation | R | Opens the reply box; ⌘Return sends what is in it |
| A conversation | E | Marks it done and goes on to the next thread |
| A conversation's list of files | Esc | Closes it; every other key goes on to the page under it |
| A conversation's menu (the dots) | ↓, ↑, Home, End, Enter, Space | Walks its rows around both ends and presses the one lit; every other key goes on to the page |
| A conversation's menu, its place panel | Esc | Closes it; Tab closes the menu too |
| The reply box's model menu | ↓, ↑, Enter, Esc | Walks its rows around both ends and picks the one lit; Esc closes it and puts the caret back in the words; any other key goes on to the page |
| The reply box | Shift Tab | Moves what the next reply may do to the agent's next mode, and says so |
| The reply box's priority menu | ↓, ↑, Enter, Esc | Walks the four levels around both ends and picks the one lit, the keys staying on the chip; Esc closes it there, where ↓, ↑, Enter or Space opens it again; any other key goes on to the page |
| The reply box's mode menu | Enter, Space, Esc | Enter and Space close it, as a press on its chip does; Esc closes it and puts the caret back in the words; any other key goes on to the page |
| Home | N, or a letter none of the list's keys | Puts the caret in the box on the page (the letter typed into it) |
| The Inbox, a conversation | N | Opens the New thread card over the page |
| The box, the card | ⌘Return | Sends the thread; Return is a new line |
| The box, the card | Esc | Closes the open menu; then lets go of the box, or closes the card, keeping the words |
| A chip of the box | ↓, ↑ | Opens its menu (after Esc left the keyboard on it) |
| A menu of the box | ↓, ↑, Enter | Walks its rows around both ends and picks the one lit |
| Anywhere | Z | Takes back the last thing done; past half a minute it asks first, and a second Z inside eight seconds does it |
| A conversation | a letter none of the above | Goes into the reply box, after its words, and opens it; E and L just after the screen changed wait a moment for a word |
| The question before a discard | 1, 2, ↓, ↑, Enter, Esc | Answers it, walks it, answers the lit one; Esc keeps the change |
| Anywhere | ⌘K | Opens the command bar, and closes it again |
| The command bar | ↓, ↑, Enter, Esc | Walks its rows and runs the one lit; Esc closes it |
| Anywhere | ⌘C | Turns the cloud on, or asks before turning it off; Copy instead while words are selected, a box has the keys or something stands over the page |
| ⌘C's question | Enter, Esc | Turns the cloud off; keeps it on. Every other key waits |
| The Inbox, a menu of its strip open | Esc | Closes it; every other key goes on to the page under it |
| The Inbox | B | Says the board is the web window's |

Everything written is one call on a channel the web window calls too, with the web window's own
words in it (`src/rules/writes.rs`): a reply, a picked option ("Option 2: …") and a close go to
`zero:answer`, as Reopen does (`status: open`, which Z marks done again), and a thread opened is told to the web window, which
alone keeps what has been read, the way a phone tells it (`zero:phone-seen`). An option that
closes ("Close this task"), leaves the work on its branch or needs the person's own words
("Change something first") does that instead of being sent, by the web window's rules. A reply
typed in the box carries the model, level and agent picked in it, the priority picked on its
chip, and what it may do, as the web window's box sends them; a level picked in a conversation's
menu and where it runs are written on their own channels ("Choosing in a conversation", below).

What is done waits three seconds before it is written, as in the web window (`deferCommit`): it
shows at once, a line says so with Z and Undo beside it, and Z inside that time takes it back
whole, because nothing was written. Once written it goes on Z's pile, newest last (`undoStack`):
a close, by opening the thread again as it was, blocked if it was blocked, so no run starts; a
reply or a pick, by stopping the run it started and withdrawing it; a thread put off or brought
back, by its moments as they were; a thread sent from the box, by closing it. Past half a minute Z
names what it would undo and only a second Z inside eight seconds does it, any other key saying
no (`undo-window.ts`). Each Z on a thread leaves one line in it, drawn as the person's own act. If
the engine refuses a write, its own reason is said in one line and what was refused is put back;
a refused Z keeps its place on the pile.

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

## Answering a thread and stopping its agent

Everything the web window does when a person answers a thread or stops its agent, line by line,
is `docs/parity-answering.md`, each line ticked with how it was checked. The code is `src/answer.rs`
(its rules `src/rules/answering.rs`) and `src/drafts.rs`.

- **Stop.** In the opened box of a thread In progress (the tab's own rule, `stoppable`), Stop
  stands in Send's place, its caret with it, while the box holds no words and no files: a square
  and "Stop" in the dim ink inside the strong line, the wash and the text's ink under the pointer,
  the square alone in a box under 460 points. Its title says whether a run is up. A press puts the
  caret back in the box, writes what is held for its three seconds first, then `zero:stop-session`,
  and once the engine has it says "Agent stopped. Back in your inbox. Reply to redirect it." or,
  with nothing started, "Stopped before it started. …". The person stays on the thread, following
  it: Escape then goes to Needs you, the row picked. The command bar has the same row, "Stop Agent
  (Moves to Inbox)" or "Stop Task (Moves to Inbox)", after the thread's rows. Nothing else stops a
  run in the web window either: no key, not the thread's menu, not the working line.
- **What was sent, at once.** A reply is on the page from the press as the person's message, its
  head saying "Sending… press Z to undo" in the time's place, its words a little fainter; a pick is
  the act its written copy will be ("Picked" and the option). One written copy takes the place of
  one drawn copy, oldest first, and the drawn one goes ten seconds after it is written. On a thread
  at rest the reply is held three seconds, its row In progress meanwhile, and the strip's line names
  the thread ("Sent → Storefront", a pick "Sent “Ship it” to …") with the time left to open it. On a
  thread a run is on, no line is said: after the three seconds the head says "Sending…", or, on
  Claude Code, "Waiting for its current step" with Send now, which writes `zero:send-now` and says
  "Sending now…". Refused, the drawn copy goes and the words come back after what the box holds now,
  the box open, with the engine's words; elsewhere a line names the thread.
- **Where it leaves the person.** A reply from a thread opened from Needs you on the Inbox moves on
  to the next thread waiting (not one opened from Home or come back to with O), and a pick does
  unless its words promise more or a run is up. Moved on, the line stands in the next thread's
  strip instead: a dot, "Sent to" and the thread, O Back, Z Undo, and a ring that runs out over five
  seconds, stopped while the pointer is on it. A number pressed, or an option pressed with the
  pointer where it was, in the moment after the app moved the person is not taken, and a line says
  it was meant for the thread before; so is a press on the options just after they moved under a
  still pointer.
- **Z after it is written.** A reply: the reminder or moment it lifted put back, then
  `zero:stop-session` with `withdraw`, then the answer withdrawn with the status it found; its
  words, files and model go back in the box unless the box holds something newer, and "Reply
  withdrawn, back in your inbox · your message is back in the reply box". A pick: the stop, the
  withdrawal, "Option 1 withdrawn, back in your inbox". A reply already with a running agent
  cannot be taken back: Z says so and opens the conversation.
- **The question before a discard.** An option that throws the change away asks "Discard the
  change?" where the options stood, inside the strip's frame: 1 Discard it, 2 Keep it
  (recommended). Its keys are the numbers, the arrows, Enter on the lit one and Escape; 1 sends the
  option as any pick, 2 and Escape leave the offer standing.
- **The waiting agent's warning.** The first E ever on a thread whose agent waits on an answer
  only says "This agent is waiting on your answer, and marking it done leaves it stuck. Press
  again to go ahead."; the first L ever opens the Later picker with the same lesson inside it. One
  marker for both keys, kept with the drafts.
- **Drafts.** Each box's words, files and model are written down as they change, beside the note
  about the app's door, and a thread whose box holds a draft opens with the box open on it. A
  reply sent is written down until it lands; one a quit or a crash lost is looked for in its
  thread at the next launch, three times over two minutes, and put back in its box with "Your reply
  didn’t send. It’s back in the box." What is held when the window closes is written before it goes.
- **The box and the options.** A letter typed on an open thread with the box folded goes into it; a
  press on nothing in the page folds an empty box; a press outside one of the box's menus that lands
  on an option or on Stop goes on to it; a choice is one line in the strip, whole while the keys are
  on it, and whole in a card over the strip while the pointer rests on it when it is cut short.

The scenes (`tools/scenes/`): `thread-working-reply` (Stop), `thread-reply-sending`,
`thread-pick-sending`, `thread-working-sending`, `thread-working-waiting`, `thread-working-handed`,
`thread-working-refused`, `thread-reply-moved-on`, `thread-discard-ask`, `thread-done-toast`,
`thread-later-warned`, `thread-pick-undone` and `thread-option-peek`. A fixture says what was done
before the picture (`replySent`, `handedOver`, `refusedWith`, `keysAfter`, `from`). `thread.mjs`
takes `hang` (the page's writes never answer, where the harness refuses them) and `runEngine`. At
1280 by 760, 2x, dark, whole window, before this and now:

| Scene | Before | Now | Where it differs |
|---|---|---|---|
| `thread-working-reply` | 0.47 | 0.37 | Stop itself 0.77 |
| `thread-reply-sending` | 6.52 | 0.35 | the strip's line 0.81, its clock |
| `thread-working-sending` | 4.49 | 0.42 | |
| `thread-working-waiting` | 4.53 | 0.41 | the head 2.0, glyph edges |
| `thread-working-handed` | 4.46 | 0.50 | |
| `thread-working-refused` | 5.55 | 0.41 | |
| `thread-reply-moved-on` | 4.83 | 0.94 | the line 1.14, its ring; the page under it is the harness's (it answers only one thread) |
| `thread-discard-ask` | 1.21 | 0.37 | the question 0.35 |
| `thread-done-toast` | 4.94 | 0.71 | |
| `thread-later-warned` | 24.13 | 0.26 | |
| `thread-option-peek` | | 0.56 | the card 0.79 |

`thread-pick-sending` and `thread-pick-undone` stand where they stood: the web window lands on the
top of the answer the act follows, this window on the page's top (see what still differs); the act
line itself lands on the web's to a hundredth of a point. The screens before this did not move:
`thread`, `thread-folded`, `thread-options`, `thread-option-picked`, `thread-options-reply`,
`thread-reply`, `thread-reply-typed`, `thread-working`, `thread-done-held` and `thread-later`.

Against a scratch engine (above), twin threads waiting on an agent's three options, this window
acting on one and the web window's own payloads sent by hand to the other, wrote the same ledger
lines in the same order: a reply and its Z (`answer`, `status: open`; then `status: blocked` from
the stop, `answer: (withdrawn)`, `status: blocked`); a pick and its Z (`Option 1: …`, `open`; the
same three); the discard asked and answered 1 (`Option 2: Discard the change`, `open`); and Stop
on a thread under way (`status: blocked`), the window saying "Stopped before it started" with the
crumb back on the Inbox. A drafts file holding two replies written down as on their way, one in the
thread's ledger and one not, came back as the first let go at the first look and the second in its
box at the last.

## Choosing in a conversation

What a conversation offers to choose is the web window's, ported where the person uses it: the
model and level of the next reply (`src/reply.rs`, its rules `src/rules/reply_pick.rs`), the
priority it sets and what it may do (`src/chips.rs`), where the thread runs and the thread's own
menu (`src/strip.rs`, the place's rules `src/rules/place.rs`), and the strip's other presses.

**The model and how hard it thinks.** The reply box's model chip opens the New thread box's own
model menu (`compose.rs`), raised out of the chip eight points over it, its left edge on the chip's
(`ReplyModel.tsx`, which draws the card's `ModelMenu`): the models used lately, each with its agent
where this Mac offers more than one, All models, and the level strip at its foot. It opens with the
keys on the model the reply is on; the arrows walk it around both ends, Enter picks, Escape closes
it and puts the caret back in the words. The model named at the folded box's right end opens the
box with the menu already open. Then, as in the web window, whose box takes the focus back as it
mounts (asked of the web page itself: the caret in the words, no row lit, ↓ moving the caret), no
row is lit until the pointer moves onto one, which then takes the keys. The chip says the model,
after the mark of the agent the reply goes to (`EngineLogo`, Claude Code's in its orange), with its
level where one will be sent ("Sonnet 5.5 · High", `effortSaid`). A model of another
agent moves the thread to that agent, and a line under the words says it starts fresh
(`freshNote`).

A pick is not written. It is kept beside the box's words until the next reply, which carries it
to `zero:answer`, and the store writes the agent, model and level before the words, so the run
that reply starts reads them (`pickToSend`, `answerItem` in `main/store.mts`): Claude Code's model
always named, the other agents' defaults by naming none, the agent only where the pick moves the
thread. A pick with nothing typed writes nothing, a picked option of the agent's carries none, and
the folded box goes on naming the thread's own model until a reply carries the pick. Z inside the
three seconds puts the words and the pick back; a refused reply comes back by its words alone, as
in the web window (`restoreFailedDraft`). The reply also says what it may do (below).

**The priority and what it may do.** The reply box's first chip is the priority the next reply
sets, the second what it may do (`PriorityPicker` and `ModePicker` in `DockComposer`;
`src/chips.rs`, its rules `src/rules/reply_chips.rs`). Each opens the web window's `.prio-menu`
over it, 168 points wide, risen eight points from the chip's left edge, kept inside the window,
coming in as `prio-in` does (a fade and a four point rise from 98%, over 0.13 s), the chip
brighter inside a stronger edge while it is up.

- The priority's four levels, each with its bars, open with the keys on the level the chip
  shows, which is the thread's own (Medium for an untagged one, as it runs) until a pick: the
  pointer moving onto a row and the arrows move the wash, Enter or a press picks, and the level
  the chip shows is bold. A pick made with Enter leaves the keys on the chip, where an arrow,
  Enter or Space opens the menu again; one made with a press leaves them on nothing, as the
  pressed row goes with its menu.
- What it may do lists the modes of the agent the thread is on now, whatever a model picked in
  the box would move it to (`claudeCode`): Claude Code's six in its own words (Manual, Accept
  edits, Plan, Auto, Don't Ask, Bypass Permissions), Codex's three (Read only, Auto, Full
  access) on any other agent, Antigravity's threads included. The mode the next reply runs in is
  bold: the box's own, which starts as the thread's (`answerMode`), else on Claude Code the
  project's (`config.permission`), else Auto. No row lights under the pointer or the keys. A pick
  is said in the strip ("Permissions · plan mode on for this thread"), puts the caret back in the
  words, and empties them only of a slash query. The chip says the mode title cased ("Accept
  Edits"), led by its shield.
- Shift+Tab in the words moves the mode round the agent's ring and says so: on Claude Code
  Manual, Accept edits, Plan, Auto, with anything outside the ring (Bypass Permissions, Don't Ask)
  going to Manual, so no key walks anybody into Bypass; on any other agent round Codex's three.
- Nothing is written when a chip is picked. The next reply carries the picks, as the web window
  sends them: the priority only once it was picked (`priority`, 9, 7, 5 or 2), what it may do
  always (`permissionMode`: the box's mode, `null` while neither the box nor the thread has one,
  which clears a mode set on an earlier reply). A reply whose files are saved first carries them
  as they stood at Send. Neither is kept with the draft: folding the box lets both go, and the box
  opened again shows the thread's own, now including what the reply wrote, as the web window's
  box is made anew each time it opens. Z puts the words and the model back, not these.
- A narrow bar gives up the mode chip first, when the three chips at their whole widths no longer
  fit beside the paper clip and Send (`chipsOverflow`): it stays laid out unseen, so it comes back
  when there is room, and Shift+Tab still changes the mode. A box under 460 points drops the
  priority's word, the shield, the model's mark and level, and Send's word for its arrow
  (`@container reply`).
- One menu stands at a time over a conversation: a chip's menu closes the model menu and the
  strip's, and theirs close it. A press outside closes it on its way down and goes on to what it
  was aimed at in the box (the other chip, the model's, the words, Send); anywhere else it only
  closes, as the model menu does.

**The choices' little arrow.** The small arrow in the corner of an agent's choices folds them away
(`ChoicesFold`): the strip goes, and the folded reply box carries a sign after its R, how many
choices and an arrow up, whose press brings them back without opening the box (`ChoicesShow`). A
fold holds one thread's current round of choices, kept from one window to the next with the
other picks (`foldKey`, the newest hundred): another thread, or the same thread's next round,
opens unfolded. The keys reach the choices either way, and one picked out with an arrow brings
the strip back while it is picked.

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
Priority with the level it runs at said in words beside it, not as a key (`.ts-more-detail`),
which turns the menu to the four levels, the one it is at ticked. A level picked is written at once (`zero:answer` with the level and no words, which starts
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
| `thread-actions` | 1280 by 760 | 0.70 | 0.25 | 0.63 |
| | 900 by 760 | 0.84 | 0.25 | 0.65 |
| | 1600 by 1000 | 0.75 | 0.25 | 0.65 |
| `thread-actions-priority` | 1280 by 760 | 0.69 | 0.08 | 0.62 |
| `thread-place` | 1280 by 760 | 0.68 | 0.24 | 0.67 |
| | 900 by 760 | 0.86 | 0.24 | 0.51 |
| | 1600 by 1000 | 0.74 | 0.24 | 0.67 |
| `thread-reply-model` | 1280 by 760 | 0.65 | 0.22 | the chip 0.64 |

Measured on the web window's newer look, whose menu says Priority's level in words (the detail,
12.5 points and faint, rather than a key's cap) and whose reply chip leads with the agent's mark.
The page under them is the waiting thread's, 0.69 with nothing open (`thread-options`); its
options and reply bar are 0.58. The screens before this did not move, to the thousandth: Home
0.844, the Inbox 0.240, a conversation 0.409 and as first opened 0.389, the Later picker 0.216,
the card 0.221, the command bar 0.146, the waiting thread 0.679. The reply box opened with R
(`thread-reply`) went from 0.435 to 0.432, its three chips' words now landing where Chromium lands
them.

Filmed at 60 frames a second, the menu, the four levels and the panel each land whole on the frame
after the press lets go or the key, close the same way, and every frame after is the same to the
pixel, as the web window's do. Rune counts no frame of the menu's film as animating, and one where
a place is picked: a colour landing at once (the picked place's fill), with no source named and
nothing in motion. The reply box's model menu comes in as the web window's does, over six frames
("What moves on its own"), and closes at once. A press on the dots or the place button acts on its
way up, as the web window's click does.

Against a scratch engine (above), each pick written by this window beside the web window's own
payload for the same pick, sent by hand to a second thread, wrote the same lines, in order:

- Sonnet 5.5 and High picked in the reply box, then a reply sent: `{"model":"sonnet"}`,
  `{"effort":"high"}`, then the words. Picked and folded with nothing typed: no line at all.
- Codex's own picked on a Claude Code thread: `{"engine":"codex"}`, `{"model":""}`,
  `{"effort":""}`, then the words, the box having said "Codex starts fresh with this thread."
- High picked in the menu: `{"priority":7}`, and "Priority: high" in the strip.
- Later in the menu, then Enter: `{"snoozedUntil":…}`, thirty minutes on, as L does.
- Z inside the hold after a reply with a pick: no line, and the words and the pick back in the box.

The reply box's chips have their own scenes (`tools/scenes/reply-chips.mjs` answers the page's
settings with invented Codex and Antigravity models and puts every thread on one agent):
`thread-reply-priority` and `thread-reply-mode` (each chip's menu open, the project on Bypass
Permissions), `thread-reply-mode-codex` and `-antigravity` (the menu on those agents),
`thread-reply-mode-moved` (a waiting thread, a line typed, the next reply moved to Antigravity's
model at High, its note under the words, the mode menu open), `thread-reply-picked` (High and
Plan picked, the line in the strip), the pointer resting on a row of each menu
(`thread-reply-priority-hover-row`, `thread-reply-mode-hover-row`), `thread-reply-compact` (the
moved one with the rail kept open, photographed at 720 by 760), and the folded choices
(`thread-choices-folded`, and `-picked` with an arrow pressed). A fixture says what was open or
picked (`actions`: `reply-priority`, `reply-mode`; `replyPicked`, `replyPick`, `choicesFolded`).
At 2x, dark, the menu's box (`.prio-menu`), the chips, the box and the bar:

| Scene | Window | Whole | The menu | The chips | The box | The bar |
|---|---|---|---|---|---|---|
| `thread-reply-priority` | 1280 by 760 | 0.43 | 0.33 | 0.72 | 0.41 | 0.50 |
| | 900 by 760 | 0.57 | 0.33 | 0.72 | 0.41 | 0.49 |
| `thread-reply-mode` | 1280 by 760 | 0.45 | 0.38 | 0.41 | 0.49 | 0.52 |
| | 900 by 760 | 0.59 | 0.38 | 0.41 | 0.51 | 0.53 |
| `thread-reply-mode-codex`, `-antigravity` | 1280 by 760 | 0.44, 0.44 | 0.39, 0.39 | 0.40, 0.40 | 0.47, 0.49 | 0.49, 0.53 |
| `thread-reply-mode-moved` | 1280 by 760 | 0.69 | 0.38 | 0.41 | 0.56 | 0.65 |
| | 900 by 760 | 0.92 | 0.38 | 0.41 | 0.59 | 0.67 |
| | 720 by 760, the mode chip folded | 1.19 | | 0.34 | 0.67 | 0.69 |
| `thread-reply-compact` | 720 by 760 | 5.16, the kept-open rail's places | | 0.10 | 0.75 | 0.76 |
| `thread-reply-picked` | 1280 by 760 | 0.47 | | 1.14 | 0.63 | 0.57 |
| `thread-choices-folded` | 1280 by 760 | 0.67 | the sign 0.12 | | the folded box 0.28 | |
| `thread-choices-folded-picked` | 1280 by 760 | 0.71 | the strip 0.81 | | 0.71 | |

Under the pointer a priority row is 0.20 (0.17 at rest) and a mode row 0.20 (0.20), the
pointing hand over both, as the web window's. With a menu open, every point of it and of the
box's chips, words and Send takes the web window's pointer (the pointer walk finds none apart);
the page around them shows the arrow, as with the box's other menus. Filmed at 60 frames a second a chip's menu comes in
over four frames after the press lets go and closes at once. The screens before this did not
move: `thread-reply`, `thread-options` and `home-team` are drawn to the same pixel as the build
before it.

Against a scratch engine, two threads written down for later so nothing ran, each reply sent by
this window (its box opened with R, the chips pressed, Send pressed) beside the web window's own
payload for the same picks sent by hand to the other thread, wrote the same ledger lines and the
same kept mode (`answer-modes.json`):

- "Ship the fix." with Plan and High: `{"answer":"Ship the fix."}`, then `{"priority":7}`, and
  `plan` kept for the thread. Opened again, the box read High and Plan.
- "Go on." after Shift+Tab, the priority untouched: `{"answer":"Go on."}` alone, and `auto`
  kept (Plan's next on Claude Code's ring).
- On two Codex threads, "Go on." with Read only and Urgent: `{"answer":"Go on."}`, then
  `{"priority":9}`, and `read-only` kept.

The engine there has no cloud set up, so it draws no place button and the panel was not pressed
against it. Its three writes were sent by hand, as `place::writes` makes them: keeping the thread
here answered `{"ok":true}` and the supervisor kept it (`threadOnMac`), letting the cloud have it
answered the same and moved it to `threadToCloud`, and a move with no run to move answered
`{"moving":0}`, on which the panel says the cloud cannot take it. Picks in the panel were pressed
only on pictures, where nothing is sent.

## The Inbox's View and filters, and the cloud's menu

The two square buttons at the right of the Inbox's head strip open the web window's menus under
them (`DisplayMenu` and `CloudButton` in `renderer/src/threads/Pages.tsx`), drawn from its
measurements of each (`src/head.rs`; the rules `src/rules/display.rs` and `src/rules/cloud.rs`).
Each hangs forty points under its button's top, its right edge on the button's. One is open at a
time and only over the Inbox: a press on its button closes it and one on the other button opens
that one instead, a press anywhere else closes it on its way down, Escape closes it, and leaving
the page closes it. Every other key goes on to the page, as the web window's menus take only
Escape (`useOutside`): J and K walk the list under the open menu and Tab turns its tabs, the
menu's foot counting the new tab. The button whose menu is open keeps the sky's dark glass, its
mark in ink.

**View and filters.** For one person it is View, Sort by, then Priority, Project (where there is
more than one), Sent and the foot; People, Privacy and Group by are a team's. A pick changes the
list at once and leaves the menu open (`setInboxDisplay`):

- Sort by Priority runs Needs you and Later by the project order, then the level each thread
  shows; Sent runs them newest sent first, a pinned thread first under either. In progress and
  Done keep their own order, and the line says so beside the choice (`order`).
- The filters keep the levels picked, the projects picked and what was sent today or this week
  (`keeps`); nothing picked keeps everything. The row that says nothing is running stays
  whatever they say. They thin every list a screen draws from the Inbox, as the web window's do:
  the tabs and their numbers, Home's Needs you and its count, the rail's count, and J and K inside
  a conversation with its strip's arrows. Home's In progress card lists the whole tab under a
  number counting what the filters keep, as the web window's does (`homeProgress`,
  `progressCount`). The strip over a conversation still names the tab that holds a thread the
  filters hide.
- The Project line offers the eight projects used last, a picked one always among them, and the
  rest behind Show all (`projectChoices`).
- The foot says how many of the open tab's threads show of how many it holds, and offers Clear
  filters while one is on; the display button wears the accent dot then (`isFiltered`). A tab
  the filters emptied says so, and how many they hold back, with a button that clears them
  (`FilteredEmpty`), and Later stays on the strip while it holds anything at all.
- The board is the web window's: Board and B say so in a line, and the list stays.

The display is this window's own, kept in `native-picks.json` beside the picks of the box a thread
is started in (`src/picks.rs`) and read when the window opens. The web window keeps its own
(`threads.display.inbox`), which the door does not hand over, so each window opens on its own. A
fixture's display is its `display`, the web window's own shape.

**The cloud's menu.** While no account can run in the cloud it is an invitation with one button.
Otherwise: a line about the setup when one is needed (an account that was not added with Try
again, one being added with the mark of a wait, one that stopped working with Fix), On and Off
with ⌘C beside them, what on means, the runs on this Mac the cloud could take now with Move them
(or that they are moving, with Everything), the accounts with Manage, the month's agent-hours, and
the line at its foot (what runs there, or why threads run on this Mac: the cloud resting, its image
too old, this Mac's Claude not signed in there, the account at its limit). Opening it on an account
that stopped working counts that account as seen once it closes, and the button stops asking for
its sign-in (`seenAfter`); this window keeps which, beside the display.

A pick is written at once, on the web window's own calls (`HeaderActions`' props in
`renderer/src/App.tsx`), and the menu says the new state once the engine does: On and Off to
`zero:settings-set-workspace` `{ key: "cloudRuns" }` (On to what on means), what on means to
`{ key: "cloudWhenOn" }`, Move them to `zero:move-to-cloud` with the runs' ids, and a line says
how many go once the engine says (`moveToCloud`). Try again lets the failed sign-in go and starts
it again (`zero:cloud-signin`), then opens the setup. The setup is the web window's own screen
(`components/CloudSetup.tsx`): Set up the cloud, Manage, Fix and Try again ask the app to bring its
window forward on it, through a channel only the desktop app's door answers
(`zero:open-cloud-setup-here`, `main/local-door.mts`, which takes nothing from the caller and
opens that screen and nothing else). An app without that channel says so in a line. The month's
hours are Agentbox Cloud's own count where its server says it (`zero:cloud-account-plan`), read
when the window opens and each time the menu does.

⌘C turns the cloud on from anywhere a key reaches, with the web window's line ("Cloud on:
threads that don’t fit here start in the cloud"), and asks before turning it off, as both ⌘C and
⌘K were pressed by accident (`CloudOffAsk`): Enter turns it off, Escape or a press outside keeps
it on. It is Copy instead while words are selected, a box has the keys or something stands over
the page (`cloudChord`), and nothing on a Mac with no cloud set up.

The scenes, each the web window photographed with the menu open (`inboxMenu` in the fixture):
`inbox-display` (as the page opens), `inbox-display-filtered` (Urgent and Low, one project, Sort by
Sent), `inbox-display-done` (over Done, drawn with `--tab done`), `inbox-filtered` and
`inbox-sorted-sent` (the list under them, closed), `inbox-filtered-empty`; `inbox-cloud-menu`
(on), `-off`, `-move`, `-all`, `-again`, `-adding`, `-failed`, `inbox-cloud-invite`, and
`inbox-cloud-off-ask` (⌘C pressed, `cloudOffAsk`). And four with the pointer resting on a
choice: `inbox-display-hover-choice` and `-hover-on`, `inbox-cloud-menu-hover-manage` and
`-hover-off`. At 1280 by 760, 2x, dark, on the web window's newer look, the menu's own box and
the whole window, where the web window's rail still has the places this one left out (the Inbox
with nothing open is 0.27):

| Scene | The menu | Whole window |
|---|---|---|
| `inbox-display` | 0.33 | 0.29 |
| `inbox-display-filtered` | 0.37 | 0.21 |
| `inbox-display-done` | 0.34 | 0.19 |
| `inbox-cloud-menu`, `-off` | 0.33, 0.32 | 0.29, 0.29 |
| `inbox-cloud-menu-move`, `-all` | 0.37, 0.33 | 0.30, 0.29 |
| `inbox-cloud-menu-again`, `-adding`, `-failed` | 0.31, 0.32, 0.36 | 0.29, 0.29, 0.30 |
| `inbox-cloud-invite` | 0.38 | 0.29 |
| `inbox-cloud-off-ask` | the card 0.58 | 0.13 |
| `inbox-filtered-empty` | the page under the tabs 0.15 | 0.19 |
| `inbox-filtered`, `inbox-sorted-sent` | | 0.18, 0.27 |

A worded cloud button wearing the dot ("Sign in again", "Cloud") leaves the dot a corner of its
own, 20 points of room on its right rather than 14 (`.th-cloud.named:has(> i)`): those states of
the button are 0.27 whole, as the others are.

At 900 by 760 and 1600 by 1000 the two menus are 0.33 and 0.37 again. What is left is glyphs, the
extent of the soft patch of the sky's dark every button of the strip wears (`.topbar :is(h1,
button)`'s text shadow, which reaches the menus' buttons, and here `inbox::shaded`), and on the
question its keys' caps, in Geist Mono where the web window's are the system's monospace.
`tools/rows.mjs` holds every one of these scenes' rows, tabs with their numbers and cloud button
against the web window's: none differs, the filtered and sorted ones included (`--rows` reads
the fixture's display). Under the pointer, each choice measured in its own box is within half a
point of 255 of the same box at rest: High 0.68 (0.58 at rest), Priority while on 1.59 (1.36),
Manage 2.05 (1.60), Off 1.18 (0.93). Every point inside a menu takes the web window's pointer,
the arrow on its ground and the pointing hand on its buttons, but one at a button's very edge.

Filmed at 60 frames a second, each menu lands whole on the frame after the press lets go, a pick
changes the list on the frame after, Escape closes on the next, and ⌘C's question comes and goes
the same way. Rune counts no frame of View and filters' film as animating, and one where the
cloud's menu opens and one where the question comes up: a colour landing at once (the cloud's
mark, the scrim), nothing in motion.

Against a door on loopback that answers an invented inbox on a cloud and records what this window
writes, pressing Off, Everything, On, Try again, Move them, then ⌘C and Enter, then Manage wrote,
in order: `cloudRuns` `mac`, `cloudWhenOn` `all`, `cloudRuns` `full`, the sign-in let go and
started again on no account, `zero:open-cloud-setup-here`, the two runs' ids to
`zero:move-to-cloud`, `cloudRuns` `mac`, and `zero:open-cloud-setup-here`. A scratch engine took
each of those settings and said them back on its snapshot's `cloud` (Off keeps what on means,
When on takes effect at once while on, and waits while off), answered a move with nothing to move
`{ "moving": 0 }`, and has no `zero:open-cloud-setup-here`, a 404 the window says in its line. A
filter picked in a window given `--picks` was in that file, and a window opened after on the same
file drew the dot.

## Starting a thread

There is one box to start a thread in, drawn in one of two places, as the web window draws its one
composer either in Home's slot or in a dialog (`ThreadComposer.tsx`, `drawnIn` in `App.tsx`):

- **On Home** it is open under the greeting (`src/compose.rs`, `inline`). N, a press on it, or a
  letter that is none of the list's own keys puts the caret in it.
- **Anywhere else** N, and New thread on the Inbox's strip, open it as the **New thread card**
  over the page: the page darkened under the command bar's own scrim, no longer blurred under a
  brown veil of its own (`.modal-backdrop.tc-veil`), the card 680 points wide a tenth above the
  middle, its head saying who the thread is to (where someone besides the agent could get it) and
  on which model, each named in the card's own faint type, then the words, then its bar. While
  the card is up Home's slot is empty, as in the web window. The empty Inbox has no box of its own
  here, so N opens the card there too.

Both hold the same words and picks: whatever was typed on Home is in the card, and the other way
round. Escape, or a press outside the card, closes it and keeps every word.

Under the words: the model (a chip on the page, the head's Model on the card), each led by its
agent's mark (Claude's in its own orange, `EngineLogo`), the project and the level, the paper clip,
and Send with the caret beside it, which dims with Send while there is nothing to send. Send, the
caret and the clip are the chips' 28 points, so the bar is one line of them, six points apart. On
a team's Home, where the thread could go to someone besides the agent, the box's bar is the quiet
one (`quietBar`): To is the agent's face alone, the model and the project follow, then the room
left, then a level nobody picked as its mark alone, the clip and Send, four points apart. On the
card To and the model's word carry their mark beside them, part of the button. Each opens the
web window's menu, built from its measurements: the models used lately, each with its agent where
there are two, then every model in a column for each agent, and how hard it thinks at the foot;
the projects in the person's order with New project last; Agent decides and the four levels; and
under the caret, Add it to Later. On the page every menu drops under its chip; on the card the
model's drops from its word and the others rise out of the bar. A menu is kept inside the window as the web window keeps
it (`fitMenu`), opens with the keys on what is picked, and is walked by the pointer and by ↓, ↑
and Enter around both ends. Escape closes it and leaves the keyboard on its chip, where an arrow
opens it again and a letter goes to the words. A press outside it closes it on the way down and
goes on to what it was aimed at in the box: another chip opens its menu, the words take the caret,
Send sends.

Send, or ⌘Return, writes the thread at once, one `zero:compose` as the web window's
(`api.compose`): the project, the title and body split out of the words by the web window's rule
(`splitMessage`), `kind: directive`, the level as a number (nought for Agent decides), the Claude
model always (another agent's only when moved off its default), the agent where this Mac offers
more than one, the effort only when one was picked, and `start: later` when Later was. The box
empties at once and the card closes; a line says where it is going, then names the thread with O
to open it and Z to take it back, which closes it and stops its agent unless the agent has
already answered. O still opens that thread for two minutes after the line has gone, unless Z took
something back since (`opensTheLastToast`, `O_STILL_OPENS_MS`): it is usually pressed within half
a minute of a send and sometimes after a minute or more. Z hands the words back and opens the card
on them. A send the engine refuses
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
`home-team` is Home's box with its quiet bar. At 1280 by 760, 2x, against the web window's newer
look:

| Scene | Whole window | Where it changed |
|---|---|---|
| `home-typed` | 0.91 | the box 2.43 (Home's own box is 2.03: see the glass below) |
| `home-team` | 0.91 | the box 2.01, its bar 2.09 |
| `home-menu-project` | 0.78 | the menu 0.27 |
| `home-menu-priority` | 0.77 | the menu 0.28 |
| `home-menu-model` | 0.74 | the menu 0.23 |
| `home-menu-models` | 0.72 | the menu 0.16 |
| `home-menu-model-engines` | 0.75 | the menu 0.29 |
| `home-menu-models-engines` | 0.66 | the menu 0.15 |
| `home-sending` | 0.87 | the line 0.86 |
| `home-sent` | 0.85 | the line 0.35 |
| `compose-card` | 0.21 | the card 0.28 |
| `compose-card-model` | 0.19 | the card 0.15, the menu 0.22 |
| `compose-card-priority` | 0.21 | the card 0.24, the menu 0.30 |
| `compose-card-team` | 0.19 | the card 0.14 |

Before following the newer look, Home with its box was 2.9 to 3.1 against it and the card 11.0
to 11.6, most of that the veil. At 900 by 760 `home-menu-model` is 0.86 and the card 0.21; at
1600 by 1000, 0.58 and 0.15.

Filmed at 60 frames a second, the card and a send each land whole on the frame after the press and
stay still, and a menu comes in over six frames as the web window's does (`tc-in`, "What moves on
its own"). Against a scratch engine (above), a thread typed on Home and sent wrote
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
glass below); and the files alone. Against the web window's newer look, whose box is four points
shorter (its bar one line of 28 point controls), so Home gives the box its room at other heights
(`give_way`'s test holds the new ones):

| Scene | Whole window | Box or card | Attachment row | The files |
|---|---|---|---|---|
| `home-attached` | 0.91 | 2.44 | 1.85 | 0.35 |
| `home-attached-picture` | 0.82 | 1.75 | 1.57 | 0.46 |
| `home-attached-too-big` | 0.91 | 2.33 | 1.90 | 1.29, and 2.32 the line saying why |
| `compose-card-attached` | 0.22 | 0.24 | 0.12 | 0.38 |

Measured against the older look, at 900 by 760 the three on Home were 0.93 to 1.05 whole (Home
with words and no file 1.08 there, and 1.08 again against the newer), their rows 1.25 to 1.44 and
their files 0.29 to 0.44, the lit chip 1.23; at 1600 by 1000, 0.72 to 0.76 whole (0.71 with no
file, 0.73 against the newer), rows 1.66 to 1.73, files 0.30 to 0.45, the lit chip 1.25. The card
was 0.26 and 0.17 whole, its row 0.12 and its files 0.38 at both. The web window's caret was
between blinks when `home-attached-picture` was photographed, and this window draws its caret.

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

### Files in a reply

A conversation's reply box takes files as the web window's does (`DockComposer` in
`renderer/src/components/Focus.tsx`), with the same code as the box a thread is started in:
`src/attach.rs` holds the files of either box (`Holder`), and the reply box's are kept with its
words (`ReplyFiles`).

- **What attaches**: ⌘V in the words, a picture on the clipboard as `pasted-<number>.png` and
  files copied in Finder as themselves, while words on the clipboard paste as words; files
  dropped anywhere on the open box; and the paper clip, the system's picker. A TIFF or a HEIC
  is turned by the engine as it comes in, and is a chip with its name until it has been.
- **Where they stand**: in the same row, between the words and the bar, eight points under the
  words and flush on the bar's hairline, eighteen in from the box's edge (`.attach-row` in
  `.dock-card`); the line saying why one cannot go under it. Leaving the conversation keeps them
  in its box, with its words, for the next visit, and so does a restart (`drafts.rs`). Folded, the box says what it holds: the words,
  on one line and dimmer, and the files counted beside them ("2 files", "1 image"), or the count
  alone where nothing was typed (`foldedReply`, ported as `src/rules/folded.rs` with its tests).
  Escape in a box holding words or files lets go of it and leaves it open, as the web window's
  does; the next Escape folds it.
- **Send**: lit for words or files. A file still being turned holds the reply and says "… is still
  being attached."; one that cannot be attached, or is over the 1 GB a message can send, holds it
  and says why in the web window's words ("Not sent: … Your draft has been kept."), where the box
  a thread is started in keeps Send off instead, as in the web window. Otherwise the files are
  saved into the thread's project first (`zero:save-attachments`), Send saying "Sending…" off
  meanwhile; then the reply is held its three seconds as any other is, its words with a link to
  each file under them after a blank line (`[words, attachedMd].join('\n\n')`). Z gives back the
  words and the files themselves, not the links; a refusal of either write keeps or puts back
  the whole draft.

The scenes: `thread-reply-files` (words, a picture pasted and a report picked), `thread-reply-files-too-big`
(a film of 1.2 GB) and `thread-reply-files-folded` (the first, folded by a press on nothing). A
fixture says what the box held, in the order it came (`replyFiles`: an entry with a `png` is a
picture pasted, one with a `path` a file dropped) and whether it was folded (`folded`). At 1280 by
760, 2x, on the web window's newer look, the reply box with what it carries and the bar
(`.dock-card`), and the folded box (`.dock-pill`):

| Scene | Whole window | The box |
|---|---|---|
| `thread-reply-files` | 0.53 | 0.78 |
| `thread-reply-files-too-big` | 0.53 | 0.88 |
| `thread-reply-files-folded` | 0.43 | 1.43, the folded box |

Every part stands where the web window's does, the chips with the agent's mark and the mode's
shield and the paper clip's square of 28; what differs is glyph edges and the picture's copy,
resampled. A conversation with people draws what is attached as tiles of one height
(`AttachRow`'s `tiles`); a reply to an agent, the only one this window has, keeps the chips. Filmed at 60 frames a second, a picture pasted and a file dropped each land
whole on the next frame, the box growing with them, with nothing animating.

Against a scratch engine, a reply typed, with a screenshot pasted and a PDF dropped, wrote
`Use this layout for the receipt.` and under it `![pasted-64740.png](attachments/…-pasted-64740.png)`
and `[week-32.pdf](attachments/…-week-32.pdf)`, both files in the project's `attachments/` folder
with the bytes pasted and dropped; once it was written the conversation drew the reply with the
picture's place under its words, and the strip's Files button counted two. A reply with a HEIC
photo dropped came in as `IMG_0042.jpg` and was
taken back with Z inside the three seconds: no answer was written, and the box opened on its words
and the photo.

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

A row leads with the web row's own line drawing, 16 points in the faint ink and in the text's own
ink on the lit row, which is a wash with no accent down its edge; Esc is a key's cap, as a row's
key is, and nothing stands under the list, where a foot once said what the arrows and Return do.
The drawings are the web window's, path for path (`src/palette/marks.rs`, from `ICONS` in
`renderer/src/palette-glyphs.ts`), and which row wears which is its rule, by the row's id and the
start of it (`icon_for`, from `iconFor`). Every row of the web's whose doing this window has is
here, under the web row's own name, drawing and key or hint, when the web offers it and where it
puts it, doing what it does, through what this window's own key, button or page already runs or
on the web row's own channel; `docs/parity-command-bar.md` lists every row of the web's, with
what this window does of each and why a row is left out:

- New thread (N), which opens the New thread card.
- On the thread under the keys (the open conversation, else the Inbox's row under the pointer,
  else its picked row, the one E acts on; Home has none), in the web's order: Approve recommended,
  Approve as proposed or Approve and run, by what the agent asks, held three seconds as an answer
  is; Run now on a thread waiting its turn, and what the engine says to it; Mark done (E), Done on
  an outside agent's row, or Reopen on a finished thread, written at once and taken back by Z
  ("Reopened: … · press Z to undo", `reopenDone`); Reply (R); Resume this agent, where Try again
  would not stand for it; Mark read on an unread thread, told to the web window as a phone tells
  it; Back to Inbox (E) or "Let it run now"; Later (L); View code changes on the open thread
  while its strip draws the code button, which opens the change; Pin thread or Unpin thread; where it runs,
  in the place button panel's words with the place said as its hint; three models, the other
  agent's first where this Mac offers it ("Sonnet 5.5 picks this up next"); Move to each other
  project, or Move to another project saying why it cannot, the move taken back by Z and the
  copy opened when it was moved from inside; Send back to agent on a blocked thread; Restart the
  agent on a stopped one; Stop Agent or Stop Task (Moves to Inbox) on a thread under way, which
  stops it as Stop in its box does; Open (↵) from the Inbox; and the four levels, "(current)" after the one
  it shows, each written even when it is that one.
- Then Projects… and Rank each project first, with where it stands; on a Mac set up for the cloud,
  Turn the cloud on or off (⌘C), off asking first and offered only to words that name the cloud,
  and each meaning of on with "On now"; Go to Home and the Inbox's tabs; Go to Files and Files in
  each project the place lists, the list read as the bar opens; Restart to update while a version
  waits, Move to Applications where the app cannot update itself, Check for updates while none
  waits, and the restart a build's main process needs, which does nothing and leaves the bar
  open; Pause or Unpause agents and Resume interrupted agents; Usage and limits (Claude Code's
  page), Settings… (General) and Keyboard shortcuts (the web window's page, which comes forward
  on it, as Settings here sends it); and the key hints switch.

What is said once the engine answers is the web window's own sentence for that answer.

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

Against a scratch engine, through a door that writes down each call, every row that writes was
pressed and wrote the web row's own payload (the table in `docs/parity-command-bar.md`): the
level, the model, the pin and its place, the move and its Z, Send back, Reopen and its Z, an
approval after its three seconds, Resume this agent, Mark read, ranking a project first, pausing
and letting the agents go, resuming the interrupted ones, the update check, keeping a thread on
this Mac and letting the cloud have it, and the cloud's three rows, off through ⌘C's question.
The ledger lines of the level, the model, the move and its Z, Send back and Reopen and its Z were
the same as those the web's payloads left, sent by hand to a second thread.

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

Pressing a picture, a folder's picture or a mention of a picture, a film, a PDF, a page or a deck
opens it big over the window ("A file opened big", below); a mention of a document, a table or a
script opens it beside the conversation ("A file opened beside the conversation"); with ⌘ held,
as a browser opens a link in a new tab, it opens with the system instead (`asksForANewTab`), and
so does any other file and a folder's name (`open`, through `files::open_with_system`). A file
macOS would run is shown in Finder instead, as the web app does (`main/opens-a-program.mts`).

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
list and opens it as the web window's list opens it (`opensBig(f.src) ? openBig : onOpenDoc`,
here `pane::open_listed`): what the big view looks at, big, and a document, a table or a script
beside the conversation. Escape closes the list, and so
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

At 2x, dark, on the web window's newer look (its strip 64 points, the button 32 tall inside the
strong line); the strip is the card's head, the button and the list the page's own boxes:

| Scene | Window | Whole window | The strip | The button | The list |
|---|---|---|---|---|---|
| `thread-files` | 1280 by 760 | 0.46 | 0.50 | 0.49 | |
| | 900 by 760 | 0.79 | 0.57 | 0.37 | |
| | 1600 by 1000 | 0.41 | 0.42 | 0.46 | |
| `thread-files-open` | 1280 by 760 | 0.46 | 0.54 | 0.64 | 0.49 |
| | 900 by 760 | 0.74 | 0.63 | 0.57 | 0.49 |
| | 1600 by 1000 | 0.43 | 0.46 | 0.58 | 0.49 |
| `thread-files-many` | 1280 by 760 | 0.77 | 0.51 | 0.70 | 0.61 |

A strip with no files is 0.49 at 1280 by 760 (`thread`). Most of what `thread-files-many` differs
by is its answer's fourteen mentions, whose underlines sit two points higher (see what still
differs).

Filmed at 60 frames a second, the list opens on the frame the press lets go, closes on the frame
of Escape or of a press outside it (on the press's way up for the button itself), and every
frame after is the same to the pixel; Rune counts no frame as animating. A press on a file in a
script says what a window would have opened (`a press opens …/notes/rounding-v2.md`).

## A file opened beside the conversation

A document, a table or a script pressed in a conversation opens in a pane on its right, the
conversation narrowed to what the file leaves it, as the web window's does (`DocPane.tsx`, laid
out by `App.tsx` with `artifact-layout.ts`, `workspace-navigation.css` and `team/rooms/rooms.css`;
`src/pane.rs` here, its rules `src/rules/pane.rs`, ported with the web's test cases, and a
sheet's grid `src/table.rs`).

**Which press opens what** (`whereAFileOpens`, `whereALinkGoes` and `openHref` in `Focus.tsx`,
the Files button's `onOpen`, `onBeside`, `besideDocument`):

| Pressed | The web window | Here |
|---|---|---|
| A mention of a document, a table, a script or any text | beside, at its newest edited version | beside, the same |
| A mention of a picture, a film, a PDF, a page or a deck | big | big |
| A picture drawn in a message, a folder's picture | big | big |
| A folder's name | the system | the system |
| A row of the Files button's list | big for what is looked at, beside for the rest | the same |
| A tile of the Files place | big | big |
| The big view's corners, drawn inward, over a file from a thread | beside, its newest version | the same |
| Any of them with ⌘ held | the system | the system |
| An address on the web or a local app's | beside, in a browser of the pane's own | the browser |
| A file of any other kind | the system | the system |
| Nothing: the newest answer sets a document, a page, a table or a PDF on a line of its own | beside, by itself | the same |

**The pane.** The body under the strip is shared by the split kept (where it opens before a hand
has moved it, 0.72 of it), held so that the conversation keeps 400 points and the file 560
(`artifactFraction`): at 1280 by 760 the file takes 776 points of 1176, at 1600 by 1000 1077 of
1496. Its card stands twelve points in all round. A document or a script is on the glass card
(`.doc-md`): an edge of `--line-strong` rounded to twelve, a ground that fades at 145° from
the text's colour at 5% to the page's at 25%, a highlight of a point along its top, and a head 53
tall, sixteen in, with a line under it. A picture and a table stand on the page's own ground, the
picture's card rounded to six and the table's square, their head 52 tall, twelve in and with no
line.

- **The path** (`crumbParts`): from the deepest of the project's folders that holds the file, its
  thread folders left out ("designs › receipt-plan.md"), the last three parts when no folder
  holds it, a file a message carried by its own path; eleven points, the file's name brighter,
  the whole path its title.
- **The marks**, 28 points square six apart, faint, and under the pointer in the text's colour
  with nothing behind them, at once both ways:
  - the globe, where the app says it shares files (`zero:file-link` names a site) and the file is
    a kind a link can draw (`canShareFile`), which opens the web window's Share panel (see what
    still differs);
  - two panes side by side ("Show notes"), on a document, a page or any text (`takesNotesSidebar`),
    which opens the file over the whole window where the web window draws its notes;
  - the frame's corners ("Give this document the full workspace"), which open it big over the
    window (`docFull`); Escape or the big view's close mark come back to it beside the
    conversation, and the big view's own corners, drawn inward, put it back beside;
  - open outside ("Open outside Mercury"), the file in the app that owns it;
  - close ("Close this document (esc)").
- **The file.** A document as the big view lays one out, at the pane's width, under "Edited 3 min
  ago" (`agoWords`, moving on with the minute), its outline's dashes at the right edge opening
  into its contents under the pointer; a script or any text as its lines, numbered and coloured,
  flush in the card (`CodeFile`); a table as its grid (`TableView` in the pane), with its bar
  (Search, Filter, Sort, how many rows), each column marked with what it holds, figures to the
  right and short lists of words as coloured tags; a picture whole in the middle of the card,
  never larger than its own pixels. Each scrolls on its own.
- **The divider** on the card's left edge: nine points to reach for, the resize pointer, and
  under the pointer and while dragged a pill 7 by 30 on it. Dragged, the file takes what is right
  of the pointer (`splitFromPointer`, between a quarter and three quarters of the body), kept in
  this window's picks (`doc_split`) once it is let go; a double press puts it back where it opens.
- **The keys.** Escape puts down a file the person opened; one the thread opened by itself does
  not stand in the way, and Escape leaves the thread (`escapeClosesDoc`). Every other key is the
  conversation's, as the web window's pane takes none: J and K go on to the next thread.
- **Another file** pressed takes the pane's place. **Leaving the thread** takes the file with it,
  and coming back with nothing new said puts it back (`leavePlace`, `placeToReturnTo`); a file
  the thread opened by itself and the person closed stays shut until its next answer
  (`closedBeside`).
- **A card under a thousand points** gives the file all of it (`artifactPlacement`): the strip
  is given up to it, leaving the Inbox, back and forward over the card's top edge, the card has
  no head, and the reply box stands under it twelve points in. The web window draws the marks
  there only over a file sent to the whole window, so neither draws them, and Escape is the way
  out.
- **Followed on disk**: while it is open the file is asked of the disk every three seconds
  (`FOLLOW_FILE_MS`) and read again when an agent has written it since (`newerOnDisk`).
- **The conversation beside it** keeps the page's own sizes rather than the measure that grows
  with the window (`.app:not(.doc-open)`): 44 points in, lined up with the back arrow, 28 from
  the file, its words at 16 points, its title 21 points where it is 560 or less (`@container
  talk`) and 27 where it is wider, six points over the line under it, the newest answer's
  headline at 17. The strip's arrows stand sixteen points further in (`rooms.css`).

The scenes (`tools/scenes/thread-pane*.mjs`) open the invented thread on an answer that names
a plan, a script and a sheet in its sentences and press one: `thread-pane` (the plan),
`thread-pane-code`, `thread-pane-table`, `thread-pane-picture` (a picture pressed big, then put
beside with the big view's mark), `thread-pane-dragged` (the divider dragged by the pointer),
`thread-pane-auto` (the plan on a line of its own, opened by itself), `thread-pane-wide` (a wide
window, a picture in the answer and the divider dragged all the way, laid out as a person reads
a plan beside its thread), and `thread-pane-hover-*` (the pointer on each mark, the path and the
divider). The page reads a file through the app, which the harness does not answer, so each
scene answers it from the planted folder, with the file's own time on disk, which the native
window reads too: photograph a scene and draw it before the next scene plants the folder again.
Each fixture says what was pressed (`docOpen`), where the divider was left (`docSplit`) and what
the app said of sharing (`fileLink`).

At 2x, dark, of 255, the whole window, the conversation beside the file, the card, its head and
the file under the head:

| Scene | Window | Whole | Conversation | Card | Head | File |
|---|---|---|---|---|---|---|
| `thread-pane` | 1280 by 760 | 1.42 | 2.55 | 1.40 | 0.90 | 1.45 |
| `thread-pane-code` | 1280 by 760 | 1.14 | 2.55 | 0.81 | 0.87 | 0.80 |
| `thread-pane-table` | 1280 by 760 | 1.01 | 2.55 | 0.53 | 0.21 | 0.56 |
| `thread-pane-picture` | 1280 by 760 | 0.36 | 0.97 | 0.03 | 0.30 | 0.00 |
| `thread-pane-auto` | 1280 by 760 | 1.03 | 1.07 | 1.40 | 0.90 | 1.45 |
| `thread-pane-dragged` | 1280 by 760 | 0.97 | 0.74 | 1.71 | 0.97 | 1.78 |
| `thread-pane` | 900 by 760 | 1.12 | the reply box 0.56 | 1.43 | | 1.43 |
| `thread-pane-code` | 900 by 760 | 0.68 | 0.56 | 0.78 | | 0.78 |
| `thread-pane-table` | 900 by 760 | 0.49 | 0.56 | 0.52 | | 0.52 |
| `thread-pane` | 1600 by 1000 | 0.97 | 0.89 | 1.25 | 0.89 | 1.27 |
| `thread-pane-code` | 1600 by 1000 | 0.75 | 0.89 | 0.87 | 0.85 | 0.87 |
| `thread-pane-table` | 1600 by 1000 | 0.42 | 0.89 | 0.28 | 0.12 | 0.29 |
| `thread-pane-dragged` | 1600 by 1000 | 0.92 | 0.45 | 1.90 | 0.96 | 1.96 |
| `thread-pane-wide` | 1800 by 870 | 0.62 | 0.29 | 1.61 | 0.95 | 1.66 |

At 900 by 760 the card is all the card has, its head given up (see above). The conversation
is what differs most where its answer names a script: the web window breaks the mention
`round-tax.ts` at its hyphen and this window moves it whole to the next line (see what still
differs). A document's text sits between 1.3 and 2.0, as the text-dense parts of a conversation
do: its lines break where the editor's do, one space short of the reader's, which is what
`white-space: break-spaces` on the editor does to a line's last word. Under the pointer, each mark against the web window's
under the pointer, and the same box at rest: the globe 0.73 and 0.73, the notes' mark 0.61 and
0.61, the corners 0.67 and 0.67, open outside 0.92 and 0.92, close 0.63 and 0.63; the divider's
pill 0.32. The pointer's shape over `thread-pane` matches the web window's at every point of
the pane but the outline's dashes, which open into their card under a moving pointer here and
are probed at rest there (`MERCURY_FEEL`).

The screens with no file beside them draw to the same pixel as before (`thread`,
`thread-folded`, `thread-links`, `thread-pictures`, `thread-files-open`, `file-doc`), and the big
view of a picture from a thread differs only by its new mark (`thread-file-picture`, 2.34 against
the web window, where it was 2.35).

Tests hold the rest on invented files (`src/pane.rs`, `src/rules/pane.rs`, `src/table.rs`): a
document opens beside and leaves the conversation 400 points at 1280 by 760, Escape puts it down
and the next leaves the thread; a picture still opens big, and the Files list opens each kind
where the web window's does; the corners open it big, Escape comes back to it, the notes' mark
opens it big, the big view's mark puts it back and the close mark closes it; the divider is
dragged to where the pointer is, kept, and put back by a double press; leaving a thread takes
the file and coming back puts it back until a new answer; the newest answer's file opens by
itself, Escape then leaves the thread, and once closed it stays shut; a card under a thousand
points gives the file all of it with no head and the reply box under it; a file rewritten on
disk is read again once its time has come round; and the rules' own cases, from the web's tests.

## The code a thread changed

A run that changed code leaves the change it made (`runs/<thread>/the-change-it-made.change`),
and the web window shows it in full: a button in the thread's head strip, the change beside the
conversation, and a chip on each step that edited one of its files (`Focus.tsx`,
`ThreadMenu.tsx`, `DocPane.tsx`, `CodeArtifact.tsx` with `code-artifact.ts`, `code-keys.ts`,
`code-copy.ts` and `code-edit.ts`). Here it is `src/change.rs`, its rules `src/rules/change.rs`
with the web's own test cases, and every behaviour is a line of `docs/parity-code-changes.md`,
ticked with how it was checked.

**The button.** A thread whose change adds or removes a line carries it after the Files button:
the code mark and the figures, `+36` green and `−88` red, each only when not zero, thousands
separated, in Done's frame, lit while the change is open beside the thread. A press opens the
change at its top, and puts it away when it is open. Its figures are the change without its rows
(`zero:code-change` with `summary`), asked when the conversation opens, when the thread's row
changes and when a run starts on it; then every four seconds while a run is up, and for two
minutes after its answer was written, when the folder may still move (`asks_for_the_figures`).
A person's own agent session is never asked.

**Beside the conversation.** The change opens in the pane every file opens in, with its split,
divider and Escape. Its card is flush, on the code's own ground, under a head that says "The
change", with the corners (the whole workspace), "Open this file in your editor" (the file at the
top of the column, found as the engine finds it) and close. While it is read, the skeleton that
holds a document's place; when it cannot be, the engine's reason. It is read once per opening, as
in the web window, which does not read it again while open. Sent to the whole workspace by its
corners, the conversation is hidden and the reply box stays; the marks over it wake when the
pointer moves, sleep 1.8 seconds after, and lie under the card, so only their eight points past
its edge show or take a press. A change a narrow window leaves the whole card to has no marks,
and Escape is its way out.

**The files.** A tree, 232 points down its left, whose head says how many files and lines, "not
committed" while the work is loose in the folder and how long the run took; folders first, a
chain of single folders one row, each folder's count; a press shuts a folder or jumps the column
to a file, and the arrows then walk the files. In a pane 800 points wide or less a picker stands
over the column instead, naming the file at its top. The file at the top of the column is lit.

**The column.** Every file in one running column, its head stuck at the top while its file is in
view and pushed off by the next; a press on a head folds the file, and folds are kept while the
window runs. Each row is its number, its mark and its line, on a green or red bed, the words that
changed between a removed line and its replacement on a stronger bed, coloured as a script is.
Between two hunks, "N lines not shown": a press reads the file through the engine
(`zero:code-file`) and draws the lines in place. ] and [ go to the next and previous change, the
arrows a row, Page Up and Down a screenful, Home and End the ends, and J and K do nothing, as
the web window keeps them from the app's own. A large change builds the rows of the files near
the view and holds the height of the rest, as the web window's unfilled files do, so nothing
waits on the rows out of sight.

**Selecting, typing, saving.** The words select, never the numbers or marks; a drag from the
margin starts at a line's front, a shifted press extends across lines, and ⌘C copies the lines
with no gutter, the removed ones left out unless they are all that is selected, built or not. A
line that is in the file now takes typing as an editor's does: a press puts the caret where it
landed; Return is refused and a paste comes in as one line; the arrows cross to the lines above
and below; a selection over several lines refuses a keystroke. Typed words are kept while the
window runs and mark the file Unsaved on its head, in the tree and in the picker. ⌘S reads each
such file, puts each typed line back exactly where the run left it and writes it through the
engine (`zero:code-file`, `zero:save-code-file`), saying "Saved name.ts, 2 lines." or why not.

**From a run's step.** In an opened run, a step that edited or wrote a file the change holds wears
it as a chip, which opens the change at that file. ⌘K offers View code changes on a thread whose
strip draws the button.

**How it was checked.** Each behaviour is a test on an invented change: 24 on the window
(`src/change.rs`), the rules' own 21 (`src/rules/change.rs`) and the bar's row
(`src/rules/palette.rs`). Two more are ignored, as they need a large change or a running engine.
The save was tried against a running scratch engine (`MERCURY_SCRATCH_DOOR`,
`MERCURY_SCRATCH_TOKEN`, `MERCURY_SCRATCH_DIR`): both hunks written in place, a change whose
folders do not hold the file refused in the engine's words, and a file moved between the read
and the write refused as stale. The scenes `thread-change`, `-open`, `-full`, `-full-marks`, `-chip`, `-hover`,
`-loading`, `-failed`, `-folded`, `-at`, `-gap`, `-typed` and `-shut`
(`tools/scenes/thread-change*.mjs`, the change in `tools/scenes/change.mjs`) are the web window
in each state; a fixture says what the page had done (`changeFolded`, `changeShut`,
`changeTyped`, `changeGaps`, `changeAt`, `docFull`). They plant their files in
`MERCURY_PLANTED`, which the next reference run plants again.

At 2x, dark, of 255, the whole window, the conversation beside the change, the card, the tree or
picker, and the column under them:

| Scene | Window | Whole | Conversation | Card | Tree or picker | Column |
|---|---|---|---|---|---|---|
| `thread-change` | 1280 by 760 | 0.41 | the strip 0.49, the button 0.59 | | | |
| `thread-change-open` | 1280 by 760 | 0.66 | 1.11 | 0.42 | | 0.41 |
| `thread-change-loading` | 1280 by 760 | 0.78 | 1.11 | 0.68 | | |
| `thread-change-failed` | 1280 by 760 | 0.58 | 1.11 | 0.25 | | |
| `thread-change-folded` | 1280 by 760 | 0.71 | 1.11 | 0.53 | | 0.55 |
| `thread-change-at` | 1280 by 760 | 0.71 | 1.11 | 0.52 | | 0.55 |
| `thread-change-gap` | 1280 by 760 | 0.73 | 1.11 | 0.57 | | 0.61 |
| `thread-change-typed` | 1280 by 760 | 0.67 | 1.11 | 0.43 | | 0.43 |
| `thread-change-shut` | 1280 by 760 | 0.66 | 1.11 | 0.42 | | 0.41 |
| `thread-change-full` | 1280 by 760 | 0.39 | the reply box 0.48 | 0.37 | | 0.37 |
| `thread-change-chip` | 1280 by 760 | 1.28 | the thread 1.42, the chip 1.82 | | | |
| `thread-change` | 1600 by 1000 | 0.34 | 0.38 | | | |
| `thread-change-open` | 1600 by 1000 | 0.63 | 0.81 | 0.61 | 0.32 | 0.71 |
| `thread-change-folded` | 1600 by 1000 | 0.63 | 0.81 | 0.61 | 0.28 | 0.72 |
| `thread-change-at` | 1600 by 1000 | 0.65 | 0.81 | 0.65 | 0.31 | 0.77 |
| `thread-change-gap` | 1600 by 1000 | 0.65 | 0.81 | 0.65 | 0.31 | 0.76 |
| `thread-change-typed` | 1600 by 1000 | 0.63 | 0.81 | 0.61 | 0.32 | 0.71 |
| `thread-change-shut` | 1600 by 1000 | 0.62 | 0.81 | 0.60 | 0.27 | 0.71 |
| `thread-change-full` | 1600 by 1000 | 0.36 | | 0.36 | | |
| `thread-change-open` | 900 by 760 | 0.42 | | 0.42 | 0.41 | 0.44 |
| `thread-change-typed` | 900 by 760 | 0.43 | | 0.44 | 0.54 | 0.43 |
| `thread-change-at` | 900 by 760 | 0.46 | | 0.48 | 0.24 | 0.53 |

Under the pointer (`tools/hover.py`), the button 1.06 against 0.58 at rest, and the line the
pointer wakes the whole workspace's marks from 0.22 and 0.22. The chip's step reads 1.82
where the run's opened lines are text set a fraction of a pixel apart; the chip itself, its mark
and its path stand where the web window's do.

The figures differ most where text does: the conversation's prose and the change's mono lines,
whose glyphs sit a fraction of a pixel apart; every row, gap and head stands where the web
window's does. Under the pointer, every point of the button, the tree, the picker, the heads,
the gaps, the lines and the chip takes the web window's shape (`MERCURY_FEEL`); the probe reads
an I-beam in the twelve points under the card, where words clipped by the card lie, and the web
window shows the arrow there as this one does. The pointer tool runs no poll, so a scene that
opens at a file (`thread-change-at`, `-gap`) was walked again with the jump landed.

**How fast.** Measured in a release build on an invented change of 80 files with lines two
thousand characters long (`MERCURY_BIG_CHANGE=<rows> cargo test --release a_big_change_opens_in
-- --ignored --nocapture`), at sizes well past what a run usually leaves.

| Rows | Opened (two frames) | A frame at rest | A frame scrolling, median and slowest | To the last file |
|---|---|---|---|---|
| 5,000 | 14.5 ms | 0.42 ms | 0.66 ms, 5.6 ms | 6.9 ms |
| 20,000 | 14.7 ms | 0.37 ms | 0.82 ms, 8.1 ms | 12.7 ms |

Opened, 100 and 52 lines are built: the files near the view, whatever the change's size.

## A file opened big

A file opens over the whole window, as the web window's big view does (`ArtifactView.tsx`, its
kinds in `ArtifactStage.tsx`; `src/viewer.rs` here, its rules `src/rules/viewer.rs`): the paper
the file lies on, the bar over it and, from Files, the line of places it was sent in under it.

**What opens it.** In a conversation: a picture, a folder's picture, and a mention or a row of
the Files button's list of what is looked at (a picture, a film, a PDF, a page, a deck); what is
read (a document, a table, a script) opens in the pane beside the conversation instead ("A file
opened beside the conversation", `whereAFileOpens`, ported with its cases), whose corners mark
opens it here. In the Files place: a tile, of any kind. With ⌘ held a press opens the file with
the system, as there; an address goes to the browser; any other file opens with the system as
before. A picture in the box a thread is started in still opens with the system (the web window
has a viewer of its own for those).

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
the mark that opens it outside, opened from a conversation the frame's corners drawn inward that
put its newest version in the pane beside the conversation (`onBeside`, "Move it to the side · Esc
closes it"), and the close mark. The web window's comments, Approve and Share are not drawn (see
what still differs).

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
  its contents stand in the right margin, or as a column of dashes where the margin is narrow,
  which open into a card of the contents under the pointer (`DocOutline`); a press on one
  scrolls to its heading. Its two largest headings follow the page's width (`5cqi`, `3.4cqi`,
  between 26 and 34 points and 20 and 23), and a table is the reader's: a frame rounded to ten
  points, the header on a quiet band, the first column heavier, the columns shared out as a
  browser shares a table's (`.doc-prose table`).
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

A long file is read once and laid out once, the stage built again only when the version, the
stage's size or the pictures held change (`rune::memo`). Measure it in a release build on large
files with the ignored test `a_big_file_opens_in` (`MERCURY_BIG=<a fixture> cargo test --release
-- --ignored --nocapture a_big_file_opens`): a script of some fifteen thousand lines and a
document of some fifteen hundred each open in about a quarter of a second, a picture of a few
megabytes in under a tenth, its decoding on the worker after that; every frame after is 1 to
2 ms. A debug build takes about five seconds to open the first two.

Tests hold the rest on invented files (`src/feel.rs`): a picture pressed in a conversation opens
big, takes every key (E closes nothing under it, + and 0 zoom), and Escape and the close mark go
back to the conversation; a mention of a document opens it beside the conversation, and with ⌘
held does not; the rail's Files opens the Files place, a tile opens big from the list and fixture of a Files place,
the place under it opens its thread, and Escape walks back from the thread to Files and from Files
to where it was opened from. No test, and no headless run, hands a file to the system.

## The Files place

The rail's Files, under the Inbox, opens the Files place (`FilesPlace.tsx`, its look
`files-place.css`, its rules `files-shown.ts`; here `src/files_page.rs` and
`src/rules/files_shown.rs`, ported with the web's test cases): every file the threads made, one
tile each however many threads it went to, from the list the app answers (`zero:files-place`,
read when the page opens and again as the store changes, at most every two seconds).

- **The strip**: back and forward, and Files over the Inbox's sky, as the web window draws it.
- **The bar**: the way back (All files, then the project and its folders, each a press away); how
  the files are ordered and drawn, Recent and Name (Name reads figures as numbers and capitals and
  accents as their letters) and Grid or List, with what the files take on this Mac before them
  once it reaches a megabyte (`showsTotal`); and the box that searches (`/` puts the caret in it;
  Escape takes back what is typed, then the keys). Every word must start a word of a file's name,
  its project or its folder, and the files named for a word come before, and instead of, those
  that only sit in a place of that word. Its controls stand 32 points tall, as the strip's do.
- **The kinds**, under the bar while there are two kinds or more or something needs review: All
  and each kind the files here are (Documents, Pictures, Films, Pages, Tables, Code…), and Needs
  review while a file has comments nobody approved.
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

The scenes: `files` (All files), `files-project` (a project's page), `files-list` (the list) and
`files-name` (ordered by name), over an invented list built on the planted folder;
`MERCURY_FILES=<a saved list>` photographs the web window on another list. At 1280 by 760, 2x,
on the web window's newer look (Recent, Name and grid or list in the bar, the kinds under it),
whole windows and each part; the strip leaves out the web window's New, which this window does not
draw (0.95 with it):

| Scene | Whole | The strip | The bar | The kinds | The cards | The tiles |
|---|---|---|---|---|---|---|
| `files` | 0.52 | 0.12 | 0.51 | 0.56 | 0.30 | 0.55 |
| `files-list` | 0.60 | 0.12 | 0.51 | 0.56 | 0.30 | 0.68 |
| `files-project` | 2.74 | 0.12 | 0.60 | 0.56 | | 0.65, and 131.5 the page's tile |

Before this window followed the newer look, `files` was 5.27 against it. At 900 by 760 the two
are 0.69 and 0.84 whole; at 1600 by 1000 the list is 0.43 and the grid 6.47, which is the page's
tile in view: the web window draws the page in it, white, and this window its word. Every part
stands where the browser puts it, the bar's and the kinds' words in the soft patch of the sky's
dark the web window gives them (`.fp-bar`, `.fp-tools button`), which took the bar from 0.79 to
0.51 and the kinds from 0.85 to 0.56; what is left is glyphs, and the rail, whose other places
this window does not draw.

The rail's Files moved the screens at rest a little, as it stands where the web window draws
Demands (the rail itself is 0.52 on each): Home from 0.844 to 0.867, the Inbox from 0.240 to
0.262, a conversation from 0.409 to 0.429.

To check a real list rather than the invented one, read it through the running app's door
(never written), photograph the web window on the same list (`MERCURY_FILES`) and draw this
window on the door (`--view files`): the project cards, the kinds and the first sixty tiles
should come out in the same order with the same names and lines, the web window's harness
having no team to tell "Your agent" from another's, and ordered by name the same. On a list of
a couple of hundred files a release build draws the page first in under a tenth of a second,
its pictures then landing from the worker (the ignored test `the_files_place_draws_in`,
`MERCURY_FILES_PLACE=<a fixture with the list>`). A document's or a script's face is read once
from the top of its file, and read again only when its versions or its size change.

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
  changes"), with how many of them failed in the stopped colour, and its whole tally as its title
  ("24 things it did · ran 18 · read 4 · edited 2", `runSummary`). A press opens it at once onto
  each thing it ran.
- Each of those lines has the web window's three states (`WorkLine`): shut, it is its mark, its
  verb, what it acted on in the code face and "· 8 lines"; a press opens it onto the whole command
  in a block (`.did-full`) and the first six lines of what came back in another (`.did-out`), with
  "show all 8 lines" under them, whose press opens all of them and says under them what the log
  did not keep (`outputCut`); a press on the line shuts it from either. A line opens when
  something came back, when its words were shortened from the command, or when the column cut
  the command, which it is measured for where it is laid out (`clipped`); otherwise it takes no
  press and does not answer the pointer. Under the pointer its verb is the text's ink and its
  mark dim.
- Escape folds back what was opened on this visit, newest first (`putAwayTheNewest`): every fold
  and line opened since the conversation was opened at once, as one step, and the working line's
  steps as another; then the reply box, then the conversation. A file opened beside the
  conversation counts too: what was opened after it goes first, then the file, then what was
  opened before it (`docOpenedAt`). Folding something by hand takes it off the step.
- Where the person was in a conversation is kept (`thread-place.ts`): the earlier messages shown,
  the middle asked back, the folds and lines opened and how far, and how far they had scrolled it
  themselves (the wheel is their hand). Coming back puts all of it back while the thread's newest
  answer is the one it had when it was left, and reaches for the scrolled place while the page
  settles; with news since, it opens as new. The newest fifty are written down with this window's
  picks, so they outlast a restart, as the web window's do.
- A conversation drawn as one (unfolded, from the answer an agent is working from, or short
  enough to be drawn whole) lands where a person starts reading (`holdAtBottom`, `landingTop`): 20
  points over the top of its last message, the outcome when there is one, or at its foot when that
  and all under it fit, and holds there for 2.5 seconds while the page settles, until the person
  scrolls; landed at its foot it stays there as it grows. A page that leads with the answer alone
  opens at its top. Off its foot, what is read stays put when something over it changes height,
  as a browser keeps it (`scroll_anchor`).
- A long conversation keeps its opening and its last forty messages, with a line for the middle
  that asks for it whole (`.thread-gap`, dim and underlined under the pointer); what was under the
  line stays where it is on the screen as the middle comes in over it (`openGap`).
- While an agent is on the thread, the work since anybody last said anything leaves the
  conversation, and the line at its foot says which step is running in the agent's own words,
  how many more run beside it, how long it has worked, and where it runs when that is news: "in
  the cloud", or "on this Mac" while the cloud takes everything, with the card saying why over it
  while the pointer rests there. The sentence the agent is typing this second (`saying`) is
  placed where it began, once. The line is drawn again every fifteen seconds while it counts.
- In every other state the line says why nothing is moving (`liveLine`, the whole of it in
  `src/rules/live.rs` with the web's cases): Asking with the time of the run waiting on a yes;
  Working, its mark turning, for an agent outside this window; Shipping, Up next, Queued and
  Paused; for signed out, at its limit and cannot start the whole sentence ("Claude Code is at its
  usage limit, so this starts on its own when it resets at 3:00pm."); Nothing came back with how
  long ago; Nothing running. Its mark stands still there, every cell at 0.35. Try again follows
  where nothing will start the thread without the person (`offersTryAgain`): a quiet button that
  takes the strong wash under the pointer. Out of In progress, a thread whose newest word is the
  app's note that a run got nowhere gets the one line true of it: Nothing running, and Try again.
  What the line says is read aloud whole.
- A stopped thread draws no line; the bar over its title says so (`.stalled-bar`: "The agent
  stopped without finishing this.", and that the person's answer still waits when it does, "try
  again →" at its end), its edge lit under the pointer, and a press anywhere on it tries again.
- Try again (on the line, the bar, a row and the command bar) is Resume on that one thread
  (`zero:resume-items` with its id), and a line says what came back, as the web window's does
  (`tryAgainWords`): "Trying again now.", "Queued.", or why not; with no app behind the window,
  "Try again did not reach the app." The lists are read again at once. Checked headless against
  a small door on loopback that answered `thread-line-idle`'s inbox: the press wrote
  `{"ids":["w-readme0"]}` and the line said "Trying again now.". On a scratch engine the channel
  answered as the web window reads it (the supervisor's queue, a list, over the count of that
  name, which is why anything that did not start now says Queued there and here).
- That line's words are a press while it works, with a chevron after them that shows under the
  pointer and turns down while it is open (`.live-disclosure`). Open, it lists the steps the run
  that is up took since it last said anything, the newest five oldest first with the rest counted
  ("3 earlier steps since it last said something"), then each call running now, each row its mark
  and its words, which a press opens onto the command behind it, in a block in the system's
  monospace (`stepsSinceLastSaid`, `previousActivity`). Opening it asks for the run's log at once;
  a read that fails says "Couldn’t read the command history." under them until one lands, with
  only the calls running now listed when no read has landed since it opened, and with nothing at
  all to list it says "No commands yet.". Another conversation or another run starts folded.
- A chat just brought in, owed the run that writes it up (`catchUpDue`), says under its title
  "Getting this chat ready. Its summary and options will appear here.", shimmering, over its
  newest answer, as the web window's does (`thread-getting-ready`: 0.69 whole, 1.19 the column
  of words, `.focus-body`).
- Under the title of a thread an agent is on, the line says what was asked: the first paragraph of
  the ask that is not a lead-in ending in a colon ("Priya Shah asked:" says who, not what), a link
  in it read as its words, and nothing when the title already says it (`askLine`, with the web's
  cases); nothing of a cause's ask; and for a run of a repeating task its schedule after it
  ("Repeats weekdays at 9:00am.", `workingContext`, `repeatsLine`), from the rules `zero:repeats`
  lists, read when such a run is open and again when the threads change in number.
- The head strip names the tab that holds the thread now (In progress while an agent is on it),
  and draws the arrows to the threads before and after only for one in Needs you.

`src/rules/notes.rs` is `renderer/src/notes.ts` whole, with its tests: the diary of a run's
sentences and its one-line summaries, which no screen of the web window draws today either.

`docs/parity-run.md` is the checklist of all of this, line by line against the web window's
code, each line with how it was checked or why it is not done.

The scenes (`tools/scenes/thread-line*.mjs`) open `thread-working`'s conversation with the run
taken away, or kept, and the supervisor saying each state (`fleet` in `thread.mjs`'s `open`);
`thread-line-tried` is Try again pressed, `thread-line-hover-*` the pointer on Try again, the bar
and the place. At 1280 by 760, 2x, of 255, whole and the line at the foot (the bar for
`stalled`):

| State | Whole | The line |
|---|---|---|
| Nothing running, with Try again (`idle`) | 0.33 | 0.48 |
| Queued, Up next, Paused, Shipping | 0.33, 0.33, 0.40, 0.33 | 0.26, 0.17, 0.20, 0.21 |
| Cannot start (asleep), Signed out, At its limit | 0.35, 0.34, 0.35 | 1.17, 1.06, 1.46 |
| Working elsewhere, Nothing came back | 0.33, 0.33 | 0.59, 0.59 |
| Asking | 2.67, the web window's approval card over the page | 0.38 |
| Working on this Mac (`here`) | 0.44 | 0.90, the mark in another phase |
| Stopped: the bar | 0.35 | 0.73 |

The long sentences sit between 1.0 and 1.5 as other long lines of text do, from where the two put
each glyph's edge. Under the pointer: Try again 2.04 (1.31 at rest), the bar 0.79 (0.79), the
place with its card 1.47 (1.71), the card itself 0.75. The line said after Try again is the web
window's to the word; it stands ten points further left in the strip, as this window's lines do
there.

A run's lines, photographed in a window 2400 points tall so the whole conversation shows
(`thread-steps-line`, `-line-all`, `thread-steps-cut`, `-cut-shut`: the git log line opened, then
all of it, and a command the column cuts, opened and shut), the opened fold: 0.80, 0.88, 1.21
and 1.11; whole 0.58, 0.66, 0.56, 0.55. The cut command gets its mark from the frame after its
first layout (`--at`), as it is measured where it is laid out. Filmed at 60 frames a second, a
press on a line draws it shut until the press lets go and opened whole from the next frame.
The long conversation (`thread-long`, forty-five checkpoints, scrolled to its top) is 0.52 whole,
its gap line 0.29, and 0.58 with the pointer on it (0.44 at rest). The working line opened
(`thread-working-steps-open`) is 1.17 whole, its steps 2.10 and the opened command 3.43, where
they were 1.20, 2.32 and 4.17 in Geist Mono; `thread-working-steps-unread` (every read of the log
failing) 0.43 whole, its steps 0.38. Every other conversation scene measured before this
(`thread`, `-folded`, `-steps*`, `-options`, `-pictures-large`, `-reply-attached`,
`-getting-ready`, `-working`) draws as it did, to the pixel.

## What moves on its own

The web window's conversation page moves in few places, and this window moves in the same ones.
What its stylesheets animate there, and what became of each:

- **The working line's mark** (`live-lattice-orbit`): nine cells in a square of 14 points, the
  eight round the edge lit in turn every 1.488 s, each 186 ms behind the one before, on
  `cubic-bezier(0.77, 0, 0.175, 1)` between faint (0.22) and lit, the middle a hole at 0.1.
- **Its word** (`live-shimmer`): in full ink, a soft band of 45% travelling through it from left
  to right every 2.6 s, linear: a mask gradient at 100 degrees, 260% of the word's width.
- **The getting-ready line** of a chat just brought in: the same shimmer, at 15 points.
- **The Inbox's running marks** (`th-breathe`, outside the conversation but asked for with it):
  the half-dot and the cloud fade to 45% and back every 1.6 s, `ease-in-out`, all in step.
- **A box's menu** (`tc-in`): the reply box's model menu, and the menus of the box a thread is
  started in, fade in as they rise their last three points over 0.13 s, and go at once.

Messages arriving, the reply box opening and folding, the options strip, a run's fold and its
mark, the Files list, the strip's menus and the toasts have no transition in the web window: they
land whole on the next frame there and here. Under Reduce motion the web window stands the mark
at 0.65, takes the band away and draws the word in dim ink, and stops the breathing; so does this
window (`cx.is_reduced_motion()`, and each loop's `still`).

**How they are built.** Every one of them is a Rune loop on the window's clock (`El::cycle`,
`rules/motion.rs`), which is an opacity, and the window draws a frame of nothing but loops as a
loop-only step, with no poll, view, layout or paint, only for a loop on a plain fill that shows.
So each is made of those:

- The lattice's cells are each a cycle whose stops lay the web's keyframes and their easing down
  in short straight pieces, closer than a frame.
- The band is a veil: the word drawn whole, and over it strips of the ground's own colour, about
  seventy whatever the word's length, each a cycle that comes up to 55% as the band's middle
  reaches it (a veil of the ground at `1 − m` over ink is the ink at `m`, exactly, where the
  ground is one colour, which under a conversation it is: the card's wash over the page and its
  glow, worked out where the word stands, `ground_under_the_card`).
- The running ring is a disc breathing, its middle in the card's colour standing still over it,
  and the middle's right half breathing over that; the cloud, a mark of strokes, stands still
  under a veil that breathes the other way.
- The menu is the one transition: an overdamped spring standing for the curve, within a
  seventieth of it at every frame of a 60 Hz display, on the menu alone (`animated_changes`),
  its rows and its size still landing at once.

The loops run only while they are on screen: the line reports itself (`on_visible`), and off
screen, scrolled up through the conversation, it has no loops at all, so the window rests. Until
it has reported itself, for the frame a conversation opens on, the mark stands as under Reduce
motion. The mark and the band start where the web window's start as the line mounts, the band off
the word's left end, from the moment the line was first seen, and keep their place across scrolling.
They wake the window 60 times a second (`fps`), where the web window's run at the display's rate.

**Measured against the web window.** `reference.mjs` films the web page (`MERCURY_FILM=186@60:x,y,w,h`
stops every animation and moves it on a 60th of a second between shutters, so each frame is the
page exactly that long after the last; `MERCURY_FILM_START=0` films from an animation's first
frame) and `mercury --out` films this one, and `tools/film.py` reads both at the same points:

```sh
MERCURY_FILM='186@60:220,602,200,34' arch -arm64 node tools/reference.mjs <checkout> /tmp/film-web dark 1280 760 thread-working
cargo run -- --fixture shots/thread-working-fixture.json --thread --out /tmp/film-native --frames 192 --fps 60 --size 1280x760 --scale 2
python3 tools/film.py /tmp/film-web /tmp/film-native /tmp/film-web/thread-working-dump.json 60
```

| What | The web window | This window |
|---|---|---|
| A lattice cell's opacity, frame by frame, the films slid into step | | mean difference 0.005, largest 0.067 (of 1) |
| One turn of the orbit | 1.483 s | 1.483 s (to the frame) |
| The band's middle | 151.7 points a second | 152.8 |
| The faintest the word gets | 0.450 of its ink | 0.454 |
| The word with the band's middle on its middle, column by column | | 0.022 apart on average, 0.037 at most |
| The model menu coming in, how much of it shows, frame by frame from the chip | 0.47, 0.73, 0.87, 0.94, 0.99, 1 | 0.48, 0.72, 0.86, 0.92, 0.96, 0.99, 1 |

The breathing ring drawn as fills is 0.27 of 255 from the ring it was, at 2x, frame by frame.

**What it costs.** `RUNE_LOOP_CHECK=1` has a headless run ask, before every frame, for the
loop-only step a window would draw, and hold it against the ordinary frame drawn instead. With the
working line on screen, every frame of three seconds is such a step and none differs (57 of 57,
the rest the app's own timers); scrolled off screen there is no loop at all; and the same holds for
the getting-ready line and for the Inbox with a running ring or a cloud. In a release build with
Rune's frame profile, an ordinary frame of the working conversation costs 0.19 ms of the drawing
thread's time (0.12 its update, 0.08 its paint), all of which a loop-only step leaves out. Run
headless in real time, which draws every frame whole, 600 frames at 60 a second took 0.70 s of
the process's time in 10 s with the line moving and 0.69 s with nothing moving: drawn whole, the
conversation costs 7% of a core at 60 frames a second, and the loops add about a tenth of a
percent to that. A window draws no frame at rest, and while the line moves draws only the steps.
A window's own figure was not taken here, since none may be opened: with a working thread on
screen in the installed window, `top -pid <pid>` gives it.

The Inbox drawn whole costs far more, 2.27 s in the same 10 s (23% of a core). A ring's border,
which no step can draw, breathing at thirty frames a second would have cost about half of that for
as long as an agent worked: that is why the marks are built of plain fills.

## The same rows as the web window

Which threads are in which list, in which order and with which words is decided by rules ported
from the web window's own (`src/rules/`, each function naming the TypeScript it follows;
`docs/rules-spec.md` is the map). `tools/rows.mjs` holds the two against each other: it reads the
rows the web window drew out of the dumps and compares them with `mercury --rows`, row by row and
field by field, on Home and under each of the Inbox's four tabs, a table row's harness mark and
its priority's word among them. Under a tab it also holds what
stands around the rows: which tabs the strip draws and with which numbers, what the time column is
called, and what the cloud button wears. Any other picture of the Inbox left in the folder (a
cloud scene, `inbox-later-kinds`) is held the same way. On the invented inbox and on the invented
cases (`--cases`, whose rows run on Claude Code, Codex and Antigravity by their labels, their run
and the engine's answer) every table row is the web window's. A row is held by its Try again
mark too, and by what its Status word says under the pointer: its sentence (the cell's `title`)
and whether the card saying why the cloud has not got it takes the sentence's place
(`.has-card`). `inbox-progress-stuck` is the In progress tab with a thread in each state a run
can be in that its row says; it, the cases (5 marks under Needs you, 16 under In progress, 39
sentences, 2 cards), the invented inbox and the running app's own inbox through `--door` all
hold with 0 differences.

The Try again mark (`th-retry`, the refresh arrow in an 18-point square after the word) takes
the strong wash under the pointer; a press tries again and does not open the row. The card under
a row's Status word is the web window's (`.th-why-card`), and as there the rows after it paint
over it, so under a row with another after it only its top shows; under the last row all of it.
`inbox-progress-stuck` is 0.29 whole against the web window's, with the pointer on a mark 0.34
(0.34 at rest), on a Status word with its card 0.32 whole (the card's sliver 0.85).

```sh
node desktop/tools/rows.mjs desktop/shots --shoot <checkout>            # the invented inbox
node desktop/tools/rows.mjs desktop/shots/real --shoot <checkout> --real   # the person's real threads
node desktop/tools/rows.mjs desktop/shots/live --shoot <checkout> --door   # the running app's inbox, this minute
```

`--door` asks the running app for its snapshot and for what its window keeps, through the local
door, so the fleet, the cloud and the team are the real ones too.

The real threads are read and never written, their words are printed as lengths only, and nothing
of them is ever committed. `cargo test` runs the rules' own tests on invented threads.

## The agents' limits

The rail's usage row and the card it opens are the web window's sidebar `UsagePill`
(`renderer/src/components/UsagePill.tsx`, its look `.usage-panel` and `.workspace-usage`):
`src/usage.rs` draws them, and `src/rules/usage.rs` holds `shared/usage.mts`'s rules with its
cases as tests.

- **The row** names the agent used last: the newest run going now, else the thread touched last
  that names its agent (`lastUsedEngine`), among the agents with a reading from this week that
  still answers for a window (`agentsInUse`). Open, it ends on the headline window's figure, the
  five hour one where there is one, else the fullest (`headlineLimit`); none when that window
  started over since it was read. Past 80% spent the figure is in the text's ink.
- **The card** rises eight points over the row from the rail's left edge, 268 points wide
  whatever the rail's width, square, on the panel's grey under a strong edge and the app's large
  shadow. One agent: its name in small capitals, each limit (its name, "39% used", a bar of what
  is spent, when it starts over), and under a hairline when it was read ("Read just now", "Read
  4m ago"). Several: the one used last first with "Last used" beside its name and when it was
  read under its limits, then each other under a hairline of its own with its limits and when it
  was read. A reset is Claude Code's clock time, with the day when it is not today ("Resets Wed
  at 10:57pm"), or, for a reading that carries an instant and no words, Codex's, counted down
  ("Resets in 6d 13h"). A five hour window that started over while another still runs says Reset,
  with no bar and no time (`limitsToShow`).
- **When it opens.** 120 ms after the pointer came onto the row, so a pointer crossing the
  rail's foot does not flash it; it stays 220 ms after the pointer left it, so one slipping off
  its edge does not snap it shut (`LINGER_MS`). On the folded rail nothing opens: a pointer that
  landed there opens it on its first move once the rail has opened, so the card never opens into
  a rail still widening, and the rail folding closes it at once. On a rail kept open (Settings'
  "the sidebar shrinks to icons" off) it opens 120 ms after the pointer moves onto the row. It
  lands whole, as the web window's has no transition, and takes no pointer: the pointer passes
  through it to what is under it. Nothing asks the agents for a fresh reading when it opens; the
  readings are the snapshot's, as the web window's are.
- **A press** on the row opens that agent's page in Settings, here (Codex's for a Codex reading,
  Claude Code's for any other: `openAgentSettings`), and the card goes.

The scenes: `home-usage` (one agent), `home-usage-two` (Claude Code read just now, Codex a day
ago), `home-usage-reset` (a five hour window that started over) and `home-usage-kept-open`
(two agents, the rail kept open). Each hands the pointer's path to this window in its fixture
(`script`, `at`), the same three moves the page was given. `tools/scenes/usage.mjs` invents
the readings. At 2x, dark, the card, the row and the rail:

| Scene | Window | Whole | The card | The row | The rail |
|---|---|---|---|---|---|
| `home-usage` | 1280 by 760 | 0.96 | 0.27 | 0.47 | 0.65 |
| `home-usage-two` | 1280 by 760 | 0.96 | 0.28 | 0.47 | 0.64 |
| | 900 by 760 | 0.98 | 0.28 | 0.47 | |
| `home-usage-reset` | 1280 by 760 | 0.96 | 0.27 | 0.32 | 0.64 |
| `home-usage-kept-open` | 1280 by 760 | 3.60, Home's sky | 0.28 | 0.47 | 0.64 |

The rail is the web window's less its Demands and Connections (see what still differs). Home
with the rail kept open is 3.60 whole without the card too: the web window paints Home's sky for
the narrower card there, and this one for the folded card. Filmed at 60 frames a second the card
lands whole on the frame its 120 ms run out, and stays still. Tests hold the rest on an invented
inbox: no card until the pointer moves on the opened rail, then 120 ms; 220 ms of linger; the
rail folding closes it; a pointer only passing over opens nothing; a press opens Claude Code's
page in Settings.

## Settings

The rail's Settings, and the person's own row under it, open Settings over the whole window as the
web window opens its own (`src/settings.rs`): the menu in the rail's place, with Back to app, the
search and each page under its group (Personal, Agents, Workspace, Billing; Team and Rooms only
where the web window shows them), and the open page on the card, in the web window's column of 640
points. Escape closes an open menu first, then puts back a name being typed, then closes the
screen; ⌘F, or / outside a box, goes to the search, where Escape empties it before it closes, the
arrows walk what it found and Enter lands on the setting with its group scrolled into view.

Each page is drawn from the engine's own reading of the settings (`zero:settings-read`), read when
the screen opens and again after each write, and from the snapshot for what moves while it is open
(the update, the accounts' usage, the projects' order). A write is the web window's own channel,
sent off the drawing thread: `zero:settings-set-workspace` for a setting of the workspace,
`zero:settings-set-project` for a project's, and the page's own for the rest
(`zero:project-rename`, `zero:project-archive`, `zero:set-product-order`, `zero:team-sign-out`,
the update's and the accounts' switch). A change shows at once and stays until the engine's answer
agrees, or says otherwise.

Drawn here (`src/settings/`):

- **General**: key hints, and the sidebar shrinking to icons, which are this window's own and kept
  with its picks (`native-picks.json`), as the web window keeps its own; the Agent Tray; counts
  and crash reports; where Mercury keeps things; updates (Check for updates, Restart to update,
  moving the app into Applications); and the account with Sign out.
- **Phone**: the App Store's code, drawn from its modules; Open the App Store; and which projects'
  threads go to the phone, from a menu that stays open while they are ticked.
- **Alpha features**: its four switches.
- **Claude Code** and **Codex**: whether the agent was found and where it runs from; each account
  as a tile (In use or Switch, its plan, where it is signed in, its meters and when they reset, or
  what is wrong with it); Use the cloud; and the permission modes, on this Mac and in the cloud.
  **Antigravity**: its sign-in and its mode.
- **Running**: what this Mac measured, how many agents run at once, holding heavy work when memory
  runs short and how many heavy commands at once, stopping the programs agents leave behind,
  Something felt slow, agents started outside Mercury, and importing new ones.
- **Projects**: the running order, moved with the arrows and To top, and with ⌥↑ and ⌥↓ on the
  row last moved (in the web window, the row the pressed button is in); its search; Select, to
  archive several, with Undo; and the archived ones, folded away, each with Bring back. A
  project's own page: its name, renamed in place (Enter or leaving the box keeps it, Escape puts
  it back), how it runs (on its own, on which Claude account, its own permissions), its browser
  (the sites its agents may open, and signing in to it), where it is kept, with its branch, and
  Archive.
- **Files**: keeping files in the cloud, and for how long, or the way to a plan.
- **Plan and Billing**: the three plans and how the cloud counts, on a Mac on no paid plan.

The rest bring the web window forward on its page (`zero:open-settings-here` with that page's id):
Appearance, Shortcuts, Instructions, Team and Rooms, from the menu or a search; and from a button,
a paid plan's own summary and choosing a plan (both go on to Stripe), signing an account in or
adding one, Get link and choosing the agent's program by hand, the Android app, signing in to use
the phone, a new project, a project's picture and its instructions. The window says in a line when
the web window is not there to come forward.

The scenes: `settings` (General), `settings-running`, `-claude`, `-antigravity`, `-codex`,
`-projects`, `-project`, `-phone`, `-cloud`, `-files` and `-alpha`; a picker's menu open
(`settings-running-menu`, `-phone-menu`, and `-claude-menu`, scrolled to its pick); the search
typed into (`settings-search`); and the pointer resting on a control (`settings-hover-*`). Each
answers the web window with invented settings in the engine's shape (`tools/scenes/settings.mjs`),
which the picture's fixture carries to this window. Measured against the web window whose menu
down the side is the app sidebar's (rows 36 points tall, marks of 18, a search with no box) and
whose switches are the text's ink when on. Of 255, at 1280 by 760, 2x, the whole window and the
open page's column:

| Page | Whole | Page |
|---|---|---|
| General | 0.38 | 0.68 |
| Running | 0.25 | 0.36 |
| Claude Code | 0.26 | 0.38 |
| Antigravity | 0.19 | 0.23 |
| Codex | 0.25 | 0.36 |
| Projects | 0.23 | 0.31 |
| A project | 0.28 | 0.42 |
| Phone | 0.33 | 0.57 |
| Plan and Billing | 0.28 | 0.44 |
| Files | 0.16 | 0.16 |
| Alpha features | 0.27 | 0.40 |
| A menu open: Running, Phone, Claude Code | 0.27, 0.34, 0.25 | 0.41, 0.59, 0.32 |
| The search typed into | 0.39 | 0.68 |

The menu down the side is 0.28 to 0.31 on every page. At 900 by 760, where the column narrows to
488 points and the plans' prices break onto a second line, General, Running, Claude Code,
Projects, a project, Phone and Plan and Billing are 0.28 to 0.46 whole; at 1600 by 1000, 0.18 to
0.29. Most of what is left is a wrapped line drawn a device pixel lower than Chromium draws it
(below), which comes more often in a narrower column.

Under the pointer (`tools/hover.py`), the control and the same box at rest: Back to app 0.29 and
0.34, a page's row in the menu 0.49 and 0.33, a ghost button 1.09 and 0.81, a picker's chip 1.61
and 1.59, a project's row 0.56 and 0.53, Switch 1.18 and 1.27, Add account 0.96 and 0.89, the
crumb back to Projects 0.58 and 0.39; and, measured on their own boxes, a row of an open menu 0.91
and 0.49, a project's name 2.81 and 2.24. The pointer walk holds the shape to the web window's at
every point of every page but one to six a page, past the end of the last line of a wrapped
description, where the web window shows the arrow and this one the I-beam.

Every write was tried on a scratch engine (`MERCURY_HOME` in a temporary folder, run from the main
checkout), never on the app's: counts and crash reports off, four agents at once, a nudge of one,
the memory check off, a project renamed with Enter and by leaving the box, a project set to run on
its own, the order moved with the arrows and To top, a project archived with Select, and another
archive taken back with Undo; each came back in the engine's next reading. Not tried there: the
phone's menu, Check for updates, Sign out and switching an account, which need an account, a
release or a second login.

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
  up a sliver apart. And its foot can rest half a point lower than the browser's: this window
  scrolls to the page's height rounded down to a device pixel, which is where the browser stops
  most conversations (1408.27 points at 1408, 1211.61 at 1211.5), but not all (1435.8 at 1436,
  1679.86 at 1680, where this window stops at 1435.5 and 1679.5). No one rounding of the height
  alone gives all of them. `thread` at 1600 by 1000 is 1.12 for it, and `thread-reply-attached`
  0.83 at 1280 by 760.
- **Under the pointer, where it still differs** (see "How it answers the pointer and the hands"):
  - The I-beam shows over the whole box of a run of words, past the last word of a short line too,
    where the web window's shows it over the words' lines only: Rune finds selectable words by
    their box. Over a title or a heading shorter than the column, the same.
  - A mention's mark takes the I-beam, not the pointing hand, and stays muted when the words
    brighten (`a.mention:hover .mention-icon`): Rune's link covers words and not a mark set among
    them.
  - The open rail's update card, which the web window draws while a new version waits
    (`home-update-rail-open`, the rail 3.39 of 255), is its Restart row here.
  - To's face on the New thread card takes the pointer over its whole square; the web window's
    is round, and Chromium finds its corners as the field's.
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
  - An address on the web or on this Mac opens in the browser. The web window opens it in the
    pane beside the conversation, in a browser of the pane's own with its own bar (`WebsitePane`,
    `BrowserChrome`): Rune draws no web page inside a window.
  - A mention cut by the end of a line moves whole to the next, where Chromium breaks it at a
    hyphen in its name (`round-` / `tax.ts`), which a conversation narrowed beside a file meets
    more often.
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
  - With pictures above the place being read still loading, the landing on the last message
    holds for its 2.5 seconds only on the turns the window takes (a picture landing is one),
    where the web window's holds on every change of height.
  - A picture on the web (`https://…`) or pasted as `data:` is a mention, not drawn: this window
    reads pictures from the disk only. An `.svg` and an `.avif` are not decoded and say missing;
    an animated picture shows its first frame.
  - The place held for a picture on its way, and a missing picture's mark, do not change under
    the pointer, and the place does not shimmer.
  - What only a conversation between people has is not drawn: tiles under a message
    (`MessageFiles`), how a file reached a teammate, a voice note. Nor a thread or a connection
    named as a link, which stays a link.
  - With more pictures than the budget holds, one less than half on screen may hold its place
    until more of it shows: Rune reports an element on screen from half of it.
- **A file opened beside the conversation, where it still differs** (see "A file opened beside
  the conversation"):
  - Nothing is typed into it. The web window's pane is an editor: a document takes typing,
    saved where it opened from when that is the project's own (`paneCanSave`, `DocText`), a
    table's cells are edited and its rows added (New row), and a teammate on the same thread
    types into the same copy. Here it is read only, and a link in a document opens at once,
    where the web window's first shows a small card with Visit, Copy, Edit and Remove.
  - The globe is drawn where the app says it shares files, and takes the pointer, but its panel
    (who can open the link, Copy link, Update link, Stop sharing, the people added, the link's
    comments) is not drawn: making a link anyone can open is a write to the cloud that could not
    be tried against a real app from here. A press says the link is made in Mercury's own window.
  - Two panes side by side open the file big, where the web window opens it big with its notes
    down the right, which the big view here does not draw. It counts no notes on its mark.
  - A page, a PDF or a film the thread sets beside itself is a card saying what it is, with Open
    outside, where the web window draws the page in a frame, the PDF's pages and the film's player:
    Rune draws no web page, PDF or film inside a window.
  - A table's search, Filter and Sort say, when pressed, that they are the web window's; its
    column menus are drawn and do nothing yet, the first column
    and the row numbers do not stay put while the rest scrolls sideways, a date column is told
    only by the ISO day, and a link, an email or a logo in a cell is its words.
  - A document's outline does not light the section being read.
  - The split is this window's own, kept with its picks: the web window keeps its own in its
    storage, which this window cannot read, so a split dragged in one is not the other's. Where
    a thread had its file open is kept while the window is open, where the web window keeps it
    across a restart too, and it is let go when the thread's row changes at all, where the web
    window waits for a new answer.
  - Over a card under a thousand points wide, the web window hides the marks too (its corner
    wakes them only over a file sent to the whole window), so there, as here, Escape is the way
    out; the web window's crumb for it is kept, hidden, in the strip it gave up.
  - The file is asked of the disk every three seconds while it is open; the web window asks when
    the app says something changed, at most that often.
  - A conversation beside a file at 560 points or less sets its code blocks smaller and wrapped
    in the web window (`@container talk`); here they keep their size and scroll.
- **The code a thread changed, where it still differs** (see "The code a thread changed"; each
  line is in `docs/parity-code-changes.md`). The first three come from one gap in Rune as pinned: a
  window can select inside one text (`Tx::select_text`) but cannot set a selection that runs
  across two, and the line holding the caret is a field, not one of the texts a selection runs
  through.
  - Shift with an arrow, Home or End inside the line holding the caret extends the selection in
    that line only. The web window carries it into the lines above and below, keeping its column
    (`extendHead`, ported here with the web's cases and not called). A drag or a shifted press
    does cross lines.
  - From the line holding the caret, a shifted press on another line lets go of the line and
    selects nothing, and a drag stays inside the line. The web window selects from where the
    caret was. From anywhere else, a press on a removed line, a gap's line, a line not yet
    holding the caret or the margin, a shifted press and a drag cross lines as in the web window.
  - A double press on a line that takes typing and does not hold the caret yet puts the caret
    there, and the word comes with the next press, the line with the one after: the first press
    makes the line a field, and Rune counts a field's presses from the field's own first. A press
    on a field reaches no handler of the window's, so the count cannot be carried over. Once
    the caret is in the line a double press takes the word and a triple press the line. A removed
    line and a gap's line take them at once.
  - A press on a line puts the caret by counting the mono face's characters, and where it
    wraps, where the browser finds the glyph under the pointer: the same place for the code's own
    letters, off where wide characters (an emoji, a CJK letter) stand before it.
- **A file opened big, where it still differs:**
  - No comments down the right, no box to write one, no Approve and no Share, which are not used
    where this window runs, so the stage takes the whole width, and everything on it
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
  - The document's outline does not light the section being read.
  - A document cannot be edited and a version cannot be restored (the web window's editor and
    Restore this version), and a file is not renamed by pressing its name.
  - A document's inline code takes the message's look, a little smaller than the web window's
    reader draws it.
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
  so when nobody else is on the team and is bare otherwise, until the Team page is drawn. Its rows
  are still counted, each person of the team and each agent teammate of one's own, so the two
  cards fit three rows or fewer as the web window's do.
- **Every row on Home wears the agent's face.** The web window gives a thread a teammate sent
  their initials or photo, and one an agent teammate is on that teammate's face; this window does
  not tell who sent a thread, so every row is an agent's.
- **Choosing in a conversation, where it still differs.** Ported is what is used day to day: the
  model and level of the next reply, Later, and where a thread runs. Left out is what this window
  cannot do and what goes unused, a thread's name, project, pin, waits, demand and shipped mark,
  and what only teammates or approvals need:
  - Nothing is sent later. The caret beside Send answers the pointer as the web window's does
    and, pressed, says a reply is sent later from Mercury's own window: its menu (`ReplyLater`: Remove the time, Tomorrow morning, Monday morning,
    a time typed in words, then the time said on the caret, the reply written with its `runAt`
    and held on the thread until then, a reply sent meanwhile taking the waiting one with it, and
    the time kept with the draft) is a box of its own and a grammar of its own, and goes unused.
  - What a reply may do is changed by its chip and Shift+Tab. The web window's box also takes
    the mode typed as a slash command (`/plan ` and its kin set it, `/clear ` puts the thread back
    on its project's setting, and `/` opens a menu of them and of Claude Code's commands); this
    box has no slash commands, so a mode set on a thread is changed but never cleared from here.
  - In a box under 460 points the model chip keeps its whole name, where the web window's is
    cut with an ellipsis once the mode chip has folded and the chips still do not fit. That takes
    a model's name longer than about twenty letters in the narrowest window with the rail open.
  - The thread's menu leaves out every row this window cannot do, which are also the ones never
    used: Open terminal (⌘J; this window has no terminal), Rename, Move to another project, Pin
    thread, Share…, Add people and Ask or hand off… (teammates), Waits on… and Blocked by…, Mark as
    shipped, Stop repeating, Close task, Run now and Demand. On a finished thread, where the web's
    menu holds only the first two, the dots say where they are. Its View code changes row is not
    left out: the web window draws it only on a thread whose corner has no code button, which is
    never while the button is drawn (`change: !!change && !codeButton` in `ThreadMenu.tsx`), so
    there is nothing to draw.
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
- **Starting a thread, where it still differs:**
  - Out of reach here: To (the agent's face on a team's Home and Agent on the card, and it opens
    no menu), @-names in the words, recording, a demand, the branch a thread starts from, who sees
    it, and the tutorial's suggestions. Of these To is drawn, where the web window draws it; who
    sees it, which a team's quiet bar puts before the level, is not, nor is a subject typed into
    that bar's own mark, nor a thread to a person or an agent teammate with its own short bar.
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
  - A menu comes in on a spring that stands for the web window's curve (see "What moves on its
    own"). Under Reduce motion Rune fades it without the rise, where the web window's still
    rises. The reply box's model menu opened by the model's name on the folded box comes in a
    frame late, its first frame already three quarters in: it hangs from the chip of a box that
    opens in the same press, and is laid out once that box has been.
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
- **Files in a reply, where they still differ** (besides what is said of a new thread's files,
  which holds here too):
  - A voice note, a reply to a person and the slash commands are not here: the reply box takes
    pictures and files.
- **Answering a thread and stopping its agent, where it still differs** (see "Answering a thread
  and stopping its agent", and `docs/parity-answering.md` line by line):
  - A press on another control of the page (a link, a run's fold, the strip's buttons) leaves an
    empty open box open, where the web window folds it on that press too: Rune hands a press to
    the control under it and to no observer of every press, so only a press on nothing in the page
    folds it. For the same reason a press outside one of the box's menus goes on only to the box's
    own controls, an option and Stop; the web window's lands anywhere. Where it lands, the option
    is picked as the press goes down rather than as it comes up.
  - Each window keeps its own drafts and its own once-ever warning: the web window's are in its
    storage, which the door does not hand over, so a draft typed in one is not in the other's box,
    and each window warns once.
  - A stop the engine refuses is said ("Could not stop it: …"), and so is a pick it refuses ("Could
    not send that option: …"); the web window's stop and pick throw and say nothing.
  - A reply lost by a restart is looked for in its thread's ledger alone: the web window also
    counts a reply the phone shows as waiting for the computer (`phoneWaiting`), a list this window
    does not read.
  - A thread whose answer is taller than the window, with something the person did after it, opens
    on the page's top, where the web window lands on that answer's top (as for a conversation drawn
    from the answer an agent works from, below).
  - The ring of the line about a send is drawn in sixty steps.
  - What is held when the window closes is written on the closing turn; a close that is not the
    window's own (the process killed) loses it, as a crash loses the web window's.
- **No card to allow or deny what an agent asks to run.** The web window draws one over the
  page, answered with ⌘1 and ⌘2. Here a line at the card's foot says an agent is waiting for a
  yes and where to give it. Where every project runs on full access no agent ever asks.
- **Several rows cannot be ticked**, so nothing is done to many threads at once.
- **The rail has only what works here**: Search, Home, the Inbox, Files, Restart to update while
  a new version waits, Settings, the usage row and the person's own row. The web window's also
  has Demands, Chats (with teammates), Connections, Record meeting and Feedback. Drawn here they
  were buttons that went nowhere, so they are left out until this window has their pages, and
  Files stands under the Inbox where the web window has Demands and Connections before it.
  Settings and the person's row open Settings here ("Settings", above), and the usage row opens
  the card of the agents' limits ("The agents' limits", above).
- **Settings has only the pages that are used**, and sends the rest to the web window
  ("Settings", above). Within its pages:
  - A line of a wrapped description can stand a device pixel lower than the web window's:
    Chromium and Rune round a line that starts on a half pixel differently. It is most of what is
    left on General (0.68 on the page). Moving the lines by half a pixel, or their pitch, put
    the other lines out instead.
  - Past the end of a wrapped description's last line the pointer is the I-beam, where the web
    window's is the arrow: Rune gives the text's whole box the I-beam, not its lines.
  - The App Store's code is drawn from its modules; its edges are up to a pixel softer.
  - The Projects search's words stand half a point higher, as at 1280 by 760 they stood half a
    point lower; which way they round follows where the box falls on a device pixel.
  - A project's rank takes the grab, as the web window's does, but dragging a row does nothing:
    it is moved with the arrows, To top and ⌥↑ ⌥↓.
  - What eases in the web window lands at once: a switch's knob, a moved row's wash fading, a
    meter's bar, and the flash on a setting a search lands on, which is not drawn.
  - A press outside an open menu closes it, and opens another picker if it was on one; any other
    control under it waits for a second press, where the web window's takes the first.
  - Not drawn: General's public links (the files made public, with Copy link and Stop sharing),
    the cloud's keys for its threads, and a paid plan's summary of its month. The last opens the
    web window; the first two are left out until they are wanted (the web window, too, shows
    them only where there are some).
- **The command bar, where it still differs** (each row of the web's is in
  `docs/parity-command-bar.md`, with what this window does of it):
  - Left out, as each needs a screen this window does not have: New project, Record meeting and
    Stop recording, Groups and each group, the team's lists, Go to Connections, teammates, General
    agent instructions (the web's instructions card), Show as a board, Import chats and agents,
    the tutorial and End the tutorial, Set Mercury up again and a demo copy; on a thread, Remote
    control, Open terminal, Rename, Stop repeating and the team's rows (who sees it, handing it
    over, asking, waits, shipped, decisions, Add people, Close task). The theme rows are left out
    as this window draws the dark look only. Filter… is left out because in the web window it
    only closes the bar, the menu it opened being gone, and Clear filter because its filter is
    set only by the tutorial. Mark unread is left out because only the web window keeps what has
    been read and the app has no channel to set a thread back to unread; Mark read is here.
  - Stop and Try again are here; View code changes is its own control's to bring; Resume this agent and
    Restart the agent already stand aside where Try again would stand for them.
  - It finds threads only, the best five, with no "See all" row, since this window has no search
    page. The web's bar also finds people and groups, repeating tasks, teammates' threads, and
    lines, files, comments and decisions inside things, which its main process searches. The
    lines, files and comments are not here yet: a line opens its thread scrolled to that message
    and marks it, which needs this window's conversation to keep each message's line id (it keeps
    none), and a comment opens its file big at the comment, which the big view does not draw.
  - Pause agents closes the bar as it is pressed; the web's closes it once the engine has
    answered, a few milliseconds later.
  - Resume this agent says what the web window says of the engine's answer, and that answer
    carries the supervisor's whole queue under the name the count goes by (`queued` in
    `zero:resume-items`, spread after the count in `main/ipc.mts`), so both windows say "1
    resumed,  waiting for a slot" (with the queue's ids where there are any) and never "Nothing
    to resume". Fixed in the engine's answer, both windows would say it right.
  - Over the Files place the web's bar still acts on the Inbox's picked row when the Inbox was
    the list last on screen; this window's acts on no thread there.
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
  terminal agent's row cannot be put off here.
- **A person's photo is their two letters.**
- **What the web window remembers only for itself is not known here**: an update card the person
  closed still shows its Restart row, and the Sign in row of someone who skipped signing in is
  not drawn.
- **Later lists threads only.** The web window also lists repeating tasks there, above the threads,
  and counts them in the tab's number. They are rules on a channel of their own that this window
  does not ask, so on a Mac with repeating tasks its Later says a smaller number and has fewer rows.
  A terminal agent put off, which the web window lists when the setting for such agents is on, is
  not there either.
- **What moves on its own, where it still differs** (see "What moves on its own"):
  - The band crosses the word as vertical strips: the web window's gradient leans ten degrees,
    which moves its edges a point and a half between the word's top and its foot.
  - The loops wake the window 60 times a second, where the web window's run at the display's rate,
    120 on a ProMotion screen.
  - The veil is drawn in the colour the ground has where the word stands; a faint dot of the page
    under the word, which the web window's mask leaves as it is, is veiled with it, by at most
    three of 255.
  - A conversation opens on one frame of the mark standing as under Reduce motion, before the line
    has said it is on screen.
  - The card that shows an option whole comes in on the spring the menus stand for the web
    window's curve, and an option in it is its words, where the web window's draws its markdown.
  - The bar along a toast's foot is drawn anew every 33 ms rather than as a loop.
- **What a run did, where it still differs:**
  - A command the column cuts gets its mark a frame after the line is first laid out: it is
    measured where Rune lays it out, after the frame is drawn, where the browser measures it
    before it paints.
  - Where the cloud says why a run is on this Mac, and on a row whose Status word carries that
    card, the web window shows the help pointer (`cursor: help`). Rune's pointers have no help
    shape, and Chromium's is a picture of its own, not one the system has; this window shows the
    arrow over the place and the pointing hand over a row, which is the row's.
  - The person's hand that ends a landing and keeps a scrolled place is the wheel here; the web
    window also counts a key or a press in the conversation (`HER_GESTURES`), which Rune does not
    report to a scrolling box without taking them from what is in it.
  - The working line's steps are read with the conversation, on every change the engine pushes
    and every eight seconds while a run is up, open or not, and opening the line asks at once,
    where the web window asks for the run's log alone while the line is open. One failed read of
    either says the history could not be read, as the web window says of its own.
  - The system's monospace is set 0.217 points a letter closer than Rune sets it at 12 points,
    which is where Chromium sets it; `'trak' 0` does not take Rune's tracking back.
  - Signed out never points at a sign-in bar at the top of the Inbox (`signInBar`): this window
    has no such bar, so the sentence says to sign in from a terminal, as the web window's does
    with its bar down.
  - The walkthrough's practice thread never reads as being answered (`answering`): the window
    has no walkthrough, and so never draws working on one.
- **The Inbox's two menus, where they still differ** (see "The Inbox's View and filters, and the
  cloud's menu"):
  - The board is the web window's. Board in the menu and B say so, and the list stays a list; a
    display kept as a board is drawn as the list.
  - With a menu open, the page under it shows the arrow and its rows do not wash, and a press on
    it only closes the menu, where the web window's press also lands on what it was aimed at (a
    row opens): the layer that catches a press outside the menu covers the page, as for the
    strip's menus over a conversation.
  - The display is this window's own and the web window's is its own, so the two can show the
    Inbox filtered differently; so is the list of stopped cloud accounts already looked at.
  - The setup of the cloud's accounts is the web window's screen, which Manage and its kin bring
    forward. That needs the app with `zero:open-cloud-setup-here`; an older one says in a line
    where the accounts are set up.
  - The month's hours from Agentbox Cloud's server are read when the window opens and when the
    menu opens; the web window reads them every two minutes while its cloud is the server's.
  - The mark of a wait on the line about an account being added stands still; the web window's
    turns.
  - ⌘C's question draws its keys' caps in Geist Mono, where the web window's are in the system's
    monospace.
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
  what a pick there writes). `engine.rs`: which coding agent a row is on, as Home names it
  (`row_engine`) and as the Inbox's mark shows it (`harness_on`, `harness_mark`).
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
  sentences, the tools it ran in plain words and what came back. `src/rules/live.rs`: whether
  anything is happening on a thread (`liveLine`), what the line at its foot draws in each state,
  the steps it opens onto, when Try again is offered and what it says, and the line under its
  title. `src/rules/status.rs` reads the same line for the Inbox's Status word.
- `src/foot.rs`: the facts a conversation's line is read from, the bar over a stopped
  conversation, Try again on the line, the bar, a row and the command bar and what a press of it
  writes, the card saying why the cloud has not got a thread, and the repeating tasks' rules.
  `src/thread.rs` draws the line itself, a run's lines in their three states, and keeps what was
  opened, where a conversation was left (in `src/picks.rs`) and where it lands.
  `docs/parity-run.md`: the checklist of all of it against the web window.
- `src/rules/motion.rs`: what moves on its own, as Rune's loops: the lattice's keyframes, the
  breath, the shimmer's veil and the ground it is drawn in. `tools/film.py` holds a film of this
  window's working line against the web window's (`MERCURY_FILM` in `tools/reference.mjs`).
- `src/later.rs`: the Later picker. `src/rules/later.rs` reads the time typed into it, on the
  calendar `src/rules/calendar.rs` keeps.
- `src/reply.rs`: the model and level a conversation's reply box is on, and its model menu (the
  box's own, from `compose.rs`). `src/rules/reply_pick.rs`: what the box opens on, what a pick
  keeps and what the next reply writes. `src/strip.rs`: what the strip over a conversation
  opens, the thread's menu and the place panel, and what a pick there writes.
- `src/answer.rs`: Stop, what was sent drawn from the press, Z's pile and its question, the lines
  a Z leaves, the question before a discard, the waiting agent's warning, the thread followed, the
  line in the strip about a send that moved the person on, replies a restart may have lost, and the
  guards on a press or a number aimed at what stood there before. `src/rules/answering.rs`: their
  rules and words. `src/drafts.rs`: what the reply boxes hold, written down across a restart.
  `docs/parity-answering.md`: every behaviour of the web window's there, ticked.
- `src/chips.rs`: the reply box's priority and mode chips, their menus, Shift+Tab, what a reply
  carries of them, the narrow bar's fold, and the little arrow that folds an agent's choices.
  `src/rules/reply_chips.rs`: each agent's modes and their words, the ring Shift+Tab walks, the
  line a change says, what a reply writes, and which round of choices a fold holds.
- `src/usage.rs`: the rail's usage row and the card of the agents' limits it opens, and when.
  `src/rules/usage.rs`: which agents are in use and which used last, which windows a reading
  still answers for, the card's rows and their resets, and how old a reading is.
- `src/compose.rs`: the box a thread is started in, on Home and as the New thread card, its
  menus and what a send does. `src/rules/compose.rs`: what a new thread is made of and the
  menus' rules. `src/picks.rs`: the picks kept from one window to the next.
- `src/attach.rs`: the files in that box and in a conversation's reply box, how they come in,
  how they are drawn and how a reply's are sent. `src/rules/attach.rs`: which can go, what the
  box says of one that cannot, and how they are saved and linked. `src/rules/folded.rs`: what the
  folded reply box says it holds. `src/rules/typed.rs`: the lines the app wrote into a person's
  message for its files, which a conversation draws.
- `src/palette.rs`: the command bar ⌘K opens, what each row does and what is said once the
  engine answers, and `src/palette/marks.rs` the line drawings its rows lead with.
  `src/rules/palette.rs` decides what it offers, in which order and which drawing each row wears,
  what each writes and the words for each answer; `src/rules/search.rs` finds a thread by its
  words. `docs/parity-command-bar.md` is every row of the web's bar beside this one's.
- `src/head.rs`: what the Inbox's head strip opens, View and filters and the cloud's menu, ⌘C and
  its question, and what each pick does. `src/rules/display.rs`: what View and filters keeps and
  in which order, laid over the lists (`display::show`); `src/rules/cloud.rs`: the cloud button,
  its menu's lines and what they write.
- `src/files.rs`: the files those words name, drawn, and the one function that opens one with the
  system. `src/pictures.rs`: the pictures, read off the disk on a worker and kept small.
- `src/viewer.rs`: a file opened big, over the window. `src/rules/viewer.rs`: which files it holds
  and steps through, where a press lands, what the thread called the file, who shared it, and a
  picture's fit and zoom. `src/rules/code.rs`: a script's colours.
- `src/pane.rs`: a file opened beside a conversation, its card, marks and divider, what opens it
  and what it keeps. `src/rules/pane.rs`: which press opens what where, the path's crumb, the
  split, Escape, and the file the newest answer opens by itself. `src/table.rs`: a sheet drawn as
  its table, each column told what it holds and each tag its colour.
  `tools/scenes/thread-pane.mjs`: the scenes that open it, answering the page's reads of a file.
- `src/change.rs`: the code a thread changed: the strip's button and when its figures are read,
  the change in the pane (its tree or picker, its running column, gaps, keys, copying, typing and
  saving), the marks over the whole workspace, the chip on a run's step, and what a picture had
  done to it. `src/rules/change.rs`: the change read off the engine's answer, its figures and
  words, the tree, the words that changed, the gaps, the keys, the copy, the typed lines put back
  in a file, and which step's file the change holds. `tools/scenes/change.mjs` and
  `tools/scenes/thread-change*.mjs`: the invented change and the scenes that show it.
  `docs/parity-code-changes.md`: every behaviour of the web window's view, ticked with how it was
  checked.
- `src/files_page.rs`: the Files place. `src/rules/files_shown.rs`: what it shows, in what order,
  and what a tile says.
  `src/rules/paths.rs`, `mentions.rs`, `roots.rs`: which words name a file, what kind it is and
  what it is called, and where it is looked for. `src/rules/thread_files.rs`: which files a
  thread made, each once at its newest version, for the Files button (`src/thread.rs` draws it
  and its list).
- `src/settings.rs`: the Settings screen, its menu and search, its keys, and its reads and writes,
  each on a worker. `src/settings/kit.rs`: the parts its pages are made of (a group, a row, a
  switch, a ghost button, a picker and its menu); `marks.rs`, its icons. `pages.rs`: General,
  Phone, Alpha features, Running, Files, Plan and Billing; `agents.rs`: Claude Code, Codex and
  Antigravity; `projects.rs`: Projects, a project's own page and the phone's menu of projects.
  `src/rules/settings.rs`: the pages and the search, and every rule behind what the pages say,
  with the web window's own cases as tests. `tools/scenes/settings.mjs`: the invented settings
  the scenes are photographed with.
- `tools/pictures.mjs`: the invented project folder the scenes that name files are drawn from.
- `tools/install-mac.sh`: builds the app and puts it in `~/Applications` as "Mercury Native".
- `src/sky.rs`: the dithered sky, ported from `renderer/src/home/sky.ts` with the same seed.
- `src/icons.rs`: the icons, with the web app's own paths and strokes; the makers' marks once
  each, for a row's tile (`HARNESS_*`) and a chip (`ENGINE_*`).
- `assets/fonts`: Geist and Geist Mono (SIL Open Font License), the files the web app ships.
