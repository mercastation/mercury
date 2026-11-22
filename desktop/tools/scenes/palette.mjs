// ⌘K pressed on the Inbox: the command bar over the page, its box empty.
import { go as toInbox } from './inbox.mjs';

export const commandK = async (h) => {
  const key = { key: 'k', code: 'KeyK', windowsVirtualKeyCode: 75, modifiers: 4 };
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', ...key });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', ...key });
  await h.wait(400);
};

// The bar's rows as its page drew them, for the fixture: a heading, a command with its mark,
// name, hint and key, or only the place of a thread, which the native window fills from its own
// search. It lists commands it cannot do yet, so a like-for-like picture draws these instead.
export const rowsOnScreen = (h) => h.evaluate(`[...document.querySelectorAll('.palette-k .palette-list > *')].map((row) => {
  const text = (sel) => row.querySelector(sel)?.textContent ?? '';
  if (row.classList.contains('palette-section')) return { section: row.textContent };
  if (row.classList.contains('palette-thread')) return { thread: true };
  if (!row.classList.contains('palette-item')) return null;
  return { glyph: text('.palette-glyph'), label: text('.palette-label'), hint: text('.palette-hint'), key: text('.palette-key') };
}).filter(Boolean)`);

export async function go(h) {
  await toInbox(h);
  await commandK(h);
  return { palette: '', paletteRows: await rowsOnScreen(h) };
}
