// `thread-needs-file` where the file is not on this computer: the engine's words, and only
// "Choose the file".
import { filed } from './thread-needs-file.mjs';

export const go = (h) => filed(h, { ok: false, error: '~/Downloads/signed-lease.pdf is not on this computer.' });
