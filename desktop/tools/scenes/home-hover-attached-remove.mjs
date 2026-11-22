// Home with a picture pasted into its box and the pointer resting on the × in its corner.
import { go as paste } from './home-attached-picture.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  const box = await paste(h);
  return { ...box, ...(await rest(h, '.hm-start .attach-remove')) };
}
