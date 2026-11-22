// The line at the foot of a thread the app is shipping: Shipping (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'shipping');
