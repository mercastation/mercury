// `thread-needs` on a Mac with no cloud set up (`cloudKeys` null): the card's own fallback, the
// line's block drawn as code, the need as JSON.
import { needs } from './thread-needs.mjs';

export const go = (h) => needs(h, { cloud: false });
