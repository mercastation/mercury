// The line at the foot of a thread whose tool is at its usage limit for two hours more: the whole
// sentence, when it resets, and Try again (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'limit');
