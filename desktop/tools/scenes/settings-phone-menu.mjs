// Settings, Phone, with the projects' menu open: Every project, None, and a tick for each
// (`PhoneProjectsPicker`, `settings.mjs`).
import { open } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'phone', { then: async (h) => {
  await h.evaluate(`document.querySelector('.set-picker-now[aria-label="Projects on your phone"]').click()`);
  await h.wait(400);
  return { picker: 'phone-projects' };
} });
