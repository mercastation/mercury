// THE LOCAL DOOR: the desktop app's own inbox, answering another window on this computer.
//
// main/serve.mts is a door for a browser tab: it boots a second copy of the
// doing half and listens on the loopback address. The native window
// (desktop/, drawn with Rune instead of a web page) needs the first half of
// that and not the second. It must reach the inbox THIS app is running, the
// store one supervisor already owns, not a second copy of it.
//
// So this takes the handlers the desktop app already filed (main/main.mts files
// each one in a Map as it hands it to Electron) and puts serve.mts's request
// handler in front of them, on 127.0.0.1 only.
//
// THE KEY IS NEW AT EVERY LAUNCH. Whoever holds it can run agents on this
// computer, so it is long, random and never a constant, and the port is
// whatever the system gives. A window that wants in finds both in a note this
// app leaves in its own folder, readable by this user only.
//
// THE NOTE OUTLIVES THE APP, ON PURPOSE. Taking it away at quit would cost one
// of the ten before-quit listeners Node allows before it warns
// (tests/quitting-stays-under-nodes-listener-warning), and it would buy
// nothing: a note left behind names a port nothing answers on with that key,
// which is a closed door and not an open one. It carries the pid, so a reader
// can tell a stale note from a live one, and the next launch writes over it.
//
// OFF UNLESS ASKED FOR. `localDoor: true` in zero.config.json turns it on
// (main/main.mts). A copy nobody pointed a native window at opens nothing.
//
// NO SCREEN IS SERVED HERE. serve.mts answers any other path with the built
// web screen, which is how a tab gets its page. This door is for a window that
// draws itself, so it is handed a folder with no screen in it and every such
// path is a plain 404.

import http from 'node:http';
import type { AddressInfo } from 'node:net';
import fs from 'node:fs';
import path from 'node:path';
import { createHandler, newToken } from './serve.mts';
import type { IpcHandler, PushListener } from './serve.mts';

/** The note's name inside the app's own folder. The native window looks for it there. */
export const LOCAL_DOOR_NOTE = 'local-door.json';

// WHAT THE WEB WINDOW KEEPS FOR ITSELF. Which threads count as delivered and
// which as read is not in the store: the web window remembers it in its own
// localStorage (zero.deliveredThrough, zero.unreadSince, zero.seenAt), and
// its lists are decided by it. A native window beside it has to decide from
// the same facts or it shows a different Inbox. So this door has one channel
// of its own, which asks the web window for exactly those three and nothing
// else: the script is fixed here and takes nothing from the caller.
export const WINDOW_KEEPS = 'zero:window-keeps';

const READ_WHAT_THE_WINDOW_KEEPS = `(() => {
  const number = (key) => Number(localStorage.getItem(key) ?? 0) || 0;
  let seenAt = {};
  try { seenAt = JSON.parse(localStorage.getItem('zero.seenAt') ?? '{}') ?? {}; } catch { /* an unreadable list is an empty one */ }
  return { deliveredThrough: number('zero.deliveredThrough'), unreadSince: number('zero.unreadSince'), seenAt };
})()`;

/**
 * Answers `WINDOW_KEEPS`. `run` is the web window's own executeJavaScript; an app with no
 * window open, or one that will not answer, gives null and the caller keeps its own facts.
 */
export function windowKeeps(run: (script: string) => Promise<unknown> | null): IpcHandler {
  return async () => {
    try { return (await run(READ_WHAT_THE_WINDOW_KEEPS)) ?? null; } catch { return null; }
  };
}

// WHAT A NATIVE WINDOW IS TOLD, AND WHAT IT IS NOT. The app tells its own
// window that the inbox changed on one channel, `zero:changed`, with nothing
// in it: the window then asks for the snapshot again. A native window beside
// it had no such word and asked every two seconds instead, which cost this
// process about 2% of a core for as long as that window was open (measured
// 2026-11-21, a 517 KB snapshot each time). So the same word goes to whoever
// listens at the door. Only that channel, and never its arguments: the other
// pushes carry things (an approval's answer, a file's line) that a window
// which never asked has no business being sent.
export const TOLD_AT_THE_DOOR = 'zero:changed';

/**
 * From now on, what the app sends its own window on `TOLD_AT_THE_DOOR` is said at the door too.
 *
 * It wraps the one `send` the app's pushes all go through rather than asking every place
 * that pushes to remember a second listener: the store, the supervisor, the approvals spool
 * and a dozen handlers each push in their own words, and one of them forgotten would be a
 * native window that is quietly stale.
 */
export function tellTheDoorToo(contents: { send(channel: string, ...args: unknown[]): void }, listeners: Set<PushListener>): void {
  const send = contents.send.bind(contents);
  contents.send = (channel: string, ...args: unknown[]) => {
    if (channel === TOLD_AT_THE_DOOR) {
      for (const tell of listeners) {
        try { tell({ channel, args: [] }); } catch { /* a window that hung up is not an error */ }
      }
    }
    send(channel, ...args);
  };
}

export interface LocalDoorNote { port: number; token: string; pid: number; since: number }

export interface LocalDoor {
  port: number;
  token: string;
  close(): Promise<void>;
}

export interface LocalDoorOptions {
  /** The handlers the window's own calls are answered by, by channel. */
  channels: Map<string, IpcHandler>;
  /** Where the note with the port and the key is left. */
  note: string;
  /** Who is told when the app pushes. Empty unless the caller feeds it. */
  listeners?: Set<PushListener>;
  /** What answers `WINDOW_KEEPS` (see `windowKeeps`). Without it the channel is not there. */
  keeps?: IpcHandler;
  token?: string;
}

/** Opens the door and leaves its note. Rejects if the port cannot be had or the note cannot be written. */
export async function openLocalDoor({ channels, note, listeners = new Set(), keeps, token = newToken() }: LocalDoorOptions): Promise<LocalDoor> {
  // A folder that is certain to hold no index.html: the note's own name with a suffix.
  const noScreen = `${note}.no-screen`;
  // The door's own channel is looked up before the app's, and the app's Map is left as it was:
  // the phone's copy is made from that same Map and must not grow a channel meant for this door.
  const answered = keeps ? new Map<string, IpcHandler>([...channels, [WINDOW_KEEPS, keeps]]) : channels;
  const server = http.createServer(createHandler({ channels: answered, token, listeners, dist: noScreen }));
  await new Promise<void>((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => { server.off('error', reject); resolve(); });
  });
  const port = (server.address() as AddressInfo).port;
  const words: LocalDoorNote = { port, token, pid: process.pid, since: Date.now() };
  try {
    fs.mkdirSync(path.dirname(note), { recursive: true });
    // Written beside itself and moved into place, so a reader never sees half a note, and
    // made with its final permissions rather than narrowed after the key is already on disk.
    const draft = `${note}.${process.pid}.tmp`;
    fs.writeFileSync(draft, JSON.stringify(words), { mode: 0o600 });
    fs.renameSync(draft, note);
  } catch (err) {
    await new Promise<void>((resolve) => server.close(() => resolve()));
    throw err;
  }
  return {
    port,
    token,
    close() {
      // Only this door's own note: a second copy of the app may have left a newer one.
      try {
        const there = JSON.parse(fs.readFileSync(note, 'utf8')) as Partial<LocalDoorNote>;
        if (there.token === token) fs.rmSync(note, { force: true });
      } catch { /* already gone, or not ours to read */ }
      return new Promise<void>((resolve) => {
        server.close(() => resolve());
        // An event stream a window left open would hold the close until it hung up.
        server.closeAllConnections();
      });
    },
  };
}
