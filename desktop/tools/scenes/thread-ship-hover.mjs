// `thread-ship-loose` with the pointer resting on "Commit it and ship": the row takes the wash
// and its words the text's own ink (`.opt-row:hover`).
import { go as loose } from './thread-ship-loose.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await loose(h);
  return { ...fed, ...(await rest(h, '.one-choices .opt-row', 200)) };
};
