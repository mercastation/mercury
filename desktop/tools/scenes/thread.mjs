// From Home to one open conversation: the first thread of the Inbox, as Enter opens it, with
// its earlier messages unfolded. (`thread-folded` is the same thread as it first opens.)
//
// The harness's bridge answers the snapshot and little else, and a thread reads more than
// that: its ledger (`zero:item-history`), what its runs did (`zero:session-trace`), the files
// they changed (`zero:code-change`) and whether it is being compacted
// (`zero:compaction-status`). So this scene answers those four in the page, with one invented
// conversation, and hands the same answers back for the fixture file, where the native window
// reads them. `docs/thread-data.md` says what each channel is asked and answers.
import { go as toInbox } from './inbox.mjs';

const MIN = 60_000;

const ASK = `Checkout fails for some customers since this morning. Support has eleven tickets so far.

Find out why and fix it, please.`;

const FOUND = `**Found it: tax rounding breaks on carts over $1,000.**

The tax service started returning four decimal places this morning, and \`roundTax()\` only handles two. Smaller carts happen to round the same either way, which is why most orders still go through. What changed is in [the tax service's release notes](https://example.com/tax/releases/4.2), and it was *not* announced.

What I checked:

- 37 failed orders since 08:10, every one of them over $1,000
- No failures on carts paid with store credit
- The same carts pass once the tax is rounded **before** the total is added up

| Cart total | Orders | Failed |
| --- | --- | --- |
| Under $1,000 | 412 | 0 |
| $1,000 to $5,000 | 31 | 29 |
| Over $5,000 | 8 | 8 |

The fix is small:

\`\`\`ts
export function roundTax(amount: number): number {
  return Math.round(amount * 100) / 100;
}
\`\`\`

## Options

1. Ship the fix now and refund the 37 orders by hand (recommended)
2. Ship the fix and let the nightly job retry the orders
3. Turn the new tax rates off until Monday`;

const REPLY = 'Go with 1. Tell support before the refunds go out.';

const SHIPPED = `**Shipped, and support knows.**

The fix is on main as \`a41c9e2\` and every checkout test passes. The 37 refunds are queued behind a note to support, so nothing goes out before they have read it.`;

// The ledger a thread like this leaves behind, one line per write, and the row those lines
// fold into (shared/work-items.mts): the newest value of each field, and who wrote it when.
function conversation(item) {
  const [asked, found, replied, shipped] = [item.createdAt, item.createdAt + 38 * MIN, item.updatedAt - 17 * MIN, item.updatedAt];
  const { id, title } = item;
  const lines = [
    { id, ts: asked, source: 'founder', patch: { title, body: ASK, status: 'open', kind: 'directive', priority: item.priority, labels: item.labels } },
    { id, ts: asked + MIN, source: 'agent', epoch: 1, claim: { holder: 'mcp-40122', leaseUntil: asked + 31 * MIN }, patch: { status: 'claimed' } },
    { id, ts: found, source: 'agent', epoch: 1, patch: { status: 'blocked', note: FOUND } },
    { id, ts: replied, source: 'founder', patch: { answer: REPLY, status: 'open' } },
    { id, ts: replied + MIN, source: 'agent', epoch: 2, claim: { holder: 'mcp-40131', leaseUntil: replied + 31 * MIN }, patch: { status: 'claimed' } },
    { id, ts: shipped, source: 'agent', epoch: 2, patch: { status: 'done', result: SHIPPED } },
  ];
  const row = {
    ...item, body: ASK, note: FOUND, answer: REPLY, result: SHIPPED,
    wrote: {
      title: { ts: asked, source: 'founder' }, body: { ts: asked, source: 'founder' },
      note: { ts: found, source: 'agent' }, answer: { ts: replied, source: 'founder' },
      result: { ts: shipped, source: 'agent' }, status: { ts: shipped, source: 'agent' },
    },
  };
  return { row, lines };
}

// `unfold` presses the line under the title, so the picture is of the whole conversation
// and not of its newest answer alone.
export async function open(h, { unfold }) {
  const snapshot = await h.evaluate('window.__SNAP__');
  // The Inbox opens on its first row; the snapshot's first thread is not always that one.
  await toInbox(h);
  const first = await h.evaluate(`document.querySelector('.list .row.selected .mark')?.getAttribute('aria-label') ?? ''`);
  const at = snapshot.items.findIndex((i) => first === `Select ${i.title}`);
  if (at < 0) throw new Error(`thread scene: the Inbox's first row ("${first}") is not in the snapshot`);
  const { row, lines } = conversation(snapshot.items[at]);
  snapshot.items[at] = row;
  // Keyed as the page asks: product and id. Each answer is under its channel's name.
  const threads = {
    [`${row.product}/${row.id}`]: {
      'zero:item-history': { ok: true, lines },
      'zero:session-trace': { ok: true, sessions: [] },
      'zero:code-change': { ok: false, error: 'no change was recorded for this thread' },
      'zero:compaction-status': null,
    },
  };
  // On every load from here on: the snapshot with the conversation's row in it, the four
  // answers, and a note of anything else the page asks the bridge for and does not find.
  await h.call('Page.addScriptToEvaluateOnNewDocument', {
    source: `(() => {
      window.__SNAP__ = ${JSON.stringify(snapshot)};
      const threads = ${JSON.stringify(threads)};
      const answer = (channel, fallback) => async (p) => threads[p?.product + '/' + p?.id]?.[channel] ?? fallback;
      const bridge = {
        ...window.zero,
        itemHistory: answer('zero:item-history', { ok: true, lines: [] }),
        sessionTrace: answer('zero:session-trace', { ok: true, sessions: [] }),
        codeChange: answer('zero:code-change', { ok: false, error: 'no change was recorded for this thread' }),
        compactionStatus: answer('zero:compaction-status', null),
      };
      window.__UNANSWERED__ = [];
      window.zero = new Proxy(bridge, {
        get(target, name) {
          if (typeof name === 'string' && !(name in target) && !window.__UNANSWERED__.includes(name)) window.__UNANSWERED__.push(name);
          return target[name];
        },
      });
    })()`,
  });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await toInbox(h);
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await h.wait(1500);
  // A thread opens on its newest answer, with the rest folded behind one line.
  if (unfold) {
    await h.evaluate(`document.querySelector('.one-earlier')?.click()`);
    await h.wait(600);
  }
  const unanswered = await h.evaluate('window.__UNANSWERED__');
  if (unanswered.length) console.error('thread scene: the page asked the bridge for', unanswered.join(', '), 'and got nothing');
  // `unfolded` tells the native window the same thing the click above told the page.
  return { snapshot, threads, unfolded: unfold ? Object.keys(threads) : [] };
}

export const go = (h) => open(h, { unfold: true });
