// Settings with "memory" typed into the menu's search: the hits in place of the pages, the
// first lit (`searchSettings`, `settings.mjs`).
import { open, typeIntoSearch } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'general', { then: (h) => typeIntoSearch(h, 'memory') });
