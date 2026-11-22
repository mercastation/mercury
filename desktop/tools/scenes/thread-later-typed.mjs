// The Later picker with a time typed into it: the moment it read stands beside the words, and
// no row is lit, because what is typed decides.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

const WORDS = 'tomorrow at noon';

export const go = (h) => open(h, {
  unfold: false,
  then: async (h) => {
    await press(h, 'l', 'KeyL', 76);
    await h.wait(300);
    await h.call('Input.insertText', { text: WORDS });
    await h.wait(200);
    return { later: WORDS };
  },
});
