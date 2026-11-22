// The `thread-folded` conversation with the logs its two runs left, as it first opens: the
// newest answer alone, and the line under the title counting what each run said as well.
import { open } from './thread.mjs';

export const go = (h) => open(h, { unfold: false, runs: 'steps' });
