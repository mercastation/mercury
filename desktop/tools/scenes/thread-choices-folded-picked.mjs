// The choices folded away, then the down arrow pressed: a choice picked out with the keys
// brings the strip back while it is picked (`.one-choices-folded:not(:has(.opt-row.selected))`).
import { open } from './thread.mjs';
import { fold } from './thread-choices-folded.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false,
  waiting: true,
  then: async (h) => {
    const did = await fold(h);
    await press(h, 'ArrowDown', 'ArrowDown', 40);
    return { ...did, option: 1 };
  },
});
