// The command bar over the Inbox with words typed, for the rows each kind of word finds. The
// scenes named `palette-words-<kind>` type one of these; `palette-words` alone types the first.
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

export const WORDS = {
  prio: 'prio',
  move: 'move to',
  settings: 'settings',
  agents: 'agents',
  update: 'update',
  keys: 'keyboard',
  rank: 'rank',
};

export const typed = (words) => async (h) => {
  await toInbox(h);
  await commandK(h);
  await h.call('Input.insertText', { text: words });
  await h.wait(500);
  return { palette: words, paletteRows: await rowsOnScreen(h) };
};

export const go = typed(WORDS.prio);
