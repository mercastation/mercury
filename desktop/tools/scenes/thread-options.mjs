// A thread that waits on the person, its agent's options still open: the strip of them over
// the reply box, as the thread first opens.
import { open } from './thread.mjs';

export const go = (h) => open(h, { unfold: false, waiting: true });
