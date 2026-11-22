// The whole conversation, unfolded, with pictures inside it: the agent's first answer ends
// on a picture under its sentence and the last one opens with one, so the room a picture
// keeps is held against what stands over and under it in a conversation.
import { open } from './thread.mjs';

const FOUND = ({ thread }) => `**Found it, and drew what the receipt should say.**

The total belongs on one line, as in designs/${thread}/total-line.png here.`;

const SHIPPED = ({ thread }) => `![The tax line](designs/${thread}/tax-line.png)

**Shipped, and support knows.** The receipt prints the total on one line again.`;

export const go = (h) => open(h, { unfold: true, found: FOUND, shipped: SHIPPED, files: {} });
