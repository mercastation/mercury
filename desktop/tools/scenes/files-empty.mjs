// Files with no file and no project: "Nothing in Files yet.", and no New, as no project is one's own.
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { empty: true });
