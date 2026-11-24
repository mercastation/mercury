// E on `inbox-trouble`'s row, the first in Needs you: for three seconds a line in the corner
// says "Done: what is not running" with Z beside it, and the row stays in the list until they
// have passed and the put-away is written (`closeTroubleRow` in `renderer/src/App.tsx`).
import { go as toInbox } from './inbox.mjs';
import { press } from './thread-option-picked.mjs';

export { dress } from './inbox-trouble.mjs';

export async function go(h) {
  await toInbox(h);
  await press(h, 'e', 'KeyE', 69);
  return { closed: true };
}
