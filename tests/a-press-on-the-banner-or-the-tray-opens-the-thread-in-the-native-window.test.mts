// A PRESS ON THE BANNER OR THE TRAY OPENS THE THREAD IN THE NATIVE WINDOW.
//
// With the day's work done in the native window (desktop/), a press on the
// banner, on a line of the island in the notch or of the corner tag, or on the
// island itself brought this app's own window forward instead, and so took the
// person out of the window they work in. Now, when a native window is at the
// local door and has said where it is within the last three beats, in front or
// not, the press goes to it on the door's event stream alone
// (`zero:open-in-native`, with the thread), and this app's window stays where
// it is. An approval's banner still brings this window: only it draws the card
// that answers one. With no native window there, a press does what it did.
// Measured against main/native-window.mts on a fake clock, the banner, the
// island and the tag driven as the app drives them, and a real door on
// loopback with an event stream open.

import { describe, it, expect, vi, afterEach } from 'vitest';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';

import { NATIVE_BEAT_MS, NATIVE_WORD_LASTS_MS } from '../shared/notify-rules.mts';
import { NATIVE_WINDOW, OPEN_IN_NATIVE, createNativeWindow } from '../main/native-window.mts';
import { createNotifier } from '../main/notify.mts';
import { LOCAL_DOOR_NOTE, openLocalDoor, tellTheDoorToo } from '../main/local-door.mts';
import type { LocalDoor } from '../main/local-door.mts';
import type { IpcHandler, PushListener } from '../main/serve.mts';
import { notchIslandRig, WAITING } from './helpers/notch-island-rig.mts';
import { cornerTagRig } from './helpers/corner-tag-rig.mts';

afterEach(() => { vi.useRealTimers(); });

// A native window as this app hears of it, with `streams` event streams open at the door, on
// timers the test turns.
function aNativeWindow({ streams = 1 } = {}) {
  let clock = 1_000_000;
  const listeners = new Set<PushListener>();
  const pushed: { channel: string; args: unknown[] }[] = [];
  for (let i = 0; i < streams; i += 1) listeners.add((m) => pushed.push(m));
  const native = createNativeWindow({
    listeners,
    now: () => clock,
    setTimer: (() => 0) as unknown as typeof setTimeout,
    clearTimer: (() => {}) as unknown as typeof clearTimeout,
  });
  const says = (focused: boolean) => native.hear({ sender: null }, { focused });
  const advance = (ms: number) => { clock += ms; };
  return { native, listeners, pushed, says, advance };
}

const opened = (id: string | null, product: string | null = null) => ({ channel: OPEN_IN_NATIVE, args: [{ id, product }] });

describe('what the app sends the native window', () => {
  it('sends a press to the event stream at the door, with the thread, once a native window has said where it is', () => {
    const r = aNativeWindow();
    r.says(true);
    expect(r.native.open({ id: 'w-1', product: 'lantern' })).toBe(true);
    expect(r.pushed).toEqual([opened('w-1', 'lantern')]);
  });

  it('sends it to a native window in the background as long as it keeps saying so', () => {
    const r = aNativeWindow();
    r.says(false);
    for (let beat = 0; beat < 10; beat += 1) {
      r.advance(NATIVE_BEAT_MS);
      r.says(false);
    }
    expect(r.native.open({ id: 'w-1', product: 'lantern' })).toBe(true);
    expect(r.pushed).toEqual([opened('w-1', 'lantern')]);
  });

  it('comes forward as it is when no thread is named', () => {
    const r = aNativeWindow();
    r.says(false);
    expect(r.native.open()).toBe(true);
    expect(r.native.open({ id: null })).toBe(true);
    expect(r.pushed).toEqual([opened(null), opened(null)]);
  });

  it('sends nothing once the window has said nothing for three beats, in front or not', () => {
    const r = aNativeWindow();
    r.says(false);
    r.advance(NATIVE_WORD_LASTS_MS - 1);
    expect(r.native.listening()).toBe(true);
    r.advance(1);
    expect(r.native.listening()).toBe(false);
    expect(r.native.open({ id: 'w-1' })).toBe(false);
    expect(r.pushed).toEqual([]);
  });

  it('sends nothing with no stream open, or to a stream no window has spoken on', () => {
    // A stream with no word is a check run from the command line (`--watch-door`), not a window.
    const r = aNativeWindow();
    expect(r.native.open({ id: 'w-1' })).toBe(false);
    const s = aNativeWindow({ streams: 0 });
    s.says(true);
    expect(s.native.open({ id: 'w-1' })).toBe(false);
    expect([...r.pushed, ...s.pushed]).toEqual([]);
  });
});

