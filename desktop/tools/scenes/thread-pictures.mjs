// A conversation whose newest answer names pictures, each a different way an agent writes
// one: bare in a sentence, as a markdown image on a line of its own, and as a markdown link.
// The pictures are planted in an invented project folder (`../pictures.mjs`), short strips so
// the three fit one window.
import { open } from './thread.mjs';

export const SHIPPED = ({ thread }) => `**The receipt, redrawn.**

The total now sits on one line, as in designs/${thread}/total-line.png, with the tax under it.

![The tax line](designs/${thread}/tax-line.png)

On a phone it is [the narrow one](designs/${thread}/total-narrow.png).`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
