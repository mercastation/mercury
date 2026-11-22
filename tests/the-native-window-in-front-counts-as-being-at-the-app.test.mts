// WHILE THE PERSON WORKS IN THE NATIVE WINDOW, THEY ARE AT THE APP.
//
// The banner and the tray decided whether the person was at the app by this
// app's own window's focus alone. With the day's work done in the native
// window (desktop/), this window had no focus, so the banner spoke and the
// island in the notch stood over the very window the person was using. Now the
// native window's word (main/native-window.mts) counts as this window's focus
// wherever either is asked (appInFront, shared/notify-rules.mts), and coming
// back to the native window counts as coming back to the app. Measured by
// driving main/notify.mts, main/notch-island.mts and main/corner-tag.mts as the
// app does, beside a native window that says where it is.

import { describe, it, expect, vi, afterEach } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

import { AWAY_AFTER_MS, NATIVE_BEAT_MS, NATIVE_WORD_LASTS_MS, appInFront } from '../shared/notify-rules.mts';
import { createNotifier, installNotifier } from '../main/notify.mts';
import { createNativeWindow } from '../main/native-window.mts';
import { notchIslandRig, WAITING } from './helpers/notch-island-rig.mts';
import { cornerTagRig } from './helpers/corner-tag-rig.mts';

const root = path.join(import.meta.dirname, '..');

afterEach(() => { vi.useRealTimers(); });

function fakeNotifications() {
  const shown = [];
  const closed = [];
  class N {
    constructor(opts) { this.opts = opts; this.handlers = {}; }
    on(event, fn) { this.handlers[event] = fn; return this; }
    show() { shown.push(this); }
    close() { closed.push(this); this.handlers.close?.(); }
    click() { this.handlers.click?.(); }
  }
  N.isSupported = () => true;
  return { N, shown, closed };
}

// This app's own window, behind whatever the person is in.
const fakeWindow = () => ({
  focusedNow: false, heard: {}, sent: [],
  isDestroyed: () => false,
  isFocused() { return this.focusedNow; },
  isMinimized: () => false, restore() {}, show() {}, focus() {},
  on(event, fn) { this.heard[event] = fn; },
  webContents: { send(ch, payload) { this.sent?.push({ ch, payload }); } },
});
const idle = (box) => ({ getSystemIdleTime: () => box.ms / 1000, on: (event, fn) => { box.heard[event] = fn; } });
const item = (id, title) => ({ id, title, product: 'lantern', productName: 'Lantern', kind: 'item' });

// A native window this app hears of, on vitest's clock (made after its timers are faked).
function aNativeWindow() {
  const native = createNativeWindow({ listeners: new Set() });
  return { native, says: (focused) => native.hear({ sender: null }, { focused }) };
}

function installed({ box = { ms: 0, heard: {} } } = {}) {
  const { N, shown, closed } = fakeNotifications();
  const { native, says } = aNativeWindow();
  const handlers = {};
  const notifier = installNotifier({
    app: { getAppPath: () => root }, window: fakeWindow(), Notification: N, powerMonitor: idle(box),
    ipcMain: { handle: (ch, fn) => { handlers[ch] = fn; } }, nativeImage: null, nativeWindow: native,
  });
  const arrive = (...arrivals) => handlers['zero:notify']({}, { arrivals });
  return { notifier, shown, closed, says, arrive, box };
}

describe('the rule', () => {
  const now = 1_000_000;

  it('the app is in front when its own window is, or when the native window says it is', () => {
    expect(appInFront({ window: true, native: null, now })).toBe(true);
    expect(appInFront({ window: false, native: { focused: true, at: now - NATIVE_BEAT_MS }, now })).toBe(true);
    expect(appInFront({ window: true, native: { focused: false, at: now }, now })).toBe(true);
  });

  it('and not when the native window is elsewhere, has been quiet too long, or was never heard', () => {
    expect(appInFront({ window: false, native: { focused: false, at: now }, now })).toBe(false);
    expect(appInFront({ window: false, native: { focused: true, at: now - NATIVE_WORD_LASTS_MS }, now })).toBe(false);
    expect(appInFront({ window: false, native: null, now })).toBe(false);
    expect(appInFront({ now })).toBe(false);
  });
});

