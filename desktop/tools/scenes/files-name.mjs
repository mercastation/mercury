// The Files place ordered by name, its Name switch pressed (`filesSort`).
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, {
  then: async (h) => {
    await h.evaluate(`[...document.querySelectorAll('.fp-switch .fp-kind')].find((b) => b.textContent.trim() === 'Name')?.click()`);
    await h.wait(600);
    return { filesSort: 'name' };
  },
});
