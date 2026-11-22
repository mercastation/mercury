// A wide window: the newest answer draws a picture and names its plan in a sentence, the plan is
// pressed open beside the conversation and its divider dragged to give the file as little as it
// keeps, so the conversation reads wide on the left and the plan on the right.
import { open } from './thread-pane.mjs';

const SHIPPED = ({ thread }) => `**The new checkout is drawn.**

![The checkout, wide](designs/${thread}/checkout-wide.png)

The plan for it is in designs/receipt-plan.md, which says what changes and what stays.`;

export const go = (h) => open(h, { shipped: SHIPPED, split: 0.25 });
