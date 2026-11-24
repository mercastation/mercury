// The pointer resting on a row of Recent: the row's wash.
import { openFiles } from './files.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: (h) => rest(h, '.fh-rrow:nth-of-type(2)') });
