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
  token?: string;
}

/** Opens the door and leaves its note. Rejects if the port cannot be had or the note cannot be written. */
export async function openLocalDoor({ channels, note, listeners = new Set(), token = newToken() }: LocalDoorOptions): Promise<LocalDoor> {
  // A folder that is certain to hold no index.html: the note's own name with a suffix.
  const noScreen = `${note}.no-screen`;
  const server = http.createServer(createHandler({ channels, token, listeners, dist: noScreen }));
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
