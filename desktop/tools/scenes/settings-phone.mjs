// Settings, Phone: the App Store's code, Android, and which projects the phone has (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'phone');
