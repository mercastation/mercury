// Home with words typed and a film over the 1 GB a message can send attached: the chip lit
// with no size in it, the line under the row saying why, and Send off.
//
// The film is invented and empty: a file of that length with nothing written in it, which the
// disk does not keep (`truncate`), planted beside the invented folder's other files.
import fs from 'node:fs';
import path from 'node:path';
import { plant } from '../pictures.mjs';
import { type, WORDS } from './home-typed.mjs';
import { attached, pickFiles } from './home-attached.mjs';

export async function go(h) {
  const planted = plant();
  const film = path.join(planted.dir, 'media', 'launch-film.mov');
  fs.writeFileSync(film, '');
  fs.truncateSync(film, Math.round(1.2 * 1024 ** 3));
  await type(h);
  await pickFiles(h, '.hm-start', [film]);
  return { composing: WORDS, attached: attached([film]) };
}
