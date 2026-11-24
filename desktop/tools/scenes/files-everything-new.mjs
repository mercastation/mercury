// New pressed in the strip on the page of every file: its menu, 38 under the button.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.topbar .th-new');
  return { filesNew: true };
} });
