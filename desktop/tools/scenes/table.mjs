// A sheet worked by the hands, beside the conversation (`TableView` in the pane): the invented
// list of leads (`LEADS` in thread-pane.mjs) opened from its mention, then pressed, typed into
// and walked with the keys as a person does it.
//
// Each press and key is made in the page, and written down in the native window's own script
// language at the same window points (`tableScript`), so the native window can be driven through
// the same steps and photographed after them: `tools/table.py` does that and measures the two.
// The page's saves are answered as written and kept (`wrote`), so what the web would have
// written is beside the picture, word for word.
import { open, LEADS } from './thread-pane.mjs';

export { LEADS };

const SHIPPED = `**The list is ready.**

Every lead we spoke to this month is in exports/leads.csv, with who owns it and when the next call is.`;

const DOM_KEYS = {
  Enter: { code: 'Enter', vk: 13 },
  Escape: { code: 'Escape', vk: 27 },
  Tab: { code: 'Tab', vk: 9 },
  Backspace: { code: 'Backspace', vk: 8 },
  Delete: { code: 'Delete', vk: 46 },
  ArrowDown: { code: 'ArrowDown', vk: 40 },
  ArrowUp: { code: 'ArrowUp', vk: 38 },
  ArrowLeft: { code: 'ArrowLeft', vk: 37 },
  ArrowRight: { code: 'ArrowRight', vk: 39 },
  Home: { code: 'Home', vk: 36 },
  End: { code: 'End', vk: 35 },
  F2: { code: 'F2', vk: 113 },
  ' ': { code: 'Space', vk: 32 },
};

