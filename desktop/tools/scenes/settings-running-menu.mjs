// Settings, Running, with the menu of "How many run at once" open: Automatic and every number
// (`Picker` in `components/Settings.tsx`, `settings.mjs`).
import { open, pressPicker } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'running', { then: (h) => pressPicker(h, 'How many run at once') });
