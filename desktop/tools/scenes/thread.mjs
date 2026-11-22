// From Home to one open conversation: the first thread of the Inbox, as Enter opens it, with
// its earlier messages unfolded. (`thread-folded` is the same thread as it first opens.)
//
// The harness's bridge answers the snapshot and little else, and a thread reads more than
// that: its ledger (`zero:item-history`), what its runs did (`zero:session-trace`), the files
// they changed (`zero:code-change`) and whether it is being compacted
// (`zero:compaction-status`). So this scene answers those four in the page, with one invented
// conversation, and hands the same answers back for the fixture file, where the native window
// reads them. `docs/thread-data.md` says what each channel is asked and answers.
import { go as toInbox } from './inbox.mjs';
import { tab } from './inbox-progress.mjs';
import { plant, standIn } from '../pictures.mjs';

const MIN = 60_000;

const ASK = `Checkout fails for some customers since this morning. Support has eleven tickets so far.

Find out why and fix it, please.`;

const FOUND = `**Found it: tax rounding breaks on carts over $1,000.**

The tax service started returning four decimal places this morning, and \`roundTax()\` only handles two. Smaller carts happen to round the same either way, which is why most orders still go through. What changed is in [the tax service's release notes](https://example.com/tax/releases/4.2), and it was *not* announced.

What I checked:

- 37 failed orders since 08:10, every one of them over $1,000
- No failures on carts paid with store credit
- The same carts pass once the tax is rounded **before** the total is added up

| Cart total | Orders | Failed |
| --- | --- | --- |
| Under $1,000 | 412 | 0 |
| $1,000 to $5,000 | 31 | 29 |
| Over $5,000 | 8 | 8 |

The fix is small:

\`\`\`ts
export function roundTax(amount: number): number {
  return Math.round(amount * 100) / 100;
}
\`\`\`

## Options

1. Ship the fix now and refund the 37 orders by hand (recommended)
2. Ship the fix and let the nightly job retry the orders
3. Turn the new tax rates off until Monday`;

const REPLY = 'Go with 1. Tell support before the refunds go out.';

const SHIPPED = `**Shipped, and support knows.**

The fix is on main as \`a41c9e2\` and every checkout test passes. The 37 refunds are queued behind a note to support, so nothing goes out before they have read it.`;

const SEC = 1000;

// The log one run leaves (`<product>/sessions/<thread id>/<startedAt>.log`), in the shape the
// supervisor writes it (`traceStreamLine` in main/harnesses/claude-stream.mts): a header, then
// each line `HH:MM:SS  <what happened>` on a UTC clock, a tool call with its id and what came
// back under it, the result block and the exit. A line here is a string as it is written, or
// `[ms after the start, words]` for one that is stamped.
function runLog(item, startedAt, lines, { continuation = false } = {}) {
  const stamp = (ms) => new Date(startedAt + ms).toISOString().slice(11, 19);
  const head = [`# ${item.title}`, `# ${item.id} · spawned ${new Date(startedAt).toISOString()}${continuation ? ' · continuation (the founder answered)' : ''}`, ''];
  return { startedAt, text: [...head, ...lines.map((l) => (typeof l === 'string' ? l : `${stamp(l[0])}  ${l[1]}`))].join('\n') };
}
const out = (id, lines, error = false) => [`#out ${id} ${lines.length}${error ? ' error' : ''}`, ...lines.map((l) => `#| ${l}`)];
const ended = (startedAt, ms, code = 0) => ['', `# exited (${code}) ${new Date(startedAt + ms).toISOString()}`];
const result = (startedAt, ms, words, error = false) => {
  const stamp = new Date(startedAt + ms).toISOString().slice(11, 19);
  return ['', `${stamp}  == RESULT (success${error ? ' ERROR' : ''} · ${error ? 1 : 14} turns) ==`, ...words.split('\n')];
};

