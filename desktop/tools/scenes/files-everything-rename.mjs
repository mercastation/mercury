// Rename on a tile's menu on the page of every file: its name a box over its own line
// (`.fp-name > .fp-name-box`), all of it chosen.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-tile .fp-more', 'More for receipt-small.png');
  await press(h, '.fp-menu-item', 'Rename');
  return { filesRenaming: { file: 'storefront/receipt-small.png' } };
} });
