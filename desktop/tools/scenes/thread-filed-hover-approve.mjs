// `thread-filed` with the pointer resting on the proposal's New thread: the accent's own wash
// behind it (`.made-approve:hover`).
import { go as filed } from './thread-filed.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await filed(h);
  return { ...fed, ...(await rest(h, '.made-approve', 200)) };
};
