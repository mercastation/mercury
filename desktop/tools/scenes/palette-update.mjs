// The command bar over the Inbox while a new version waits, the app's main process changed on
// disk, the agents are paused and a project was made for the person and never ranked: Restart
// to update, the restart a build needs, Unpause agents, and "made for you".
import { dress as withUpdate } from './home-update.mjs';
import { go as toInbox } from './inbox.mjs';
import { commandK, rowsOnScreen } from './palette.mjs';

export function dress(snapshot) {
  const updated = withUpdate(snapshot);
  return {
    ...updated,
    products: [...updated.products, { slug: 'workspace', name: 'My workspace', dir: '/fixtures/workspace', personal: true, createdAt: 1 }],
    restartNeeded: { files: ['main/ipc.mts', 'main/store.mts'] },
    supervisor: { ...updated.supervisor, paused: true },
  };
}

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
