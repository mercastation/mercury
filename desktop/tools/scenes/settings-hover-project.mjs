// Settings, Mobile's row, with the pointer resting on it (`settings.mjs`, `pointer.mjs`).
import { open } from './settings.mjs';
import { rest } from './pointer.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'projects', { then: (h) => rest(h, '.pp-row:nth-child(2) .pp-text') });
