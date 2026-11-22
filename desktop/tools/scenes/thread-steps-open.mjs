// `thread-steps` with one fold pressed open: the third, the shipping run's change, its tests, the
// commit and the push, each its own line.
import { open, openRun } from './thread.mjs';

export const go = (h) => open(h, { unfold: true, runs: 'steps', then: (page) => openRun(page, 2) });