function fakeNotifications() {
  const shown: { opts: { title: string; body: string }; click(): void }[] = [];
  class N {
    opts: { title: string; body: string };
    handlers: Record<string, () => void> = {};
    static isSupported = () => true;
    constructor(opts: { title: string; body: string }) { this.opts = opts; }
    on(event: string, fn: () => void) { this.handlers[event] = fn; return this; }
    show() { shown.push(this); }
    close() { this.handlers.close?.(); }
    click() { this.handlers.click?.(); }
  }
  return { N, shown };
}

// This app's own window, behind whatever the person is in: what was asked of it.
function fakeWindow() {
  const asked = { shown: 0, focused: 0, sent: [] as [string, unknown][] };
  const window = {
    isDestroyed: () => false, isFocused: () => false, isMinimized: () => false,
    restore() {}, show() { asked.shown += 1; }, focus() { asked.focused += 1; }, on() {},
    webContents: { send: (ch: string, payload: unknown) => { asked.sent.push([ch, payload]); } },
  };
  return { window, asked };
}

const idle = { getSystemIdleTime: () => 0, on() {} };
const item = (id: string, title: string) => ({ id, title, product: 'lantern', productName: 'Lantern', kind: 'item' });
const approval = (id: string, title: string) => ({ id, title, product: 'lantern', productName: 'Lantern', kind: 'approval' });

// A banner up for `arrived`, with a native window that said it is elsewhere, at a door with
// `streams` event streams open.
function aBanner(arrived: ReturnType<typeof item>, { streams = 1 } = {}) {
  vi.useFakeTimers();
  const { N, shown } = fakeNotifications();
  const listeners = new Set<PushListener>();
  const pushed: { channel: string; args: unknown[] }[] = [];
  for (let i = 0; i < streams; i += 1) listeners.add((m) => pushed.push(m));
  const native = createNativeWindow({ listeners });
  native.hear({ sender: null }, { focused: false });
  const { window, asked } = fakeWindow();
  const n = createNotifier({ window, Notification: N, powerMonitor: idle, nativeWindow: native } as unknown as Parameters<typeof createNotifier>[0]);
  n.add([arrived]);
  vi.advanceTimersByTime(2_000);
  expect(shown).toHaveLength(1);
  return { press: () => shown[0]!.click(), pushed, asked };
}

describe('the banner', () => {
  it('a press opens the thread it was about in the native window, and leaves this window where it was', () => {
    const r = aBanner(item('w-1', 'The rota page shows this week'));
    r.press();
    expect(r.pushed).toEqual([opened('w-1', 'lantern')]);
    expect(r.asked).toEqual({ shown: 0, focused: 0, sent: [] });
  });

  it('a press brings this window and opens the thread there, as before, with no native window at the door', () => {
    const r = aBanner(item('w-1', 'The rota page shows this week'), { streams: 0 });
    r.press();
    expect(r.pushed).toEqual([]);
    expect(r.asked).toEqual({ shown: 1, focused: 1, sent: [['zero:open-item', { id: 'w-1' }]] });
  });

  it('a press on an approval still brings this window, which alone draws the card that answers it', () => {
    const r = aBanner(approval('a-1', 'Run the migration on the staging copy'));
    r.press();
    expect(r.pushed).toEqual([]);
    expect(r.asked).toEqual({ shown: 1, focused: 1, sent: [] });
  });
});

