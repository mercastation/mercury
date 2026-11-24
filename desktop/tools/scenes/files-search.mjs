// Files' home with words in its search box: every file a word of whose name starts with them,
// as tiles under Matches and their count (`filesQuery`).
import { openFiles, search } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true, then: (h) => search(h, 'receipt') });
