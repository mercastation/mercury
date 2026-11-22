// The Inbox with the pointer resting on Back, which goes to Home.
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.topbar .back-forward button:not(.back-forward-inbox)');
}
