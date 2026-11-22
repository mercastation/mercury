// One agent whose five hour window started over since it was read: that limit says Reset with
// no bar and no time, and the row on the rail has no figure (`limitsToShow`).
import { claude, dressed, pointAt } from './usage.mjs';

export const dress = dressed((now) => {
  const read = claude(now, { read: 4 * 3_600_000 });
  read.limits[0] = { ...read.limits[0], resetsAt: now - 30 * 60_000, percent: 71 };
  return [read];
});
export const go = (h) => pointAt(h);
