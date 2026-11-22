// The conversation whose runs left logs, unfolded, with the pointer resting on the first fold of
// what a run ran.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.did-run-head', { unfold: true, runs: 'steps' });
