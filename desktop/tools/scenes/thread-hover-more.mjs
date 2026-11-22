// A conversation with the pointer resting on ⋯, the thread's own menu.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.ts-more[aria-label="More actions"]');
