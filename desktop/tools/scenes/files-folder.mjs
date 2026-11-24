// A folder's own page, its section's name pressed on its project's home: the path back, its
// name as the heading, and every file in it and in the folders inside it (`Shelf`).
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', folder: 'rec' });
