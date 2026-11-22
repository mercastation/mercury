# What an open conversation reads

The Home and Inbox screens draw from the snapshot alone. A conversation needs more, and the
web window asks the engine for it the moment a thread opens (`renderer/src/components/ItemThread.tsx`,
through `renderer/src/api.ts`). This is what it asks, what comes back, and what the native
window does with it today.

Every channel is reachable through the engine's door the same way `zero:snapshot` is
(`src/source.rs`): `POST <door>/api/<channel, percent-encoded>` with the header
`x-agentbox-token`, a JSON body that is the payload below, and a JSON answer
(`main/serve.mts`). All of these only read.

## The channels

### `zero:item-history`: the conversation itself

The one the page cannot do without. Every write to a thread is a line in its project's
ledger, and this hands the lines over, oldest first (`main/ipc.mts`, `store.readHistory`).

- Payload: `{ "product": "<slug>", "id": "<thread id>" }`. An optional `"before": <ms>` keeps
  only lines written at or before that moment; the page does not send it when it opens a thread.
- Answer: `{ "ok": true, "lines": [Line, …] }`, or `{ "ok": false, "error": "<words>", "lines": [] }`.
- A `Line` (`LedgerLine` in `renderer/src/thread-history.ts`):

  ```json
  { "id": "w-…", "ts": 1795000000000, "source": "founder" | "agent" | "system",
    "patch": { "title": "…", "body": "…", "status": "open", "note": "…", "answer": "…", "result": "…" },
    "claim": { "holder": "mcp-40122", "leaseUntil": 1795000000000 },
    "epoch": 1, "heartbeat": true, "release": true, "by": "<person id>", "uid": "<line id>" }
  ```

  Only `ts`, `source` and what the line carries are always there. `patch` holds the fields the
  line wrote: `body` is the ask, `note` a checkpoint, `answer` the person's reply (or
  `"Option 2: …"` for a pick), `result` the agent's finished word. A line with `claim` is a run
  picking the thread up; `heartbeat` and `release` lines are the lease being kept and let go.

`src/talk.rs` reads these lines into the conversation (`Talk::read`), by the same rules as the
web window's `threadEvents` and `itemThread`, and decides the page's shape (`Talk::page`): the
newest answer alone with the rest folded, or the conversation from some point on.

### `zero:session-trace`: what each run did

- Payload: `{ "product": "<slug>", "id": "<thread id>" }`. `"whole": true` asks for every run's
  whole log (the page sends it when the fold is opened); `"have": { "<startedAt>": "<stamp>" }`
  names the logs the screen already holds, so only what grew comes back.
- Answer: `{ "ok": true, "sessions": [{ "startedAt": <ms>, "stamp": "<size>:<mtime>", "text": "<log>" }, …] }`.
  With `have`, a log that has not changed answers `{ startedAt, stamp, "same": true }` and one
  that grew `{ startedAt, stamp, "from": <byte>, "more": "<new text>" }` (`shared/trace-delta.mts`).
  Without `whole` it is the newest twenty logs, each cut to its last 120,000 characters behind a
  line saying so (`main/ipc.mts`, `main/run-traces.mts`). The logs are the files under
  `sessions/<thread id>/<startedAt>.log` in the project's records folder.
- A log is the supervisor's own (`traceStreamLine` in `main/harnesses/claude-stream.mts`): a
  `#` header, then lines of `HH:MM:SS  <what happened>` on a UTC clock with no date: a sentence
  the agent said, `[Tool] <its main argument>` with `#id <call>` under it and, when it came
  back, `#out <call> <lines>[ error]` and the output as `#| ` lines; a `stderr:` chunk; the
  `== RESULT (…) ==` block; and `# exited (<code>) <ISO moment>`.

