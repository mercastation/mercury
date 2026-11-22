// The Later picker with words in it that are not a time, before Enter: the line beside the
// box says so, faintly.
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
    return { later: WORDS };
  },
});
