// The command bar over In progress, on a thread the person filed that waits its turn in the
// queue, on a Mac with the cloud on: Run now, Pin thread, where it runs, Move to the other
// project and its level, as the web window offers them for that row.
import { CLOUDS } from './inbox-cloud.mjs';
import { tab } from './inbox-progress.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

const MIN = 60_000;

export function dress(snapshot, now) {
  const asked = now - 20 * MIN;
  const waiting = {
    id: 'w-readme-queued', product: 'storefront', productName: 'Storefront', status: 'open', kind: 'directive',
    title: 'Add the receipt number to the order email', body: 'Add the receipt number to the order email', labels: ['founder'], priority: 7,
    epoch: 0, claim: null, createdAt: asked, updatedAt: asked,
    wrote: { title: { ts: asked, source: 'founder' }, body: { ts: asked, source: 'founder' } },
  };
  return {
    ...snapshot,
    items: [...snapshot.items, waiting],
    supervisor: { ...snapshot.supervisor, queued: [waiting.id] },
    cloud: CLOUDS.on,
  };
}

export async function go(h) {
  await tab(h, 'In progress');
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
