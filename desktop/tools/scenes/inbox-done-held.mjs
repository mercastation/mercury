// E pressed on the Inbox's first row: the row is out of the list at once, and for three
// seconds a line in the head strip says what was done and that Z takes it back.
import { go as toInbox } from './inbox.mjs';
import { press } from './thread-option-picked.mjs';

export async function go(h) {
  await toInbox(h);
  await press(h, 'e', 'KeyE', 69);
  return { closed: true };
}
