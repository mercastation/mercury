// A conversation with the pointer resting on the sidebar's Home: the sidebar open beside it,
// with Recent under Workspace holding the thread that is open, which was opened a moment ago
// (`recentThreads` in renderer/src/recent-threads.ts). Home, because this window's sidebar has
// it where the web window's does, so the pointer rests on the same row in both.
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, 'button.workspace-tab[data-tab="home"]', { ms: 900 });
