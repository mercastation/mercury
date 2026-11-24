// `thread-needs` with two sign-ins: the facts are said on the first, and the second says only
// where its token is got, "Kept the same way." (`showFacts`).
import { needs } from './thread-needs.mjs';

export const go = (h) => needs(h, {
  said: [
    'Both are needed to read why the checkout fails.',
    '',
    'AGENTBOX-NEEDS: sign-in sentry | to read the checkout errors',
    'AGENTBOX-NEEDS: sign-in vercel | to read the deploy log',
  ].join('\n'),
});
