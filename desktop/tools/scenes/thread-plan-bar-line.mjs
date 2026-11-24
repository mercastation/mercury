// A thread whose run stopped because Claude Code is signed out, opened while the plan bar's words
// stand (Claude Code on the computer, signed out): no bar over the conversation, and the line at
// its foot points at the bar on the Inbox (`signInBar` in `renderer/src/live-line.ts`).
import { open } from './thread.mjs';
import { SIGNED_OUT, NOTHING } from './inbox-plan-bar.mjs';

const READINGS = { claude: SIGNED_OUT, codex: NOTHING };

export async function go(h) {
  await h.call('Page.addScriptToEvaluateOnNewDocument', {
    source: `(() => {
      const readings = ${JSON.stringify(READINGS)};
      const was = window.zero;
      Object.defineProperty(window, 'zero', {
        configurable: true,
        get() { return this.__zero__; },
        set(v) { this.__zero__ = new Proxy(v, { get: (t, name) => (name === 'engineSetup' ? async ({ engine } = {}) => readings[engine] : t[name]) }); },
      });
      if (was) window.zero = was;
    })()`,
  });
  const fed = await open(h, {
    unfold: false,
    fromTab: 'Needs you',
    fleet: (snapshot, row) => {
      row.status = 'open';
      snapshot.supervisor.signInNeeded = { [row.id]: 'Claude Code' };
    },
  });
  return { ...fed, 'zero:engine-setup': READINGS };
}
