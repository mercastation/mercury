// New on All files, then Document: which project the document goes in.
import { openFiles, press } from './files.mjs';

export const go = (h) => openFiles(h, { then: async (h) => {
  await press(h, '.fh-top .th-new');
  await press(h, '.fp-menu-item', 'Document');
  return { filesNew: 'document' };
} });
