// Settings, the crumb back to Projects, with the pointer resting on it (`settings.mjs`, `pointer.mjs`).
import { open } from './settings.mjs';
import { rest } from './pointer.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'projects:storefront', { then: (h) => rest(h, '.set-crumb') });
