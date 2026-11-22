// Hold the native lists against the rows the web window drew: same count, same order, every
// field equal. The web side is read out of `reference.mjs`'s dumps, the native side is
// `mercury --fixture <the dump's fixture> --rows`, so both worked from the same snapshot, the
// same clock and the same localStorage facts.
//
//   node tools/rows.mjs <dir>                             the dumps already in <dir>
//   node tools/rows.mjs <dir> --shoot <mercury checkout>  photograph the five scenes first (reference.mjs's invented inbox)
//   node tools/rows.mjs <dir> --shoot <checkout> --cases  ... of the invented threads in tools/rows.cases.mjs
//   node tools/rows.mjs <dir> --shoot <checkout> --real   ... of the person's real threads, read and never written
//   node tools/rows.mjs <dir> --shoot <checkout> --real --stir   ... with a busy fleet invented around them
//   node tools/rows.mjs <dir> --shoot <checkout> --door   ... of the running app's inbox this minute, asked through its local door
//
// Five scenes: home (Needs you and In progress as Home draws them), inbox, inbox-progress,
// inbox-later and inbox-done (the Inbox page's tabs). A scene with no dump in <dir> is skipped.
// Under each tab the page's own furniture is held too: which tabs the strip draws and with
// which numbers, what the time column is called, and what the cloud button wears.
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
  console.error('usage: node tools/rows.mjs <dir> [--shoot <mercury checkout> [--cases | --real [--stir] | --door]] [--show]');
  process.exit(2);
}
const show = flag('--show');
const SCENES = ['home', 'inbox', 'inbox-progress', 'inbox-later', 'inbox-done'];

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

