// The reply box with the caret in it and nothing typed: R pressed on an open thread.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false,
  then: async (h) => { await press(h, 'r', 'KeyR', 82); return { replying: '' }; },
});
