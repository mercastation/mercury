// The cloud button while an account is being added: the square, with the dot.
import { CLOUDS, withCloud } from './inbox-cloud.mjs';

export const go = (h) => withCloud(h, CLOUDS.adding);
