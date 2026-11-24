// `home-update`'s Home, with the sidebar open under the pointer: the new version's card risen
// over its row, as the open sidebar draws it (`home-rail-open` says when it opens and why the
// pointer rests on Home).
import { rest } from './pointer.mjs';

export { dress } from './home-update.mjs';

export const go = (h) => rest(h, 'button.workspace-tab[data-tab="home"]', 900);
