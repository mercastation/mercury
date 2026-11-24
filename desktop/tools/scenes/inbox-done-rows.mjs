// The Done tab with rows in it: five of the invented inbox's threads closed by the person, one
// of them after a reply the agent never answered, whose second line on Done is still its
// result (`rowSummary`, which reads every row of Done as finished), and one whose result only
// says its title again, which keeps one line. Nothing here is anyone's real thread.
import { tab } from './inbox-progress.mjs';

const MIN = 60_000;

export function dress(snapshot, now) {
  const closed = ['w-readme1', 'w-readme3', 'w-readme5', 'w-readme7', 'w-readme8'];
  const items = snapshot.items.map((i, n) => {
    if (!closed.includes(i.id)) return i;
    const at = now - (20 + n) * MIN;
    const item = { ...i, wrote: { ...i.wrote, status: { ts: at, source: 'founder' } }, updatedAt: at };
    // A reply after the result, never answered: open, it would read the reply.
    if (i.id === 'w-readme3') return { ...item, answer: 'Go with the second one.', wrote: { ...item.wrote, answer: { ts: at - MIN, source: 'founder' }, body: { ts: at - MIN, source: 'founder' } } };
    if (i.id === 'w-readme7') return { ...item, result: i.title };
    return item;
  });
  return { ...snapshot, items };
}

export async function go(h) {
  await tab(h, 'Done');
}
