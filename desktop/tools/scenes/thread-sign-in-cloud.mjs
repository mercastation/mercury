// `thread-sign-in` for a run on another machine, whose link comes back to localhost: the tick
// that saves the sign-in for every cloud thread, and the way in for a page that did not load.
import { signIn } from './thread-sign-in.mjs';

export const VERCEL = { service: 'Vercel', url: 'https://vercel.com/oauth/authorize?client_id=cl_1&redirect_uri=http%3A%2F%2Flocalhost%3A8976%2Fcallback', why: 'to deploy the preview' };

export const go = (h) => signIn(h, VERCEL, { cloud: true });
