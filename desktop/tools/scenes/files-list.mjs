// The Files place drawn as a list rather than tiles, its List button pressed (`filesAsList`).
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, {
  then: async (h) => {
    await h.evaluate(`document.querySelector('.fp-view-icon[aria-label="List"]')?.click()`);
    await h.wait(600);
    return { filesAsList: true };
  },
});