// The first run: it reads the code and this morning's orders, a test fails the way the
// customers' carts do, and it writes what it found to the thread as a checkpoint. A run's
// last words repeat the answer it wrote, which the page draws once.
function findingRun(item, start, wrote, found) {
  const at = wrote - start;
  return runLog(item, start, [
    [3 * SEC, `[mcp__mercury__claim_work_item] ${item.id}`], '#id toolu_a1',
    [9 * SEC, 'I\'ll start with this morning\'s failed orders and the checkout code they went through.'],
    [12 * SEC, '[Bash] cd /Users/you/dev/storefront && git log --oneline -8'], '#id toolu_a2',
    ...out('toolu_a2', ['b72f1d4 Update the tax service client to 4.2', '91c03ee Show the delivery date in the cart', '3e8a5b0 Retry a declined card once', '7d1f2a9 Move the cart total into its own module', 'c40e6b1 Log the tax service\'s answer', '5a9d3f2 Cache the shipping zones', 'e18b7c4 Round the shipping cost to the cent', '0f6c2d8 Start the checkout tests on node 22']),
    [20 * SEC, '[Read] /Users/you/dev/storefront/src/checkout/tax.ts'], '#id toolu_a3',
    ...out('toolu_a3', ['export function roundTax(amount: number): number {', '  return Number(amount.toFixed(2));', '}']),
    [27 * SEC, '[Grep] roundTax'], '#id toolu_a4',
    ...out('toolu_a4', ['src/checkout/tax.ts:12', 'src/checkout/total.ts:40']),
    [4 * MIN, 'The tax service started returning four decimal places this morning, and `roundTax()` only handles two. Carts over $1,000 are where the difference shows.'],
    [4 * MIN + 5 * SEC, '[Bash] node scripts/failed-orders.mjs --since 08:00'], '#id toolu_a5',
    ...out('toolu_a5', ['37 failed orders since 08:10', 'all 37 over $1,000', '0 paid with store credit']),
    [6 * MIN, '[Bash] npm test -- tax'], '#id toolu_a6',
    ...out('toolu_a6', ['FAIL src/checkout/tax.test.ts', '  ✕ rounds a $1,240.5550 cart to the cent', '    Expected: 1240.56', '    Received: 1240.555'], true),
    [7 * MIN, 'The same carts pass once the tax is rounded before the total is added up. Writing it up with three ways forward.'],
    [at - 2 * SEC, `[mcp__mercury__update_work_item] ${item.id}`], '#id toolu_a7',
    [at + 5 * SEC, found.split('\n')[0]],
    '',
    'The fix and the three ways forward are on the thread.',
    ...result(start, at + 6 * SEC, found.split('\n')[0]),
    ...ended(start, at + 8 * SEC),
  ]);
}

// The run after the reply: it ships the fix and writes to support before the refunds. Cut off
// at `upTo` it is a run still going, its last call still running. `flaky`: its first run of
// the tests fails, and it changes the file again before they pass.
function shippingRun(item, start, wrote, said, upTo = Infinity, { flaky = false } = {}) {
  const at = wrote - start;
  const lines = [
    [3 * SEC, `[mcp__mercury__claim_work_item] ${item.id}`], '#id toolu_b1',
    [8 * SEC, 'Shipping option 1: the rounding fix first, then the 37 refunds once support has the note.'],
    [15 * SEC, '[Edit] /Users/you/dev/storefront/src/checkout/tax.ts'], '#id toolu_b2',
    ...out('toolu_b2', ['The file src/checkout/tax.ts has been updated.']),
    ...(flaky ? [
      [25 * SEC, '[Bash] npm test -- checkout'], '#id toolu_b9',
      ...out('toolu_b9', ['FAIL src/checkout/total.test.ts', '  ✕ adds the tax after rounding it', '    Expected: 1241.80', '    Received: 1241.79'], true),
      [28 * SEC, '[Edit] /Users/you/dev/storefront/src/checkout/total.ts'], '#id toolu_b10',
      ...out('toolu_b10', ['The file src/checkout/total.ts has been updated.']),
    ] : []),
    [30 * SEC, '[Bash] npm test -- checkout'], '#id toolu_b3',
    ...out('toolu_b3', ['Test Files  12 passed (12)', '     Tests  214 passed (214)']),
    [4 * MIN, '[Bash] git commit -am "Round tax before adding it to the total"'], '#id toolu_b4',
    ...out('toolu_b4', ['[main a41c9e2] Round tax before adding it to the total', ' 2 files changed, 9 insertions(+), 2 deletions(-)']),
    [4 * MIN + 10 * SEC, '[Bash] git push origin main'], '#id toolu_b5',
    ...out('toolu_b5', ['To example.com:storefront.git', '   b72f1d4..a41c9e2  main -> main']),
    [5 * MIN, 'Main is at `a41c9e2`. Writing to support before any refund goes out.'],
    [6 * MIN, '[Write] /Users/you/dev/storefront/notes/refunds-for-support.md'], '#id toolu_b6',
    [8 * MIN, '[Bash] node scripts/queue-refunds.mjs --after-note'], '#id toolu_b7',
    ...out('toolu_b7', ['37 refunds queued', 'they go out once support marks the note read']),
    [at - 2 * SEC, `[mcp__mercury__update_work_item] ${item.id}`], '#id toolu_b8',
    [at + 3 * SEC, said.split('\n')[0]],
    '',
    said.split('\n').slice(2).join('\n'),
    ...result(start, at + 4 * SEC, said),
    ...ended(start, at + 6 * SEC),
  ];
  // A stamped line past the cut, and everything after it, has not been written yet.
  const cut = lines.findIndex((l) => typeof l !== 'string' && l[0] > upTo);
  return runLog(item, start, cut < 0 ? lines : lines.slice(0, cut), { continuation: true });
}

