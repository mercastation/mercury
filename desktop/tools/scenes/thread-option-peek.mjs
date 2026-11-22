// A waiting thread whose second option runs past its two lines, the pointer resting on it: the
// option shows whole in a card over the strip (`.opt-peek`).
import { open } from './thread.mjs';
import { rest } from './pointer.mjs';

const FOUND = `**Found it: tax rounding breaks on carts over $1,000.**

The tax service started returning four decimal places this morning.

## Options

1. Ship the fix now and refund the 37 orders by hand (recommended)
2. Ship the fix, let the nightly job retry every failed order, write to each customer whose card was charged twice, refund the shipping on the orders that went out late, and tell support which tickets that closes so they can answer them before the weekend
3. Turn the new tax rates off until Monday`;

export const go = (h) => open(h, {
  unfold: false, waiting: true, found: FOUND,
  then: async (h) => rest(h, '.opt-row:nth-of-type(2)', 400),
});
