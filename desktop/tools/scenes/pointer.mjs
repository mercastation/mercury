// The pointer resting on one thing on the page, as a hand leaves it there: moved onto it in two
// steps, so the page sees a pointer that moved (the rail opens only for one, `pointerMoved` in
// renderer/src/sidebar-shrinks.ts, and the Inbox's rows take the keys only from one), then held
// still. `at` is a selector, whose middle the pointer goes to, or a point in the window.
//
// The fixture says where it rests (`pointer`), and the native window is moved there by a script
// (`--script`, a `move`), so both pictures are of the same pointer. `ms` is how long it rests
// before the shutter: under the 700 ms a hint plate waits for, the picture is of what the pointer
// changes by itself; past it, of the plate too.
export async function rest(h, at, ms = 200) {
  const [x, y] = typeof at === 'string' ? await middleOf(h, at) : at;
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x: x - 8, y: y - 4 });
  await h.wait(30);
  await h.call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
  // The page is photographed this long after the pointer stopped; `settle` is what the
  // reference tool waits after the scene, before the shutter.
  return { pointer: [x, y], rested: ms, settle: ms };
}

/** The middle of the first element `selector` finds whose middle is in the window, in window points. */
export async function middleOf(h, selector) {
  const box = await h.evaluate(`(() => {
    for (const el of document.querySelectorAll(${JSON.stringify(selector)})) {
      const r = el.getBoundingClientRect();
      const [x, y] = [r.x + r.width / 2, r.y + r.height / 2];
      if (r.width > 0 && x >= 0 && y >= 0 && x < innerWidth && y < innerHeight) return [x, y];
    }
    return null;
  })()`);
  if (!box) throw new Error(`nothing on the page answers ${selector}`);
  return box.map((v) => Math.round(v * 2) / 2);
}
