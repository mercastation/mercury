// The Inbox with the bar that says agents cannot start until a plan is signed in (`PlanBar` in
// `renderer/src/components/PlanSetup.tsx`, drawn by `App.tsx` while `planWords` holds words).
// The page asks `zero:engine-setup` with `ready` for each agent as it opens (`checkPlan`), and
// the harness's bridge has no such channel, so this answers it: here neither agent is on the
// computer, and each reading is sure of it (`certain`), which is what raises the bar.
//
// `readings` is what the bridge answers, by agent, in the engine's own shape (`{ ok, phase,
// found, signedIn, certain }`), and goes in the fixture under the channel's name for the native
// window to read the same.
import { go as toInbox } from './inbox.mjs';

export const NOTHING = { ok: true, phase: 'idle', found: false, signedIn: false, certain: true };
export const SIGNED_OUT = { ok: true, phase: 'idle', found: true, signedIn: false, certain: true };

export async function withReadings(h, readings, then = toInbox) {
  await h.call('Page.addScriptToEvaluateOnNewDocument', {
    source: `(() => {
      const readings = ${JSON.stringify(readings)};
      window.zero = { ...window.zero, engineSetup: async ({ engine } = {}) => readings[engine] ?? { ok: false, error: 'Unknown coding agent.' } };
    })()`,
  });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await then(h);
  return { 'zero:engine-setup': readings };
}

export const go = (h) => withReadings(h, { claude: NOTHING, codex: NOTHING });
