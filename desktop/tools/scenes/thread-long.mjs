// A long conversation with its middle left out: an agent at work on a thread whose run has written
// forty-five checkpoints since the person asked, and nothing from the person since. The page is
// drawn from the ask, so it keeps the opening and the last forty messages and puts the line that
// asks for the middle where it was cut (`conversationGap`, `.thread-gap`). The picture is of the
// top of the page, scrolled there once the page has settled; the fixture says so (`scrolledTo`).
import { open } from './thread.mjs';

const MIN = 60_000;

/** The ask, the claim, and forty-five checkpoints a minute apart, the newest just now. */
export function chatty(lines, row) {
  const [asked, claimed] = lines;
  const first = row.updatedAt - 46 * MIN;
  const notes = Array.from({ length: 45 }, (_, n) => ({
    id: row.id, ts: first + n * MIN, source: 'agent', epoch: 2,
    patch: { note: `Checkpoint ${n + 1}: tried rounding step ${n + 1} of the tax table, and the totals still differ by a cent.` },
  }));
  return [asked, { ...claimed, ts: first - MIN }, ...notes];
}

/** Scrolls the conversation to its top, as a person reading back would. */
export async function toTheTop(h) {
  await h.wait(2000);
  await h.evaluate(`(() => { const box = document.querySelector('.focus-scroll'); if (box) box.scrollTop = 0; })()`);
  await h.wait(400);
  return { scrolledTo: 0 };
}

export const go = (h) => open(h, {
  unfold: false,
  runs: 'working',
  trace: () => [],
  ledger: chatty,
  dress: (row) => {
    delete row.answer;
    row.note = `Checkpoint 45: tried rounding step 45 of the tax table, and the totals still differ by a cent.`;
    row.wrote = { ...row.wrote, note: { ts: row.updatedAt - MIN, source: 'agent' } };
    delete row.wrote.answer;
  },
  then: toTheTop,
});
