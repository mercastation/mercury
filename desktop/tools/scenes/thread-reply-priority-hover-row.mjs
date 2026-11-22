// The priority chip's menu open, the pointer resting on its last row: the row under the pointer
// takes the keys and their wash (`onPointerEnter` in `PriorityPicker`).
import { rest } from './pointer.mjs';
import { CHIP, click, on, reply } from './reply-chips.mjs';

export const dress = on('claude');
export const go = (h) => reply(h, {
  then: async (h) => {
    await click(h, CHIP.priority);
    return { actions: 'reply-priority', ...(await rest(h, '.prio-menu .prio-menu-row:last-child', 300)) };
  },
});
