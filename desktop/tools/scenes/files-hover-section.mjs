// The pointer resting on a section's name: underlined, and the folder's ⋯ shown.
import { openFiles } from './files.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => openFiles(h, { kept: true, project: 'Storefront', then: (h) => rest(h, '.fh-main .fh-link') });
