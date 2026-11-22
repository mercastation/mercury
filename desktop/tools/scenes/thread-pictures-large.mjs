// A picture wider than the reading column, drawn from a copy the column's width, then one
// taller than the window.
import { open } from './thread.mjs';

const SHIPPED = ({ thread }) => `![The checkout, wide](designs/${thread}/checkout-wide.png)

![On a phone](designs/${thread}/phone-tall.png)`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
