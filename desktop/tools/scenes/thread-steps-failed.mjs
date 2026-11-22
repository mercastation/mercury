// A conversation with two things that failed: the run after the reply, turned away at the usage
// limit and said in the app's words, and a test the run after it ran, in the fold pressed open
// here (the third on the page), the failed line in red and its count on the fold.
import { open, openRun } from './thread.mjs';

export const go = (h) => open(h, { unfold: true, runs: 'refused', then: (page) => openRun(page, 2) });
