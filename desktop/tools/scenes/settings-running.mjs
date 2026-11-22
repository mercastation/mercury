// Settings, Running: what it measured, how many run at once, the memory check, the leftovers, and the agents started by hand (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'running');
