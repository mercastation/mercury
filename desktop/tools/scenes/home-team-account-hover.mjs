// `home-team-account` with the pointer gone on from the row to rest on the menu's Settings: that
// row washed, its words and mark a step brighter, and the person's own row lit for the menu
// over it without the pointer (`account.mjs`).
import { dressed, openAccount, restOn } from './account.mjs';

export const dress = dressed();
export const go = (h) => openAccount(h, { then: (h, step) => restOn(h, step, '.th-acct-menu .th-acct-row:not(.th-acct-signout)') });
