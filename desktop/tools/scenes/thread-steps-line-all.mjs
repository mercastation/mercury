// `thread-steps-line` with "show all 8 lines" pressed: every line that came back (`WorkLine`, all).
import { open, openRun } from './thread.mjs';
import { openLine } from './thread-steps-line.mjs';

export const go = (h) => open(h, {
  unfold: true,
  runs: 'steps',
  then: async (page) => ({ ...(await openRun(page, 0)), ...(await openLine(page, 0, 0, true)) }),
});
