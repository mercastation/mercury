// `inbox-plan-bar` where both agents are on the computer and both signed out: two sign-ins, and
// the sentence names both plans (`barSignedOutBoth`).
import { SIGNED_OUT, withReadings } from './inbox-plan-bar.mjs';

export const go = (h) => withReadings(h, { claude: SIGNED_OUT, codex: SIGNED_OUT });
