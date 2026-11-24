// The page of a project's files in no folder, "Not in a folder" pressed on its home.
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', folder: 'not-in-a-folder' });
