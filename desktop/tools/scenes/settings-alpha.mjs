// Settings, Alpha features: four switches, all off (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'alpha');
