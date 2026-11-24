// A tile's Rename: its name a box over its own line, all of it chosen.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for Rounding');
  await press(h, '.fh-item', 'Rename');
  return { filesRenaming: { file: 'storefront/rounding-v2.md' } };
} });
