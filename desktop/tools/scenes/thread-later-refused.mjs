// The Later picker after Enter on words it cannot read: it stays open and says so beside the
// box, with what it would have read.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

const WORDS = 'whenever';

export const go = (h) => open(h, {
  unfold: false,
  then: async (h) => {
    await press(h, 'l', 'KeyL', 76);
    await h.wait(300);
    await h.call('Input.insertText', { text: WORDS });
    await h.wait(200);
    await press(h, 'Enter', 'Enter', 13);
    await h.wait(200);
    return { later: WORDS, laterRefused: true };
  },
});
