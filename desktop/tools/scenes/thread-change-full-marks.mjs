// The change over the whole workspace with the pointer just moved over it: the marks wake in
// the top right corner (`data-chrome="up"`).
import { openChange } from './thread-change-open.mjs';
import { click } from './thread-pane.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openChange(h, {
  then: async (h) => {
    const at = await h.evaluate(`(() => { const b = document.querySelector('.doc-pane .focus-mode-control'); const r = b.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
    await click(h, at[0], at[1]);
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
    await h.wait(2500);
    return { docFull: true, ...(await rest(h, [900, 500], 400)) };
  },
});
