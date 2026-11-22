// Several pictures named in one sentence: the sentence keeps their names and the pictures
// sit in one row under it, each at the row's height whatever its shape.
import { open } from './thread.mjs';

const SHIPPED = () => `**Three carts.**

Compare shots/cart-a.png, shots/cart-b.png and shots/cart-c.png before I pick one.`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
