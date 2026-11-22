// Settings, Files: keeping files in the cloud, which a cloud of one's own does not come with (`settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'files');
