// The page of every file, "Every file, with sizes and the Trash" pressed on a project's home:
// the bar, the kinds, the folder cards and the tiles this window drew before as the whole place
// (`filesEverything`).
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true });
