// `home-team-account`, and then the pointer gone off the sidebar onto the page: the sidebar
// folds and the menu stays open over its foot, 240 points wide (`.workspace-collapsed
// .th-acct-menu`, `account.mjs`).
import { dressed, openAccount } from './account.mjs';

export const dress = dressed();
export const go = (h) => openAccount(h, {
  rested: 900,
  then: async (h, step) => {
    await step('move', 700, 600);
    await step('move', 720, 610);
    return [720, 610];
  },
});
