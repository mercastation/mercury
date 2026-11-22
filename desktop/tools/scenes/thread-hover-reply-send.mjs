// A conversation's reply box opened with R, the pointer resting on Send, which with nothing
// typed has nothing to send.
import { hover } from './thread-hover.mjs';
import { reply } from './thread-hover-reply-chip.mjs';

export const go = (h) => hover(h, '.dock-send', { then: reply });
