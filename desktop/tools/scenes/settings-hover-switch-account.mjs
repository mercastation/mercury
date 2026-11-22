// Settings, the Switch on the account not in use, with the pointer resting on it (`settings.mjs`, `pointer.mjs`).
import { open } from './settings.mjs';
import { rest } from './pointer.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'claude', { then: (h) => rest(h, '.acct-switch') });