// A run turned away before it began: the tool types its own refusal into the run as if the
// agent had said it, and repeats it as an error result (`errorResultText` in terminal.ts).
function refusedRun(item, start) {
  const refusal = 'API Error: Claude AI usage limit reached. Your limit resets at 4pm.';
  return runLog(item, start, [[5 * SEC, refusal], ...result(start, 5 * SEC, refusal, true), ...ended(start, 6 * SEC, 1)], { continuation: true });
}

// The ledger a thread like this leaves behind, one line per write, and the row those lines
// fold into (shared/work-items.mts): the newest value of each field, and who wrote it when.
// `runs` gives it the logs its runs left: `steps` (each run finished), `refused` (the run
// after the reply was turned away and a later one shipped), `working` (the run after the reply
// is going on right now, and the thread is In progress). `reply` is what the person said to the
// first answer.
function conversation(item, found, waiting = false, said = SHIPPED, runs = null, reply = REPLY) {
  const [asked, checked, replied, shipped] = [item.createdAt, item.createdAt + 38 * MIN, item.updatedAt - 17 * MIN, item.updatedAt];
  const { id, title } = item;
  // A worker claims its thread as its first action, so the claim lands seconds after its run began.
  const began = (claim) => claim - 10 * SEC;
  if (runs === 'working') {
    // Replied to a few minutes ago, and the run that took the reply is running its tests.
    const replied = item.updatedAt - 2 * MIN;
    const picked = replied + MIN;
    const lines = [
      { id, ts: asked, source: 'founder', patch: { title, body: ASK, status: 'open', kind: 'directive', priority: item.priority, labels: item.labels } },
      { id, ts: asked + MIN, source: 'agent', epoch: 1, claim: { holder: 'mcp-40122', leaseUntil: asked + 31 * MIN }, patch: { status: 'claimed' } },
      { id, ts: checked, source: 'agent', epoch: 1, patch: { status: 'blocked', note: found } },
      { id, ts: replied, source: 'founder', patch: { answer: reply, status: 'open' } },
      { id, ts: picked, source: 'agent', epoch: 2, claim: { holder: 'mcp-40131', leaseUntil: picked + 31 * MIN }, patch: { status: 'claimed' } },
    ];
    const { result: _result, ...rest } = item;
    const row = {
      ...rest, body: ASK, note: found, answer: reply, status: 'claimed', epoch: 2, updatedAt: picked,
      claim: { holder: 'mcp-40131', leaseUntil: picked + 31 * MIN },
      wrote: {
        title: { ts: asked, source: 'founder' }, body: { ts: asked, source: 'founder' },
        note: { ts: checked, source: 'agent' }, answer: { ts: replied, source: 'founder' },
        status: { ts: picked, source: 'agent' },
      },
    };
    // Up to the tests, which are running as the picture is taken.
    const start = began(picked);
    const sessions = [findingRun(item, began(asked + MIN), checked, found), shippingRun(item, start, shipped, said, 30 * SEC)];
    const session = {
      itemId: id, product: item.product, startedAt: start, engine: 'claude-code', tail: ['tool: Edit', 'tool: Bash'],
      activity: [{ id: 'toolu_b3', label: 'Running the tests', detail: 'npm test -- checkout', startedAt: start + 30 * SEC }],
    };
    return { row, lines, sessions, session };
  }
  // `waiting`: the thread as it stood when the agent's first answer came back, its options
  // still open and nothing said to them yet.
  if (waiting) {
    const lines = [
      { id, ts: asked, source: 'founder', patch: { title, body: ASK, status: 'open', kind: 'directive', priority: item.priority, labels: item.labels } },
      { id, ts: asked + MIN, source: 'agent', epoch: 1, claim: { holder: 'mcp-40122', leaseUntil: asked + 31 * MIN }, patch: { status: 'claimed' } },
      { id, ts: item.updatedAt, source: 'agent', epoch: 1, patch: { status: 'blocked', note: found } },
    ];
    const { result: _result, ...rest } = item;
    const row = {
      ...rest, body: ASK, note: found, status: 'blocked',
      wrote: {
        title: { ts: asked, source: 'founder' }, body: { ts: asked, source: 'founder' },
        note: { ts: item.updatedAt, source: 'agent' }, status: { ts: item.updatedAt, source: 'agent' },
      },
    };
    return { row, lines };
  }
  // `refused`: the first run after the reply was turned away at the usage limit, and the one
  // that shipped picked the thread up once it reset.
  const refused = runs === 'refused';
  const shipping = refused ? replied + 6 * MIN : replied + MIN;
  const lines = [
    { id, ts: asked, source: 'founder', patch: { title, body: ASK, status: 'open', kind: 'directive', priority: item.priority, labels: item.labels } },
    { id, ts: asked + MIN, source: 'agent', epoch: 1, claim: { holder: 'mcp-40122', leaseUntil: asked + 31 * MIN }, patch: { status: 'claimed' } },
    { id, ts: checked, source: 'agent', epoch: 1, patch: { status: 'blocked', note: found } },
    { id, ts: replied, source: 'founder', patch: { answer: reply, status: 'open' } },
    ...(refused ? [{ id, ts: replied + MIN, source: 'agent', epoch: 2, claim: { holder: 'mcp-40127', leaseUntil: replied + 31 * MIN }, patch: { status: 'claimed' } }] : []),
    { id, ts: shipping, source: 'agent', epoch: refused ? 3 : 2, claim: { holder: 'mcp-40131', leaseUntil: shipping + 31 * MIN }, patch: { status: 'claimed' } },
    { id, ts: shipped, source: 'agent', epoch: refused ? 3 : 2, patch: { status: 'done', result: said } },
  ];
  const row = {
    ...item, body: ASK, note: found, answer: reply, result: said,
    wrote: {
      title: { ts: asked, source: 'founder' }, body: { ts: asked, source: 'founder' },
      note: { ts: checked, source: 'agent' }, answer: { ts: replied, source: 'founder' },
      result: { ts: shipped, source: 'agent' }, status: { ts: shipped, source: 'agent' },
    },
  };
  const sessions = !runs ? [] : [
    findingRun(item, began(asked + MIN), checked, found),
    ...(refused ? [refusedRun(item, began(replied + MIN))] : []),
    shippingRun(item, began(shipping), shipped, said, Infinity, { flaky: refused }),
  ];
  return { row, lines, sessions };
}

