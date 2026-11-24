// Needs you on `inbox-later-kinds`' inbox: the cause the threads in Later are stuck behind is a
// row of its own here ("1 waiting", where most of them are), among the invented inbox's rows,
// each with its second line. Nothing here is anyone's real thread.
import { withSnapshot } from './inbox.mjs';
import { laterThreads } from './inbox-later-kinds.mjs';

export async function go(h) {
  return withSnapshot(h, (snapshot) => {
    const { items, stalled } = laterThreads(Date.now(), snapshot.products);
    return { ...snapshot, items: [...snapshot.items, ...items], supervisor: { ...snapshot.supervisor, stalled } };
  });
}
