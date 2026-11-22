// `inbox-progress-stuck` with the pointer resting on the first row's Try again: the text's ink on
// the strong wash (`.th-retry:hover`).
import { go as stuck } from './inbox-progress-stuck.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await stuck(h);
  return { ...fed, ...(await rest(h, '.th-retry', 200)) };
};
