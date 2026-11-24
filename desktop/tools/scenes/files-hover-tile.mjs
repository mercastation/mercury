// The pointer resting on a tile of a project's home: its edge and ground lit, a point higher,
// and its ⋯ shown.
import { openFiles } from './files.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: (h) => rest(h, '.fh-main .fh-tile') });
