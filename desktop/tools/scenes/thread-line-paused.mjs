// The line at the foot of a thread while agents are paused: Paused (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'paused');
