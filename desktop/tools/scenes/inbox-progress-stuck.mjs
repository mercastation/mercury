// The In progress tab with a thread in each of the states an agent's run can be in that its row
// says: nothing running (Try again after the word), at its limit (Try again), a run that came
// back with nothing, queued, up next, unable to start, shipping, working outside this window,
// working on this Mac while the cloud takes everything (a card says why), and queued for the
// cloud (a card says why). Nothing here is anyone's real thread.
import { withSnapshot } from './inbox.mjs';
import { tab } from './inbox-progress.mjs';

const min = 60_000;
const hour = 60 * min;

export function stuckThreads(now, products) {
  const by = (source, ts) => ({ ts, source });
  const items = [];
  const thread = (title, fields = {}) => {
    const at = now - hour - items.length * 3 * min;
    const item = {
      id: `w-stuck${items.length}`, product: 'storefront', productName: products.find((p) => p.slug === 'storefront').name, status: 'open', kind: 'directive',
      title, body: title, labels: ['founder'], priority: 0, epoch: 1, claim: null, createdAt: at, updatedAt: at,
      wrote: { title: by('founder', at), body: by('founder', at), status: by('founder', at), kind: by('founder', at), labels: by('founder', at) },
      ...fields,
    };
    items.push(item);
    return item;
  };
  const idle = thread('Archive last year\'s invoices');
  const limit = thread('Resize the product photos');
  const silent = thread('Check the coupon codes');
  const queued = thread('Translate the checkout page');
  const next = thread('Fix the receipt footer');
  const cannot = thread('Rebuild the search index');
  const shipping = thread('Ship the cart badge', { result: 'Built and tested.', wrote: { result: by('agent', now - 20 * min) } });
  thread('Tidy the order emails', { status: 'claimed', claim: { holder: 'mcp-77' } });
  const here = thread('Import the supplier prices', { status: 'claimed', claim: { holder: 'mcp-78' } });
  const waits = thread('Compress the backups');
  return {
    items,
    supervisor: {
      queued: [queued.id, next.id, cannot.id, waits.id], runNow: [next.id],
      cannotStart: { [cannot.id]: { why: 'asleep', tool: 'Claude Code' } },
      atLimit: { [limit.id]: { tool: 'Claude Code', until: now + 2 * hour } },
      silent: { [silent.id]: { runs: 1, endedAt: now - 12 * min } },
      shipping: [shipping.id],
      running: [{ itemId: here.id, product: 'storefront', startedAt: now - 7 * min, engine: 'claude-code', tail: [], activity: [] }],
      whyNotCloud: { [here.id]: { at: 'mac', why: 'kept' }, [waits.id]: { at: 'wait-cloud', why: 'cloud-full', detail: 3 } },
      stalled: [],
    },
    idle,
  };
}

export async function go(h) {
  const fed = await withSnapshot(h, (snapshot) => {
    const { items, supervisor } = stuckThreads(Date.now(), snapshot.products);
    return { ...snapshot, items: [...snapshot.items, ...items], supervisor: { ...snapshot.supervisor, ...supervisor } };
  });
  await tab(h, 'In progress');
  return fed;
}
