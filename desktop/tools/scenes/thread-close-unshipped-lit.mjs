// `thread-close-unshipped` with the down arrow pressed twice: the second answer, "Close it
// anyway", is lit and says what Enter does.
import { unshipped } from './thread-close-unshipped.mjs';

export const go = (h) => unshipped(h, { keys: ['ArrowDown', 'ArrowDown'] });
