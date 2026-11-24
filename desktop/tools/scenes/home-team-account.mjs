// `home-team`'s Home with the sidebar opened by the pointer and the person's account menu open
// over their row, the pointer still on the row it pressed. Multiplayer is off, as `home-team`
// has it: the menu is who they are, Settings and Sign out (`account.mjs`).
import { dressed, openAccount } from './account.mjs';

export const dress = dressed();
export const go = (h) => openAccount(h);
