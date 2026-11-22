// `home-team`'s Home, with the rail open under the pointer: Chats, Record meeting, the reading
// of usage and the person's own row show their words too (`home-rail-open` says when and why
// Demands).
import { rest } from './pointer.mjs';

export { dress } from './home-team.mjs';

export const go = (h) => rest(h, 'button.workspace-tab[data-tab="demands"]', 900);