describe('the island in the notch', () => {
  // The island up over another app, with a native window there, in the background.
  function up({ listening = true } = {}) {
    const r = notchIslandRig({ native: true });
    if (listening) r.nativeListens();
    r.nativeSays(false);
    r.hand(WAITING(r.clock() + 60_001));
    r.pageUp();
    expect(r.win().visible).toBe(true);
    return r;
  }
  const openedThere = (r: ReturnType<typeof notchIslandRig>) => r.main.sent.filter(([ch]: [string]) => ch === 'zero:open-item');

  it('a line pressed opens its thread in the native window, and the island shuts', () => {
    const r = up();
    r.point();
    expect(r.lastState().mode).toBe('open');
    r.call('notch-island:go', 'w-1');
    expect(r.pushed).toEqual([opened('w-1')]);
    expect(r.lastState().mode).toBe('shut');
    expect(r.main.shown).toBe(0);
    expect(openedThere(r)).toEqual([]);
  });

  it('the island pressed brings the native window forward as it is', () => {
    const r = up();
    r.call('notch-island:go', null);
    expect(r.pushed).toEqual([opened(null)]);
    expect(r.main.shown).toBe(0);
    expect(r.main.focused).toBe(false);
  });

  it('with no native window at the door, a press brings this window as before', () => {
    const r = up({ listening: false });
    r.call('notch-island:go', 'w-1');
    expect(r.pushed).toEqual([]);
    expect(r.main.shown).toBe(1);
    expect(openedThere(r)).toEqual([['zero:open-item', { id: 'w-1' }]]);
  });

  it('"Turn off in Settings…" still opens this window, where Settings are', () => {
    const r = up();
    r.call('notch-island:settings');
    expect(r.pushed).toEqual([]);
    expect(r.main.shown).toBe(1);
    expect(r.main.sent).toContainEqual(['zero:open-settings', { pane: 'general' }]);
  });
});

describe('the corner tag', () => {
  it('a line pressed opens its thread in the native window, and the tag pressed brings it forward', () => {
    const r = cornerTagRig({ native: true });
    r.nativeListens();
    r.nativeSays(false);
    r.call('corner-tag:go', 'w-1');
    r.call('corner-tag:go', null);
    expect(r.pushed).toEqual([opened('w-1'), opened(null)]);
    expect(r.main.shown).toBe(0);
    expect(r.main.sent).toEqual([]);
  });

  it('with no native window at the door, a press brings this window as before', () => {
    const r = cornerTagRig({ native: true });
    r.nativeSays(false);
    r.call('corner-tag:go', 'w-1');
    expect(r.pushed).toEqual([]);
    expect(r.main.shown).toBe(1);
    expect(r.main.sent).toEqual([['zero:open-item', { id: 'w-1' }]]);
  });
});

describe('the door', () => {
  const doors: LocalDoor[] = [];
  const folders: string[] = [];
  afterEach(async () => {
    for (const door of doors.splice(0)) await door.close();
    for (const dir of folders.splice(0)) fs.rmSync(dir, { recursive: true, force: true });
  });

  it('carries the press to the native window on its event stream, and never to this app\'s own window', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'open-in-native-'));
    folders.push(dir);
    const listeners = new Set<PushListener>();
    const native = createNativeWindow({ listeners });
    const ownWindow: [string, unknown[]][] = [];
    tellTheDoorToo({ send: (channel: string, ...args: unknown[]) => { ownWindow.push([channel, args]); } }, listeners);
    const channels = new Map<string, IpcHandler>([['zero:snapshot', () => ({ items: [] })]]);
    const door = await openLocalDoor({ channels, note: path.join(dir, LOCAL_DOOR_NOTE), listeners, native });
    doors.push(door);
    let heard = '';
    const stream = await new Promise<http.ClientRequest>((resolve) => {
      const req = http.get({ host: '127.0.0.1', port: door.port, path: `/events?token=${door.token}&device=mercury-desktop` }, (res) => {
        res.setEncoding('utf8');
        res.on('data', (chunk: string) => { heard += chunk; });
        res.once('data', () => resolve(req));
      });
    });
    const said = await fetch(`http://127.0.0.1:${door.port}/api/${encodeURIComponent(NATIVE_WINDOW)}`, {
      method: 'POST', headers: { 'content-type': 'application/json', 'x-agentbox-token': door.token }, body: JSON.stringify({ focused: false }),
    });
    expect(said.status).toBe(200);
    expect(native.open({ id: 'w-1', product: 'lantern' })).toBe(true);
    for (let tries = 0; tries < 100 && !heard.includes(OPEN_IN_NATIVE); tries += 1) await new Promise((r) => setTimeout(r, 20));
    const pushes = heard.split('\n').filter((line) => line.startsWith('data:')).map((line) => JSON.parse(line.slice('data:'.length)));
    expect(pushes).toEqual([opened('w-1', 'lantern')]);
    expect(ownWindow).toEqual([]);
    stream.destroy();
  });
});
