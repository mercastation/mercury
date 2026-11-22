// The Inbox with the pointer resting on its In progress tab.
import { go as toInbox } from './inbox.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  await toInbox(h);
  return rest(h, '.tm-tabs button.tm-tab:nth-child(2)');
}
