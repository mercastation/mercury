// The line at the foot of a thread waiting its turn: Queued, its mark still (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'queued');
