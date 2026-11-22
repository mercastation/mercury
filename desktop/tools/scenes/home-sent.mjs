// Home just after a thread was sent from its box: the box empty again, the new thread in its
// list, and the line in the corner that names it, says where it went, and offers O and Z.
//
// The harness's bridge refuses every write, so this scene answers the one a send makes
// (`zero:compose`) in the page, the way the engine does: with the row it wrote, which is in
// the snapshot from then on. Nothing leaves the page.
import { press } from './thread-option-picked.mjs';
import { type, WORDS } from './home-typed.mjs';

export async function go(h) {
  await h.call('Page.addScriptToEvaluateOnNewDocument', {
    source: `(() => {
      window.__SENT__ = [];
      window.zero.compose = async (p) => {
        const now = Date.now();
        const by = { ts: now, source: 'founder' };
        const made = {
          id: 'w-sent1', product: p.product, productName: window.__SNAP__.products.find((x) => x.slug === p.product)?.name ?? p.product,
          status: 'open', kind: p.kind ?? 'directive', title: p.title, body: p.body ?? '', labels: ['founder'], priority: p.priority ?? 0,
          epoch: 1, claim: null, createdAt: now, updatedAt: now, wrote: { title: by, body: by, status: by },
          ...(p.model ? { model: p.model } : {}), ...(p.engine ? { engine: p.engine } : {}), ...(p.effort ? { effort: p.effort } : {}),
        };
        window.__SENT__.push({ asked: p, made, at: now });
        window.__SNAP__ = { ...window.__SNAP__, items: [...window.__SNAP__.items, made] };
        return made;
      };
    })()`,
  });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await type(h);
  // ⌘Return, as the Send button prints it.
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, modifiers: 4 });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, modifiers: 4 });
  await h.wait(700);
  const [sent] = await h.evaluate('window.__SENT__');
  if (!sent) console.error('home-sent: the page sent nothing');
  // What the page asked the engine for, the row it was answered with, and when: the native
  // window is handed the same answer, and says the same line about it.
  return { snapshot: await h.evaluate('window.__SNAP__'), sent, typed: WORDS };
}
