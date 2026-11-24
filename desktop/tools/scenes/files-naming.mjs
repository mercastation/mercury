// New, then Folder, on a project's home: the box for the new folder's name, as a section.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-top .th-new');
  await press(h, '.fp-menu-item', 'Folder');
  return { filesNaming: true };
} });
