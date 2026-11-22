// A conversation with the pointer resting on the strip's Next thread arrow until its key hint is
// drawn under it.
import { hover } from './thread-hover.mjs';
import { PLATE_MS } from './hint.mjs';

export const go = (h) => hover(h, 'button[aria-label="Next thread (J)"]', { ms: PLATE_MS });
