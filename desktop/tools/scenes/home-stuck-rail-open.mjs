// Home on a copy of the app that cannot update itself, run from the disk image it came in, with
// the sidebar open under the pointer: the card "Updates are stuck" stands where the new
// version's would, over Move to Applications (`SidebarMove` in `SidebarUpdate.tsx`).
import { restOnTheSidebar } from './pointer.mjs';

export function dress(snapshot) {
  return {
    ...snapshot,
    update: {
      canMove: true, place: 'disk-image', moveError: null, phase: 'idle', currentVersion: '0.1.0', newVersion: null,
      percent: null, error: null, checkedAt: null, readyAt: null, ready: false, source: false, changes: [], behind: null, installing: false,
    },
  };
}

export const go = (h) => restOnTheSidebar(h, 'button.workspace-tab[data-tab="home"]', 900);
