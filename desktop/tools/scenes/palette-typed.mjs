// The command bar with a word typed that some threads carry: the threads it found, then the
// commands whose names match.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

const WORDS = 'check';

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  await h.call('Input.insertText', { text: WORDS });
  await h.wait(500);
  return { palette: WORDS, paletteRows: await rowsOnScreen(h) };
}
