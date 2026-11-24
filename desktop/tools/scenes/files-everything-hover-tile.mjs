// The pointer resting on a tile on the page of every file: its face's edge brighter and its ⋯
// shown at its corner.
import { openFiles } from './files.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', everything: true, then: (h) => rest(h, '.fp-tile .fp-face') });
