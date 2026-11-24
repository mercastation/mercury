// `home-team`'s Home, with the sidebar open under the pointer: Chats and Record meeting show
// their words too, and the person's own row its name, with the bar of what is left of their
// limit at its end (`home-rail-open` says when it opens and why the pointer rests on Home).
import { restOnTheSidebar } from './pointer.mjs';

export { dress } from './home-team.mjs';

export const go = (h) => restOnTheSidebar(h, 'button.workspace-tab[data-tab="home"]', 900);
