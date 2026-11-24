// A key the run says is in the Keychain (`FromHere`): "Use the one in your Keychain" with the
// item beside it, the facts, and the box under it as a second way ("Or paste it", a quiet Save).
// The key's words are the title, so they are not said again under it.
import { needs } from './thread-needs.mjs';

export const go = (h) => needs(h, {
  said: 'The daily report reads PostHog.\n\nAGENTBOX-NEEDS: key POSTHOG_PERSONAL_KEY | the PostHog personal API key the daily report reads | keychain posthog-read-key',
});
