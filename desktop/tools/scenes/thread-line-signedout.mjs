// The line at the foot of a thread whose tool is signed out: the whole sentence, and no Try
// again, which a sign-in alone fixes (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'signedout');
