// `thread-reply-mode-moved` with the rail kept open (Settings' "the sidebar shrinks to icons"
// off), for a window at its narrowest: the reply box is under 460 points, so its chips drop
// their words and marks (`@container reply` in renderer/src/narrow-conversation.css), and the
// mode chip, which no longer fits, folds out of sight. Photographed at 720 by 760.
import { go as moved } from './thread-reply-mode-moved.mjs';

export { dress } from './thread-reply-mode-moved.mjs';

export async function go(h) {
  await h.evaluate(`localStorage.setItem('agentbox.sidebarShrinks', '0')`);
  const did = await moved(h);
  return { ...did, picks: { rail_shrinks: false } };
}
