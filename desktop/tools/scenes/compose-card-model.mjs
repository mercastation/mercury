// The New thread card with its model menu open: it drops from the Model field of the head.
import { openCard, pressOnCard } from './compose-card.mjs';
import { WORDS } from './home-typed.mjs';

export async function go(h) {
  await openCard(h);
  await pressOnCard(h, { field: 'Model' });
  return { card: true, composing: WORDS, menu: 'model' };
}
