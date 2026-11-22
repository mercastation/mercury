// A conversation with ten screenshots in it, each 2880 by 1800 pixels, as a Retina Mac takes
// them: what `desktop/README.md` measures the window's memory on. The earlier answer holds
// five and the newest five, so the whole conversation, unfolded, draws all ten.
import { open } from './thread.mjs';

const screens = (from, to) => Array.from({ length: to - from + 1 }, (_, n) => `![Screen ${from + n}](screens/screen-${String(from + n).padStart(2, '0')}.png)`).join('\n\n');

const FOUND = () => `**The first five screens of the new checkout.**\n\n${screens(1, 5)}`;
const SHIPPED = () => `**And the other five.**\n\n${screens(6, 10)}`;

export const go = (h) => open(h, { unfold: true, found: FOUND, shipped: SHIPPED, files: { large: true } });
