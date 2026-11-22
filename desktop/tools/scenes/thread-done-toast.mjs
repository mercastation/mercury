// E pressed on a waiting thread: the page goes back to the list with the thread out of it, and
// for three seconds a line at the foot says what was done and that Z takes it back.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => { await press(h, 'e', 'KeyE', 69); return { closed: true }; },
});
