// A conversation whose agent waits on an answer, with the pointer resting on Done until its key
// hint is drawn under it, its right edge on Done's.
import { hover } from './thread-hover.mjs';
import { PLATE_MS } from './hint.mjs';

export const go = (h) => hover(h, '.ts-more-wrap .ts-done:not(.ts-reopen):not(.ts-files)', { waiting: true, ms: PLATE_MS });
