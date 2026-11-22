// Settings, a project's own page: how it runs, its instructions, its browser, where it is kept and
// archiving it (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'projects:storefront');
