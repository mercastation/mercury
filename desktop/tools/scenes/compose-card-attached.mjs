// The New thread card on the Inbox with words typed and two files attached through its paper
// clip, a picture and a note: the picture drawn small, the note a chip with its size, both in
// the row between the words and the bar.
import { openCard } from './compose-card.mjs';
import { WORDS } from './home-typed.mjs';
import { attached, pickFiles } from './home-attached.mjs';
import { plant } from '../pictures.mjs';

export async function go(h) {
  const planted = plant();
  const files = ['receipt-small.png', 'notes/rounding.md'].map((end) => planted.files.find((f) => f.endsWith(end)));
  await openCard(h);
  await pickFiles(h, '.tc-card:not(.tc-inline)', files);
  return { card: true, composing: WORDS, attached: attached(files) };
}
