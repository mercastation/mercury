// The reply box with words in it, before they are sent.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

const WORDS = 'Thanks. Tell support the refunds went out, and close this once they answer.';

export const go = (h) => open(h, {
  unfold: false,
  then: async (h) => {
    await press(h, 'r', 'KeyR', 82);
    await h.wait(300);
    await h.call('Input.insertText', { text: WORDS });
    return { replying: WORDS };
  },
});
