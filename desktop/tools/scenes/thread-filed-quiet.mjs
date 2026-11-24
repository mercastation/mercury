// `thread-filed` with nothing waiting: one filed thread finished and one already approved, so the
// list is "Filed from this thread", each row saying the tab it sits in.
import { filed, FINISHED } from './thread-filed.mjs';

export const go = (h) => filed(h, [{ id: 'w-f11ed00005', title: 'Refund the 37 orders once support has read the note', answer: 'Approved. Run it.' }, FINISHED]);
