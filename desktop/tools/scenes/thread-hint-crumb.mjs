// A conversation with the pointer resting on the tab named in the strip's crumb until its key
// hint is drawn under it.
import { hover } from './thread-hover.mjs';
import { PLATE_MS } from './hint.mjs';

export const go = (h) => hover(h, '.thread-crumb-back', { ms: PLATE_MS });
