// Rename on a folder card's menu on the page of every file: the card's name and count give way to
// a box holding its name, all of it chosen.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-folder-more .fp-more', 'More for Receipts');
  await press(h, '.fp-menu-item', 'Rename');
  return { filesRenaming: { folder: 'rec' } };
} });
