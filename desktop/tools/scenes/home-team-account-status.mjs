// `home-team-account-multi`, and "Set a status" pressed: the box a status is written in takes
// the menu's place, the caret in the sentence (`StatusComposer`, `account.mjs`). With `typed`
// the person has written in it; with `until` they have opened the menu of when it ends.
import { dressed, openAccount, pressOn } from './account.mjs';

export const dress = dressed({ multiplayer: true });

export const writing = ({ typed = null, until = false } = {}) => (h) => openAccount(h, {
  kept: true,
  then: async (h, step) => {
    let at = await pressOn(h, step, '.th-acct-status', 200);
    if (typed) await step('type', typed, null, 120);
    if (until) at = await pressOn(h, step, '.tm-status-until .compose-word', 200);
    return at;
  },
});

export const go = writing();
