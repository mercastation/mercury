// A film and a sound named in an answer. The web window plays each where it is named; the
// native window names them and opens them with the system, so this scene is what
// `desktop/README.md` counts as a difference, not a match.
import { open } from './thread.mjs';

const SHIPPED = () => `**A film of the fix, with its sound.**

The film is media/checkout-demo.mp4 and it runs for a minute.

Its sound alone:

media/voice-over.mp3`;

export const go = (h) => open(h, { unfold: false, shipped: SHIPPED, files: {} });
