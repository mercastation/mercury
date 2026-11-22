// The document beside the conversation with the pointer resting on its globe (`.doc-marks .av-share .doc-mark`).
import { open } from './thread-pane.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => open(h, { then: (h) => rest(h, '.doc-marks .av-share .doc-mark') });
