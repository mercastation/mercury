// ⌘K over a finished conversation (`thread`): Reopen, Resume this agent, and the move it
// cannot make, saying why.
import { open } from './thread.mjs';
import { typedOver } from './palette-thread.mjs';

export const go = (h) => open(h, { unfold: false, then: typedOver('') });
