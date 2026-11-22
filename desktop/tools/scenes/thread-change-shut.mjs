// A wide pane's tree with a folder shut (`shut`): its chevron turned, what is under it hidden,
// the column as it was. The fixture says which (`changeShut`).
import { openChange } from './thread-change-open.mjs';
import { click } from './thread-pane.mjs';

export const go = (h) => openChange(h, {
  then: async (h) => {
    const at = await h.evaluate(`(() => { const f = [...document.querySelectorAll('.code-node.code-folder')].find((b) => b.textContent.startsWith('scripts')); const r = f.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; })()`);
    await click(h, at[0], at[1]);
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
    await h.wait(400);
    return { changeShut: ['scripts/'] };
  },
});
