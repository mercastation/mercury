// A tile's menu, its ⋯ pressed on a project's home: Open, Rename, Change icon…, Move to… and
// Move to the Trash (`TileMenu`). The fixture says whose (`filesMenu`).
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for Rounding');
  return { filesMenu: { file: 'storefront/rounding-v2.md' } };
} });
