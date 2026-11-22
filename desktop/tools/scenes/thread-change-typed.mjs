// The change with words typed into one of its lines, then let go with Escape: the file marked
// Unsaved on its head and, in a wide pane, its dot in the tree. The fixture says what was typed
// (`changeTyped`: the file, the hunk, the row and the line as it reads now).
import { openChange } from './thread-change-open.mjs';
import { click } from './thread-pane.mjs';

export const go = (h) => openChange(h, {
  then: async (h) => {
    const at = await h.evaluate(`(() => {
      const line = [...document.querySelectorAll('.code-row.cr-live .code-line')][2];
      const r = line.getBoundingClientRect();
      return [r.x + 3, r.y + r.height / 2];
    })()`);
    await click(h, at[0], at[1]);
    await h.wait(200);
    await h.call('Input.insertText', { text: 'Every ' });
    await h.wait(200);
    await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
    await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
    await h.wait(500);
    const said = await h.evaluate(`[...document.querySelectorAll('.code-row.cr-live .code-line')][2].textContent`);
    return { changeTyped: [['notes/refunds-for-support.md', 0, 2, said]] };
  },
});
