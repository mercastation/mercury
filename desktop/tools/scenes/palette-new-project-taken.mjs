// `palette-new-project` where the store already has a project of that name: nothing is made, and
// the line says so in the walk's own words (`whyNotMade`). The store's refusal is answered here
// as the app's bridge throws it.
import { plantedFolder } from './palette-new-project.mjs';
import { go as toInbox } from './inbox.mjs';
import { commandK } from './palette.mjs';
import { press } from './thread-option-picked.mjs';

const REFUSED = "Error invoking remote method 'zero:create-product': Error: product storefront already exists";

export async function go(h) {
  const folder = plantedFolder('storefront');
  await h.evaluate(`Object.assign(window.zero, {
    chooseFolder: async () => ({ path: ${JSON.stringify(folder)} }),
    createProduct: async () => { throw new Error(${JSON.stringify(REFUSED)}); },
  })`);
  await toInbox(h);
  await commandK(h);
  await press(h, 'Enter', 'Enter', 13);
  await h.wait(600);
  return { newProjectFolder: folder, newProjectRefused: REFUSED };
}
