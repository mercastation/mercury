// The cloud button on a Mac offered the cloud and never set up: it says Cloud.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';

export const go = (h) => withCloud(h, CLOUDS.named);
