// ⌘K pressed on the Inbox: the command bar over the page, its box empty.
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { go as toInbox } from './inbox.mjs';

export const commandK = async (h) => {
  const key = { key: 'k', code: 'KeyK', windowsVirtualKeyCode: 75, modifiers: 4 };
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', ...key });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', ...key });
  await h.wait(400);
};

// The web's own table of the drawings at the head of a row (`ICONS` in
// renderer/src/palette-glyphs.ts), read from the checkout being photographed (the reference
// tool's first argument), so a row's drawing is named by the paths it was drawn with.
const drawings = async () => (await import(pathToFileURL(path.join(process.argv[2], 'renderer/src/palette-glyphs.ts')).href)).ICONS;

// The bar's rows as its page drew them, for the fixture: a heading, a command with the name of
// its drawing, its name, hint and key, or only the place of a thread, which the native window
// fills from its own search. It lists commands it cannot do yet, so a like-for-like picture
// draws these instead.
export const rowsOnScreen = async (h) => {
  const rows = await h.evaluate(`[...document.querySelectorAll('.palette-k .palette-list > *')].map((row) => {
    const text = (sel) => row.querySelector(sel)?.textContent ?? '';
    if (row.classList.contains('palette-section')) return { section: row.textContent };
    if (row.classList.contains('palette-thread')) return { thread: true };
    if (!row.classList.contains('palette-item')) return null;
    const paths = [...row.querySelectorAll('.palette-glyph path')].map((p) => p.getAttribute('d'));
    return { paths, label: text('.palette-label'), hint: text('.palette-hint'), key: text('.palette-key') };
  }).filter(Boolean)`);
  const icons = await drawings();
  const named = (paths) => Object.entries(icons).find(([, ds]) => ds.length === paths.length && ds.every((d, i) => d === paths[i]))?.[0] ?? '';
  return rows.map(({ paths, ...row }) => (paths ? { icon: named(paths), ...row } : row));
};

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
