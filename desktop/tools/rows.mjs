// Hold the native lists against the rows the web window drew: same count, same order, every
// field equal. The web side is read out of `reference.mjs`'s dumps, the native side is
// `mercury --fixture <the dump's fixture> --rows`, so both worked from the same snapshot, the
// same clock and the same localStorage facts.
//
//   node tools/rows.mjs <dir>                             the dumps already in <dir>
//   node tools/rows.mjs <dir> --shoot <mercury checkout>  photograph the four scenes first (reference.mjs's invented inbox)
//   node tools/rows.mjs <dir> --shoot <checkout> --cases  ... of the invented threads in tools/rows.cases.mjs
//   node tools/rows.mjs <dir> --shoot <checkout> --real   ... of the person's real threads, read and never written
//   node tools/rows.mjs <dir> --shoot <checkout> --real --stir   ... with a busy fleet invented around them
//
// Four scenes: home (Needs you and In progress as Home draws them), inbox, inbox-progress and
// inbox-done (the Inbox page's three tabs). A scene with no dump in <dir> is skipped.
//
// Differences are printed by scene, row and field. A thread's own words (title, project,
// aside) are printed as lengths only, so a real title never reaches a terminal or a log;
// `--show` prints them, for invented threads. Run from `desktop/`, after `cargo build`.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const flag = (name) => args.includes(name);
const after = (name) => (args.includes(name) ? args[args.indexOf(name) + 1] : null);
const dir = args[0];
if (!dir || dir.startsWith('--')) {
  console.error('usage: node tools/rows.mjs <dir> [--shoot <mercury checkout> [--cases | --real [--stir]]] [--show]');
  process.exit(2);
}
const show = flag('--show');
const SCENES = ['home', 'inbox', 'inbox-progress', 'inbox-done'];

/* ------------------------------------------------------------ photographing */

// The person's real threads, in the shape the harness's own `herSnapshot` gives. That one
// reads a store layout this Mac no longer has (a ledger beside each project's files); the
// ledgers are in the store's `projects` folder, named for the project's whole path.
async function realSnapshot(mercury) {
  const { foldWorkItems } = await import(pathToFileURL(path.join(mercury, 'shared/work-items.mts')).href);
  const root = after('--store') ?? process.env.MERCURY_HOME ?? path.join(os.homedir(), 'Mercury');
  const products = [];
  const items = [];
  for (const account of fs.readdirSync(path.join(root, 'accounts'))) {
    const accountDir = path.join(root, 'accounts', account);
    if (!fs.statSync(accountDir).isDirectory()) continue;
    for (const slug of fs.readdirSync(accountDir)) {
      const ledger = path.join(root, 'projects', path.join(accountDir, slug).replace(/[^A-Za-z0-9]/g, '-'), 'work-items.jsonl');
      if (!fs.existsSync(ledger)) continue;
      let name = slug;
      try { name = JSON.parse(fs.readFileSync(path.join(accountDir, slug, 'project.json'), 'utf8')).name ?? slug; } catch { /* keep the slug */ }
      products.push({ slug, name });
      const lines = fs.readFileSync(ledger, 'utf8').split('\n').filter(Boolean).map((l) => { try { return JSON.parse(l); } catch { return null; } }).filter(Boolean);
      for (const item of foldWorkItems(lines).values()) items.push({ ...item, product: slug, productName: name });
    }
  }
  return {
    products, items, approvals: [],
    supervisor: { paused: false, pausedProducts: [], running: [], stalled: [], queued: [], scheduled: [], productOrder: [], hiddenProducts: [], personalProducts: [], capacity: 0 },
    restartNeeded: null, config: {},
  };
}

