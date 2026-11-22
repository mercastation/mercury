// `thread-line-stalled` with the pointer resting on the bar: its edge comes up in the faint ink
// (`.stalled-bar:hover`).
import { line } from './thread-line.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await line(h, 'stalled');
  return { ...fed, ...(await rest(h, '.stalled-bar-label', 200)) };
};
