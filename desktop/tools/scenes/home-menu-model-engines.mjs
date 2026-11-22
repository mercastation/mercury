// The model menu on a Mac that offers two coding agents: each model used lately says which
// agent runs it. The second agent's models are not in the snapshot: the page reads them with
// its settings (`zero:settings-read`), so this scene answers that in the page with an invented
// list and hands the same answer on for the fixture file.
import { openMenu } from './home-menu-project.mjs';

export const SETTINGS = {
  ok: true,
  workspace: {
    codexModels: [
      { id: 'gpt-6-astra', label: 'GPT-6 Astra', levels: ['low', 'medium', 'high', 'xhigh'], defaultLevel: 'medium' },
      { id: 'gpt-6-astra-mini', label: 'GPT-6 Astra Mini', levels: ['low', 'medium', 'high'], defaultLevel: 'low' },
    ],
    codexModelDefault: 'gpt-6-astra',
  },
};

export function dress(snapshot) {
  const engines = [
    { id: 'claude', label: 'Claude Code', word: 'Claude Code' },
    { id: 'codex', label: 'Codex', word: 'Codex' },
  ];
  return { ...snapshot, engines: { choices: engines, workspace: 'claude', byItem: {} } };
}

// The page loaded again, answered its settings from here on.
export async function withSettings(h) {
  await h.call('Page.addScriptToEvaluateOnNewDocument', {
    source: `window.zero.settingsRead = async () => (${JSON.stringify(SETTINGS)});`,
  });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
}

export async function go(h) {
  await withSettings(h);
  await openMenu(h, 4);
  return { menu: 'model', settings: SETTINGS };
}
