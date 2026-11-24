// `thread-sign-in` with the pointer resting on "I've signed in": the wash under a quiet button
// (`.need-btn:hover`).
import { signIn } from './thread-sign-in.mjs';
import { rest } from './pointer.mjs';

export const go = async (h) => {
  const fed = await signIn(h);
  return { ...fed, ...(await rest(h, '.need-acts .need-btn:not(.need-main)', 200)) };
};
