// Settings, Claude Code, with the permission mode's menu open at the end of its row: Claude
// Code's six modes, Bypass Permissions ticked (`settings.mjs`).
import { open, pressPicker } from './settings.mjs';

export { dress } from './settings.mjs';
export const go = (h) => open(h, 'claude', { then: async (h) => {
  // Scrolled to its foot first, where the permissions are; the fixture says how far.
  await h.evaluate(`document.querySelector('.set-pane').scrollTo({ top: 600 })`);
  await h.wait(300);
  const scrolled = await h.evaluate(`document.querySelector('.set-pane').scrollTop`);
  return { ...(await pressPicker(h, 'Permission mode', 0)), scrolled };
} });
