// A tile's menu at Move to…: the project's folders, Calls under Receipts, the one the file is
// in ticked, Not in a folder, and New folder….
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for The receipt, line by line');
  await press(h, '.fh-item', 'Move to…');
  return { filesMenu: { file: 'storefront/receipt-plan.md', step: 'move' } };
} });
