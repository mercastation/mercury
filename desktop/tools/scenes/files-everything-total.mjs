// The page of every file with files that take megabytes: the quiet total beside the sort, and its
// menu pressed open (`TotalMenu`): what the place uses, the biggest first, the older versions,
// choosing files for the Trash, and keeping files in the cloud.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, heavy: true, project: 'Storefront', everything: true, then: async (h) => {
  await press(h, '.fp-total');
  return { filesMenu: { total: true } };
} });
