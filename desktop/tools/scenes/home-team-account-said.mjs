// `home-team-account-multi` with a status said, long enough to need its two lines and held two
// days, the pointer resting on who the person is (`account.mjs`).
import { dressed, openAccount, restOn } from './account.mjs';

const DAY = 86_400_000;

export const dress = dressed({ multiplayer: true, said: (now) => ({ text: 'In meetings with the design team about the spring launch all week', until: now + 2 * DAY }) });
export const go = (h) => openAccount(h, { kept: true, then: (h, step) => restOn(h, step, '.th-acct-who') });
