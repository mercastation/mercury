// "Choose files to move to the Trash" pressed on the total's menu: the line that says so, Cancel
// and a Move to the Trash with nothing to move yet, a ring on every tile Files may move, and the
// one it may not faded (`chosen`).
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, heavy: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-total');
  await press(h, '.fp-menu-item', 'Choose files to move to the Trash');
  return { filesChosen: [] };
} });
