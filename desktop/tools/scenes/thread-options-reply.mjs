// A waiting thread with the reply box opened over its options: what stays of the strip.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => { await press(h, 'r', 'KeyR', 82); return { replying: '' }; },
});
