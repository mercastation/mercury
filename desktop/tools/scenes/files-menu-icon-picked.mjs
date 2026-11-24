// Change icon… on a file whose icon a person picked: that mark outlined, Automatic not ticked.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-more-btn', 'More for failed-orders.csv');
  await press(h, '.fh-item', 'Change icon…');
  return { filesMenu: { file: 'storefront/failed-orders.csv', step: 'icon' } };
} });
