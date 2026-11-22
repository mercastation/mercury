// The command bar with words typed that nothing answers: the line that says so where the
// first row would have been.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

const WORDS = 'zebra crossing';

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  await h.call('Input.insertText', { text: WORDS });
  await h.wait(500);
  return { palette: WORDS, paletteRows: await rowsOnScreen(h) };
}
