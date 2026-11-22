// The Inbox with the pointer resting on New thread until its key hint is drawn under it.
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';
import { PLATE_MS } from './hint.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.th-new', PLATE_MS);
}
