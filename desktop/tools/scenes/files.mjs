// The Files place (renderer/src/files/FilesPlace.tsx), on All files: a card for each project
// and a tile for every file its threads made, each file once at its newest version.
//
// The harness's bridge answers the snapshot and little else, and the Files place reads its own
// channels: the list (`zero:files-place`), one file opened big (`zero:files-place-open`), what
// the cloud keeps (`zero:files-cloud`), a file's words (`zero:read-doc`) and where a file is
// (`zero:open-artifact` asked to resolve). This scene answers them in the page with an invented
// list over the planted folder (`../pictures.mjs`), and hands the same list and views back for
// the fixture, where the native window reads them (`filesPlace`, `filesPlaceViews`). Nothing in
// it is anybody's real file.
//
// The page draws a file opened big from `<img>` addresses on the app's own scheme, which only
// the app answers, so those are answered here from the planted pictures as well.
import fs from 'node:fs';
import path from 'node:path';
import { plant, standIn } from '../pictures.mjs';

const MIN = 60_000;

// Files only these scenes name: a script, a longer note with sections, and a sheet's newer copy.
const EXTRA = [
  ['scripts/round-tax.ts', `// Rounds each tax line to the cent before the total is added up.
import { cents } from './money';

export interface TaxLine {
  label: string;
  amount: number;
}

/** Each line rounded on its own, so the receipt adds up to what was charged. */
export function roundTax(lines: TaxLine[]): TaxLine[] {
  return lines.map((line) => ({ ...line, amount: Math.round(line.amount * 100) / 100 }));
}

export function totalOf(lines: TaxLine[]): number {
  const sum = roundTax(lines).reduce((total, line) => total + line.amount, 0);
  return cents(sum);
}
`],
  ['notes/receipt-plan.md', `# The receipt, line by line

What the customer sees after paying, and what each line is made of. The totals are added up
after every line is rounded, so the receipt always matches the charge.

## What is on it

- The items, each with its own price
- The tax, one line per rate
- The delivery, when there is one
- The total, on a line of its own

## How it adds up

Each tax line is rounded to the cent first. Only then are the lines added, so a cart of
**$1,240.555** in tax shows $1,240.56 and charges the same.

## What is left

Two things are not decided yet: whether a refund shows on the same receipt, and how a gift
card is named on it.
`],
];

// The invented threads the files were sent in, from the invented inbox (`reference.mjs`).
const THREADS = {
  checkout: { kind: 'thread', product: 'storefront', id: 'w-readme0', title: 'Checkout fails for some customers since this morning' },
  empty: { kind: 'thread', product: 'storefront', id: 'w-readme3', title: 'Redesign the empty inbox' },
  notes: { kind: 'thread', product: 'storefront', id: 'w-readme8', title: 'Write the release notes for today' },
  signin: { kind: 'thread', product: 'mobile', id: 'w-readme1', title: 'Sign in loops back to the start screen on Android' },
};

// Each file: its versions by where they are planted (oldest first), its kind, its project and
// thread, how long ago its newest version came, and who made it ('' an agent, null a person).
const FILES = [
  { versions: ['designs/<thread>/total-line.png', 'designs/<thread>/total-line-v2.png'], kind: 'image', in: 'checkout', ago: 6, agent: '' },
  { versions: ['notes/rounding.md', 'notes/rounding-v2.md'], kind: 'markdown', in: 'checkout', ago: 9, agent: '' },
  { versions: ['notes/receipt-plan.md'], kind: 'markdown', in: 'checkout', ago: 14, agent: '' },
  { versions: ['scripts/round-tax.ts'], kind: 'code', in: 'checkout', ago: 21, agent: '' },
  { versions: ['exports/failed-orders.csv'], kind: 'table', in: 'checkout', ago: 33, agent: '' },
  { versions: ['designs/<thread>/checkout-wide.png'], kind: 'image', in: 'empty', ago: 48, agent: '' },
  { versions: ['designs/<thread>/receipt-small.png'], kind: 'image', in: 'empty', ago: 75, agent: '' },
  { versions: ['designs/<thread>/phone-tall.png'], kind: 'image', in: 'empty', ago: 130, agent: '' },
  { versions: ['pages/checkout.html'], kind: 'page', in: 'empty', ago: 190, agent: '' },
  { versions: ['reports/week-32.pdf'], kind: 'pdf', in: 'notes', ago: 260, agent: '' },
  { versions: ['shots/cart-a.png'], kind: 'image', in: 'signin', ago: 340, agent: null },
  { versions: ['shots/cart-b.png'], kind: 'image', in: 'signin', ago: 410, agent: '' },
];

