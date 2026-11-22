// The document beside the conversation with the pointer resting on its outside (`.doc-marks > .doc-mark:nth-last-of-type(2)`).
import { open } from './thread-pane.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => open(h, { then: (h) => rest(h, '.doc-marks > .doc-mark:nth-last-of-type(2)') });
