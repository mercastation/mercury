// `thread-line-idle` with Try again pressed: the line in the corner says what came back. The page
// has no app behind it, so it says Try again did not reach the app, as this window's picture
// does with no door (`tryAgainWords`).
import { line, tryAgain } from './thread-line.mjs';

export const go = (h) => line(h, 'idle', tryAgain);
