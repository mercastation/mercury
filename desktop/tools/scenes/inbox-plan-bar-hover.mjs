// `inbox-plan-bar` with the pointer resting on its quiet button: the hover ground and the text's
// ink (`.ps-bar button:hover`).
import { NOTHING, withReadings } from './inbox-plan-bar.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await withReadings(h, { claude: NOTHING, codex: NOTHING });
  return { ...fed, ...(await rest(h, '.ps-bar button.quiet', 200)) };
};
