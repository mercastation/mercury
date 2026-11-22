// `home-update`'s Home, with the rail open under the pointer: the Restart row standing as the
// open rail draws it (`home-rail-open` says when and why Demands).
import { rest } from './pointer.mjs';

export { dress } from './home-update.mjs';

export const go = (h) => rest(h, 'button.workspace-tab[data-tab="demands"]', 900);
