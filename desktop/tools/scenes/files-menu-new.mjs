// Move to…'s New folder… pressed: a box for the new folder's name at the menu's foot.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for The receipt, line by line');
  await press(h, '.fh-item', 'Move to…');
  await press(h, '.fh-item', 'New folder…');
  return { filesMenu: { file: 'storefront/receipt-plan.md', step: 'new' } };
} });
