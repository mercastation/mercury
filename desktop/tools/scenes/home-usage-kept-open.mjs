// `home-usage-two` with the rail kept open (Settings' "the sidebar shrinks to icons" off):
// the card comes 120 ms after the pointer moves onto the row.
import { claude, codex, dressed, pointAtOpen } from './usage.mjs';

export const dress = dressed((now) => [claude(now), codex(now)]);
export const go = (h) => pointAtOpen(h);