// `unfold` presses the line under the title, so the picture is of the whole conversation
// and not of its newest answer alone. `found` is the agent's first answer, for a scene that
// wants other markdown in it.
// `waiting` leaves the thread at that first answer, waiting on the person. `then` is what a
// scene does once the thread is open (a key, some typing), before the picture.
// `shipped` is the agent's last answer. `files` plants an invented project folder for a scene
// whose words name files (`../pictures.mjs`): the thread's project is moved into it, the
// page's pictures are answered from it, and `found` and `shipped` may then be functions of
// what was planted (`{ dir, repo, thread }`), for words that hold a full path.
// `files.arriving` leaves every picture on its way for ever, and `files.large` plants ten
// screenshots besides, for a measurement.
// `runs` gives the thread's runs their logs (see `conversation`); with `working` an agent is
// on the thread as the picture is taken, so it is opened from In progress. `reply` is what the
// person answered, a function of what was planted as `found` may be.
// `marks` is what the store folds a thread's marks on its files into (`item.artifacts`): the
// notes on them, which the list of the thread's files counts.
// `dress` changes the thread's row before the page opens (a label, a status), and `fromTab`
// names the Inbox's tab the thread is then opened from, whichever run is on it. `fleet` changes
// what the supervisor says about it once the run is set (`fleet(snapshot, row, now)`): queued,
// stopped, signed out, the run taken away.
export async function open(h, { unfold, found = FOUND, shipped = SHIPPED, files = null, waiting = false, then = null, runs = null, reply = REPLY, marks = null, dress = null, fromTab = null, fleet = null, trace = null }) {
  const snapshot = await h.evaluate('window.__SNAP__');
  // The Inbox opens on its first row; the snapshot's first thread is not always that one.
  await toInbox(h);
  // Found by the row's own id: a thread named by subject is not drawn under its title, and a
  // real one's words stay out of whatever this prints.
  const first = await h.evaluate(`document.querySelector('.list .row.selected')?.dataset.itemId ?? ''`);
  const at = snapshot.items.findIndex((i) => i.id === first || `${i.product}/${i.id}` === first);
  if (at < 0) throw new Error('thread scene: the Inbox\'s first row is not in the snapshot');
  const planted = files ? plant({ thread: snapshot.items[at].id, large: !!files.large }) : null;
  const words = (text) => (typeof text === 'function' ? text(planted) : text);
  const made = conversation(snapshot.items[at], words(found), waiting, words(shipped), runs, words(reply));
  const { row, lines, session = null } = made;
  // `trace` changes the runs' logs before they are served (a line added to one).
  const sessions = trace ? trace(made.sessions ?? []) : made.sessions ?? [];
  if (marks) row.artifacts = marks;
  if (dress) dress(row);
  snapshot.items[at] = row;
  if (session) snapshot.supervisor.running = [session];
  if (fleet) fleet(snapshot, row, Date.now());
  if (planted) {
    const project = snapshot.products.find((p) => p.slug === row.product);
    project.dir = planted.dir;
    project.repoPath = planted.repo;
  }
  // Keyed as the page asks: product and id. Each answer is under its channel's name. The logs
  // are answered whole, with no stamps, which the page takes as they come (`traceReader`).
  const threads = {
    [`${row.product}/${row.id}`]: {
      'zero:item-history': { ok: true, lines },
      'zero:session-trace': { ok: true, sessions },
      'zero:code-change': { ok: false, error: 'no change was recorded for this thread' },
      'zero:compaction-status': null,
    },
  };
  // On every load from here on: the snapshot with the conversation's row in it, the four
  // answers, and a note of anything else the page asks the bridge for and does not find.
  if (planted) await h.call('Page.addScriptToEvaluateOnNewDocument', { source: standIn(planted, { arriving: !!files.arriving }) });
  await h.call('Page.addScriptToEvaluateOnNewDocument', {
    source: `(() => {
      window.__SNAP__ = ${JSON.stringify(snapshot)};
      const threads = ${JSON.stringify(threads)};
      const answer = (channel, fallback) => async (p) => threads[p?.product + '/' + p?.id]?.[channel] ?? fallback;
      const bridge = {
        ...window.zero,
        itemHistory: answer('zero:item-history', { ok: true, lines: [] }),
        sessionTrace: answer('zero:session-trace', { ok: true, sessions: [] }),
        codeChange: answer('zero:code-change', { ok: false, error: 'no change was recorded for this thread' }),
        compactionStatus: answer('zero:compaction-status', null),
        // Which pictures a folder holds is the one thing a page cannot see for itself.
        ...(window.__FOLDER_PICTURES__ ? { folderPictures: window.__FOLDER_PICTURES__ } : {}),
      };
      window.__UNANSWERED__ = [];
      window.zero = new Proxy(bridge, {
        get(target, name) {
          if (typeof name === 'string' && !(name in target) && !window.__UNANSWERED__.includes(name)) window.__UNANSWERED__.push(name);
          return target[name];
        },
      });
    })()`,
  });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(2000);
  // A thread an agent is on is In progress, the only row there.
  if (fromTab) await tab(h, fromTab);
  else if (session) await tab(h, 'In progress');
  else await toInbox(h);
  await h.call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await h.call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
  await h.wait(1500);
  // A thread opens on its newest answer, with the rest folded behind one line.
  if (unfold) {
    await h.evaluate(`document.querySelector('.one-earlier')?.click()`);
    await h.wait(600);
  }
  const did = then ? (await then(h)) ?? {} : {};
  if (then) await h.wait(500);
  const unanswered = await h.evaluate('window.__UNANSWERED__');
  if (unanswered.length) console.error('thread scene: the page asked the bridge for', unanswered.join(', '), 'and got nothing');
  // `unfolded` tells the native window the same thing the click above told the page.
  // And whatever `then` did, for the native window to do the same.
  // `arriving` says that no picture had come when the photograph was taken.
  // `thread` is the one open, for a picture of a thread that is not the first in Needs you.
  return { snapshot, threads, unfolded: unfold ? Object.keys(threads) : [], ...(files?.arriving ? { arriving: true } : {}), ...(session || fromTab ? { thread: row.id } : {}), ...did };
}

// Presses the folded line of the run at `index` among those on the page (`.did-run-head`), and
// says so for the native window, which opens the same one (`runsOpen`).
export async function openRun(h, index) {
  await h.evaluate(`document.querySelectorAll('.did-run-head')[${index}]?.click()`);
  await h.wait(400);
  return { runsOpen: [index] };
}

export const go = (h) => open(h, { unfold: true });
