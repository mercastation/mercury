// 2026-11-20, the owner after signing in: "quando vou abrir a thread não está
// mostrando os modelos do antigravity". The app reads agy's model list once,
// at launch, off the window's thread, and agy answers over the network (1.5 s
// warm, measured). The screen had already read the lists and got none, and
// nothing read them again. Two things were missing: main telling the screen
// when the list lands, and the pickers redrawing from it, since the list is a
// module value (renderer/src/models.ts) and their memos never named it.
import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import { PUSH_CHANNELS } from '../shared/bridge-map.mts';
import { antigravityModelRows, setAntigravityModels } from '../renderer/src/models.ts';
import { allModels } from '../renderer/src/threads/composer-rules.ts';

const read = (p: string) => fs.readFileSync(new URL(`../${p}`, import.meta.url), 'utf8');

describe('agy’s model list, landing after the screen first asked', () => {
  it('is announced by main once agy has answered', () => {
    const ipc = read('main/ipc.mts');
    const body = ipc.slice(ipc.indexOf('const readAntigravity = async'), ipc.indexOf("if (config.antigravityBin) void readAntigravity();"));
    expect(body).toContain("window.webContents.send('zero:models-changed')");
  });

  it('reaches the screen over its own channel, which the screen listens to', () => {
    expect(PUSH_CHANNELS.onModelsChanged).toBe('zero:models-changed');
    expect(read('preload.cjs')).toMatch(/onModelsChanged: \(fn\) => \{[\s\S]{0,120}ipcRenderer\.on\('zero:models-changed'/);
    expect(read('renderer/src/App.tsx')).toMatch(/api\.onModelsChanged\(\(\)\s*=>\s*void readModels\(\)\)/);
  });

  it('is a new list each time, so a picker keyed on it redraws, and both pickers are', () => {
    setAntigravityModels([]);
    const before = antigravityModelRows();
    setAntigravityModels([{ id: 'gemini-4-neon', label: 'Gemini 4 Neon', levels: ['high'] }]);
    expect(antigravityModelRows()).not.toBe(before);
    expect(allModels({ codexOffered: false, antigravityOffered: true }).antigravity).toEqual([{ engine: 'antigravity', model: 'gemini-4-neon' }]);
    for (const file of ['renderer/src/threads/ThreadComposer.tsx', 'renderer/src/components/Focus.tsx']) {
      const src = read(file);
      expect(src).toMatch(/const agyRows = antigravityModelRows\(\);/);
      // Anywhere in its keys: Agentbox's further harnesses are keyed beside it (2026-11-23 sync).
      expect(src).toMatch(/allModels\(\{[^}]*\}\), \[[^\]]*\bagyRows\b[^\]]*\]/);
    }
    setAntigravityModels([]);
  });
});
