// THE APP HEARS WHETHER THE NATIVE WINDOW IS IN FRONT, AND STOPS BELIEVING A
// WINDOW THAT WENT QUIET.
//
// The native window (desktop/) does the day's work beside this app's own
// window, through main/local-door.mts. This app judged whether the person was
// at it by its own window's focus alone, so while they worked in the native
// window it took them to be in another app. Now the native window says whether
// it is in front, on a channel only the local door answers, each time that
// changes and every beat while it stays, and this app keeps the word with the
// moment it heard it. Measured against main/native-window.mts with a fake
// clock, and against a real door on loopback: a word not said again stops
// counting after three beats, and the word of a window that hung up its event
// stream stops counting at once.

import { describe, it, expect, afterEach } from 'vitest';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';

import { NATIVE_BEAT_MS, NATIVE_WORD_LASTS_MS, nativeInFront } from '../shared/notify-rules.mts';
import { NATIVE_WINDOW, createNativeWindow } from '../main/native-window.mts';
import { LOCAL_DOOR_NOTE, openLocalDoor } from '../main/local-door.mts';
import type { LocalDoor } from '../main/local-door.mts';
import type { IpcHandler, PushListener } from '../main/serve.mts';

const root = path.join(import.meta.dirname, '..');

// A native window as this app hears of it, on a clock and timers the test turns.
function rig({ streams = 1 } = {}) {
  let clock = 1_000_000;
  let timerId = 0;
  const timers: { id: number; fn: () => void; at: number }[] = [];
  const listeners = new Set<PushListener>();
  const open: PushListener[] = [];
  for (let i = 0; i < streams; i += 1) {
    const stream: PushListener = () => {};
    listeners.add(stream);
    open.push(stream);
  }
  const native = createNativeWindow({
    listeners,
    now: () => clock,
    setTimer: ((fn: () => void, ms: number) => { const id = ++timerId; timers.push({ id, fn, at: clock + ms }); return id; }) as unknown as typeof setTimeout,
    clearTimer: ((id: number) => { const i = timers.findIndex((t) => t.id === id); if (i >= 0) timers.splice(i, 1); }) as unknown as typeof clearTimeout,
  });
  const told: string[] = [];
  native.on('focus', () => told.push('focus'));
  native.on('blur', () => told.push('blur'));
  const say = (word: unknown) => native.hear({ sender: null }, word);
  const advance = (ms: number) => {
    const target = clock + ms;
    for (;;) {
      timers.sort((a, b) => a.at - b.at);
      const next = timers[0];
      if (!next || next.at > target) break;
      timers.shift();
      clock = next.at;
      next.fn();
    }
    clock = target;
  };
  // The stream at the door closes, as it does when the window quits or crashes.
  const hangUp = (i = 0) => { listeners.delete(open[i]!); native.hungUp(); };
  const inFront = () => nativeInFront(native.said(), clock);
  return { native, say, advance, told, hangUp, inFront, listeners, clock: () => clock };
}

describe('the rule: a native window is in front while its word is fresh', () => {
  it('is believed for three beats after it said so, and not a moment longer', () => {
    expect(NATIVE_WORD_LASTS_MS).toBe(3 * NATIVE_BEAT_MS);
    const word = { focused: true, at: 1_000 };
    expect(nativeInFront(word, 1_000)).toBe(true);
    expect(nativeInFront(word, 1_000 + NATIVE_WORD_LASTS_MS - 1)).toBe(true);
    expect(nativeInFront(word, 1_000 + NATIVE_WORD_LASTS_MS)).toBe(false);
  });

  it('is not in front when it said it is not, or has said nothing', () => {
    expect(nativeInFront({ focused: false, at: 1_000 }, 1_000)).toBe(false);
    expect(nativeInFront(null, 1_000)).toBe(false);
    expect(nativeInFront(undefined, 1_000)).toBe(false);
  });

  it('beats as often as the native window says it again', () => {
    const front = fs.readFileSync(path.join(root, 'desktop', 'src', 'front.rs'), 'utf8');
    expect(front).toContain(`const BEAT: Duration = Duration::from_secs(${NATIVE_BEAT_MS / 1000});`);
    expect(front).toContain(`pub const SAY_WHERE: &str = "${NATIVE_WINDOW}";`);
  });
});

