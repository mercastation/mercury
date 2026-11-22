// The Inbox with the pointer resting on its third row until the row's key hint is drawn under
// it, lined up with the row's title.
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';
import { PLATE_MS } from './hint.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.list.th-table .row:not(.th-head):nth-child(4) .subject', PLATE_MS);
}
