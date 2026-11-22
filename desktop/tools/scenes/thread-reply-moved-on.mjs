// A reply sent from a waiting thread opened from Needs you: the page moves on to the next thread
// waiting, and the line over its reply box says where the reply went (`replyMovesOn`,
// `SentLine`).
import { open } from './thread.mjs';
import { WORDS, send } from './thread-reply-sending.mjs';

export const go = (h) => open(h, {
  unfold: false, waiting: true,
  then: async (h) => {
    await send(h, WORDS);
    await h.wait(700);
    return { replySent: WORDS, from: 'needs' };
  },
});
