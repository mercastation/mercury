// A project's home, its tile pressed on All files: Recent by day on the left, its folders as
// sections (Pinned first), the empty folders in a sentence, the files in no folder, and Every
// file (`FilesHome`). The fixture says which project (`filesProject`).
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront' });
