// `thread-sign-in` with the pointer resting on "Not now": the link's words in the text's own
// ink (`.need-link:hover`).
import { signIn } from './thread-sign-in.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await signIn(h);
  return { ...fed, ...(await rest(h, '.need-skip', 200)) };
};
