// `home-update-rail-open`'s Home once Restart to update was pressed and the new version is being
// built: no card to rise, since there is nothing to press; the foot's first row says Updating
// and its mark turns (`SidebarUpdate.tsx`, `if (!rail || installing) return row`).
import { restOnTheSidebar } from './pointer.mjs';
import { dress as waiting } from './home-update.mjs';

export function dress(snapshot) {
  const out = waiting(snapshot);
  return { ...out, update: { ...out.update, phase: 'installing', ready: false, installing: true } };
}

export const go = (h) => restOnTheSidebar(h, 'button.workspace-tab[data-tab="home"]', 900);