// The harness's snapshot has a fleet at rest: nothing running, queued or stopped, no order of
// projects. This gives the supervisor something to say about the real threads, by their place
// in the list and nothing else, so the real words meet every status the page can draw.
function stir(snapshot) {
  const at = (n) => snapshot.items.filter((_, k) => k % 11 === n);
  const ids = (n) => at(n).map((i) => i.id);
  const now = Date.now();
  snapshot.approvals = at(6).map((i, k) => ({ id: `a-${k}`, at: now, product: i.product, item: i.id, tool: 'Bash', input: { command: 'ls', description: 'List the folder' } }));
  snapshot.engines = { workspace: 'claude', byItem: Object.fromEntries(ids(9).map((id, k) => [id, k % 2 ? 'codex' : 'antigravity'])) };
  Object.assign(snapshot.supervisor, {
    productOrder: snapshot.products.map((p) => p.slug).reverse(),
    running: [...at(0), ...at(6)].map((i, k) => ({ itemId: i.id, product: i.product, startedAt: now - 5 * 60_000, ...(k % 3 === 0 ? { where: 'cloud' } : {}) })),
    queued: [...ids(1), ...ids(10)], runNow: ids(10),
    stalled: [...ids(2), ...ids(4)], signInNeeded: Object.fromEntries(ids(4).map((id) => [id, 'Claude Code'])),
    shipping: ids(3), silent: Object.fromEntries(ids(5).map((id) => [id, { runs: 1, endedAt: now - 60_000 }])),
    starting: ids(7),
  });
  return snapshot;
}

// `reference.mjs` takes its snapshot from the checkout's own harness. To hand it another one
// without touching either, it is pointed at a stand-in checkout: the real harness, with the
// snapshot read from a file, and the real built renderer.
function standIn(mercury, snapshotFile) {
  const shim = fs.mkdtempSync(path.join(os.tmpdir(), 'mercury-rows-'));
  fs.mkdirSync(path.join(shim, 'scripts/lib'), { recursive: true });
  fs.mkdirSync(path.join(shim, 'renderer'));
  fs.symlinkSync(path.join(mercury, 'renderer/dist'), path.join(shim, 'renderer/dist'));
  fs.writeFileSync(path.join(shim, 'scripts/lib/inbox-harness.mjs'), [
    `import fs from 'node:fs';`,
    `export { openInbox } from ${JSON.stringify(pathToFileURL(path.join(mercury, 'scripts/lib/inbox-harness.mjs')).href)};`,
    `export const herSnapshot = () => JSON.parse(fs.readFileSync(${JSON.stringify(snapshotFile)}, 'utf8'));`,
    '',
  ].join('\n'));
  return shim;
}

async function shoot(mercury) {
  fs.mkdirSync(dir, { recursive: true });
  let checkout = mercury;
  let env = process.env;
  let shim = null;
  if (flag('--cases') || flag('--real')) {
    const made = flag('--real') ? await realSnapshot(mercury) : (await import('./rows.cases.mjs')).snapshot(Date.now());
    const snapshot = flag('--stir') ? stir(made) : made;
    const file = path.resolve(dir, 'snapshot.json');
    fs.writeFileSync(file, JSON.stringify(snapshot));
    shim = standIn(mercury, file);
    checkout = shim;
    env = { ...process.env, MERCURY_REAL: '1' };
  }
  try {
    // One capture at a time. A tall window, so a scroll area lays out more of its rows.
    for (const scene of SCENES) {
      const run = spawnSync('arch', ['-arm64', 'node', path.join(here, 'reference.mjs'), checkout, dir, 'dark', '1280', '2400', scene], { env, encoding: 'utf8' });
      if (run.status !== 0) throw new Error(`reference.mjs ${scene} failed:\n${run.stderr}`);
    }
  } finally {
    if (shim) fs.rmSync(shim, { recursive: true, force: true });
  }
}

/* ------------------------------------------------------------ the rows as drawn */

function tree(dump) {
  const kids = new Map();
  for (const n of dump.nodes) {
    if (!kids.has(n.parent)) kids.set(n.parent, []);
    kids.get(n.parent).push(n);
  }
  const has = (n, cls) => String(n.cls).split(/\s+/).includes(cls);
  const under = (n) => (kids.get(n.id) ?? []).flatMap((k) => [k, ...under(k)]);
  const first = (n, test) => under(n).find(test) ?? null;
  return { kids, has, under, first };
}

