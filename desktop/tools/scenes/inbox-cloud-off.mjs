// The cloud button with the cloud off: the plain square.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';

export const go = (h) => withCloud(h, CLOUDS.off);
