// The reply box with words in it and a film over the 1 GB a message can send picked with its
// paper clip: the chip lit with no size in it and the line under the row saying why. Unlike the
// box a thread is started in, the reply box leaves Send lit; pressing it says the file cannot go.
//
// The film is the same invented, empty one `home-attached-too-big` plants.
import fs from 'node:fs';
import path from 'node:path';
import { open } from './thread.mjs';
import { plant } from '../pictures.mjs';
import { attached, pickFiles } from './home-attached.mjs';
import { replyCarrying } from './thread-reply-files.mjs';

export const go = (h) => open(h, {
  unfold: false,
  then: replyCarrying(async (h) => {
    const planted = plant();
    const film = path.join(planted.dir, 'media', 'launch-film.mov');
    fs.writeFileSync(film, '');
    fs.truncateSync(film, Math.round(1.2 * 1024 ** 3));
    await pickFiles(h, '.dock-card', [film]);
    return { replyFiles: attached([film]) };
  }),
});
