// Home with the pointer resting on the sidebar's Home: the sidebar opened whole, its words
// and keys showing and the card moved over, as it stays while the pointer is on it. It opens
// 140 ms after a pointer moves on it and slides for 240 ms, and its words fade in for 140 ms
// after 80 (`onHover` in renderer/src/components/WorkspaceNavigation.tsx, the rail's rules in
// renderer/src/team/rooms/rooms.css), so the picture waits for all of that. Home, for two
// reasons. This window's sidebar has it where the web window's does (it has no Demands, and
// Workspace's name stands where the web window's Demands is), so the pointer rests on the same
// row in both. And open it has no title (`title={collapsed ? 'Home' : undefined}`): the
// system's tooltip never shows in a picture of the web window, and Rune draws its own in one.
// The pointer comes onto the sidebar from the card beside it, the only way it opens
// (`restOnTheSidebar` in ./pointer.mjs).
import { restOnTheSidebar } from './pointer.mjs';

export const go = (h) => restOnTheSidebar(h, 'button.workspace-tab[data-tab="home"]', 900);