const PROJECT = { storefront: 'Storefront', mobile: 'Mobile' };

// What the cloud keeps (`zero:files-cloud`): no paid plan, so Keep files in the cloud says Pro.
const CLOUD = { cap: null, bytes: 0, full: false, on: false, offAfterDays: 30, offBytes: 0 };

// What a team kept, for the scenes of the Files home that show folders (`kept`): Storefront's
// Pinned, Receipts and the Calls inside it, and a Brand kit nobody put anything in yet. Each
// file is filed by its name; one has a name a person gave it and one an icon a person picked,
// and three are older, so Recent has more than one day.
const FOLDERS = [
  { id: 'pin', project: 'Storefront', name: 'Pinned', parent: null },
  { id: 'rec', project: 'Storefront', name: 'Receipts', parent: null },
  { id: 'calls', project: 'Storefront', name: 'Calls', parent: 'rec' },
  { id: 'brand', project: 'Storefront', name: 'Brand kit', parent: null },
];
const KEPT = {
  'rounding-v2.md': { folder: 'pin' },
  'receipt-plan.md': { folder: 'rec' },
  'round-tax.ts': { folder: 'calls', title: 'Tax rounding script' },
  'failed-orders.csv': { folder: 'rec', icon: 'chart' },
  'total-line-v2.png': { folder: 'rec' },
  'checkout.html': { ago: 60 * 27 },
  'week-32.pdf': { ago: 60 * 30 },
  'cart-b.png': { ago: 60 * 24 * 4 },
};

/**
 * The invented list and each file's view over a planting, with the page's clock `now`. With
 * `kept`, Storefront has folders and files in them (`FOLDERS`, `KEPT`); with `empty`, the list
 * has no file at all; `projects` adds projects with nothing in them; with `heavy`, what each
 * file takes on the Mac is four hundred times its planted size, so the page of every file has a
 * total to say and its menu older versions to move.
 */
export function invent(planted, now, { kept = false, empty = false, projects = [], heavy = false } = {}) {
  for (const [rel, words] of EXTRA) {
    const file = path.join(planted.dir, rel);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, words);
    planted.files.push(file);
  }
  const at = (rel) => path.join(planted.dir, rel.replace('<thread>', planted.thread));
  const files = [];
  const views = {};
  for (const f of FILES) {
    const paths = f.versions.map(at);
    const sizes = paths.map((p) => fs.statSync(p).size);
    const newest = paths[paths.length - 1];
    const name = path.basename(newest);
    const sent = THREADS[f.in];
    const key = `${sent.product}/${name}`;
    const own = (kept && KEPT[name]) || {};
    const when = now - (own.ago ?? f.ago) * MIN;
    const agent = f.agent === null ? {} : { agent: f.agent };
    const onMac = sizes.reduce((a, b) => a + b, 0) * (heavy ? 400 : 1);
    const newestTakes = sizes[sizes.length - 1] * (heavy ? 400 : 1);
    if (empty) continue;
    files.push({
      key, project: PROJECT[sent.product], name, kind: f.kind, versions: paths.length, at: when, by: null, ...agent,
      open: 0, approvedBy: [], sentIn: [sent], folder: own.folder ?? null, opened: newest,
      ...(own.title ? { title: own.title } : {}), ...(own.icon ? { icon: own.icon } : {}),
      bytes: sizes[sizes.length - 1], onMac, older: onMac - newestTakes, movable: onMac, trashable: true,
      // A heavy list has one file that sits in a code folder, which Files counts but never moves.
      ...(heavy && name === 'receipt-small.png' ? { movable: 0, trashable: false } : {}),
    });
    // One line per version, at the path it opens from (`linesOf` in main/files-place.mts).
    views[key] = {
      key, project: PROJECT[sent.product], name, kind: f.kind, sentIn: [sent], editsTo: null,
      lines: paths.map((p, i) => ({ ts: when - (paths.length - 1 - i) * 7 * MIN, by: null, source: f.agent === null ? 'system' : 'agent', patch: { note: `[${path.basename(p)}](${p})` } })),
      marks: { notes: [], approvals: [], seen: {} },
    };
  }
  const folders = kept ? FOLDERS.map((f, i) => ({ ...f, at: now - (60 - i) * MIN })) : [];
  const named = empty ? projects : ['Storefront', 'Mobile', ...projects];
  return { list: { files, projects: named, folders, mine: named }, views };
}

