// A folder's menu, the ⋯ beside its section's name pressed: Rename and Delete folder.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for Receipts');
  return { filesMenu: { folder: 'rec' } };
} });
