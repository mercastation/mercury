// `thread-sign-in` with the pointer resting on Open: the main button gives up a little of
// itself (`.need-main:hover`, opacity .88).
import { signIn } from './thread-sign-in.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await signIn(h);
  return { ...fed, ...(await rest(h, '.need-acts .need-main', 200)) };
};
