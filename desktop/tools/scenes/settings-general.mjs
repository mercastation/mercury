// Settings, General: the keyboard's hints, the sidebar, the tray, what Mercury sends, where it keeps things, the version and the account (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'general');
