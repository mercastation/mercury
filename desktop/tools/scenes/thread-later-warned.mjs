// The first L ever on a thread whose agent waits on an answer: the Later picker opens with the
// warning inside it, under the thread's name (`openSnooze`, `stoppedWarning`).
import { open } from './thread.mjs';
import { press } from './thread-option-picked.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => { await press(h, 'l', 'KeyL', 76); await h.wait(300); return { keysAfter: ['l'] }; },
});
