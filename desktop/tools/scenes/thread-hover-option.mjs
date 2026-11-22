// A conversation whose agent offers three options, with the pointer resting on the second.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.opt-strip .opt-row:nth-of-type(2) .opt-text', { waiting: true });
