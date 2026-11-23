// The command bar with a word typed that the app finds inside threads: the thread the word is in,
// then what the app found in the threads, under Lines and Files (`findInside` in Palette.tsx).
//
// The harness's bridge does not search threads, so the page is answered from here, as the app's
// finder answers (`zero:find-in-threads`, main/everything-found.mts): lines said in two of the
// invented inbox's threads and one in a thread it does not list, a file found by its name and one
// by what it says. The same answer goes into the fixture, where the native window reads it.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

export const WORDS = 'tax';

const MIN = 60_000;

// Where a found thing sits: a thread of the person's own, by its product and id.
const at = (snapshot, n) => {
  const item = snapshot.items[n];
  return { kind: 'thread', product: item.product, id: item.id, ownerId: null, threadId: null, updatedAt: item.updatedAt };
};

export function answer(snapshot, now) {
  return {
    ok: true,
    lines: [
      { ...at(snapshot, 0), uid: 'at-1', by: null, source: 'agent', at: now - 12 * MIN, comment: false, text: 'The tax service started returning four decimal places this morning, and roundTax() only handles two.' },
      { ...at(snapshot, 0), uid: 'at-2', by: null, source: 'founder', at: now - 3 * MIN, comment: false, text: 'Round the tax before the total is added up, then ship it.' },
      { kind: 'thread', product: 'storefront', id: 'w-gone', ownerId: null, threadId: null, updatedAt: now, uid: 'at-3', by: null, source: 'agent', at: now - MIN, comment: false, text: 'Tax rates for next year are in.' },
      { ...at(snapshot, 8), uid: 'at-4', by: null, source: 'agent', at: now - 40 * MIN, comment: false, text: 'The notes say the tax fix went out with the nine merges.' },
    ],
    files: [
      { ...at(snapshot, 0), how: 'name', src: '/fixtures/storefront/tax-rounding.md', name: 'tax-rounding.md', by: null, at: now - 20 * MIN, text: '' },
      { ...at(snapshot, 8), how: 'contents', src: '/fixtures/storefront/release-notes.md', name: 'release-notes.md', by: null, at: now - 30 * MIN, text: 'Checkout: tax on carts over $1,000 is rounded the same way as on smaller ones.' },
    ],
    comments: [],
    total: { lines: 4, files: 2, comments: 0 },
  };
}

export async function go(h) {
  await toInbox(h);
  const snapshot = await h.evaluate('window.__SNAP__');
  const found = answer(snapshot, Date.now());
  await h.evaluate(`(() => {
    const found = ${JSON.stringify(found)};
    window.zero.findInThreads = async () => found;
    window.zero.warmFindInThreads = async () => true;
  })()`);
  await commandK(h);
  await h.call('Input.insertText', { text: WORDS });
  await h.wait(700);
  return { palette: WORDS, paletteRows: await rowsOnScreen(h), 'zero:find-in-threads': found };
}
