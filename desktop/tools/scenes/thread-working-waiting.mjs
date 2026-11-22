// The handed-over reply on a run that says it is Claude Code's (`engine: 'claude'`, as the
// supervisor writes it): its head says it waits for the step the run is in, with Send now after
// it (`onSendNow`, `cutsARunningStep`).
import { open } from './thread.mjs';
import { send } from './thread-reply-sending.mjs';
import { RUN_WORDS } from './thread-working-sending.mjs';

export const go = (h) => open(h, {
  unfold: false, runs: 'working', hang: true, runEngine: 'claude',
  then: async (h) => {
    await send(h, RUN_WORDS);
    await h.wait(3600);
    return { replySent: RUN_WORDS, handedOver: true };
  },
});
