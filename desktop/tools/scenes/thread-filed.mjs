// A thread whose agent filed threads under it (`renderer/src/components/ThreadsMade.tsx`): the
// list drawn on the turn that filed them, here the newest, so at the conversation's foot under
// its last word. One proposal waits on the person and one was finished: "1 proposal waiting on
// you", the proposal first with its dot and its answers, the finished one quiet with its tab.
//
// `filed(h, kids, { unfold, at, then })`: `kids` are the threads filed, each `{ id, title, ...}`
// laid over an open task an agent filed (`status`, `answer`, `kind`), filed a minute after the
// thread's last word unless `at` says when (a function of the thread's row). The kids are added
// to the snapshot the page and the native window read (`fleet` in ./thread.mjs).
import { open } from './thread.mjs';

const MIN = 60_000;

export const PROPOSAL = { id: 'w-f11ed00001', title: 'Show the refunds queued for support on the order page' };
export const FINISHED = { id: 'w-f11ed00002', title: 'Round the shipping cost before tax', status: 'done' };

export function filed(h, kids, { unfold = false, at = (row) => row.updatedAt + MIN, then = null, dress = null } = {}) {
  return open(h, {
    unfold,
    fromTab: 'Needs you',
    dress,
    then,
    fleet: (snapshot, row) => {
      kids.forEach((kid, n) => {
        const when = at(row) + n * 1000;
        snapshot.items.push({
          product: row.product, productName: row.productName, kind: 'task', status: 'open', labels: [], priority: 5,
          body: `${kid.title}. Filed while fixing the checkout.`,
          parent: row.id, createdAt: when, updatedAt: when,
          wrote: { title: { ts: when, source: 'agent' }, body: { ts: when, source: 'agent' }, status: { ts: when, source: 'agent' } },
          ...kid,
        });
      });
    },
  });
}

export const go = (h) => filed(h, [PROPOSAL, FINISHED]);