// The level the priority bars draw: four bars is Urgent, then three with 3, 2 or 1 filled.
const LEVEL = { p4: 'urgent', p3: 'high', p2: 'medium', p1: 'low' };
const levelOf = (t, n) => {
  const bars = n && t.first(n, (k) => t.has(k, 'prio-bars'));
  const cls = bars && Object.keys(LEVEL).find((c) => t.has(bars, c));
  return cls ? LEVEL[cls] : 'medium';
};

/** Home's two cards: Needs you, and In progress. */
function homeRows(dump) {
  const t = tree(dump);
  const card = (name) => dump.nodes.find((n) => t.has(n, 'hm-card') && t.first(n, (k) => t.has(k, 't'))?.text === name);
  const rows = (name) => (card(name) ? t.under(card(name)).filter((n) => t.has(n, 'hm-row')) : null);
  const needs = rows('Needs you')?.map((r) => ({
    letter: t.first(r, (k) => t.has(k, 'hm-face'))?.text ?? '',
    title: t.first(r, (k) => k.tag === 'b')?.text ?? '',
    priority: levelOf(t, t.first(r, (k) => t.has(k, 'hm-prio'))),
    project: t.under(r).filter((k) => t.has(k, 'hm-tag')).map((k) => k.text ?? '').join(' | '),
    when: t.first(r, (k) => k.tag === 'time')?.text ?? '',
    unread: t.has(r, 'unread'),
  }));
  const progress = rows('In progress')?.map((r) => ({
    letter: t.first(r, (k) => t.has(k, 'hm-face'))?.text ?? '',
    title: t.first(r, (k) => k.tag === 'b')?.text ?? '',
    progressLine: t.first(r, (k) => k.tag === 'small')?.text ?? '',
    when: t.first(r, (k) => k.tag === 'time')?.text ?? '',
  }));
  return { needs, progress };
}

/** The Inbox page's table: one row a thread, under whichever tab is up. */
function tableRows(dump) {
  const t = tree(dump);
  const table = dump.nodes.find((n) => t.has(n, 'list') && t.has(n, 'th-table'));
  if (!table) return [];
  return t.under(table).filter((n) => t.has(n, 'row') && !t.has(n, 'th-head')).map((r) => {
    const cell = (cls) => t.first(r, (k) => t.has(k, cls));
    const titleCell = cell('th-cell-title');
    if (!titleCell) throw new Error('a row of the table is not drawn as a thread: an import row, or the walkthrough');
    // With faint words after it the title is in a span of its own; without, it is the cell's text.
    const title = t.first(titleCell, (k) => t.has(k, 'th-title-text'))?.text ?? titleCell.text ?? '';
    const asideBox = t.first(titleCell, (k) => t.has(k, 'th-aside-box'));
    const state = cell('th-cell-state');
    const dot = t.first(state, (k) => t.has(k, 'th-st') || t.has(k, 'th-st-cloud'));
    const mark = !dot ? 'none' : t.has(dot, 'th-st-cloud') ? 'cloud' : t.has(dot, 'live') ? 'running' : ['queued', 'held', 'need'].find((m) => t.has(dot, `s-${m}`)) ?? 'other';
    const proj = cell('th-cell-proj');
    const more = t.first(proj, (k) => t.has(k, 'th-proj-more'))?.text ?? '';
    return {
      title,
      aside: (asideBox && t.first(asideBox, (k) => t.has(k, 'th-aside'))?.text) ?? '',
      // "not started" is faint words of its own, straight in the cell.
      held: (t.kids.get(titleCell.id) ?? []).some((k) => t.has(k, 'th-aside') && !t.has(k, 'th-demand')),
      pinned: !!t.first(titleCell, (k) => t.has(k, 'th-pin')),
      status: state?.text ?? '',
      mark,
      place: t.first(proj, (k) => t.has(k, 'th-proj-name'))?.text ?? proj?.text ?? '',
      placeMore: more ? Number(more.replace('+', '')) : 0,
      priority: levelOf(t, cell('th-cell-prio')),
      priorityLabel: cell('th-cell-prio')?.text ?? '',
      when: t.first(r, (k) => t.has(k, 'th-when'))?.text ?? '',
      unread: t.has(r, 'unread'),
    };
  });
}

