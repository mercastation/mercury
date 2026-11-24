// Files' home on All files with what the team kept: each project's tile says how many files are
// in a folder and how many in none, and Recent spans more than one day.
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { kept: true });
