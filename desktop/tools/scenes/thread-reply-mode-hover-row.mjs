// The mode chip's menu open, the pointer resting on its third row, Plan: whatever the row does
// under the pointer, it is drawn here as the web window draws it.
import { rest } from './pointer.mjs';
import { CHIP, click, on, reply } from './reply-chips.mjs';

export const dress = on('claude', 'bypassPermissions');
export const go = (h) => reply(h, {
  then: async (h) => {
    await click(h, CHIP.mode);
    return { actions: 'reply-mode', ...(await rest(h, '.prio-menu .prio-menu-row:nth-child(3)', 300)) };
  },
});
