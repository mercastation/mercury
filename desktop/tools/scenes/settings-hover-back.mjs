// Settings, Back to app, with the pointer resting on it (`settings.mjs`, `pointer.mjs`).
import { open } from './settings.mjs';
import { rest } from './pointer.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'general', { then: (h) => rest(h, '.set-nav-back') });
