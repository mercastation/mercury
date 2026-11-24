// A GitHub sign-in: GitHub's own sign-in first ("Sign in with GitHub", "Paste a token
// instead"), no box, and no token page in the facts (`GitHubOwnSignIn`).
import { needs } from './thread-needs.mjs';

export const GITHUB = 'The failing job\'s log is behind a sign-in.\n\nAGENTBOX-NEEDS: sign-in github | to read the failure log';

export const go = (h) => needs(h, { said: GITHUB });
