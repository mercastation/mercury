// The waiting thread with the dots at the right of its head strip pressed: the thread's own menu
// under them (`ThreadMenuList` in `renderer/src/threads/ThreadMenu.tsx`), its first row holding
// the keys.
//
// The web window's menu offers rows this window does not do (the terminal, renaming, moving,
// pinning), so a like-for-like picture draws what the page drew: the scene reads each row's
// name, its key or its detail, and which of the menu's marks it wears, into the fixture
// (`actionRows`), as the command bar's scenes do with `paletteRows`.
import { open } from './thread.mjs';

// The rows as the page drew them, by the mark each wears: `ThreadMenu.tsx` names a row by its
// verb, and the mark is the verb's own. A key is a cap (`kbd`); a value said in words, such as
// a priority, is a detail (`.ts-more-detail`).
export const rowsOnScreen = (h) => h.evaluate(`[...document.querySelectorAll('.ts-more-menu .ts-more-row')].map((row) => ({
  label: row.querySelector('.ts-more-label')?.textContent ?? '',
  key: row.querySelector('kbd')?.textContent ?? '',
  detail: row.querySelector('.ts-more-detail')?.textContent ?? '',
  mark: row.querySelector('.ico svg')?.innerHTML ?? '',
}))`);

// The dots by their name: the arrows to the threads before and after wear the same class.
export const pressDots = async (h) => {
  await h.evaluate(`document.querySelector('.ts-more-wrap > button[aria-label="More actions"]')?.click()`);
  await h.wait(400);
};

export const go = (h) => open(h, {
  unfold: false,
  waiting: true,
  then: async (h) => {
    await pressDots(h);
    return { actions: 'menu', actionRows: await rowsOnScreen(h) };
  },
});
