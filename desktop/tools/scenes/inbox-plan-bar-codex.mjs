// `inbox-plan-bar` where ChatGPT's Codex is on the computer, signed out, and Claude Code is not:
// the agent already here leads, and what it is short of is a sign-in (`planBar`).
import { NOTHING, SIGNED_OUT, withReadings } from './inbox-plan-bar.mjs';

export const go = (h) => withReadings(h, { claude: NOTHING, codex: SIGNED_OUT });
