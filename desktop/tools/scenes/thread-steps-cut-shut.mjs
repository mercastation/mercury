// `thread-steps-cut` with the line still shut: the linter's command cut short by the column,
// with the mark that says it opens.
import { open, openRun } from './thread.mjs';
import { withLint } from './thread-steps-cut.mjs';

export const go = (h) => open(h, { unfold: true, runs: 'steps', trace: withLint, then: (page) => openRun(page, 0) });