/* ------------------------------------------------------------ the native rows */

// Written beside the dump it answers, so the two can be read together and nothing of a real
// thread leaves the folder the dumps are in.
function nativeLists(scene) {
  const out = path.join(dir, `${scene}-rows.json`);
  const bin = path.join(here, '..', 'target', 'debug', 'mercury');
  const run = spawnSync(bin, ['--fixture', path.join(dir, `${scene}-fixture.json`), '--rows', out], { encoding: 'utf8' });
  if (run.status !== 0) throw new Error(`mercury --rows failed: ${run.stderr || run.error}`);
  return JSON.parse(fs.readFileSync(out, 'utf8'));
}

const LABEL = { urgent: 'Urgent', high: 'High', medium: '', low: 'Low' };

/** A native row, said the way one of the two pages would draw it. */
const asHomeNeeds = (r) => ({ letter: r.letter, title: r.title, priority: r.priority, project: r.project, when: r.when, unread: r.unread });
const asHomeProgress = (r) => ({ letter: r.letter, title: r.title, progressLine: `${r.engine} · ${r.project}`, when: r.when });
const asTable = (r) => ({
  title: r.title, aside: r.aside, held: r.held, pinned: r.pinned, status: r.status, mark: r.mark, place: r.place, placeMore: r.placeMore,
  priority: r.priority, priorityLabel: LABEL[r.priority], when: r.when, unread: r.unread,
});

/* ------------------------------------------------------------ comparing */

const WORDS_OF_THE_THREAD = new Set(['title', 'project', 'place', 'aside', 'progressLine']);
const said = (field, value) => (show || !WORDS_OF_THE_THREAD.has(field) ? JSON.stringify(value) : `«${String(value).length} characters»`);

function compare(name, drawn, native) {
  const lines = [];
  if (drawn.length !== native.length) lines.push(`  ${drawn.length} rows drawn, ${native.length} native`);
  const byField = {};
  for (let i = 0; i < Math.min(drawn.length, native.length); i += 1) {
    for (const field of Object.keys(drawn[i])) {
      if (drawn[i][field] === native[i][field]) continue;
      byField[field] = (byField[field] ?? 0) + 1;
      lines.push(`  row ${i} ${field}: drawn ${said(field, drawn[i][field])}, native ${said(field, native[i][field])}`);
    }
  }
  const fields = Object.entries(byField).map(([f, n]) => `${n} in ${f}`).join(', ');
  console.log(`${name}: ${drawn.length} rows, ${lines.length} difference${lines.length === 1 ? '' : 's'}${fields ? ` (${fields})` : ''}`);
  for (const l of lines) console.log(l);
  return lines.length;
}

if (flag('--shoot')) await shoot(path.resolve(after('--shoot')));

let differences = 0;
let checked = 0;
for (const scene of SCENES) {
  const dumpFile = path.join(dir, `${scene}-dump.json`);
  if (!fs.existsSync(dumpFile) || !fs.existsSync(path.join(dir, `${scene}-fixture.json`))) continue;
  checked += 1;
  const dump = JSON.parse(fs.readFileSync(dumpFile, 'utf8'));
  const native = nativeLists(scene);
  if (scene === 'home') {
    const drawn = homeRows(dump);
    // Home shows the first eighty that need the person and the first sixty in progress.
    if (drawn.needs) differences += compare('home, Needs you', drawn.needs, native.needs.slice(0, 80).map(asHomeNeeds));
    if (drawn.progress) differences += compare('home, In progress', drawn.progress, native.progress.slice(0, 60).map(asHomeProgress));
  } else {
    const list = scene === 'inbox' ? native.needs : scene === 'inbox-progress' ? native.progressTab : native.done;
    differences += compare(scene, tableRows(dump), list.map(asTable));
  }
}
if (!checked) {
  console.error(`no dumps in ${dir}`);
  process.exit(2);
}
process.exit(differences ? 1 : 0);
