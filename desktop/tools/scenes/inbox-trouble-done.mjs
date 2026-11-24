// E on `inbox-trouble`'s row, the first in Needs you: it is out of the list at once, and for
// three seconds the head strip says "Done: what is not running" with Z beside it
// (`closeTroubleRow` in `renderer/src/App.tsx`).
import { go as toInbox } from './inbox.mjs';
import { press } from './thread-option-picked.mjs';

export { dress } from './inbox-trouble.mjs';

export async function go(h) {
  await toInbox(h);
  await press(h, 'e', 'KeyE', 69);
  return { closed: true };
}