Read, the way the web window reads it (`src/reads.rs`). One read is this channel and
`zero:item-history` together, one read at a time and off the main thread. The logs are asked
for with `have`, and `rules::trace::Reader` (the window's half of `trace-delta.mts`) puts each
log back together from what it holds and what grew; an answer it cannot put together is asked
again with no stamps, and an answer with no stamps is taken as it came. Once the person unfolds
the conversation's earlier messages, every log is read whole once more (`whole: true`), and the
regular reads are laid over that copy as they arrive (`withEveryRun`).

When: as the thread opens, when its row's `updatedAt` changes, when a run starts on it, and,
while a run is up (`supervisor.running` names the thread), again on every snapshot the engine
pushes and every eight seconds besides (`REFRESH_MS` in `ItemThread.tsx`). A finished thread is
read once and not again until it changes. `MERCURY_READS=1` prints each read.

Measured on a scratch engine with a thread whose two logs were planted on disk (one of 277
bytes, one of 399 growing to 781), read the way `--watch-door` reads the newest thread with a
run up: the first read brought the ledger (4 lines, 740 bytes) and both logs whole (1,114 bytes);
each read eight seconds later with nothing new cost 170 bytes of logs; after a log grew by 137
bytes the read carried 321. The ledger is read whole every time, as the web window reads it.

What the page makes of the logs is `src/talk.rs`: each run hangs under the pickup that started
it (`withRuns`), and its sentences become messages and its tool calls the quiet lines between
them (`runNodes`, `traceLines`), each at its own moment; `src/rules/` holds the reading itself
(`notes.rs`, `trace.rs`, `work.rs`).

### `zero:code-change`: the files a thread's runs changed

- Payload: `{ "product": "<slug>", "src": "runs/<thread id>/the-change-it-made.change", "summary": true }`.
- Answer: `{ "ok": true, "change": { "files": [{ "path": "…" }, …], … } }`, or
  `{ "ok": false, "error": "<words>" }` when the thread changed no code, which is most threads.

Not read yet. It only turns a step that names a changed file into something to press.

### `zero:compaction-status`: whether the conversation is being compacted

- Payload: `{ "product": "<slug>", "id": "<thread id>" }`.
- Answer: `{ "state": "running" | "done" | "failed" | …, "at": <ms> }`, sometimes with a `name`
  and a `text`, or `null`, which is nearly always.

Not read yet. A page that cannot ask it takes the missing answer for a failure and draws
"Couldn't compact this conversation. Try again." (`components/CompactionResult.tsx`), which
is what the reference picture showed before the scene answered it.

### `zero:snapshot`: the row

Already fetched. The page takes from the thread's own row its name (`label`, else `title`),
its `status` (a finished thread offers Reopen, any other Done), `labels` (a thread brought in
from another app always opens folded) and `wrote.status.source` (who closed it), and from
`supervisor.running` whether an agent is on it now. With no ledger to read, `body`, `note`,
`answer` and `result` with their `wrote.<field>.ts` still make a conversation of up to four
messages, which is what `thread::ledger` falls back to.

The run on the thread, from `supervisor.running`, gives the line at the conversation's foot:
`startedAt` (or `turnStartedAt`, a cloud run's reply) for how long it has worked, `activity`
(the calls running now, each with its `label`) for the step it is on, `where` for a run in the
cloud, `saying` and `sayingAt` for the sentence being typed this second, which the page places
by when it started and drops once the log holds the same words. `supervisor.whyNotCloud[id].at`
of `mac` says a local run is on this Mac while the cloud takes everything, and
`approvals[].item` naming the thread means its agent waits on a yes, which outranks the working
line (`liveLine` in `renderer/src/live-line.ts`).

## The files a conversation names

The web window asks the engine for every file: a picture on the app's own picture scheme
(`renderer/src/picture.ts`, served by `main/img-scheme.mts`), where a link's file really is
(`zero:open-artifact` with `mode: "resolve"`), and which pictures a folder holds
(`zero:folder-pictures`). The native window is on the same Mac as the files and asks the engine
for none of that. It reads them itself, from what the snapshot already says:

- Each product's `dir`, its own folder, and `repoPath`, its code folder. A short path a message
  names is looked for under them in the web window's order (`pictureRoots` in
  `renderer/src/message-folders.ts`, ported in `src/rules/roots.rs`): the folders the thread's own
  words name, `<dir>/designs/<thread id>`, `<repoPath>/.claude/worktrees/<thread id>`, `<dir>`,
  `<dir>/designs`, `<dir>/attachments`, `<repoPath>`, `<repoPath>/designs`.
- The row's `result`, `note`, `body` and `answer`, which are the words those folders are read
  out of, and which say whether the conversation draws pictures of its own.

A full path is that file and nothing else. A folder's pictures are listed off the disk, one level
deep, newest first (`main/folder-pictures.mts`). `src/files.rs` builds the places and
`src/pictures.rs` reads them, on a worker thread.

What the engine's finder does besides and this window does not: it hunts the project's folders
for a file by its name when no root has it, reads a file off a thread's branch when its checkout
is gone, and knows a teammate's file that has not arrived yet.

A fixture may say `"arriving": true`: its picture was taken before any picture had come, and the
native window then reads none either, so both draw the place a picture is held.

## Keeping it current

The web window reads the first three again when the row's `updatedAt` changes, and, while an
agent is on the thread, on every push of the event stream and every eight seconds besides. The
native window does the same for the first two: a push reaches it as the next snapshot
`source::follow_door` brings (the snapshot changes while a run works, since the run's own
`activity` and `saying` are in it), and `poll` wakes every eight seconds while the open thread
has a run up.

## Where the native window looks

`thread::ledger(s, item)` is the one place the page gets its lines from. Today it reads a
fixture: `tools/scenes/thread.mjs` answers the four channels in the page for the reference
picture and writes the same answers into the fixture file, keyed by channel, and
`talk::Answers` reads them back:

```json
{ "now": …, "snapshot": { … }, "local": { … },
  "threads": { "<product>/<id>": { "zero:item-history": { "ok": true, "lines": [ … ] },
                                   "zero:session-trace": { "ok": true, "sessions": [ … ] },
                                   "zero:code-change": { "ok": false, "error": "…" },
                                   "zero:compaction-status": null } },
  "unfolded": ["<product>/<id>"], "runsOpen": [2], "thread": "<id>" }
```

`unfolded` lists the threads whose earlier messages the page was showing when its picture
was taken. `runsOpen` lists the folds of what a run ran that the scene pressed open, by their
place among the page's folds, and `thread` names the thread that was open when it is not the
first in Needs you (one an agent is on is In progress). A scene that names files moves its
thread's product into a planted folder, so the snapshot's `dir` and `repoPath` for that product
point there (`tools/pictures.mjs`).

Live, the answers of the open thread are kept in the same shape in `State::threads`, with the
whole logs read once under `zero:session-trace/whole`, and `thread::ledger` reads either. A
conversation is read from them again only when they change (`thread::read`), and each run's log
is parsed once per length it reaches (`talk::run_nodes_once`): a long thread's logs can run to
megabytes and the page is drawn every frame.

## What opening a thread does not do here

The web window also tells the engine that the thread was seen and is being looked at
(`zero:seen-here`, `zero:looking`). Those write, and phase 1 only reads, so a
thread opened in the native window stays unread everywhere else.
