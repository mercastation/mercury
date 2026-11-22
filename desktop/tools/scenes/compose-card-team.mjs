// The New thread card for someone with teammates: its head has a To field over the Model one,
// because someone besides the agent could get the thread. Nothing typed yet.
import { openCard } from './compose-card.mjs';

export { dress } from './home-team.mjs';

export async function go(h) {
  await openCard(h, '');
  return { card: true };
}
