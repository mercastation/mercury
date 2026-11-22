// `thread-line-idle` with the pointer resting on Try again: the text's ink on the strong wash
// inside the strong edge (`.live-retry:hover`).
import { line } from './thread-line.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await line(h, 'idle');
  return { ...fed, ...(await rest(h, '.live-retry', 200)) };
};
