// The cloud button while three threads run in the cloud: it carries the number.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';

export const go = (h) => withCloud(h, CLOUDS.count);
