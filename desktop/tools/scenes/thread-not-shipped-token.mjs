// `thread-not-shipped` after "Paste a token instead" was pressed: the three facts, the box
// for the token, and Save, which takes no press over an empty box.
import { unshipped } from './thread-not-shipped.mjs';

export const go = (h) => unshipped(h, {
  then: async (h) => {
    await h.evaluate(`document.querySelector('.need .need-skip')?.click()`);
    return { gitHubPressed: ['token'] };
  },
});
