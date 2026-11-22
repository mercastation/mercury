// A conversation, as it first opens, with the pointer resting on the rail's Demands button: the
// rail open beside it, the card and its head strip moved over (`home-rail-open` says when it
// opens, and why Demands).
import { hover } from './thread-hover.mjs';

export const go = (h) => hover(h, 'button.workspace-tab[data-tab="demands"]', { ms: 900 });
