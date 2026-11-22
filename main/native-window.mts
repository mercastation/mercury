// THE NATIVE WINDOW, AS THIS APP HEARS OF IT. desktop/ draws the inbox natively
// beside this app's own window and reaches it through main/local-door.mts.
// While the person works there this app's window has no focus, and the banner
// and the tray took that to mean they were in another app. So the native
// window says whether it is in front, and this keeps its last word with the
// moment it was heard, and says 'focus' and 'blur' as a BrowserWindow does.
//
// A word stops counting when it is not said again (shared/notify-rules.mts,
// nativeInFront) or when the last event stream at the door hangs up: a window
// that quit or crashed hangs up, and one that hung stops beating.

import { NATIVE_WORD_LASTS_MS, nativeInFront } from '../shared/notify-rules.mts';
import type { NativeWord } from '../shared/notify-rules.mts';

/** What the native window says of itself at the door: `{ focused }`. Only the local door answers it. */
export const NATIVE_WINDOW = 'zero:native-window';

type Heard = 'focus' | 'blur';
// `PushListener` and `IpcHandler` of main/serve.mts, named here rather than imported: the
// banner and the tray load this file, and a window that is only a hub's client loads them
// without reaching the back end behind serve.mts
// (tests/a-window-can-be-a-client-of-a-hub-and-runs-no-agents).
type PushListener = (message: { channel: string; args: unknown[] }) => void;
type Handler = (event: { sender: unknown }, payload?: unknown) => unknown;

export interface NativeWindow {
  /** Answers `NATIVE_WINDOW`. */
  hear: Handler;
  /** Its last word, or null once it stopped counting. */
  said(): NativeWord | null;
  /**
   * Whether a native window is at the door to be sent to: a stream is open and a window has
   * spoken. A stream that never spoke is a check run from the command line (`--watch-door`).
   */
  listening(): boolean;
  /** An event stream at the door closed. */
  hungUp(): void;
  on(event: Heard, fn: () => void): void;
  off(event: Heard, fn: () => void): void;
}

export interface NativeWindowOptions {
  /** Who listens on the door's event stream (main/local-door.mts). */
  listeners: Set<PushListener>;
  now?: () => number;
  setTimer?: typeof setTimeout;
  clearTimer?: typeof clearTimeout;
}

export function createNativeWindow({ listeners, now = Date.now, setTimer = setTimeout, clearTimer = clearTimeout }: NativeWindowOptions): NativeWindow {
  let word: NativeWord | null = null;
  // What 'focus' and 'blur' last told, so each change is told once. Worked out
  // again from the word at the moment it stops counting would already read
  // "not in front", and the blur would never be told.
  let front = false;
  let quiet: ReturnType<typeof setTimeout> | null = null;
  const heard: Record<Heard, Set<() => void>> = { focus: new Set(), blur: new Set() };

  const settle = () => {
    const is = nativeInFront(word, now());
    if (is === front) return;
    front = is;
    for (const fn of heard[is ? 'focus' : 'blur']) {
      try { fn(); } catch { /* one listener's fault is not the others' */ }
    }
  };
  const become = (next: NativeWord | null) => {
    word = next;
    if (quiet) { clearTimer(quiet); quiet = null; }
    // Nothing else would ask again once a window in front falls silent.
    if (next?.focused) quiet = setTimer(() => { quiet = null; become(null); }, NATIVE_WORD_LASTS_MS);
    settle();
  };

  return {
    hear: (_event, payload) => {
      const focused = (payload as { focused?: unknown } | null | undefined)?.focused;
      if (typeof focused === 'boolean') become({ focused, at: now() });
      return null;
    },
    said: () => word,
    listening: () => listeners.size > 0 && word !== null,
    hungUp: () => { if (!listeners.size && word) become(null); },
    on: (event, fn) => { heard[event].add(fn); },
    off: (event, fn) => { heard[event].delete(fn); },
  };
}
