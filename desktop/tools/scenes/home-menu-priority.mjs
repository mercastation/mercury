// Home with the box's priority menu open: Agent Decides, which a new thread starts on, and the
// four levels.
import { openMenu } from './home-menu-project.mjs';

export async function go(h) {
  await openMenu(h, 1);
  return { menu: 'priority' };
}