describe('the banner', () => {
  it('says nothing while the person is in the native window, and keeps nothing for later', () => {
    vi.useFakeTimers();
    const { N, shown } = fakeNotifications();
    const { native, says } = aNativeWindow();
    says(true);
    const n = createNotifier({ window: fakeWindow(), Notification: N, powerMonitor: idle({ ms: 0, heard: {} }), nativeWindow: native });
    n.add([item('w-1', 'The rota page shows this week')]);
    vi.advanceTimersByTime(5_000);
    expect(shown).toHaveLength(0);
    expect(n._pending()).toHaveLength(0);
  });

  it('speaks when the person left the native window for another app', () => {
    vi.useFakeTimers();
    const { N, shown } = fakeNotifications();
    const { native, says } = aNativeWindow();
    says(true);
    says(false);
    const n = createNotifier({ window: fakeWindow(), Notification: N, powerMonitor: idle({ ms: 0, heard: {} }), nativeWindow: native });
    n.add([item('w-1', 'The rota page shows this week')]);
    vi.advanceTimersByTime(2_000);
    expect(shown).toHaveLength(1);
  });

  it('speaks once a native window in front has stopped saying so', () => {
    vi.useFakeTimers();
    const { N, shown } = fakeNotifications();
    const { native, says } = aNativeWindow();
    says(true);
    vi.advanceTimersByTime(NATIVE_WORD_LASTS_MS);
    const n = createNotifier({ window: fakeWindow(), Notification: N, powerMonitor: idle({ ms: 0, heard: {} }), nativeWindow: native });
    n.add([item('w-1', 'The rota page shows this week')]);
    vi.advanceTimersByTime(2_000);
    expect(shown).toHaveLength(1);
  });

  it('speaks when the native window is in front and nobody has touched the computer', () => {
    vi.useFakeTimers();
    const { N, shown } = fakeNotifications();
    const { native, says } = aNativeWindow();
    says(true);
    const n = createNotifier({ window: fakeWindow(), Notification: N, powerMonitor: idle({ ms: AWAY_AFTER_MS + 1, heard: {} }), nativeWindow: native });
    n.add([item('w-1', 'The rota page shows this week')]);
    vi.advanceTimersByTime(2_000);
    expect(shown).toHaveLength(1);
  });

  it('is taken down when the person comes back to the native window, and the next time away may speak again', () => {
    vi.useFakeTimers();
    const r = installed();
    r.arrive(item('w-1', 'one'));
    vi.advanceTimersByTime(2_000);
    expect(r.shown).toHaveLength(1);
    expect(r.notifier.told().map((t) => t.id)).toEqual(['w-1']);
    r.says(true);
    expect(r.closed).toEqual([r.shown[0]]);
    expect(r.notifier.told()).toEqual([]);
    r.says(false);
    r.arrive(item('w-2', 'two'));
    vi.advanceTimersByTime(2_000);
    expect(r.shown).toHaveLength(2);
    expect(r.shown[1].opts.body).toBe('two');
  });

  it('counts unlocking the screen with the native window still in front as coming back', () => {
    vi.useFakeTimers();
    const r = installed({ box: { ms: AWAY_AFTER_MS + 1_000, heard: {} } });
    r.says(true);
    r.arrive(item('w-1', 'one'));
    vi.advanceTimersByTime(2_000);
    expect(r.shown).toHaveLength(1);
    r.box.ms = 0;
    // Still in front: its beat says so again, and nothing comes to the front to be heard.
    r.says(true);
    expect(r.closed).toEqual([]);
    r.box.heard['unlock-screen']();
    expect(r.closed).toEqual([r.shown[0]]);
  });
});

