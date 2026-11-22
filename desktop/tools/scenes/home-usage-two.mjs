// The same card with two agents read this week: the one used last first with "Last used"
// beside its name and when it was read under its limits, then a hairline and the other.
import { claude, codex, dressed, pointAt } from './usage.mjs';

export const dress = dressed((now) => [claude(now), codex(now)]);
export const go = (h) => pointAt(h);
