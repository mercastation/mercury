// Settings, the menu of How many run at once, open, with the pointer resting on its third row
// (`settings.mjs`, `pointer.mjs`).
import { open, pressPicker } from './settings.mjs';
import { rest } from './pointer.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'running', {
  then: async (h) => ({ ...(await pressPicker(h, 'How many run at once')), ...(await rest(h, '.set-picker-row:nth-child(4)')) }),
});
