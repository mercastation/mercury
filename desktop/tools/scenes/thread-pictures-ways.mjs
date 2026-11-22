// The other ways an agent names a picture: a path in a code span, a full path into the
// project's folder, and a short path that is only found beside the code.
import { open } from './thread.mjs';

const SHIPPED = ({ dir, thread }) => `The cart is \`shots/cart-strip.png\` in the project.

Written out in full it is ${dir}/designs/${thread}/tax-line.png today.

A worker left shots/from-the-code.png beside the code.`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
