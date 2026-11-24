// The Inbox with the pointer resting on the sidebar's Home: the sidebar open beside the table
// and the card moved over (`home-rail-open` says when it opens and why the pointer rests on Home).
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, 'button.workspace-tab[data-tab="home"]', 900);
}
