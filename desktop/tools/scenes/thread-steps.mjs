// The conversation whose two runs left logs, unfolded: what each run said between the messages,
// and what it ran folded to one line under each thing it said. A failed test is said on its fold.
import { open } from './thread.mjs';

export const go = (h) => open(h, { unfold: true, runs: 'steps' });
