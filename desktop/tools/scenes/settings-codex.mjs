// Settings, Codex: its one account, on this computer only, and its permission mode (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'codex');
