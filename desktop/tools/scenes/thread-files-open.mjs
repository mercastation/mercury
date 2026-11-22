// The same thread with its Files button pressed: the list it opens under the button, each
// file its newest version, newest first, with its kind and its name. The fixture says the list
// was open (`filesOpen`).
import { open } from './thread.mjs';
import { FOUND, SHIPPED } from './thread-files.mjs';

export const go = (h) => open(h, {
  unfold: false,
  found: FOUND,
  shipped: SHIPPED,
  files: {},
  then: async (h) => {
    await h.evaluate(`document.querySelector('.ts-files')?.click()`);
    await h.wait(300);
    return { filesOpen: true };
  },
});
