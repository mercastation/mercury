// A reply sent to a thread an agent is on: a second after Send it is on the page, its head
// saying Z still takes it back, and no line says it was sent (`answerWith`, `isRunning`).
import { open } from './thread.mjs';
import { send } from './thread-reply-sending.mjs';

export const RUN_WORDS = 'Also refund the two orders from Friday.';

export const go = (h) => open(h, {
  unfold: false, runs: 'working',
  then: async (h) => {
    await send(h, RUN_WORDS);
    await h.wait(500);
    return { replySent: RUN_WORDS };
  },
});
