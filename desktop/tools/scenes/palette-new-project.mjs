// ⌘K on the Inbox, then Return on its first row, New project…: the Mac's folder window, a folder
// picked in it, and the project made of it, said in the line at the head of the page
// (`NewProject.tsx` and its call in `App.tsx`). A picture cannot show the system's window, so the
// page's bridge answers it with a folder this scene plants (`chooseFolder`), and answers the
// store's write (`createProduct`) as the store would. The fixture says where the folder is
// (`newProjectFolder`), for the native window's script to pick the same, and what the store
// answered (`zero:create-product`).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { go as toInbox } from './inbox.mjs';
import { commandK } from './palette.mjs';
import { press } from './thread-option-picked.mjs';

export const plantedFolder = (name) => {
  const root = process.env.MERCURY_PLANTED ?? path.join(os.tmpdir(), 'mercury-planted');
  const dir = path.join(root, name);
  fs.mkdirSync(dir, { recursive: true });
  return fs.realpathSync(dir);
};

export async function pickAndMake(h, folder, made) {
  await h.evaluate(`Object.assign(window.zero, {
    chooseFolder: async () => ({ path: ${JSON.stringify(folder)} }),
    createProduct: async () => (${JSON.stringify(made)}),
  })`);
  await toInbox(h);
  await commandK(h);
  await press(h, 'Enter', 'Enter', 13);
  await h.wait(600);
  return { newProjectFolder: folder, 'zero:create-product': made };
}

export const go = (h) => pickAndMake(h, plantedFolder('greet-js'), { slug: 'greet-js', name: 'Greet JS' });
