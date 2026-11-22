// L pressed on an open thread: the Later picker over the page, its box empty and its first
// moment lit.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false,
  then: async (h) => { await press(h, 'l', 'KeyL', 76); await h.wait(300); return { later: '' }; },
});
