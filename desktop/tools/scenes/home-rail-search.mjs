// Home with the pointer resting on the sidebar's Search: the sidebar open, and Search a field
// on the card's black inside a rule, its word and keys a step brighter under the pointer
// (`.workspace-find button` in renderer/src/sidebar.css). The picture waits for the sidebar to
// open and its words to come in, as `home-rail-open` does. Search has a title open or folded,
// which a headless picture of this window draws (Rune's own stand-in) and the web's does not.
import { rest } from './pointer.mjs';

export const go = (h) => rest(h, '.workspace-find button', 900);
