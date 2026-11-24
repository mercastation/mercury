// `home-team-account-rooms`, and Go invisible pressed: its card takes the menu's place, saying
// what happens and for how long (`GoInvisible`, `account.mjs`). The pointer is left on the
// card's first end, which is washed.
import { dressed, openAccount, pressOn, restOn, withRooms } from './account.mjs';

export const dress = withRooms(dressed());
export const go = (h) => openAccount(h, {
  kept: true,
  then: async (h, step) => {
    await pressOn(h, step, '.th-acct-menu .th-acct-row', 200);
    return restOn(h, step, '.rm-ipop .when');
  },
});
