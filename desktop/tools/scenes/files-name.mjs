// The page of every file ordered by name, its Name switch pressed (`filesSort`). The page is
// reached from a project's home, as All files has none.
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, {
  project: 'Storefront',
  everything: true,
  then: async (h) => {
    await h.evaluate(`[...document.querySelectorAll('.fp-switch .fp-kind')].find((b) => b.textContent.trim() === 'Name')?.click()`);
    await h.wait(600);
    return { filesSort: 'name' };
  },
});
