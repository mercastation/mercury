// The Later tab with one invented thread of each kind it holds, for a picture of how each
// row is drawn: written down and not begun (pinned, and urgent), put off until tomorrow, put
// off past the week (its day is dated, and wider than its column), a reminder for later
// today, a task an agent proposed, a thread stuck behind a cause, one with a comment on its
// link, and one whose worker died.
//
// `reference.mjs`'s invented inbox has nothing in Later, so the strip has no Later to press;
// these are added to it. Nothing here is anyone's real thread.
import { withSnapshot } from './inbox.mjs';
import { tab } from './inbox-progress.mjs';

const min = 60_000;
const hour = 60 * min;
const day = 24 * hour;

export function laterThreads(now, products) {
  const by = (source, ts) => ({ ts, source });
  const items = [];
  // A thread written three hours ago, by the person unless `who` says an agent filed it.
  const thread = (title, { product = 'storefront', who = 'founder', ...fields } = {}) => {
    const at = now - 3 * hour - items.length * min;
    const item = {
      id: `w-later${items.length}`, product, productName: products.find((p) => p.slug === product).name, status: 'open', kind: 'directive',
      title, body: title, labels: who === 'founder' ? ['founder'] : [], priority: 0, epoch: 1, claim: null, createdAt: at, updatedAt: at,
      wrote: { title: by(who, at), body: by(who, at), status: by(who, at), kind: by(who, at), labels: by(who, at) },
      ...fields,
    };
    items.push(item);
    return item;
  };
  // What happened to it since, `ago` before now.
  const set = (item, field, value, source, ago) => {
    item[field] = value;
    item.wrote[field] = by(source, now - ago);
    item.updatedAt = Math.max(item.updatedAt, now - ago);
    return item;
  };
  const answered = (item) => set(item, 'result', 'Done as far as it goes.', 'agent', hour);

  thread('Rewrite the onboarding emails', { start: 'later', pinned: true });
  thread('Move the nightly export to the new bucket', { start: 'later', priority: 9, product: 'mobile' });
  set(answered(thread('Check the signup numbers again')), 'runAt', now + day, 'founder', 30 * min);
  set(answered(thread('Renew the push certificate', { product: 'mobile' })), 'runAt', now + 9 * day, 'founder', 31 * min);
  set(answered(thread('Ask the print shop about the banner')), 'snoozedUntil', now + 3 * hour, 'founder', 32 * min);
  thread('Add a retry to the tax call', { who: 'agent', parent: 'w-readme0', priority: 7 });
  const cause = thread('GitHub is signed out', { who: 'system', kind: 'cause', labels: ['cause'] });
  set(set(thread('Publish the changelog'), 'status', 'blocked', 'agent', 40 * min), 'blockedBy', [cause.id], 'agent', 40 * min);
  thread('Draft the pricing page', { start: 'later', linkComments: [{ name: 'Ines', file: 'plan.md', text: 'When does this start?', ts: now - 5 * min }] });
  const stopped = thread('Tidy the staging database', { start: 'later', priority: 2 });
  return { items, stalled: [stopped.id] };
}

export async function go(h) {
  const fed = await withSnapshot(h, (snapshot) => {
    const { items, stalled } = laterThreads(Date.now(), snapshot.products);
    return { ...snapshot, items: [...snapshot.items, ...items], supervisor: { ...snapshot.supervisor, stalled } };
  });
  await tab(h, 'Later');
  return fed;
}
