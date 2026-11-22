// The line at the foot of a thread whose agent stopped on a permission request: Asking, the run's
// time, its mark still (`thread-line.mjs`). The web window also draws the request's card over the
// page; the line is compared under it.
import { line } from './thread-line.mjs';

export const go = (h) => line(h, 'asking');
