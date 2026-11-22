// A conversation with the pointer resting on Back.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.topbar .back-forward button[aria-label^="Back"]');
