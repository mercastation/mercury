// Home with the bar that says agents cannot start until they have a plan to run on: it stands
// first on the page whatever the page is, but over a conversation (`inbox-plan-bar`).
import { NOTHING, withReadings } from './inbox-plan-bar.mjs';

export const go = (h) => withReadings(h, { claude: NOTHING, codex: NOTHING }, async () => {});
