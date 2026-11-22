// Photograph the real Mercury inbox (built renderer, invented README inbox) and
// dump every visible element's box and computed style, so a native port can be
// built from numbers and compared pixel by pixel.
//
//   arch -arm64 node desktop/tools/reference.mjs <mercury checkout with a built renderer/dist> <out dir> [dark|light] [width] [height] [scene]
//
// Writes, for the scene (home by default):
//   <scene>-real.png      the picture
//   <scene>-dump.json     each element's box, font, colours, borders, shadows and spacing, plus its SVG,
//                         how it answers the pointer (cursor, user-select, app region, transitions),
//                         and the pointer's shape at the middle of each (`pointers`)
//   <scene>-fixture.json  the snapshot and the clock the picture was taken with, for `mercury --fixture`,
//                         and whatever else the scene's `go` returns
//
// A scene is a file in tools/scenes/<name>.mjs exporting `go(h)`, which takes the page from Home
// to the screen to photograph (h is the harness: evaluate, call, wait). Home needs none. A scene
// may also export `dress(snapshot, now)`, which returns the invented inbox with more on it.
//
// Read-only: the page is fed an invented inbox through the same bridge the other shot scripts
// use, and no store or agent is touched. MERCURY_REAL=1 feeds it the person's real threads
// instead, read from the store and never written; MERCURY_SNAPSHOT=<file> feeds it that snapshot,
// and MERCURY_KEEPS=<file> what a real window keeps for itself.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const mercury = process.argv[2];
const outDir = process.argv[3];
const theme = process.argv[4] ?? 'dark';
const width = Number(process.argv[5] ?? 1280);
const height = Number(process.argv[6] ?? 760);
const scene = process.argv[7] ?? 'home';
fs.mkdirSync(outDir, { recursive: true });

const { openInbox, herSnapshot } = await import(path.join(mercury, 'scripts/lib/inbox-harness.mjs'));
const sceneFile = path.join(path.dirname(fileURLToPath(import.meta.url)), 'scenes', `${scene}.mjs`);
const sceneOwn = fs.existsSync(sceneFile) ? await import(pathToFileURL(sceneFile).href) : {};
const go = sceneOwn.go ?? null;

// A fixed clock, so the row ages read the same on every run.
const now = Date.now();
const min = 60_000;
const products = [
  { slug: 'storefront', name: 'Storefront', dir: '/fixtures/storefront', repoPath: '/Users/you/dev/storefront' },
  { slug: 'mobile', name: 'Mobile', dir: '/fixtures/mobile', repoPath: '/Users/you/dev/mobile' },
];
const rows = [
  ['storefront', 9, 'Checkout fails for some customers since this morning', 'Found it. Tax rounding broke on carts over $1,000. Fixed and tested. Merge it?'],
  ['mobile', 9, 'Sign in loops back to the start screen on Android', 'The token was saved after the redirect, not before. Fixed on both builds. Merge it?'],
  ['storefront', 5, 'Snooze is lost on restart', 'Snooze survives a restart now. Every test passes. Merge it?'],
  ['storefront', 5, 'Redesign the empty inbox', 'Three directions are drawn in the app. Nothing is built yet. Pick one.'],
  ['mobile', 5, 'J and K skip rows after an undo', 'The cursor kept its old index. Fixed and tested. Merge it?'],
  ['storefront', 5, 'The weekly usage total is off by one', 'The usage test passes now. I updated the expected total. Merge it?'],
  ['storefront', 5, 'Command K should search closed tasks', 'Closed tasks show under their own heading. Merge it?'],
  ['mobile', 2, 'Onboarding checklist copy', 'Two versions of the copy are written. Which one?'],
  ['storefront', 2, 'Write the release notes for today', 'Drafted from the nine merges since this morning. Ship it?'],
];
const items = rows.map(([product, priority, title, result], i) => ({
  id: `w-readme${i}`, product, productName: products.find((p) => p.slug === product).name,
  status: 'done', kind: 'directive', title, body: title, result, labels: ['founder'], priority,
  epoch: 2, claim: null,
  createdAt: now - (90 + i * 7) * min, updatedAt: now - (4 + i * 6) * min,
  wrote: {
    title: { ts: now - (90 + i * 7) * min, source: 'founder' },
    body: { ts: now - (90 + i * 7) * min, source: 'founder' },
    result: { ts: now - (4 + i * 6) * min, source: 'agent' },
    status: { ts: now - (4 + i * 6) * min, source: 'agent' },
  },
}));
const snapshot = {
  products, items, approvals: [],
  supervisor: {
    paused: false, pausedProducts: [], running: [], stalled: [], queued: [], scheduled: [],
    productOrder: ['storefront', 'mobile'], hiddenProducts: [], personalProducts: [], capacity: 0,
  },
  restartNeeded: null, config: {},
};

