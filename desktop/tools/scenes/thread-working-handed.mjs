// The same reply once its three seconds are over and it has been handed to the run, which has
// not taken it yet: its head says it waits for the step the run is in, with Send now after it
// (`onSendNow`).
import { open } from './thread.mjs';
import { send } from './thread-reply-sending.mjs';
import { RUN_WORDS } from './thread-working-sending.mjs';

export const go = (h) => open(h, {
  unfold: false, runs: 'working', hang: true,
  then: async (h) => {
    await send(h, RUN_WORDS);
    await h.wait(3600);
    return { replySent: RUN_WORDS, handedOver: true };
  },
});
