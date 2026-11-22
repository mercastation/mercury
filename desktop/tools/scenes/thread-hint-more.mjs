// A conversation with the pointer resting on ⋯ until its hint is drawn under it: a plate with
// words and no key.
import { hover } from './thread-hover.mjs';
import { PLATE_MS } from './hint.mjs';

export const go = (h) => hover(h, '.ts-more[aria-label="More actions"]', { ms: PLATE_MS });
