// The change open, its first file folded shut from its head (`foldFile`): the head alone, its
// chevron turned, and in a wide pane the file dimmed in the tree. The fixture says which
// (`changeFolded`).
import { openChange } from './thread-change-open.mjs';
import { click } from './thread-pane.mjs';
import { middleOf } from './pointer.mjs';

export const go = (h) => openChange(h, {
  then: async (h) => {
    const [x, y] = await middleOf(h, '.code-file-head');
    await click(h, x, y);
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
    await h.wait(500);
    return { changeFolded: ['notes/refunds-for-support.md'] };
  },
});
