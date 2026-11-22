// Home with a file attached to its box and the pointer resting on the file's chip.
import { go as attach } from './home-attached.mjs';
import { rest } from './pointer.mjs';

export async function go(h) {
  const box = await attach(h);
  return { ...box, ...(await rest(h, '.hm-start .attach-chip')) };
}
