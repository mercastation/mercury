// The reply box's priority chip pressed: its four levels rise over it, the thread's own lit and
// bold (`PriorityPicker`, `.dock-card .compose-word-wrap .prio-menu`).
import { CHIP, click, on, reply } from './reply-chips.mjs';

export const dress = on('claude');
export const go = (h) => reply(h, { then: async (h) => { await click(h, CHIP.priority); return { actions: 'reply-priority' }; } });
