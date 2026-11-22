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
  Each log is lines of `HH:MM:SS  <what happened>` in UTC (`renderer/src/notes.ts`).

Not read yet. The web window turns each log into the run's steps ("Read 3 files", "Ran the
tests") between the messages; the native page draws each run as the one line the web window
shows for a run that left no log, "An agent picked it up".

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
door's `/events` stream is the same push `source::follow_door` already waits on, so the natural
place to fetch is there: when the open thread's row changes, or on any push while it is running.

## Where the native window looks

`thread::ledger(s, item)` is the one place the page gets its lines from. Today it reads a
fixture: `tools/scenes/thread.mjs` answers the four channels in the page for the reference
picture and writes the same answers into the fixture file, keyed by channel, and
`talk::Answers` reads them back:

```json
{ "now": …, "snapshot": { … }, "local": { … },
  "threads": { "<product>/<id>": { "zero:item-history": { "ok": true, "lines": [ … ] },
                                   "zero:session-trace": { "ok": true, "sessions": [] },
                                   "zero:code-change": { "ok": false, "error": "…" },
                                   "zero:compaction-status": null } },
  "unfolded": ["<product>/<id>"] }
```

`unfolded` lists the threads whose earlier messages the page was showing when its picture
was taken. A scene that names files moves its thread's product into a planted folder, so the
snapshot's `dir` and `repoPath` for that product point there (`tools/pictures.mjs`). For the live path, keep the answers of the open thread beside the snapshot in the
same shape and have `thread::ledger` read them from there.

## What opening a thread does not do here

The web window also tells the engine that the thread was seen and is being looked at
(`zero:seen-here`, `zero:looking`). Those write, and phase 1 only reads, so a
thread opened in the native window stays unread everywhere else.