/**
 * A script for the page: the Files place's channels answered from the invented list, a file's
 * words and place from the planted folder, and `<img>` addresses on the app's scheme drawn
 * from the planted pictures.
 */
export function filesBridge(planted, { list, views }) {
  const texts = {};
  const pictures = {};
  for (const file of planted.files) {
    if (/\.(png|jpe?g|gif|webp)$/i.test(file)) pictures[file] = fs.readFileSync(file).toString('base64');
    else texts[file] = fs.readFileSync(file, 'utf8');
  }
  return `(() => {
    const list = ${JSON.stringify(list)};
    const views = ${JSON.stringify(views)};
    const texts = ${JSON.stringify(texts)};
    const pictures = ${JSON.stringify(pictures)};
    const dir = ${JSON.stringify(planted.dir)};
    const where = (src) => {
      const p = String(src ?? '').replace(/^file:\\/\\//, '').split(/[?#]/)[0];
      const whole = p.startsWith('/') ? p : dir + '/' + p.replace(/^\\.\\//, '');
      return whole in texts || whole in pictures ? whole : null;
    };
    const inPicture = (url) => {
      const at = /^[a-z0-9-]+-img:\\/\\/file(\\/[^?#]*)/i.exec(String(url ?? ''));
      const bytes = at && pictures[decodeURIComponent(at[1])];
      return bytes ? 'data:image/png;base64,' + bytes : null;
    };
    const set = Element.prototype.setAttribute;
    Element.prototype.setAttribute = function (name, value) {
      if (this instanceof HTMLImageElement && name === 'src') { const drawn = inPicture(value); if (drawn) value = drawn; }
      return set.call(this, name, value);
    };
    const src = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
    Object.defineProperty(HTMLImageElement.prototype, 'src', { ...src, set(value) { src.set.call(this, inPicture(value) ?? value); } });
    const before = window.zero ?? {};
    window.zero = {
      ...before,
      filesPlace: async () => list,
      filesPlaceOpen: async ({ key } = {}) => views[key] ?? null,
      filesCloud: async () => (${JSON.stringify(CLOUD)}),
      fileLine: async () => [],
      fileElsewhere: async () => [],
      readDoc: async ({ src } = {}) => { const at = where(src); return at && at in texts ? { ok: true, text: texts[at], mtime: 0 } : { ok: false, error: String(src) + ' could not be found.' }; },
      openArtifact: async ({ src, mode } = {}) => { const at = where(src); return mode === 'resolve' ? (at ? { ok: true, opened: at } : { ok: false, error: String(src) + ' could not be found.' }) : { ok: true }; },
    };
  })()`;
}

/**
 * The Files place: its home on All files, or, with `project`, that project's home, its tile
 * pressed on All files. `kept`, `empty` and `projects` shape the invented list (`invent`).
 * `folder` then presses a section's name, which opens that folder's page ("Not in a folder"
 * for the files in none); `everything` presses "Every file, with sizes and the Trash". `then`
 * is what the scene does on it before the picture (a tile pressed, a menu opened).
 *
 * The fixture says the same for the native window: `filesProject`, `filesFolder` (the
 * folder's id, or `not-in-a-folder`) and `filesEverything`.
 */
