// A thread whose agent handed a sign-in link to its `sign_in` tool and waits on the person
// (`shared/sign-in-ask.mts`): the card under the conversation (`SignInCard.tsx`, `.need`), over
// the line that says Asking. The conversation is `thread-working`'s, its run still up.
//
// `ask` is the thread's `signIn` field but for its id and its time. `cloud` says the run is on
// another machine, which is when the card offers to save the sign-in and, for a link that
// comes back to localhost, to paste where the browser ended. `bridge` is what the engine
// answers the card's writes with, by the bridge's own names (`signInAnswer`, `signInKeep`):
// a string of JavaScript for the function. `then` is what is done to the card after it is up,
// and what it returns goes in the fixture for the native window to do the same
// (`signInPressed`, `signInTyped`).
import { open } from './thread.mjs';

const MIN = 60_000;

export const CLOUDFLARE = { service: 'Cloudflare', url: 'https://dash.cloudflare.com/oauth2/auth?response_type=code&client_id=54d11594', why: 'to point the shop\'s domain at the new server: sign in there and press Allow' };

export function signIn(h, ask = CLOUDFLARE, { cloud = false, bridge = null, then = null } = {}) {
  return open(h, {
    unfold: false,
    runs: 'working',
    fromTab: 'Needs you',
    then: async (h) => {
      if (bridge) await h.evaluate(`Object.assign(window.zero, { ${Object.entries(bridge).map(([name, fn]) => `${name}: ${fn}`).join(', ')} })`);
      return then ? then(h) : {};
    },
    dress: (row) => {
      row.signIn = { id: 's-ask0000001', at: row.updatedAt - 3 * MIN, ...ask };
    },
    fleet: (snapshot) => {
      snapshot.supervisor.queued = [];
      if (cloud) for (const run of snapshot.supervisor.running) run.where = 'cloud';
    },
  });
}

export const go = (h) => signIn(h);
