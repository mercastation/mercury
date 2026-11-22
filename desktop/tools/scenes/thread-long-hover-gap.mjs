// `thread-long` with the pointer resting on the line that asks for the missing middle: dim and
// underlined (`.thread-gap:hover`).
import { open } from './thread.mjs';
import { chatty, toTheTop } from './thread-long.mjs';
import { rest } from './pointer.mjs';

const MIN = 60_000;

export const go = async (h) => {
  const fed = await open(h, {
    unfold: false,
    runs: 'working',
    trace: () => [],
    ledger: chatty,
    dress: (row) => {
      delete row.answer;
      row.note = 'Checkpoint 45: tried rounding step 45 of the tax table, and the totals still differ by a cent.';
      row.wrote = { ...row.wrote, note: { ts: row.updatedAt - MIN, source: 'agent' } };
      delete row.wrote.answer;
    },
    then: toTheTop,
  });
  return { ...fed, ...(await rest(h, '.thread-gap', 200)) };
};
