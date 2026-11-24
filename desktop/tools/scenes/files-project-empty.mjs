// A project with nothing in Files yet: its page says so, and has no Recent.
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { projects: ['Notes'], project: 'Notes' });