// Runs in the page: every element with a box on screen, with what it takes to redraw it.
const DUMP = `(() => {
  const keep = ['color','backgroundColor','backgroundImage','fontFamily','fontSize','fontWeight','fontStyle',
    'lineHeight','letterSpacing','textTransform','textAlign','whiteSpace','textOverflow','fontVariantNumeric',
    'fontFeatureSettings','fontVariationSettings','webkitFontSmoothing',
    'borderTopWidth','borderRightWidth','borderBottomWidth','borderLeftWidth',
    'borderTopColor','borderRightColor','borderBottomColor','borderLeftColor','borderTopStyle',
    'borderTopLeftRadius','borderTopRightRadius','borderBottomRightRadius','borderBottomLeftRadius',
    'boxShadow','opacity','paddingTop','paddingRight','paddingBottom','paddingLeft',
    'marginTop','marginRight','marginBottom','marginLeft','columnGap','rowGap',
    'display','flexDirection','alignItems','justifyContent','flexGrow','flexShrink','flexBasis',
    'position','overflowX','overflowY','backdropFilter','filter','mixBlendMode','transform','outlineWidth','outlineColor',
    'gridTemplateColumns','gridTemplateRows','zIndex','visibility',
    // How it answers the pointer and the hands: the pointer's shape over it, whether its words
    // can be selected, whether a press there moves the window, whether it takes the pointer at
    // all, and how a change of its look eases in.
    'cursor','userSelect','webkitUserSelect','appRegion','webkitAppRegion','pointerEvents',
    'transitionProperty','transitionDuration','transitionTimingFunction','transitionDelay','outlineStyle','outlineOffset',
    'textDecorationLine','textDecorationColor','textUnderlineOffset'];
  const out = [];
  const walk = (el, depth, parent) => {
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    if (cs.display === 'none') return;
    const id = out.length;
    const style = {};
    for (const k of keep) style[k] = cs[k];
    let own = '';
    for (const n of el.childNodes) if (n.nodeType === 3) own += n.textContent;
    const before = getComputedStyle(el, '::before'); const after = getComputedStyle(el, '::after');
    const pseudo = (p) => (p.content && p.content !== 'none' && p.content !== 'normal') ? {
      content: p.content, color: p.color, backgroundColor: p.backgroundColor, backgroundImage: p.backgroundImage,
      width: p.width, height: p.height, position: p.position, top: p.top, left: p.left, right: p.right, bottom: p.bottom,
      borderRadius: p.borderTopLeftRadius, opacity: p.opacity, boxShadow: p.boxShadow, maskImage: p.maskImage || p.webkitMaskImage,
      backgroundSize: p.backgroundSize, inset: p.inset, border: p.borderTopWidth + ' ' + p.borderTopColor,
    } : undefined;
    out.push({
      id, parent, depth, tag: el.tagName.toLowerCase(), cls: typeof el.className === 'string' ? el.className : (el.className?.baseVal ?? ''),
      rect: [r.x, r.y, r.width, r.height].map((v) => Math.round(v * 100) / 100),
      text: own.trim() || undefined, style, before: pseudo(before), after: pseudo(after),
      svg: el.tagName.toLowerCase() === 'svg' ? el.outerHTML : undefined,
      src: el.tagName.toLowerCase() === 'img' ? el.getAttribute('src') : undefined,
      attrs: { role: el.getAttribute('role') || undefined, aria: el.getAttribute('aria-label') || undefined, title: el.getAttribute('title') || undefined },
    });
    if (el.tagName.toLowerCase() === 'svg') return;
    for (const c of el.children) walk(c, depth + 1, id);
  };
  walk(document.body, 0, -1);
  const root = getComputedStyle(document.documentElement);
  const vars = {};
  for (const sheet of document.styleSheets) {
    let rules; try { rules = sheet.cssRules; } catch { continue; }
    for (const rule of rules) if (rule.style) for (const name of rule.style) if (name.startsWith('--')) vars[name] = root.getPropertyValue(name).trim();
  }
  const ta = document.querySelector('textarea.tc-text');
  const ph = ta ? getComputedStyle(ta, '::placeholder') : null;
  const at = (x, y) => document.elementsFromPoint(x, y).slice(0, 4).map((e) => {
    const r = e.getBoundingClientRect(); const c = getComputedStyle(e);
    return { tag: e.tagName.toLowerCase(), cls: String(e.className?.baseVal ?? e.className).slice(0, 60), rect: [r.x, r.y, r.width, r.height], color: c.color, opacity: c.opacity, html: e.outerHTML.slice(0, 700) };
  });
  return {
    placeholder: ta ? { text: ta.getAttribute('placeholder'), color: ph.color, opacity: ph.opacity, weight: ph.fontWeight } : null,
    probes: { back: at(135, 70), fwd: at(163, 70) },
    viewport: [innerWidth, innerHeight], dpr: devicePixelRatio,
    html: { cls: document.documentElement.className, data: { ...document.documentElement.dataset }, bg: root.backgroundColor, bgImage: root.backgroundImage },
    body: { bg: getComputedStyle(document.body).backgroundColor, bgImage: getComputedStyle(document.body).backgroundImage, bgSize: getComputedStyle(document.body).backgroundSize },
    fonts: [...document.fonts].filter((f) => f.status === 'loaded').map((f) => ({ family: f.family, weight: f.weight, style: f.style, range: f.unicodeRange.slice(0, 40) })),
    vars, nodes: out,
  };
})()`;

