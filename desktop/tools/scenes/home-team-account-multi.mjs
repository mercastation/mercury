// `home-team-account` with Multiplayer on: the menu has the person's status ("Set a status",
// nothing said yet) and Invite people too, the sidebar kept open (`account.mjs`).
import { dressed, openAccount } from './account.mjs';

export const dress = dressed({ multiplayer: true });
export const go = (h) => openAccount(h, { kept: true });
