// A chat just brought in, owed the one run that writes it up (`catchUpDue`): its page says it is
// getting the chat ready, the sentence shimmering as the working line's word does, over the
// chat's newest answer (`ItemThread.tsx`). The invented thread is dressed as one: open, with the
// labels an import gives it, and opened from In progress, where it waits.
import { open } from './thread.mjs';

export const go = (h) => open(h, {
  unfold: false,
  waiting: true,
  fromTab: 'In progress',
  dress: (row) => {
    row.status = 'open';
    row.labels = [...new Set([...(row.labels ?? []), 'thread-import', 'catch-up'])];
    row.wrote = { ...row.wrote, status: { ts: row.updatedAt, source: 'system' } };
  },
});
