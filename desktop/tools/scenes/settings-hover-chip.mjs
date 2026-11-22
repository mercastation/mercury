// Settings, the chip of How many run at once, with the pointer resting on it (`settings.mjs`, `pointer.mjs`).
import { open } from './settings.mjs';
import { rest } from './pointer.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'running', { then: (h) => rest(h, '.set-picker-now') });
