// The pointer resting on a folder card on the page of every file: its edge brighter and its ⋯
// shown.
import { openFiles } from './files.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: (h) => rest(h, '.fp-folder') });
