// The New thread card with its priority menu open: a menu of the bar rises out of it.
import { openCard, pressOnCard } from './compose-card.mjs';
import { WORDS } from './home-typed.mjs';

export async function go(h) {
  await openCard(h);
  await pressOnCard(h, { chip: 'Priority' });
  return { card: true, composing: WORDS, menu: 'priority' };
}
