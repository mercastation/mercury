// A conversation with the pointer resting on the Inbox button at the strip's left end until its
// key hint is drawn under it.
import { hover } from './thread-hover.mjs';
import { PLATE_MS } from './hint.mjs';

export const go = (h) => hover(h, '.back-forward-inbox', { ms: PLATE_MS });
