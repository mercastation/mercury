// The Inbox after its first thread was opened and left with Escape: that row wears the faint
// wash of the last thread the person stood on (`.row.recent` in renderer/src/styles.css, the
// trail's `lastThread` in renderer/src/back-and-forward.ts), the hairlines over and under it run
// to the card's edge, and Back, lit, leads to the thread. The thread stays on screen past the
// 300 ms a place needs to count before Escape is pressed. `lastThread` in the fixture tells the
// native window the same thing happened.
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false,
  then: async (h) => {
    await h.wait(500);
    await press(h, 'Escape', 'Escape', 27);
    return { lastThread: true };
  },
});
