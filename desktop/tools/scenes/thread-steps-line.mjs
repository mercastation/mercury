// `thread-steps` with its first fold pressed open and the first line in it pressed open too: the
// whole command behind `git log` (its `cd` taken off on the line) and the first six of the eight
// lines that came back, with "show all 8 lines" under them (`WorkLine`, opened). The fixture says
// which lines were open and how far (`linesOpen`: the fold's place, the line's place in it, 1 open
// or 2 all).
import { open, openRun } from './thread.mjs';

/** Presses the line at `line` in the fold at `run` open, and then, with `all`, its "show all". */
export async function openLine(h, run, line, all = false) {
  await h.evaluate(`document.querySelectorAll('.did-run')[${run}]?.querySelectorAll('.did-run-body .did-head')[${line}]?.click()`);
  await h.wait(300);
  if (all) {
    await h.evaluate(`document.querySelectorAll('.did-run')[${run}]?.querySelectorAll('.did-run-body > .did')[${line}]?.querySelector('.did-more')?.click()`);
    await h.wait(300);
  }
  return { linesOpen: [[run, line, all ? 2 : 1]] };
}

export const go = (h) => open(h, {
  unfold: true,
  runs: 'steps',
  then: async (page) => ({ ...(await openRun(page, 0)), ...(await openLine(page, 0, 0)) }),
});
