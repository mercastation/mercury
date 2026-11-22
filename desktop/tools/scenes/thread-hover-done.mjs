// A conversation whose agent waits on an answer, with the pointer resting on Done.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.ts-more-wrap .ts-done:not(.ts-reopen):not(.ts-files)', { waiting: true });
