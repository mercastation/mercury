// The change open and moved to a file, as the picker or a step's chip moves it (`goTo`): the
// file's block at the top of the column, its head lit, the picker or the tree naming it.
// `at` is the file; `then` what the scene does after. The fixture says the file (`changeAt`).
import { openChange } from './thread-change-open.mjs';

export const AT = 'src/checkout/tax.ts';

export function openAt(h, { at = AT, then = null } = {}) {
  return openChange(h, {
    then: async (h) => {
      const moved = await h.evaluate(`(() => {
        const pick = document.querySelector('.code-file-picker');
        if (pick && getComputedStyle(pick.parentElement).display !== 'none') {
          pick.value = ${JSON.stringify(at)};
          pick.dispatchEvent(new Event('change', { bubbles: true }));
          return true;
        }
        const row = [...document.querySelectorAll('.code-node.code-file')].find((b) => b.title === ${JSON.stringify(at)});
        row?.click();
        return !!row;
      })()`);
      if (!moved) throw new Error('change scene: nothing moved to the file');
      await h.wait(900);
      const more = then ? (await then(h)) ?? {} : {};
      return { changeAt: at, ...more };
    },
  });
}

export const go = (h) => openAt(h);
