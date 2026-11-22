// A conversation's reply box opened with R, the pointer resting on its paper clip.
import { hover } from './thread-hover.mjs';
import { reply } from './thread-hover-reply-chip.mjs';

export const go = (h) => hover(h, '.dock-clip', { then: reply });
