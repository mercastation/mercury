// The thread's own menu turned to Priority: the four levels in its place, the one the thread is
// at ticked (`leveling` in `renderer/src/threads/ThreadMenu.tsx`).
import { open } from './thread.mjs';
import { pressDots } from './thread-actions.mjs';

export const go = (h) => open(h, {
  unfold: false,
  waiting: true,
  then: async (h) => {
    await pressDots(h);
    await h.evaluate(`[...document.querySelectorAll('.ts-more-menu .ts-more-row')].find((row) => row.querySelector('.ts-more-label')?.textContent === 'Priority')?.click()`);
    await h.wait(400);
    return { actions: 'priority' };
  },
});
