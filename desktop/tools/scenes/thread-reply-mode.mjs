// The reply box's mode chip pressed on a Claude Code thread whose project runs with full
// access: Claude Code's six modes rise over it, the one the next reply runs in bold
// (`ModePicker`, `MODE_ORDER`, `MODE_WORDS`).
import { CHIP, click, on, reply } from './reply-chips.mjs';

export const dress = on('claude', 'bypassPermissions');
export const go = (h) => reply(h, { then: async (h) => { await click(h, CHIP.mode); return { actions: 'reply-mode' }; } });
