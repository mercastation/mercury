// `inbox-progress-stuck` with the pointer resting on the Status word of the first row the cloud
// says why of: the card under it (`.th-cell-state.has-card:hover .th-why-card`).
import { go as stuck } from './inbox-progress-stuck.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await stuck(h);
  return { ...fed, ...(await rest(h, '.th-cell-state.has-card', 200)) };
};
