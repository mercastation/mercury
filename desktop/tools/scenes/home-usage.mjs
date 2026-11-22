// Home with the pointer on the rail's usage row: the rail opens under it, and the card of the
// one agent read rises over the row (`UsagePill`), as wide as its own 268 points.
import { claude, dressed, pointAt } from './usage.mjs';

export const dress = dressed((now) => [claude(now)]);
export const go = (h) => pointAt(h);
