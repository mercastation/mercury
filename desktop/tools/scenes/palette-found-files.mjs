// The command bar with words the app finds only in files: a file found by its name, and one found
// by what it says, its name drawn before the line that matched. Nothing else answers the words,
// so the first file is lit as it comes.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';
import { answer } from './palette-found.mjs';

const WORDS = 'rounded';

export async function go(h) {
  await toInbox(h);
  const snapshot = await h.evaluate('window.__SNAP__');
  const all = answer(snapshot, Date.now());
  const found = { ...all, lines: [], files: all.files.map((f) => (f.how === 'name' ? { ...f, name: 'rounded-totals.md', src: '/fixtures/storefront/rounded-totals.md' } : f)), total: { lines: 0, files: 2, comments: 0 } };
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
