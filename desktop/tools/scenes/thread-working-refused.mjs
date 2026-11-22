// The same reply refused as it is handed over (the harness's bridge refuses every write): its
// drawn copy goes, its words are back in the box, opened, and the engine's words stand in the
// strip (`putBack`).
import { open } from './thread.mjs';
import { send } from './thread-reply-sending.mjs';
import { RUN_WORDS } from './thread-working-sending.mjs';

export const go = (h) => open(h, {
  unfold: false, runs: 'working',
  then: async (h) => {
    await send(h, RUN_WORDS);
    await h.wait(3600);
    return { replySent: RUN_WORDS, handedOver: true, refusedWith: 'read-only shot bridge' };
  },
});
