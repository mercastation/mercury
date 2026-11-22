// A conversation that made files, with the pointer resting on its Files button.
import { hover } from './thread-hover.mjs';
import { FOUND, SHIPPED } from './thread-files.mjs';

export const go = (h) => hover(h, '.ts-files', { found: FOUND, shipped: SHIPPED, files: {} });