export async function openFiles(h, { project = null, kept = false, empty = false, projects = [], heavy = false, folder = null, everything = false, then = null } = {}) {
  const snapshot = await h.evaluate('window.__SNAP__');
  const planted = plant({ thread: 'w-readme0' });
  const now = await h.evaluate('Date.now()');
  // MERCURY_FILES names a list the running app answered (`zero:files-place`, read through its
  // door into a file outside the repository), to hold the native page's tiles to the web
  // window's on a person's own files. Its files are not planted, so their faces stay blank.
  const invented = process.env.MERCURY_FILES ? { list: JSON.parse(fs.readFileSync(process.env.MERCURY_FILES, 'utf8')), views: {} } : invent(planted, now, { kept, empty, projects, heavy });
  await h.call('Page.addScriptToEvaluateOnNewDocument', { source: standIn(planted) });
  await h.call('Page.addScriptToEvaluateOnNewDocument', { source: filesBridge(planted, invented) });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  await h.evaluate(`document.querySelector('button.workspace-tab[data-tab="files"]')?.click()`);
  await h.wait(1500);
  if (project) {
    await h.evaluate(`[...document.querySelectorAll('.fh-open')].find((b) => b.getAttribute('aria-label') === ${JSON.stringify(`Open ${project}`)})?.click()`);
    await h.wait(900);
  }
  const fixture = { snapshot, filesPlace: invented.list, filesPlaceViews: invented.views, filesCloud: CLOUD, ...(project ? { filesProject: project } : {}) };
  if (folder) {
    const name = folder === 'not-in-a-folder' ? 'Not in a folder' : invented.list.folders.find((f) => f.id === folder)?.name;
    await h.evaluate(`[...document.querySelectorAll('.fh-link')].find((b) => b.textContent.trim() === ${JSON.stringify(name)})?.click()`);
    await h.wait(900);
    fixture.filesFolder = folder;
  }
  if (everything) {
    await h.evaluate(`document.querySelector('.fh-every')?.click()`);
    await h.wait(900);
    fixture.filesEverything = true;
  }
  const did = then ? (await then(h, invented.list)) ?? {} : {};
  if (then) await h.wait(900);
  return { ...fixture, ...did };
}

/**
 * Presses the file called `name` where a person finds it on Files' home: its row under Recent,
 * which lists the ten files that changed last, newest first. Says so for the native window
 * (`fileOpen`).
 */
export async function openTile(h, name) {
  await h.evaluate(`(async () => {
    const list = await window.zero.filesPlace();
    const order = [...list.files].sort((a, b) => b.at - a.at);
    document.querySelectorAll('.fh-rrow')[order.findIndex((f) => f.name === ${JSON.stringify(name)})]?.click();
  })()`);
  await h.wait(1200);
  return { fileOpen: name };
}

/** Types `words` into the home's search box, as a person does after `/`. */
export async function search(h, words) {
  await h.evaluate(`(() => {
    const box = document.querySelector('.fh-find input');
    box.focus();
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    set.call(box, ${JSON.stringify(words)});
    box.dispatchEvent(new Event('input', { bubbles: true }));
  })()`);
  await h.wait(600);
  return { filesQuery: words };
}

/** A press on the element `selector` finds whose words or label are `label` (or the first). */
export async function press(h, selector, label = null) {
  await h.evaluate(`(() => {
    const all = [...document.querySelectorAll(${JSON.stringify(selector)})];
    const hit = ${JSON.stringify(label)} === null ? all[0] : all.find((b) => b.textContent.trim() === ${JSON.stringify(label)} || b.getAttribute('aria-label') === ${JSON.stringify(label)});
    hit?.click();
  })()`);
  await h.wait(500);
}

export const go = (h) => openFiles(h);
