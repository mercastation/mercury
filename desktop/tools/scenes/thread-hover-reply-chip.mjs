// A conversation's reply box opened with R, the pointer resting on its first chip.
import { hover } from './thread-hover.mjs';
import { press } from './thread-option-picked.mjs';

/** R pressed on the open thread, as `thread-reply` presses it. */
export const reply = async (h) => { await press(h, 'r', 'KeyR', 82); return { replying: '' }; };

export const go = (h) => hover(h, '.dock-bar .tc-chip', { then: reply });
