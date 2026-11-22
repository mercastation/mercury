// A conversation whose agent waits on an answer, with the pointer resting on the link its answer
// names, drawn as a mention.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, '.focus-body a.mention .mention-name', { waiting: true });
