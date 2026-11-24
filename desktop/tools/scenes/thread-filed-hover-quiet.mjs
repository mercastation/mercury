// `thread-filed` with the pointer resting on the finished thread's row while a proposal waits: the
// row takes the wash and its faint title goes dim (`.made-quiet:hover .made-title`).
import { go as filed } from './thread-filed.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await filed(h);
  return { ...fed, ...(await rest(h, '.made-quiet .made-title', 200)) };
};