describe('what the app keeps of the native window', () => {
  it('keeps what it said with the moment it was heard, and hears it come to the front', () => {
    const r = rig();
    r.say({ focused: true });
    expect(r.native.said()).toEqual({ focused: true, at: r.clock() });
    expect(r.told).toEqual(['focus']);
    expect(r.inFront()).toBe(true);
  });

  it('hears it come to the front once however many beats keep it there, and leave once', () => {
    const r = rig();
    for (let beat = 0; beat < 10; beat += 1) {
      r.say({ focused: true });
      r.advance(NATIVE_BEAT_MS);
    }
    expect(r.told).toEqual(['focus']);
    expect(r.inFront()).toBe(true);
    r.say({ focused: false });
    expect(r.told).toEqual(['focus', 'blur']);
    expect(r.native.said()).toEqual({ focused: false, at: r.clock() });
    // A word that it is elsewhere, going stale, leaves nothing to stop believing.
    r.advance(10 * NATIVE_WORD_LASTS_MS);
    expect(r.told).toEqual(['focus', 'blur']);
  });

  it('stops believing a window in front that stopped saying so', () => {
    const r = rig();
    r.say({ focused: true });
    r.advance(NATIVE_WORD_LASTS_MS - 1);
    expect(r.told).toEqual(['focus']);
    expect(r.inFront()).toBe(true);
    r.advance(1);
    expect(r.told).toEqual(['focus', 'blur']);
    expect(r.native.said()).toBeNull();
    expect(r.inFront()).toBe(false);
  });

  it('forgets it the moment the last event stream at the door closes', () => {
    const r = rig();
    r.say({ focused: true });
    r.hangUp();
    expect(r.told).toEqual(['focus', 'blur']);
    expect(r.native.said()).toBeNull();
    expect(r.native.listening()).toBe(false);
  });

  it('keeps it while another stream is still open at the door', () => {
    const r = rig({ streams: 2 });
    r.say({ focused: true });
    r.hangUp(1);
    expect(r.told).toEqual(['focus']);
    expect(r.inFront()).toBe(true);
  });

  it('counts a native window as listening once it has spoken and while its stream is open', () => {
    const r = rig();
    // A stream with no word is a check run from the command line, not a window.
    expect(r.native.listening()).toBe(false);
    r.say({ focused: false });
    expect(r.native.listening()).toBe(true);
    r.hangUp();
    expect(r.native.listening()).toBe(false);
    // A word with no stream: in front, but nothing at the door to send a thread to.
    const quiet = rig({ streams: 0 });
    quiet.say({ focused: true });
    expect(quiet.inFront()).toBe(true);
    expect(quiet.native.listening()).toBe(false);
  });

  it('takes only a yes or a no', () => {
    const r = rig();
    for (const word of [{ focused: 'yes' }, { focused: 1 }, {}, null, undefined, 'focused']) r.say(word);
    expect(r.native.said()).toBeNull();
    expect(r.told).toEqual([]);
  });
});

describe('the door', () => {
  const doors: LocalDoor[] = [];
  const folders: string[] = [];
  afterEach(async () => {
    for (const door of doors.splice(0)) await door.close();
    for (const dir of folders.splice(0)) fs.rmSync(dir, { recursive: true, force: true });
  });
  async function aDoor({ native = true } = {}) {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'native-window-door-'));
    folders.push(dir);
    const listeners = new Set<PushListener>();
    const channels = new Map<string, IpcHandler>([['zero:snapshot', () => ({ items: [] })]]);
    const said = createNativeWindow({ listeners });
    const door = await openLocalDoor({ channels, note: path.join(dir, LOCAL_DOOR_NOTE), listeners, ...(native ? { native: said } : {}) });
    doors.push(door);
    return { door, said, listeners, channels };
  }
  const say = (door: LocalDoor, word: unknown) => fetch(`http://127.0.0.1:${door.port}/api/${encodeURIComponent(NATIVE_WINDOW)}`, {
    method: 'POST', headers: { 'content-type': 'application/json', 'x-agentbox-token': door.token }, body: JSON.stringify(word),
  });

  it('is told where the native window is, on a channel of its own that the app does not have', async () => {
    const { door, said, channels } = await aDoor();
    expect((await say(door, { focused: true })).status).toBe(200);
    expect(said.said()).toMatchObject({ focused: true });
    // The phone's copy is made from the app's own Map, and it did not grow.
    expect([...channels.keys()]).toEqual(['zero:snapshot']);
    const { door: other } = await aDoor({ native: false });
    expect((await say(other, { focused: true })).status).toBe(404);
  });

  it('hears a native window hang up its event stream, once it has left the listeners', async () => {
    const { door, said, listeners } = await aDoor();
    const stream = await new Promise<http.ClientRequest>((resolve) => {
      const req = http.get({ host: '127.0.0.1', port: door.port, path: `/events?token=${door.token}&device=mercury-desktop` }, (res) => { res.once('data', () => resolve(req)); });
    });
    expect(listeners.size).toBe(1);
    await say(door, { focused: true });
    expect(said.listening()).toBe(true);
    stream.destroy();
    for (let tries = 0; tries < 100 && said.said(); tries += 1) await new Promise((r) => setTimeout(r, 20));
    expect(said.said()).toBeNull();
    expect(listeners.size).toBe(0);
  });
});

describe('main/main.mts', () => {
  const source = fs.readFileSync(path.join(root, 'main', 'main.mts'), 'utf8');

  it('keeps what the native window says only for a copy whose config opens the door, and hands it to the door', () => {
    expect(source).toMatch(/const nativeWindow = config\.localDoor === true \? createNativeWindow\(\{ listeners: atTheDoor \}\) : null;/);
    expect(source).toMatch(/openLocalDoor\(\{ channels: phoneChannels,[^\n]*listeners: atTheDoor, native: nativeWindow \}\)/);
  });
});
