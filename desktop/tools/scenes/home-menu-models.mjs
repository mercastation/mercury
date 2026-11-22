// The model menu turned to its second page: every model, a column for each coding agent.
import { openMenu } from './home-menu-project.mjs';

// The row that turns the page is the one after the models used lately.
export async function allModels(h) {
  await h.evaluate(`(() => {
    const rows = [...document.querySelectorAll('.tc-model-menu [data-item]')];
    rows.find((r) => r.textContent.startsWith('All models')).click();
  })()`);
  await h.wait(500);
}

export async function go(h) {
  await openMenu(h, 4);
  await allModels(h);
  return { menu: 'models' };
}
