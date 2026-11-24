// `thread-sign-in` for a sign-in that gives the person a code to type on the service's page
// (`gh auth login`): the service's own logo in the mark, since the link is the service's own,
// and the code in its box with Copy.
import { signIn } from './thread-sign-in.mjs';

export const GITHUB = { service: 'GitHub', url: 'https://github.com/login/device', code: 'A1B2-C3D4', why: 'to push the branch from this Mac' };

export const go = (h) => signIn(h, GITHUB);