// Runs in the page: the pointer's shape at the middle of every element on screen, as the person
// would see it there. `cursor` is inherited, so what the topmost element under the point computes
// is what shows, except for `auto`, which Chromium draws as the I-beam over words a hand can
// select (`user-select` is not `none` on the nearest element that sets it, and the point is on
// one of the words' line boxes) and as the arrow everywhere else, a <button> included.
const POINTERS = (rest) => `(() => {
  const out = [];
  const seen = new Set();
  const selectable = (el) => {
    for (let e = el; e; e = e.parentElement) {
      const v = getComputedStyle(e).userSelect;
      if (v === 'none') return false;
      if (v !== 'auto') return true;
    }
    return true;
  };
  const overWords = (x, y) => {
    const at = document.caretRangeFromPoint ? document.caretRangeFromPoint(x, y) : null;
    if (!at || at.startContainer.nodeType !== 3) return null;
    const words = document.createRange();
    words.selectNodeContents(at.startContainer);
    for (const b of words.getClientRects()) if (x >= b.left && x <= b.right && y >= b.top && y <= b.bottom) return at.startContainer.parentElement;
    return null;
  };
  const at = (x, y) => {
    const hit = document.elementFromPoint(x, y);
    if (!hit) return null;
    let cursor = getComputedStyle(hit).cursor;
    const words = overWords(x, y);
    if (cursor === 'auto') cursor = hit.isContentEditable || (words && selectable(words)) ? 'text' : 'default';
    return { x, y, cursor, tag: hit.tagName.toLowerCase(), cls: String(hit.className?.baseVal ?? hit.className).slice(0, 60), label: (hit.getAttribute('aria-label') || hit.textContent || '').trim().slice(0, 40) };
  };
  for (const el of document.querySelectorAll('body *')) {
    const r = el.getBoundingClientRect();
    if (r.width < 2 || r.height < 2) continue;
    const x = Math.round((r.x + r.width / 2) * 2) / 2;
    const y = Math.round((r.y + r.height / 2) * 2) / 2;
    if (x < 1 || y < 1 || x > innerWidth - 1 || y > innerHeight - 1 || seen.has(x + ',' + y)) continue;
    seen.add(x + ',' + y);
    const shape = at(x, y);
    if (shape) out.push(shape);
  }
  // And where a scene left the pointer resting, first.
  const rest = ${JSON.stringify(rest ?? null)};
  const resting = rest && at(rest[0], rest[1]);
  return resting ? [{ ...resting, resting: true }, ...out] : out;
})()`;

// MERCURY_SNAPSHOT names a snapshot to feed as it is: the one `rows.mjs --door` leaves, say.
const shown = process.env.MERCURY_SNAPSHOT ? JSON.parse(fs.readFileSync(process.env.MERCURY_SNAPSHOT, 'utf8'))
  : process.env.MERCURY_REAL === '1' ? herSnapshot()
  // A scene may dress the invented inbox before the page opens: a team, a meeting, a reading of usage.
  : sceneOwn.dress ? sceneOwn.dress(snapshot, now) : snapshot;
