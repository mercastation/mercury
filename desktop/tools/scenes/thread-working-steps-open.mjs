// `thread-working-steps` with its first step pressed open onto the command behind it. The
// fixture says which rows were open, by their place (`liveRowsOpen`).
import { open } from './thread.mjs';
import { unfold } from './thread-working-steps.mjs';

export const go = (h) => open(h, {
  unfold: false,
  runs: 'working',
  then: async (h) => {
    const did = await unfold(h);
    await h.evaluate(`document.querySelector('.live-command summary')?.click()`);
    await h.wait(400);
    return { ...did, liveRowsOpen: [0] };
  },
});
