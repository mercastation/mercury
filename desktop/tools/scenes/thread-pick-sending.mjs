// Option 1 picked on a waiting thread opened from Home, a moment before its three seconds are
// over: the pick is on the page as the line its written copy will be (`pickOption`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';
import { fromHome } from './thread-reply-sending.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => {
    await fromHome(h);
    await press(h, '1', 'Digit1', 49);
    await h.wait(400);
    return { keysAfter: ['1'] };
  },
});
