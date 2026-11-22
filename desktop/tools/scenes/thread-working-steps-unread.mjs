// `thread-working-steps` when the run's log cannot be read: the working line pressed open while
// every read of the log fails, so under what it holds it says it could not read the command
// history (`Live.tsx`, `historyError`). The fixture says so (`historyUnread`).
import { open } from './thread.mjs';
import { unfold } from './thread-working-steps.mjs';

export const go = (h) => open(h, {
  unfold: false,
  runs: 'working',
  then: async (page) => {
    await page.evaluate(`(() => { window.zero.sessionTrace = async () => { throw new Error('the log could not be read'); }; })()`);
    return { ...(await unfold(page)), historyUnread: true };
  },
});
