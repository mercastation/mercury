// The document beside the conversation with the pointer resting on its crumb (`.doc-crumb`).
import { open } from './thread-pane.mjs';
import { rest } from './pointer.mjs';

export const go = (h) => open(h, { then: (h) => rest(h, '.doc-crumb') });
