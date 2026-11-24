// A folder card's ⋯ pressed on the page of every file (`FolderMenu`): Rename and Delete folder.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-folder-more .fp-more', 'More for Receipts');
  return { filesMenu: { folder: 'rec' } };
} });
