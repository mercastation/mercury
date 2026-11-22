// The line at the foot of a thread whose last run ended three minutes ago and wrote nothing:
// Nothing came back, how long ago, and Try again (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'silent');
