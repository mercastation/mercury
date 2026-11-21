// The cloud button while an account has stopped working: it says Sign in again.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';

export const go = (h) => withCloud(h, CLOUDS.again);
