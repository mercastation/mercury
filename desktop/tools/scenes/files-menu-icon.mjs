// A tile's menu at Change icon…: the marks a person picks from, and Automatic ticked.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for Rounding');
  await press(h, '.fh-item', 'Change icon…');
  return { filesMenu: { file: 'storefront/rounding-v2.md', step: 'icon' } };
} });