describe('the island in the notch', () => {
  // A thread that started waiting after everything up to now.
  const newer = (r, ids) => WAITING(r.clock() + 60_001, ids);

  it('stays away while the person is in the native window, however new what waits', () => {
    const r = notchIslandRig({ native: true });
    r.nativeSays(true);
    r.advance(1_000);
    r.hand(newer(r));
    r.pageUp();
    expect(r.win()?.visible ?? false).toBe(false);
    // The same with nobody in the native window shows.
    const s = notchIslandRig({ native: true });
    s.hand(newer(s));
    s.pageUp();
    expect(s.win().visible).toBe(true);
  });

  it('comes when they leave the native window and something new arrives, counting what they had there as seen', () => {
    const r = notchIslandRig({ native: true });
    r.nativeSays(true);
    r.hand(newer(r));
    r.pageUp();
    r.advance(1_000);
    r.nativeSays(false);
    expect(r.config.cornerTagLookedAt).toBe(r.clock());
    expect(r.win()?.visible ?? false).toBe(false);
    r.hand(newer(r, ['w-2']));
    r.pageUp();
    expect(r.win().visible).toBe(true);
    expect(r.lastState().ready.map((x) => x.id)).toEqual(['w-2']);
  });

  it('goes the moment the person is back in the native window', () => {
    const r = notchIslandRig({ native: true });
    r.hand(newer(r));
    r.pageUp();
    expect(r.win().visible).toBe(true);
    r.nativeSays(true);
    expect(r.win().visible).toBe(false);
  });

  it('no longer stays away for a native window that stopped saying it is in front, or hung up', () => {
    const r = notchIslandRig({ native: true });
    r.nativeSays(true);
    r.advance(NATIVE_WORD_LASTS_MS);
    r.hand(newer(r));
    r.pageUp();
    expect(r.win().visible).toBe(true);
    const s = notchIslandRig({ native: true });
    s.nativeListens();
    s.nativeSays(true);
    s.nativeHangsUp();
    s.hand(newer(s));
    s.pageUp();
    expect(s.win().visible).toBe(true);
  });

  it('stops listening to the native window when this app\'s window goes', () => {
    const r = notchIslandRig({ native: true });
    r.hand(newer(r));
    r.pageUp();
    r.mainListeners.closed();
    const saves = r.saved.length;
    r.nativeSays(true);
    r.nativeSays(false);
    expect(r.saved).toHaveLength(saves);
  });
});

describe('the corner tag', () => {
  const waiting = (r, id = 'w-1') => ({ ready: [{ id, title: `Thread ${id}`, says: 'is ready for you', since: r.clock() + 1 }], working: 0 });

  it('stays away while the person is in the native window, and comes when they leave it and something new arrives', () => {
    const r = cornerTagRig({ native: true });
    r.nativeSays(true);
    r.hand(waiting(r));
    r.pageUp();
    expect(r.win()?.visible ?? false).toBe(false);
    r.advance(1_000);
    r.nativeSays(false);
    expect(r.config.cornerTagLookedAt).toBe(r.clock());
    r.hand(waiting(r, 'w-2'));
    r.pageUp();
    expect(r.win().visible).toBe(true);
    r.nativeSays(true);
    expect(r.win().visible).toBe(false);
  });
});

describe('main/main.mts', () => {
  const source = fs.readFileSync(path.join(root, 'main', 'main.mts'), 'utf8');

  it('hands the native window to the banner and to the tray', () => {
    expect(source).toMatch(/installNotifier\(\{[\s\S]{0,400}\n {4}nativeWindow,\n {2}\}\);/);
    expect(source).toMatch(/trayCommon = \{[\s\S]{0,300}told: \(\) => notifier\.told\(\),\n {4}nativeWindow,\n/);
  });
});
