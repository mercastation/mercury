// Pictures that are nowhere: a quoted name in a sentence, a markdown image on a line of its
// own and a path in a code span. Each says which picture is missing, and the one that is
// there is still drawn.
import { open } from './thread.mjs';

const SHIPPED = ({ thread }) => `**Two drawings did not survive the move.**

The old one, "receipt old.png", is gone.

![The first draft](designs/${thread}/first-draft.png)

Nor is \`shots/cart-before.png\` anywhere, though designs/${thread}/total-narrow.png is.`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
