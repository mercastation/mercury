// A conversation with the pointer resting on the sidebar's Home: the sidebar open beside it,
// with Recent under Workspace holding the thread that is open, which was opened a moment ago
// (`recentThreads` in renderer/src/recent-threads.ts). Home, because this window's sidebar has
// it where the web window's does, so the pointer rests on the same row in both. The pointer
// comes onto the sidebar from the card beside it, the only way it opens (`restOnTheSidebar`).
import { open } from './thread.mjs';
import { restOnTheSidebar } from './pointer.mjs';

export async function go(h) {
  const fed = await open(h, {});
  return { ...fed, ...(await restOnTheSidebar(h, 'button.workspace-tab[data-tab="home"]', 900)) };
}
