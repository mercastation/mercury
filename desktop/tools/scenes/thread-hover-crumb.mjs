// A conversation with the pointer resting on the tab named in the strip's crumb, which goes back.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.thread-crumb-back');
