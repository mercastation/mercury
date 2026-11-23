// The change at src/checkout/tax.ts, the lines between its two hunks pressed open
// (`openGap`): read from the file as it is now and drawn in place, numbered. The fixture says
// which gap (`changeGaps`).
import { openAt } from './thread-change-at.mjs';
import { click } from './thread-pane.mjs';
import { middleOf } from './pointer.mjs';

export const go = (h) => openAt(h, {
  then: async (h) => {
    const [x, y] = await middleOf(h, '.code-skip');
    await click(h, x, y);
    await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: 10, y: 400 });
    await h.wait(600);
    return { changeGaps: [['src/checkout/tax.ts', 1]] };
  },
});
