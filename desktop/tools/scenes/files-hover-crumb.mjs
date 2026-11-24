// The pointer resting on the first crumb of the path back, on a folder's page.
import { openFiles } from './files.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', folder: 'rec', then: (h) => rest(h, '.fh-crumb') });
