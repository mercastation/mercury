// Home with the pointer resting on the rail's Search: the rail open, and Search washed as the
// places under it are, with no line under it (`.workspace-find` in
// renderer/src/workspace-navigation.css). The picture waits for the rail to open and its words to
// come in, as `home-rail-open` does.
import { rest } from './pointer.mjs';

export const go = (h) => rest(h, '.workspace-find button', 900);
