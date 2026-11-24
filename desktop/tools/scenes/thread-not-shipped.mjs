// A thread whose ship was refused because git on this computer could not sign in to GitHub
// (`shared/github-push-sign-in.mts`). The app says so in a note of its own and opens the
// thread again; unfolded, the conversation ends on that note under "Not shipped", with the card
// that signs in (`GitHubSignInCard.tsx`): GitHub's own sign-in first, a pasted token where
// that cannot run.
import { noted } from './thread-app-note.mjs';

export const NO_SIGN_IN = 'It did not ship because this computer could not sign in to GitHub.';
export const REFUSED_SIGN_IN = 'It did not ship because GitHub refused the sign-in saved on this computer.';

export const unshipped = (h, { note = NO_SIGN_IN, unfold = true, bridge = null, then = null } = {}) => noted(h, note, { unfold, status: 'open', fromTab: 'Needs you', bridge, then });

export const go = (h) => unshipped(h);
