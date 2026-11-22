// The line at a conversation's foot in each of its states (`renderer/src/live-line.ts`, drawn by
// `components/Live.tsx`), and the bar over a stopped one (`.stalled-bar`). The conversation is
// `thread-working`'s: the person replied a few minutes ago and a run took the reply. Each state
// then takes that run away, or keeps it, and says what the supervisor says about the thread
// (`fleet`): queued, pushed to the front, unable to start, paused, at its limit, nothing
// running, shipping, held by an agent outside this window, signed out, a run that came back with
// nothing, stopped, waiting on a yes. `thread-line-<state>.mjs` each open one.
import { open } from './thread.mjs';

const MIN = 60_000;
const HOUR = 60 * MIN;

/** What the supervisor says, by state, of the thread `id`, `now` being the page's clock. */
const STATES = {
  queued: (s, id) => ({ queued: [id] }),
  next: (s, id) => ({ queued: [id], runNow: [id] }),
  cannot: (s, id) => ({ queued: [id], cannotStart: { [id]: { why: 'asleep', tool: 'Claude Code' } } }),
  paused: () => ({ queued: [], paused: true }),
  limit: (s, id, now) => ({ queued: [], atLimit: { [id]: { tool: 'Claude Code', until: now + 2 * HOUR } } }),
  idle: () => ({ queued: [] }),
  shipping: (s, id) => ({ queued: [], shipping: [id] }),
  elsewhere: () => ({ queued: [] }),
  signedout: (s, id) => ({ queued: [], signInNeeded: { [id]: 'Claude Code' } }),
  silent: (s, id, now) => ({ queued: [], silent: { [id]: { runs: 1, endedAt: now - 3 * MIN } } }),
  stalled: (s, id) => ({ queued: [], stalled: [id] }),
  asking: () => ({ queued: [] }),
  here: (s, id) => ({ queued: [], whyNotCloud: { [id]: { at: 'mac', why: 'kept' } } }),
};

// The states with an agent on the thread keep its run; the rest have none.
const KEEPS_THE_RUN = new Set(['asking', 'here']);
// And those the lists put in Needs you are opened from there.
const IN_NEEDS_YOU = new Set(['signedout', 'stalled', 'asking']);

/** Opens the conversation in `state`, and then does `then` (a press, a pointer resting). */
export function line(h, state, then = null) {
  return open(h, {
    unfold: false,
    runs: 'working',
    fromTab: IN_NEEDS_YOU.has(state) ? 'Needs you' : 'In progress',
    then,
    dress: (row) => {
      if (KEEPS_THE_RUN.has(state)) return;
      // Held outside this window, the claim stays and no run of this window's is on it.
      if (state === 'elsewhere') return;
      row.status = 'open';
      row.claim = null;
      row.wrote = { ...row.wrote, status: { ts: row.wrote.answer.ts, source: 'founder' } };
      // A run that came back with nothing left nothing on the row to read.
      if (state === 'silent') delete row.note;
    },
    fleet: (snapshot, row, now) => {
      if (!KEEPS_THE_RUN.has(state)) snapshot.supervisor.running = [];
      Object.assign(snapshot.supervisor, STATES[state](snapshot, row.id, now));
      // An approval in the spool names the thread.
      if (state === 'asking') {
        snapshot.approvals = [{ id: 'a-line', at: now - MIN, product: row.product, item: row.id, tool: 'Bash', input: { command: 'npm run deploy', description: 'Deploy the fix' } }];
      }
    },
  });
}

/** Presses Try again on the line, and the line said back is the one in the corner. */
export async function tryAgain(h) {
  await h.evaluate(`document.querySelector('.live-retry')?.click()`);
  await h.wait(400);
  return { pressed: '.live-retry' };
}
