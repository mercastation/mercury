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
//
// WHAT GOES THROUGH IT, EACH WAY. The native window calls the channels the web
// window calls, and four that only this door answers:
// - WINDOW_KEEPS (below): what the web window keeps for itself, so both windows
//   show the same Inbox.
// - NATIVE_WINDOW (main/native-window.mts): whether the native window is in
//   front, at each change and every five seconds, so no banner and no island
//   comes up over the window the person is working in.
// - OPEN_SETTINGS (below): its rail's Settings button, which brings this app's
//   window forward on its Settings until the native window has its own.
// - OPEN_CLOUD_SETUP (below): its cloud menu's Manage, Fix, Try again and Set
//   up the cloud, which bring this app's window forward on the cloud's setup.
// On its event stream the app tells it two things, and nothing else:
// - TOLD_AT_THE_DOOR (below): the inbox changed, so it reads it again now.
// - OPEN_IN_NATIVE (main/native-window.mts): the banner or the tray was
//   pressed, and on which thread, so the person lands in the window they work
//   in. The app's own word, never one of its window's pushes.

import http from 'node:http';
import type { AddressInfo } from 'node:net';
import fs from 'node:fs';
import path from 'node:path';
import { createHandler, newToken } from './serve.mts';
import type { IpcHandler, PushListener } from './serve.mts';
import { NATIVE_WINDOW } from './native-window.mts';
import type { NativeWindow } from './native-window.mts';

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
  /**
   * What answers `NATIVE_WINDOW` and hears the event stream close (main/native-window.mts).
   * Without it the channel is not there.
   */
  native?: Pick<NativeWindow, 'hear' | 'hungUp'> | null;
  /** What answers `OPEN_SETTINGS` (see `openSettings`). Without it the channel is not there. */
  settings?: IpcHandler;
  /** What answers `OPEN_CLOUD_SETUP` (see `openCloudSetup`). Without it the channel is not there. */
  cloudSetup?: IpcHandler;
  token?: string;
}

// THE SETTINGS ARE STILL THE WEB WINDOW'S. The native window has no Settings page
// yet, and a Settings button there that went nowhere left no way to reach them
// from it. So this door has a channel that does what the tray's "Turn off in
// Settings…" does (main/corner-tag.mts): bring this app's window forward on its
// Settings. It takes nothing from the caller, so it can open Settings and nothing else.
export const OPEN_SETTINGS = 'zero:open-settings-here';

/**
 * Answers `OPEN_SETTINGS`. `forward` brings the app's window to the front and says whether
 * there is one; `tell` sends that window its own push, the one the tray sends.
 */
export function openSettings(forward: () => boolean, tell: (channel: string, payload: unknown) => void): IpcHandler {
  return () => {
    if (!forward()) return { ok: false, error: 'the window is gone' };
    tell('zero:open-settings', { pane: 'general' });
    return { ok: true };
  };
}

// SO IS THE CLOUD'S SETUP. The native window draws the cloud menu, and four of
// its buttons (Manage, Fix, Try again, Set up the cloud) open, in the web
// window, its own screen for the cloud's accounts and sign-ins
// (components/CloudSetup.tsx), which the native window does not have. So this
// door has a channel that brings this app's window forward on that screen. It
// takes nothing from the caller either, so it can open that screen and nothing else.
export const OPEN_CLOUD_SETUP = 'zero:open-cloud-setup-here';

/**
 * Answers `OPEN_CLOUD_SETUP`. `forward` brings the app's window to the front and says whether
 * there is one; `tell` sends that window the push that opens its cloud setup.
 */
export function openCloudSetup(forward: () => boolean, tell: (channel: string) => void): IpcHandler {
  return () => {
    if (!forward()) return { ok: false, error: 'the window is gone' };
    tell('zero:open-cloud-setup');
    return { ok: true };
  };
}

/** Opens the door and leaves its note. Rejects if the port cannot be had or the note cannot be written. */
export async function openLocalDoor({ channels, note, listeners = new Set(), keeps, native = null, settings, cloudSetup, token = newToken() }: LocalDoorOptions): Promise<LocalDoor> {
  // A folder that is certain to hold no index.html: the note's own name with a suffix.
  const noScreen = `${note}.no-screen`;
  // The door's own channels are looked up before the app's, and the app's Map is left as it was:
  // the phone's copy is made from that same Map and must not grow a channel meant for this door.
  const own: [string, IpcHandler][] = [];
  if (keeps) own.push([WINDOW_KEEPS, keeps]);
  if (native) own.push([NATIVE_WINDOW, native.hear]);
  if (settings) own.push([OPEN_SETTINGS, settings]);
  if (cloudSetup) own.push([OPEN_CLOUD_SETUP, cloudSetup]);
  const answered = own.length ? new Map<string, IpcHandler>([...channels, ...own]) : channels;
  const server = http.createServer(createHandler({ channels: answered, token, listeners, hungUp: () => native?.hungUp(), dist: noScreen }));
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
