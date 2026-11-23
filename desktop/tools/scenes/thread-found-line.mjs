// A line found from the command bar, pressed while its conversation is open and unfolded: the
// line brought to the middle of the page and marked (`revealLine`), photographed a second after
// the press, with the mark whole.
//
// Each ledger line of the invented conversation is given a uid, as the store's lines have, and the
// app's finder is answered from here with the agent's finding, by its line's uid: a message in the
// middle of the conversation, taller than the page, so the picture shows it centred rather than
// held at either end. The fixture says which line was pressed (`foundLine`), and the native window
// looks for the same one.
import { open } from './thread.mjs';
import { commandK } from './palette.mjs';

const WORDS = 'smaller carts';

export async function go(h) {
  let uid = null;
  const ledger = (lines) => lines.map((line, n) => {
    const named = { ...line, uid: `line-${n}` };
    if (line.source === 'agent' && line.patch?.note && uid === null) uid = named.uid;
    return named;
  });
  const then = async (h) => {
    // The page has settled where it lands, as a person reading it would have.
    await h.wait(2500);
    const snapshot = await h.evaluate('window.__SNAP__');
    const row = snapshot.items.find((i) => i.answer);
    const reply = await h.evaluate(`[...document.querySelectorAll('[data-line="${uid}"]')].length`);
    if (!reply) throw new Error('thread-found-line: the reply is not drawn');
    await h.evaluate(`(() => {
      const found = { ok: true, lines: [{ kind: 'thread', product: ${JSON.stringify(row.product)}, id: ${JSON.stringify(row.id)}, ownerId: null, threadId: null, updatedAt: 1,
        uid: ${JSON.stringify(uid)}, by: null, source: 'agent', at: Date.now() - 600000, comment: false, text: 'Smaller carts happen to round the same either way, which is why most orders still go through.' }],
        files: [], comments: [], total: { lines: 1, files: 0, comments: 0 } };
      window.zero.findInThreads = async () => found;
      window.zero.warmFindInThreads = async () => true;
    })()`);
    await commandK(h);
    await h.call('Input.insertText', { text: WORDS });
    await h.wait(700);
    await h.evaluate(`document.querySelector('.palette-k .palette-found')?.click()`);
    await h.wait(500);
    return { foundLine: uid, settle: 500 };
  };
  return open(h, { unfold: true, ledger, then });
}
