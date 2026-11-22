// The line at the foot of a thread pushed to the front with Run now: Up next (`thread-line.mjs`).
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'next');
