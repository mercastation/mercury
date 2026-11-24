// New pressed on a project's home: Document, Table, Folder and Upload from computer (`NewMenu`).
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: async (h) => {
  await press(h, '.fh-top .th-new');
  return { filesNew: true };
} });
