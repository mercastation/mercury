// A line of what a run ran whose command is longer than the column and came back with nothing:
// the browser says the column cut it, so it carries the mark and opens onto the whole command
// (`WorkLine`'s `clipped`). The first run of `thread-steps` is given one more call, a linter run
// over many files, after its search; its fold is pressed open, and the line too.
import { open, openRun } from './thread.mjs';
import { openLine } from './thread-steps-line.mjs';

const LINT = "awk -F, '$3 > 1000 { print $1, $3, $4 }' exports/orders-2026-11-20.csv exports/orders-2026-11-21.csv exports/orders-2026-11-22.csv";

/** The first log with the linter's call after its search, stamped a second after it. */
export function withLint(sessions) {
  const [first, ...rest] = sessions;
  const lines = first.text.split('\n');
  const grep = lines.findIndex((l) => l.includes('[Grep] roundTax'));
  const stamp = lines[grep].slice(0, 8);
  const after = lines.findIndex((l, k) => k > grep && !l.startsWith('#'));
  lines.splice(after, 0, `${stamp}  [Bash] ${LINT}`, '#id toolu_a9');
  return [{ ...first, text: lines.join('\n') }, ...rest];
}

export const go = (h) => open(h, {
  unfold: true,
  runs: 'steps',
  trace: withLint,
  then: async (page) => ({ ...(await openRun(page, 0)), ...(await openLine(page, 0, 3)) }),
});
