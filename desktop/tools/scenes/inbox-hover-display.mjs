// The Inbox with the pointer resting on the display button at the strip's right end.
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.th-disp');
}
