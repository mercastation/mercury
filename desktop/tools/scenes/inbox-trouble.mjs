// The Inbox with the row the app makes itself when tasks have not been able to run
// (`troubleRow` in `renderer/src/trouble-row.ts`): three tasks stopped, two on a signed-out
// login and one at the limit, the pile begun an hour and a half ago. Two of the three are
// threads of the invented inbox; the third is no longer in it, which the opened row says.
//
// The supervisor's `spawnTrouble` is written here in the shape it sends (`_strandedNews` in
// main/supervisor.mts), each sentence as `causeWhat` and `causeNext` in
// shared/spawn-trouble.mts word it.
import { go as toInbox } from './inbox.mjs';

const MIN = 60_000;

export function trouble(now, ids = ['w-readme2', 'w-readme4', 'w-gone0001']) {
  const [first, second, third] = ids;
  return {
    since: now - 95 * MIN,
    cause: 'mixed',
    message: '3 tasks have not been able to run since 95 minutes ago.',
    remedy: 'They are not all stuck on the same thing. Each row says which.',
    retryAt: 0,
    count: 3,
    resetsAt: '6pm',
    byCause: [
      { cause: 'signed-out', count: 2, what: '2 stopped by a signed-out Claude login', next: 'Type /login in a terminal and they pick straight up.', act: 'Sign in', hers: true },
      { cause: 'at-limit', count: 1, what: '1 waiting on your Claude limit', next: 'They start again on their own at 6pm.', act: null, hers: false },
    ],
    ids: [first, second, third],
    stopped: [{ id: first, cause: 'signed-out' }, { id: third, cause: 'signed-out' }, { id: second, cause: 'at-limit' }],
  };
}

export function dress(snapshot, now) {
  return { ...snapshot, supervisor: { ...snapshot.supervisor, spawnTrouble: trouble(now) } };
}

export const go = (h) => toInbox(h);
