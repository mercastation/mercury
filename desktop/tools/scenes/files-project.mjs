// The Files place on one project's page, its card pressed on All files (`filesProject`).
import { openFiles } from './files.mjs';

export const go = (h) => openFiles(h, { project: 'Storefront' });
