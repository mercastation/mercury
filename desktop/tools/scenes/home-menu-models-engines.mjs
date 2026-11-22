// Every model on a Mac that offers two coding agents: a column each, under the agent's name.
import { openMenu } from './home-menu-project.mjs';
import { allModels } from './home-menu-models.mjs';
import { SETTINGS, withSettings } from './home-menu-model-engines.mjs';

export { dress } from './home-menu-model-engines.mjs';

export async function go(h) {
  await withSettings(h);
  await openMenu(h, 4);
  await allModels(h);
  return { menu: 'models', settings: SETTINGS };
}
