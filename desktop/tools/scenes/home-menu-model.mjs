// Home with the box's model menu open on its first page: the models used lately, the way to
// the rest, and how hard the picked one thinks.
import { openMenu } from './home-menu-project.mjs';

export async function go(h) {
  await openMenu(h, 4);
  return { menu: 'model' };
}
