// `home-team-account` with Rooms (alpha) on: the menu has Go invisible under who the person is,
// the sidebar kept open (`account.mjs`).
import { dressed, openAccount, withRooms } from './account.mjs';

export const dress = withRooms(dressed());
export const go = (h) => openAccount(h, { kept: true });
