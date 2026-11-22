// Home in the moment between Send and the engine's answer: the box already empty, and the line
// in the corner saying where the thread is going. The page's send is left unanswered here, so
// the moment lasts long enough to photograph.
import { type, WORDS } from './home-typed.mjs';

export async function go(h) {
  await h.call('Page.addScriptToEvaluateOnNewDocument', { source: 'window.zero.compose = () => new Promise(() => {});' });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await type(h);
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, modifiers: 4 });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, modifiers: 4 });
  return { sending: WORDS };
}
