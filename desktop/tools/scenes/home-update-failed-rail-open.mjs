// `home-update-rail-open`'s Home after a restart that did not finish: the card says why, in the
// app's own words, and that a press tries again, on as many lines as that takes, so the card
// rises that much higher over its button (`failed` in `SidebarUpdate.tsx`).
import { restOnTheSidebar } from './pointer.mjs';
import { dress as waiting } from './home-update.mjs';

export function dress(snapshot) {
  const out = waiting(snapshot);
  return { ...out, update: { ...out.update, error: 'The new code would not build' } };
}

export const go = (h) => restOnTheSidebar(h, 'button.workspace-tab[data-tab="home"]', 900);
