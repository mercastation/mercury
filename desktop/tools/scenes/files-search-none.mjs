// Files' home with words its search finds nothing for: "No file matches".
import { openFiles, search } from './files.mjs';

export const go = (h) => openFiles(h, { then: (h) => search(h, 'zebra') });
