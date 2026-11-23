// The strip's code button under the pointer (`.ts-done:hover`): the text's ink on the strong wash.
import { open } from './thread.mjs';
import { changeFor, hexId } from './change.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => open(h, { unfold: false, files: {}, change: changeFor(), dress: hexId, then: (h) => rest(h, '.ts-code') });