const h = await openInbox({ dist: path.join(mercury, 'renderer/dist'), width, height, snapshot: shown });
try {
  await h.goto(h.origin);
  await h.evaluate(`localStorage.setItem('zero.firstRun.done', '1'); localStorage.setItem('zero.theme', ${JSON.stringify(theme)});`);
  // MERCURY_KEEPS names a file with what a real window keeps for itself (delivered through, unread
  // since, seen at), so this page decides its lists the way that window does.
  if (process.env.MERCURY_KEEPS) {
    const keeps = JSON.parse(fs.readFileSync(process.env.MERCURY_KEEPS, 'utf8'));
    await h.evaluate(`(() => {
      const k = ${JSON.stringify(keeps)};
      localStorage.setItem('zero.deliveredThrough', String(k.deliveredThrough ?? 0));
      localStorage.setItem('zero.unreadSince', String(k.unreadSince ?? 0));
      localStorage.setItem('zero.seenAt', JSON.stringify(k.seenAt ?? {}));
      if (k.cloudBrokenSeen) localStorage.setItem('zero.cloudBrokenSeen', JSON.stringify(k.cloudBrokenSeen));
    })()`);
  }
  await h.goto(h.origin);
  await h.evaluate(`(async () => { await document.fonts.ready; })()`);
  await h.wait(2000);
  const err = await h.evaluate('window.__ERR__ ?? null');
  if (err) console.error('page error:', err);
  // A scene may hand back what it fed the page besides the snapshot (a thread's words), or a
  // snapshot it changed; the fixture file carries it, so the native window is told the same.
  const fed = go ? (await go(h)) ?? {} : {};
  // A scene that leaves the pointer resting on something says how long the page waits after it
  // before the shutter (`settle`, in milliseconds): a hint plate arrives 700 ms after the pointer
  // stops (`HINT_WAIT` in renderer/src/hint-timing.ts), so a picture of what the pointer changes
  // by itself is taken before then.
  const settle = fed.settle ?? 800;
  delete fed.settle;
  if (go) await h.wait(settle);
  const png = path.join(outDir, `${scene}-real.png`);
  // The page's own clock at the shutter, so a native picture can tell the same ages.
  const at = await h.evaluate('Date.now()');
  // MERCURY_FILM=<frames>@<fps>[:x,y,w,h] films what moves on its own: every animation on the
  // page is stopped where it is and moved on a frame's time at each shutter, so each frame is
  // the page exactly that long after the one before, whatever the machine's speed. The frames
  // are `<scene>-film-0000.png` on, of the region in points when one is named.
  if (process.env.MERCURY_FILM) {
    const [count, rest] = process.env.MERCURY_FILM.split('@');
    const [fps, crop] = rest.split(':');
    const [x, y, w, hgt] = crop ? crop.split(',').map(Number) : [0, 0, width, height];
    await h.evaluate('(() => { window.__FILM__ = document.getAnimations().map((a) => [a, a.currentTime]); for (const [a] of window.__FILM__) a.pause(); })()');
    for (let k = 0; k < Number(count); k += 1) {
      await h.evaluate(`(() => { for (const [a, t] of window.__FILM__) a.currentTime = t + ${(k * 1000) / Number(fps)}; })()`);
      const { data } = await h.call('Page.captureScreenshot', { format: 'png', clip: { x, y, width: w, height: hgt, scale: 1 } });
      fs.writeFileSync(path.join(outDir, `${scene}-film-${String(k).padStart(4, '0')}.png`), Buffer.from(data, 'base64'));
    }
    await h.evaluate('(() => { for (const [a] of window.__FILM__) a.play(); })()');
  }
  await h.capture(png);
  const dump = await h.evaluate(DUMP);
  dump.pointers = await h.evaluate(POINTERS(fed.pointer));
  fs.writeFileSync(path.join(outDir, `${scene}-dump.json`), JSON.stringify(dump));
  // What the page keeps for itself and decides its lists by, besides the snapshot.
  const local = await h.evaluate(`(() => {
    const num = (k) => Number(localStorage.getItem(k) ?? 0) || 0;
    let seenAt = {}; try { seenAt = JSON.parse(localStorage.getItem('zero.seenAt') ?? '{}') ?? {}; } catch {}
    // And the cloud accounts that stopped working and were already looked at, which decide
    // whether the cloud button still asks for a sign-in.
    let cloudBrokenSeen = []; try { cloudBrokenSeen = JSON.parse(localStorage.getItem('zero.cloudBrokenSeen') ?? '[]') ?? []; } catch {}
    return { deliveredThrough: num('zero.deliveredThrough'), unreadSince: num('zero.unreadSince'), seenAt, cloudBrokenSeen };
  })()`);
  fs.writeFileSync(path.join(outDir, `${scene}-fixture.json`), JSON.stringify({ now: at, snapshot: shown, local, ...fed }));
  console.log(png, 'nodes', dump.nodes.length, 'fonts', JSON.stringify(dump.fonts.map((f) => `${f.family} ${f.weight}`)));
} finally {
  h.close();
}