// The running app's own snapshot and what its window keeps for itself, asked through the local
// door the native window reads (`main/local-door.mts`): the inbox as it is this minute, with the
// fleet as it really is. Two reads, and nothing is written.
async function throughTheDoor() {
  const noteFile = after('--door-file') ?? path.join(os.homedir(), 'Library/Application Support/Mercury/local-door.json');
  if (!fs.existsSync(noteFile)) throw new Error(`no door note at ${noteFile}: the app opens its door only with "localDoor": true in zero.config.json`);
  const note = JSON.parse(fs.readFileSync(noteFile, 'utf8'));
  const ask = async (channel) => {
    const res = await fetch(`http://127.0.0.1:${note.port}/api/${encodeURIComponent(channel)}`, {
      method: 'POST', headers: { 'content-type': 'application/json', 'x-agentbox-token': note.token }, body: '{}',
    });
    if (!res.ok) throw new Error(`the door answered ${res.status} to ${channel}`);
    return res.json();
  };
  return { snapshot: await ask('zero:snapshot'), keeps: await ask('zero:window-keeps') };
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
  if (flag('--cases') || flag('--real') || flag('--door')) {
    const live = flag('--door') ? await throughTheDoor() : null;
    const made = live ? live.snapshot : flag('--real') ? await realSnapshot(mercury) : (await import('./rows.cases.mjs')).snapshot(Date.now());
    const snapshot = flag('--stir') ? stir(made) : made;
    const file = path.resolve(dir, 'snapshot.json');
    fs.writeFileSync(file, JSON.stringify(snapshot));
    shim = standIn(mercury, file);
    checkout = shim;
    env = { ...process.env, MERCURY_REAL: '1' };
    if (live) {
      const keeps = path.resolve(dir, 'keeps.json');
      fs.writeFileSync(keeps, JSON.stringify(live.keeps));
      env.MERCURY_KEEPS = keeps;
    }
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
  // Asked of nothing (a cell the table is not drawing) it finds nothing.
  const first = (n, test) => (n ? under(n).find(test) ?? null : null);
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
  // What a row wears before its title: an agent's thread the agent face (`.hm-agent`), anyone
  // else their letters.
  const face = (r) => {
    const f = t.first(r, (k) => t.has(k, 'hm-face'));
    return f && t.has(f, 'hm-agent') ? 'agent' : f?.text ?? '';
  };
  const needs = rows('Needs you')?.map((r) => ({
    face: face(r),
    title: t.first(r, (k) => k.tag === 'b')?.text ?? '',
    priority: levelOf(t, t.first(r, (k) => t.has(k, 'hm-prio'))),
    project: t.under(r).filter((k) => t.has(k, 'hm-tag')).map((k) => k.text ?? '').join(' | '),
    when: t.first(r, (k) => k.tag === 'time')?.text ?? '',
    unread: t.has(r, 'unread'),
  }));
  const progress = rows('In progress')?.map((r) => ({
    face: face(r),
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
  // A narrow table gives up Priority, then Project (`@container th` in pages.css). A column
  // given up says nothing about its rows, so it is not held; one that is only empty still is.
  const wide = table.rect[2];
  const gaveUp = { priority: wide <= 915, project: wide <= 785 };
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
    // The mark before the title: a harness's in its tile (`HarnessMark`), or the agent's face.
    const tile = t.first(titleCell, (k) => t.has(k, 'harness-mark'));
    return {
      harness: tile ? ['claude', 'codex', 'antigravity'].find((engine) => t.has(tile, engine)) ?? 'other' : 'face',
      title,
      aside: (asideBox && t.first(asideBox, (k) => t.has(k, 'th-aside'))?.text) ?? '',
      // "not started" is faint words of its own, straight in the cell.
      held: (t.kids.get(titleCell.id) ?? []).some((k) => t.has(k, 'th-aside') && !t.has(k, 'th-demand')),
      pinned: !!t.first(titleCell, (k) => t.has(k, 'th-pin')),
      status: state?.text ?? '',
      mark,
      // Try again after the word, and what the word says under the pointer: its sentence, or
      // the card saying why the cloud has not got the thread in its place.
      retry: !!t.first(state, (k) => t.has(k, 'th-retry')),
      statusTitle: state?.attrs?.title ?? '',
      whyCard: !!state && t.has(state, 'has-card'),
      ...(gaveUp.project ? {} : { place: t.first(proj, (k) => t.has(k, 'th-proj-name'))?.text ?? proj?.text ?? '', placeMore: more ? Number(more.replace('+', '')) : 0 }),
      ...(gaveUp.priority ? {} : { priority: levelOf(t, cell('th-cell-prio')), priorityLabel: cell('th-cell-prio')?.text ?? '' }),
      when: t.first(r, (k) => t.has(k, 'th-when'))?.text ?? '',
      unread: t.has(r, 'unread'),
    };
  });
}

// The tabs as the strip words them, the name `mercury --tab` knows each by, and the list of
// `mercury --rows` its rows are in.
const TABS = {
  'Needs you': { name: 'needs', list: 'needs' },
  'In progress': { name: 'progress', list: 'progressTab' },
  Later: { name: 'later', list: 'later' },
  Done: { name: 'done', list: 'done' },
};

/** What the Inbox page draws around its rows: the strip of tabs, the time column's heading, the cloud button. */
function pageFurniture(dump) {
  const t = tree(dump);
  const tabs = dump.nodes.filter((n) => t.has(n, 'tm-tab')).map((n) => ({
    label: n.text ?? '', count: Number(t.first(n, (k) => k.tag === 'b')?.text ?? 0), on: t.has(n, 'on'),
  }));
  const head = dump.nodes.find((n) => t.has(n, 'th-head'));
  const cloud = dump.nodes.find((n) => n.tag === 'button' && t.has(n, 'th-cloud'));
  const word = cloud && t.first(cloud, (k) => t.has(k, 'th-cloud-word'));
  return {
    open: tabs.find((tab) => tab.on)?.label ?? null,
    tabs: stripWords(tabs),
    // An empty tab has no table, and so no heading to name its time column.
    heading: head ? t.first(head, (k) => t.has(k, 'num'))?.text ?? '' : null,
    cloud: !cloud ? 'no button' : [
      t.has(cloud, 'lit') ? 'lit' : 'unlit',
      ...(t.has(cloud, 'count') ? [`counts ${t.first(cloud, (k) => k.tag === 'b')?.text}`] : []),
      ...(word ? [`says "${word.text}"`] : []),
      ...(t.first(cloud, (k) => k.tag === 'i') ? ['dot'] : []),
    ].join(', '),
  };
}

/** The same, of what `mercury --rows --tab <the open tab>` says the native page draws. */
function nativeFurniture(native, rows) {
  const { tabs, heading, cloud } = native.page;
  const WORDS = { cloud: 'Cloud', again: 'Sign in again' };
  return {
    open: tabs.find((tab) => tab.on)?.label ?? null,
    tabs: stripWords(tabs),
    heading: rows.length ? heading : null,
    cloud: !cloud ? 'no button' : [
      cloud.lit ? 'lit' : 'unlit',
      ...(cloud.count !== null ? [`counts ${cloud.count}`] : []),
      ...(cloud.word ? [`says "${WORDS[cloud.word]}"`] : []),
      ...(cloud.dot ? ['dot'] : []),
    ].join(', '),
  };
}

// "Needs you 2 | In progress 1 | [Later 5] | Done": the strip in one line, the open tab in brackets.
function stripWords(tabs) {
  return tabs.map((tab) => { const words = `${tab.label}${tab.count ? ` ${tab.count}` : ''}`; return tab.on ? `[${words}]` : words; }).join(' | ');
}

/* ------------------------------------------------------------ the native rows */

// Written beside the dump it answers, so the two can be read together and nothing of a real
// thread leaves the folder the dumps are in. `tab` is the tab the page was photographed on.
function nativeLists(scene, tab = null) {
  const out = path.join(dir, `${scene}-rows.json`);
  const bin = path.join(here, '..', 'target', 'debug', 'mercury');
  const run = spawnSync(bin, ['--fixture', path.join(dir, `${scene}-fixture.json`), '--rows', out, ...(tab ? ['--tab', tab] : [])], { encoding: 'utf8' });
  if (run.status !== 0) throw new Error(`mercury --rows failed: ${run.stderr || run.error}`);
  return JSON.parse(fs.readFileSync(out, 'utf8'));
}

// The table says every level, Medium included (`RowCells`).
const LABEL = { urgent: 'Urgent', high: 'High', medium: 'Medium', low: 'Low' };

/** A native row, said the way one of the two pages would draw it. Home draws every row with the
 *  agent face: this window does not tell a teammate's thread from an agent's. */
const asHomeNeeds = (r) => ({ face: 'agent', title: r.title, priority: r.priority, project: r.project, when: r.when, unread: r.unread });
const asHomeProgress = (r) => ({ face: 'agent', title: r.title, progressLine: `${r.engine} · ${r.project}`, when: r.when });
const asTable = (r) => ({
  harness: r.harness,
  title: r.title, aside: r.aside, held: r.held, pinned: r.pinned, status: r.status, mark: r.mark,
  retry: r.retry, statusTitle: r.why ? '' : (r.statusTitle ?? ''), whyCard: !!r.why,
  place: r.place, placeMore: r.placeMore,
  priority: r.priority, priorityLabel: LABEL[r.priority], when: r.when, unread: r.unread,
});

/* ------------------------------------------------------------ comparing */

// A Stopped row's sentence is its result's first line, so the hover sentence is the thread's words too.
const WORDS_OF_THE_THREAD = new Set(['title', 'project', 'place', 'aside', 'progressLine', 'statusTitle']);
const said = (field, value) => (show || !WORDS_OF_THE_THREAD.has(field) ? JSON.stringify(value) : `«${String(value).length} characters»`);

function compare(name, drawn, native, what = 'rows') {
  const lines = [];
  if (drawn.length !== native.length) lines.push(`  ${drawn.length} ${what} drawn, ${native.length} native`);
  const byField = {};
  for (let i = 0; i < Math.min(drawn.length, native.length); i += 1) {
    for (const field of Object.keys(drawn[i])) {
      if (drawn[i][field] === native[i][field]) continue;
      byField[field] = (byField[field] ?? 0) + 1;
      lines.push(`  row ${i} ${field}: drawn ${said(field, drawn[i][field])}, native ${said(field, native[i][field])}`);
    }
  }
  const fields = Object.entries(byField).map(([f, n]) => `${n} in ${f}`).join(', ');
  console.log(`${name}: ${what === 'rows' ? `${drawn.length} rows, ` : ''}${lines.length} difference${lines.length === 1 ? '' : 's'}${fields ? ` (${fields})` : ''}`);
  for (const l of lines) console.log(l);
  return lines.length;
}

if (flag('--shoot')) await shoot(path.resolve(after('--shoot')));

// The five scenes, and any other picture of the Inbox page left in the folder: a scene that
// invents a cloud for the button (`tools/scenes/inbox-cloud*.mjs`) is held the same way.
const others = fs.existsSync(dir) ? fs.readdirSync(dir).filter((f) => /^inbox-.+-dump\.json$/.test(f)).map((f) => f.slice(0, -'-dump.json'.length)).filter((s) => !SCENES.includes(s)).sort() : [];

let differences = 0;
let checked = 0;
for (const scene of [...SCENES, ...others]) {
  const dumpFile = path.join(dir, `${scene}-dump.json`);
  if (!fs.existsSync(dumpFile) || !fs.existsSync(path.join(dir, `${scene}-fixture.json`))) continue;
  checked += 1;
  const dump = JSON.parse(fs.readFileSync(dumpFile, 'utf8'));
  if (scene === 'home') {
    const native = nativeLists(scene);
    const drawn = homeRows(dump);
    // Home shows the first eighty that need the person and the first sixty in progress.
    if (drawn.needs) differences += compare('home, Needs you', drawn.needs, native.needs.slice(0, 80).map(asHomeNeeds));
    if (drawn.progress) differences += compare('home, In progress', drawn.progress, native.progress.slice(0, 60).map(asHomeProgress));
  } else {
    // The native page is opened on the tab the picture was taken on. That is the scene's own
    // tab, except for Later on an inbox with nothing in it, where the strip has no Later to press.
    const furniture = pageFurniture(dump);
    const tab = TABS[furniture.open];
    if (!tab) throw new Error(`${scene}: no tab of the strip is open in the dump`);
    const native = nativeLists(scene, tab.name);
    const list = native[tab.list];
    differences += compare(scene, tableRows(dump), list.map(asTable));
    differences += compare(`${scene}, around the rows`, [furniture], [nativeFurniture(native, list)], 'pages');
  }
}
if (!checked) {
  console.error(`no dumps in ${dir}`);
  process.exit(2);
}
process.exit(differences ? 1 : 0);
