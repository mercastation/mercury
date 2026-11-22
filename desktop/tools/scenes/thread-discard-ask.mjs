// A waiting thread whose second option throws the change away, and 2 pressed: nothing is sent,
// and the question stands where the options stood (`discardAsk`, `closeAskStrip`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

const FOUND = `**The redesign is on a branch and the tests pass.**

It changes the checkout's three steps into one page. Nothing is on main yet.

## Options

1. Ship it (recommended)
2. Discard the change
3. Keep it on the branch`;

export const go = (h) => open(h, {
  unfold: false, waiting: true, found: FOUND,
  then: async (h) => { await press(h, '2', 'Digit2', 50); await h.wait(300); return { keysAfter: ['2'] }; },
});
