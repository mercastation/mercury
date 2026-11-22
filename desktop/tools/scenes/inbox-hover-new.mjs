// The Inbox with the pointer resting on New thread, in the card's top strip.
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.th-new');
}
