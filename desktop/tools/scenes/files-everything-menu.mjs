// A tile's ⋯ pressed on the page of every file (`MoveMenu`): Rename, Move to with the box that
// finds a folder (the project has more than three), its folders, New folder and Move to the
// Trash with what that frees. The fixture says whose (`filesMenu`).
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, heavy: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-tile .fp-more', 'More for checkout-wide.png');
  return { filesMenu: { file: 'storefront/checkout-wide.png' } };
} });
