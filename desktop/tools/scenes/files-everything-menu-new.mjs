// New folder pressed in a tile's menu on the page of every file: the box for the folder's name
// in its place, Rename and Move to the Trash put away while it is there.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-tile .fp-more', 'More for checkout-wide.png');
  await press(h, '.fp-menu-item', 'New folder');
  return { filesMenu: { file: 'storefront/checkout-wide.png', step: 'new' } };
} });
