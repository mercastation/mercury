// `thread-line-here` with the pointer resting on "on this Mac": the card saying why the run is
// here and not in the cloud, opening upward over the line (`.live-place .th-why-card`).
import { line } from './thread-line.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await line(h, 'here');
  return { ...fed, ...(await rest(h, '.live-place', 200)) };
};
