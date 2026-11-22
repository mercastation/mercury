// Home with the pointer resting on the rail's Demands button: the rail opened to the whole
// sidebar, its words and keys showing and the card moved over, as it stays while the pointer is
// on it. It opens 140 ms after a pointer moves on it and slides for 240 ms, and its words fade
// in for 140 ms after 80 (`onHover` in renderer/src/components/WorkspaceNavigation.tsx, the
// rail's rules in renderer/src/team/rooms/rooms.css), so the picture waits for all of that.
// Demands, because open it has no title (`title={collapsed ? 'Demands' : undefined}`): the
// system's tooltip never shows in a picture of the web window, and Rune draws its own in one.
import { rest } from './pointer.mjs';

export const go = (h) => rest(h, 'button.workspace-tab[data-tab="demands"]', 900);
