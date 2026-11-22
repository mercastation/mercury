// The Inbox with the pointer resting on its third row: the faint line under the row's title,
// and the row the keys now act on (`pointed` in renderer/src/App.tsx).
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.list.th-table .row:not(.th-head):nth-child(4) .subject');
}
