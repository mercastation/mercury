// `thread-filed-many` with the pointer resting on Start all: the strong wash behind it and its
// words in the text's ink (`.made-all-start:hover`).
import { go as many } from './thread-filed-many.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await many(h);
  return { ...fed, ...(await rest(h, '.made-all-start', 200)) };
};
