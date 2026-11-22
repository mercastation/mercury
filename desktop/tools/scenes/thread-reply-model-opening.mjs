// `thread-reply-model` photographed the moment its menu opens, for a film of the menu coming
// in (`tc-in`): the menu is opened last and the shutter waits for nothing (`settle`), so with
// MERCURY_FILM_START=0 the film starts at the menu's first frame.
import { open } from './thread.mjs';

export async function go(h) {
  const did = await open(h, { unfold: false, waiting: true });
  await h.evaluate(`document.querySelector('.dock-pill .ts-model')?.click()`);
  return { ...did, replying: '', actions: 'model', settle: 0 };
}