/** The hands: each step in the page, and the same step in the native window's script. */
function hands(h) {
  let t = 0.5;
  const script = [];
  const next = (d = 0.35) => { const at = Math.round(t * 1000) / 1000; t += d; return at; };
  const mouse = async (type, x, y, extra = {}) => h.call('Input.dispatchMouseEvent', { type, x, y, ...extra });
  const self = {
    script,
    end: () => t,
    /** A press at `x`, `y`, with Shift or ⌘ held, or the right button, or twice. */
    async tap(x, y, { shift = false, cmd = false, right = false, twice = false } = {}) {
      const modifiers = (shift ? 8 : 0) | (cmd ? 4 : 0);
      const button = right ? 'right' : 'left';
      await mouse('mouseMoved', x, y);
      for (let n = 1; n <= (twice ? 2 : 1); n += 1) {
        await mouse('mousePressed', x, y, { button, clickCount: n, modifiers });
        await mouse('mouseReleased', x, y, { button, clickCount: n, modifiers });
      }
      await h.wait(250);
      const at = next();
      const hold = [shift && 'shift', cmd && 'cmd'].filter(Boolean).join('+');
      if (hold) script.push({ t: at - 0.02, hold });
      script.push(twice ? { t: at, type: 'double-click', x, y } : { t: at, type: 'tap', x, y, ...(right ? { button: 'secondary' } : {}) });
      if (hold) script.push({ t: at + 0.12, hold: '' });
      if (twice) next(0.2);
    },
    /** The pointer moved onto `x`, `y`, in two steps, as a hand moves. */
    async move(x, y) {
      await mouse('mouseMoved', x - 8, y - 4);
      await mouse('mouseMoved', x, y);
      await h.wait(250);
      const at = next();
      script.push({ t: at, type: 'move', x: x - 8, y: y - 4 }, { t: at + 0.03, type: 'move', x, y });
    },
    /** A key by its DOM name, with Shift, ⌘ or ⌥ held. */
    async key(name, { shift = false, cmd = false, alt = false } = {}) {
      const known = DOM_KEYS[name] ?? { code: `Key${name.toUpperCase()}`, vk: name.toUpperCase().charCodeAt(0) };
      const modifiers = (shift ? 8 : 0) | (cmd ? 4 : 0) | (alt ? 1 : 0);
      const key = { key: name, code: known.code, windowsVirtualKeyCode: known.vk, modifiers };
      // A character typed carries its text, as a keyboard's does.
      await h.call('Input.dispatchKeyEvent', { type: 'keyDown', ...key, ...(name.length === 1 && !cmd ? { text: name } : {}) });
      await h.call('Input.dispatchKeyEvent', { type: 'keyUp', ...key });
      await h.wait(200);
      script.push({ t: next(), type: 'key', name, ...(shift ? { shift } : {}), ...(cmd ? { cmd } : {}), ...(alt ? { alt } : {}) });
    },
    /** Words typed into what has the keys. */
    async type(text) {
      await h.call('Input.insertText', { text });
      await h.wait(250);
      script.push({ t: next(0.1 + text.length * 0.04), type: 'type', text, every: 0.035 });
      next(0.25);
    },
    /** A wheel turned over `x`, `y` by `dx`, `dy` points (the page's sense: positive goes right
     *  and down), then a second for the sheet to settle. */
    async wheel(x, y, dx, dy = 0) {
      await mouse('mouseMoved', x, y);
      await mouse('mouseWheel', x, y, { deltaX: dx, deltaY: dy });
      await h.wait(1000);
      const at = next(1.0);
      // Rune moves the content by a wheel's points, the other way round.
      script.push({ t: at - 0.03, type: 'move', x, y }, { t: at, type: 'wheel', x, y, dx: -dx, dy: -dy });
    },
    async wait(ms) { await h.wait(ms); },
    /** The middle of what `css` finds, the part of it inside the table's viewport. */
    async at(css) {
      const hit = await h.evaluate(`(() => {
        const el = document.querySelector(${JSON.stringify(css)});
        const box = el?.closest('.tbl-scroll');
        if (!el) return null;
        const r = el.getBoundingClientRect();
        const v = box ? box.getBoundingClientRect() : r;
        const left = Math.max(r.left, v.left), right = Math.min(r.right, v.right);
        const top = Math.max(r.top, v.top), bottom = Math.min(r.bottom, v.bottom);
        return [Math.round((left + right) / 2), Math.round((top + bottom) / 2)];
      })()`);
      if (!hit) throw new Error(`table scene: nothing at ${css}`);
      return hit;
    },
    /** A cell by its place in the view and its column, nearer its left than its middle. */
    async cell(r, c) {
      const hit = await h.evaluate(`(() => {
        const el = document.querySelector('.tbl-td[data-r="${r}"][data-c="${c}"]');
        const v = document.querySelector('.tbl-scroll')?.getBoundingClientRect();
        if (!el || !v) return null;
        const b = el.getBoundingClientRect();
        const left = Math.max(b.left, v.left);
        return [Math.round(left + Math.min(40, (Math.min(b.right, v.right) - left) / 2)), Math.round(b.top + b.height / 2)];
      })()`);
      if (!hit) throw new Error(`table scene: no cell ${r}, ${c}`);
      return hit;
    },
    rowNumber: (r) => self.at(`.tbl-td.tbl-rn[data-rn="${r}"]`),
    header: (c) => self.at(`.tbl-th[data-hc="${c}"]`),
    /** A row of an open menu, a tag list or a calendar, by its words. */
    async row(words) {
      const hit = await h.evaluate(`(() => {
        const all = [...document.querySelectorAll('.tbl-menu button, .tbl-pick button, .tbl-sheet button')];
        const el = all.find((b) => (b.textContent ?? '').trim() === ${JSON.stringify(words)}) ?? all.find((b) => (b.textContent ?? '').trim().startsWith(${JSON.stringify(words)}));
        if (!el) return null;
        const r = el.getBoundingClientRect();
        return [Math.round(r.left + Math.min(40, r.width / 2)), Math.round(r.top + r.height / 2)];
      })()`);
      if (!hit) throw new Error(`table scene: no row ${words}`);
      return hit;
    },
  };
  return self;
}

/** The sheet open beside the conversation, then `act(hands)`; what it returns goes into the fixture. */
export function sheet(h, act, { split = null } = {}) {
  return open(h, {
    pressed: 'exports/leads.csv',
    shipped: SHIPPED,
    split,
    then: async (h) => {
      await h.evaluate(`(() => {
        window.__WROTE__ = [];
        window.zero.writeDoc = async (p) => { window.__WROTE__.push(p); return { ok: true, mtime: Date.now() }; };
      })()`);
      await h.wait(300);
      const hand = hands(h);
      const more = (await act(hand)) ?? {};
      // What the page would have written, once its 400 ms have gone by.
      await h.wait(700);
      return { tableScript: hand.script, tableEnd: hand.end(), wrote: await h.evaluate('window.__WROTE__'), ...more };
    },
  });
}

export const go = (h) => sheet(h, async () => {});
