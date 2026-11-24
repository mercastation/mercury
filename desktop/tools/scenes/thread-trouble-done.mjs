// E inside `thread-trouble`: the row is put away, the page goes on to the next thread that needs
// the person, and the head strip says "Done: what is not running" with Z beside it.
import { openTrouble } from './thread-trouble.mjs';
import { press } from './thread-option-picked.mjs';

export { dress } from './inbox-trouble.mjs';

export async function go(h) {
  const opened = await openTrouble(h);
  await press(h, 'e', 'KeyE', 69);
  return { ...opened, closed: true };
}
