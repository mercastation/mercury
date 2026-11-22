// Option 1 picked on a waiting thread opened from Home, and Z inside its three seconds: nothing
// was written, the options stand again, and the thread keeps one line of the person's own
// saying what Z took back (`undo-marks.ts`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';
import { fromHome } from './thread-reply-sending.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => {
    await fromHome(h);
    await press(h, '1', 'Digit1', 49);
    await h.wait(400);
    await press(h, 'z', 'KeyZ', 90);
    await h.wait(400);
    return { keysAfter: ['1', 'z'] };
  },
});
