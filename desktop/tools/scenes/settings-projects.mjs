// Settings, Projects: the running order, with its search, Select and New project, and the archived ones folded away (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'projects');
