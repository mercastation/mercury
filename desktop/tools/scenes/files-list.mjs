// The page of every file drawn as a list rather than tiles, its List button pressed
// (`filesAsList`). The page is reached from a project's home, as All files has none.
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, {
  project: 'Storefront',
  everything: true,
  then: async (h) => {
    await h.evaluate(`document.querySelector('.fp-view-icon[aria-label="List"]')?.click()`);
    await h.wait(600);
    return { filesAsList: true };
  },
});
