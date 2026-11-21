// Invented threads, one for each thing the list rules decide: who filed it, who spoke last,
// what the supervisor says about it, where it was put. `tools/rows.mjs --cases` feeds them to
// the real web window and to the native one, so the two are held against each other on every
// branch and not only on the ones a tidy inbox happens to reach.
//
// Every title says which case it is. Nothing here is anyone's real thread.

const min = 60_000;
const hour = 60 * min;
const day = 24 * hour;

export function snapshot(now) {
  const products = [
    { slug: 'shop', name: 'Storefront' },
    { slug: 'desk', name: 'Front desk' },
    { slug: 'lab', name: 'Lab' },
    { slug: 'practice', name: 'Practice', practice: true },
  ];
  const nameOf = (slug) => products.find((p) => p.slug === slug).name;
  const items = [];
  const by = (source, ts) => ({ ts, source });

  // A thread, written `ago` before now. `who` filed it: the person (the founder tag) or an agent.
  const thread = (title, { product = 'shop', ago = 3 * hour, who = 'founder', ...fields } = {}) => {
    const at = now - ago;
    const item = {
      id: `w-case${String(items.length).padStart(2, '0')}`, product, productName: nameOf(product), status: 'open', kind: 'directive',
      title, body: title, labels: who === 'founder' ? ['founder'] : [], priority: 0, epoch: 1, claim: null, claimExpired: false,
      createdAt: at, updatedAt: at, createdBy: 'p-you',
      wrote: { title: by(who, at), body: by(who, at), status: by(who, at), kind: by(who, at), labels: by(who, at) },
      ...fields,
    };
    items.push(item);
    return item;
  };
  // What happened to a thread afterwards, each `ago` before now.
  const set = (item, field, value, source, ago) => {
    const ts = now - ago;
    item[field] = value;
    item.wrote[field] = by(source, ts);
    item.updatedAt = Math.max(item.updatedAt, ts);
    return item;
  };
  const answered = (item, words, ago = hour) => set(item, 'result', words, 'agent', ago);
  const replied = (item, words, ago = 30 * min) => set(item, 'answer', words, 'founder', ago);
  const status = (item, value, source, ago = 50 * min) => set(item, 'status', value, source, ago);
  const offer = 'It is built and every test passes.\n\n## Options\n1. Ship it (recommended)\n2. Change the copy first';

  /* ---------- finished threads: news, or the archive ---------- */
  status(answered(thread('Done by its agent on your thread: the checkout total rounds wrong', { priority: 9 }), 'Tax rounding fixed and tested.'), 'done', 'agent');
  status(answered(thread('Done by its agent, medium, newer', { ago: 2 * hour }), 'Fixed.', 40 * min), 'done', 'agent', 35 * min);
  status(answered(thread('Done by its agent, medium, older', { ago: 5 * hour }), 'Fixed.', 4 * hour), 'done', 'agent', 4 * hour);
  status(answered(thread('Done by its agent in the second project, urgent', { product: 'desk', priority: 9 }), 'Fixed.'), 'done', 'agent');
  status(answered(thread('Done by its agent in a project never placed, high', { product: 'lab', priority: 7 }), 'Fixed.'), 'done', 'agent');
  status(answered(thread('Done by its agent, low', { priority: 2 }), 'Fixed.'), 'done', 'agent');
  status(answered(thread('Done by its agent, level eight reads High', { priority: 8 }), 'Fixed.'), 'done', 'agent');
  status(answered(thread('Done by its agent, level three reads Medium', { priority: 3 }), 'Fixed.'), 'done', 'agent');
  status(answered(thread('Done and archived by you: goes to Done'), 'Fixed.'), 'done', 'founder', 20 * min);
  status(answered(thread('Done, an agent filed it and you never touched it: goes to Done', { who: 'agent' }), 'Noted.'), 'done', 'agent');
  status(replied(answered(thread('Done, an agent filed it and you answered: news', { who: 'agent' }), 'Which one?', 2 * hour), 'The first', hour), 'done', 'agent', 30 * min);
  status(replied(answered(thread('Done after you asked for the close: goes to Done'), 'Fixed.', 2 * hour), 'Looks right, close this task', hour), 'done', 'agent', 30 * min);
  status(answered(thread('Done long ago: the date is written', { ago: 50 * day }), 'Fixed.', 45 * day), 'done', 'founder', 40 * day);
  status(answered(thread('Done yesterday or the day before', { ago: 60 * hour }), 'Fixed.', 55 * hour), 'done', 'founder', 50 * hour);
  set(status(answered(thread('Done and shipped by the app'), 'Fixed.', 2 * hour), 'done', 'founder', hour), 'note', 'Shipped to main as 0a1b2c3.', 'system', 30 * min);
  set(status(answered(thread('Done, closed without its work reaching main'), 'Fixed.', 2 * hour), 'done', 'founder', hour), 'note', 'Closed without its work reaching main.', 'system', 30 * min);
  status(answered(thread('Done, marked shipped by you', { shippedAt: now - hour }), 'Fixed.', 2 * hour), 'done', 'founder', hour);
  status(answered(thread('Done, a clean run of a repeating task: goes to Done', { labels: ['founder', 'repeat:r-1', 'clean'] }), 'Nothing to report.'), 'done', 'agent');
  status(answered(thread('Done, a run of a repeating task with news', { labels: ['founder', 'repeat:r-1'] }), 'Two bookings failed overnight.'), 'done', 'agent');

  // Thirty more, an hour apart: Home shows the first eighty, and the times run from hours ago
  // through Yesterday to a date.
  for (let k = 0; k < 30; k += 1) status(answered(thread(`One of thirty more finished threads, number ${k + 1}`, { ago: (6 + 2 * k) * hour }), 'Fixed.', (5 + 2 * k) * hour), 'done', 'agent', (5 + 2 * k) * hour);
  status(answered(thread('Done, shipped by the app, and you said it did not', { shippedAt: 0 }), 'Fixed.', 2 * hour), 'done', 'founder', hour);
  items.at(-1).note = 'Shipped to main as 0a1b2c3.';
  items.at(-1).wrote.note = by('system', now - 30 * min);

  /* ---------- open threads the agent answered ---------- */
  answered(thread('Answered and left open: says Answered'), 'The export writes one file per clinic now.');
  answered(thread('Answered with a question: says Waiting on you'), 'I can do either. Which clinic should go first?');
  answered(thread('Answered with options: says Answered'), offer);
  replied(answered(thread('Replied to after the answer: in progress'), 'Fixed.', 2 * hour), 'Now do the second clinic');
  set(replied(answered(thread('Your reply was acted on and the thread left open: news'), 'Fixed.', 3 * hour), 'Now do the second clinic', 2 * hour), 'answeredThrough', now - 2 * hour, 'system', hour);
  items.at(-1).wrote.result = by('agent', now - hour);
  answered(replied(thread('The app gave up carrying your reply: says Stopped'), 'Try again', 2 * hour), 'Your reply did not reach this task. The agent ran once and finished without writing anything back on the row, so Mercury has stopped carrying it. Reply again and it goes to a fresh session.', hour);
  answered(thread('A dead run wrote its own word: says Stopped'), 'Claude Code could not sign in, so nothing has been done on this task.');
  set(answered(thread('A ship that failed: says Not shipped'), 'Shipping it now.', 2 * hour), 'note', 'It did not ship, and it went back to its agent: the tests on main are red', 'system', hour);
  set(answered(thread('Shipped by the app and left open: says Shipped'), 'Shipping it now.', 2 * hour), 'note', 'Shipped to main as 0a1b2c3.', 'system', hour);
  answered(thread('A cloud run asks for a sign-in: says Needs Vercel'), 'The preview is built.\nAGENTBOX-NEEDS: sign-in vercel | to deploy the preview');
  answered(thread('A cloud run asks for two things'), 'AGENTBOX-NEEDS: key SENTRY_AUTH_TOKEN | a Sentry auth token\nAGENTBOX-NEEDS: code npm | the one-time password');
  answered(thread('Answered after you last looked: unread'), 'Fixed, a moment from now.', -hour);
  status(answered(thread('Blocked by its agent: needs you'), 'I cannot go on without the print shop.'), 'blocked', 'agent');
  status(thread('Blocked by you and never answered'), 'blocked', 'founder');

  /* ---------- names ---------- */
  answered(thread('The label names this thread, not this long opening sentence that goes on', { label: 'Label wins' }), 'Fixed.');
  set(answered(thread('Renamed by you after the label', { label: 'The old label' }), 'Fixed.'), 'title', 'Renamed by you after the label', 'founder', 10 * min);
  items.at(-1).wrote.label = by('system', now - 2 * hour);
  answered(thread('A name you typed is kept to the letter', { label: 'A name you typed, however long it runs on, is kept to the letter' }), 'Fixed.');
  items.at(-1).wrote.label = by('founder', now - 2 * hour);
  answered(thread('Hey, could you look at why the reminder emails for the Lisbon clinics go out twice on Mondays?'), 'Fixed.');
  answered(thread('ok então o botão de exportar não faz nada quando clico duas vezes'), 'Corrigido.');
  answered(thread('![shot.png](attachments/shot.png)'), 'Seen.');

  /* ---------- engines ---------- */
  answered(thread('On Codex by its engine', { engine: 'codex' }), 'Fixed.');
  answered(thread('On Codex by its labels', { labels: ['founder', 'codex', 'codex:abc123'] }), 'Fixed.');
  answered(thread('On Antigravity', { engine: 'antigravity' }), 'Fixed.');
  const routed = answered(thread('On the engine the app answers for it'), 'Fixed.');

  /* ---------- where you put it ---------- */
  answered(thread('Pinned, low, in a project never placed', { product: 'lab', priority: 2, pinned: true }), 'Fixed.');
  answered(thread('Dragged above its project', { product: 'lab', priority: 2, place: 207.5 }), 'Fixed.');
  status(answered(thread('Pinned and done: the pin is not drawn', { pinned: true }), 'Fixed.'), 'done', 'agent');
  set(answered(thread('Put off by you until tomorrow: Later'), 'Fixed.'), 'runAt', now + day, 'founder', 30 * min);
  set(answered(thread('A reminder you set, then answered past: back'), 'Fixed.', 3 * hour), 'snoozedUntil', now + day, 'founder', 2 * hour);
  replied(items.at(-1), 'Never mind, go on', hour);
  thread('Written down in Later and not begun', { start: 'later' });
  thread('Written down in Later, with a comment on its link: says not started', { start: 'later', linkComments: [{ name: 'Ines', file: 'plan.md', text: 'When does this start?', ts: now - 5 * min, place: { kind: 'time', s: 75.4 } }] });

  /* ---------- Later: what comes back, and when its row says it does ---------- */
  set(answered(thread('Put off by you until next week: its day is dated'), 'Fixed.'), 'runAt', now + 9 * day, 'founder', 30 * min);
  set(answered(thread('A reminder you set for three hours from now'), 'Fixed.'), 'snoozedUntil', now + 3 * hour, 'founder', 30 * min);
  thread('Pinned and written down in Later: first', { start: 'later', pinned: true });
  thread('Written down in Later, urgent', { start: 'later', priority: 9 });
  // Two put off in the same minute, so nothing but the moment each comes back tells them apart.
  set(answered(thread('Put off until the day after tomorrow, in the same minute as the next'), 'Fixed.'), 'runAt', now + 2 * day, 'founder', 31 * min);
  set(answered(thread('Put off until tomorrow, in the same minute as the last: comes first'), 'Fixed.'), 'runAt', now + day + hour, 'founder', 31 * min);
  const putOffQueued = set(answered(thread('Put off by you with a run still queued for it: not in Later yet'), 'Fixed.'), 'runAt', now + day, 'founder', 30 * min);
  const laterStalled = thread('Written down in Later, and its worker died: says Stopped', { start: 'later' });
  status(answered(thread('Done by its agent while still written down in Later: Later and Done', { start: 'later' }), 'Fixed.'), 'done', 'agent');

  /* ---------- questions, reviews and proposals ---------- */
  thread('A question from an agent, unanswered', { who: 'agent', kind: 'question', body: 'Which clinic first?\n\n## Options\n1. Riverside (recommended)\n2. Northgate' });
  replied(thread('A question you answered: in progress', { who: 'agent', kind: 'question' }), 'Riverside');
  thread('A review from an agent, unanswered', { who: 'agent', kind: 'review', priority: 7 });
  set(thread('A question its agent parked: still asks, says Scheduled', { who: 'agent', kind: 'question' }), 'runAt', now + 2 * hour, 'agent', hour);
  thread('Work an agent filed under no thread: waits for a yes', { who: 'agent' });
  replied(thread('Work an agent filed that you approved: in progress', { who: 'agent' }), 'Approved. Run it.');
  const parent = answered(thread('A thread with two tasks to approve'), 'Two follow-ups are filed under this.');
  thread('A proposal under that thread', { who: 'agent', parent: parent.id, ago: 2 * hour });
  thread('A second proposal under that thread', { who: 'agent', parent: parent.id, ago: hour });
  const closedParent = status(answered(thread('A thread you closed, with a proposal a day old'), 'Filed one follow-up.', 3 * day), 'done', 'founder', 3 * day);
  thread('A proposal under the closed thread', { who: 'agent', parent: closedParent.id, ago: 2 * day });
  const owedParent = status(answered(thread('A thread its agent finished long ago, owed an answer on its proposal', { ago: 4 * day }), 'Filed one follow-up.', 3 * day), 'done', 'agent', 3 * day);
  thread('A proposal two days old', { who: 'agent', parent: owedParent.id, ago: 2 * day });

  /* ---------- waits ---------- */
  const blocker = thread('The thread others wait on', { ago: 20 * min });
  set(status(answered(thread('Waits on another thread, by its agent: Later'), 'Waiting for the other one.'), 'blocked', 'agent'), 'blockedBy', [blocker.id], 'agent', 50 * min);
  set(answered(thread('Waits on another thread because you said so: Later'), 'Fixed.'), 'blockedBy', [blocker.id], 'founder', 50 * min);
  set(status(answered(thread('Waits on a thread that is gone: needs you'), 'Waiting.'), 'blocked', 'agent'), 'blockedBy', ['w-gone'], 'agent', 50 * min);
  set(status(answered(thread('Waits on the print shop, in plain words: Later'), 'Waiting.'), 'blocked', 'agent'), 'blockedBy', ['name:the print shop'], 'agent', 50 * min);
  answered(thread('What it waited on has ended: back, and says so', { waitEnded: { said: '“The other thread” finished', at: now - 10 * min } }), 'Fixed.');
  answered(thread('Wait for the export job', { waitEnded: { said: '“Wait for the export job” finished', at: now - 10 * min } }), 'Fixed.');
  answered(thread('Handed over from the cloud, and says so', { handedOver: { said: 'Carried on from the cloud run', at: now - 10 * min } }), 'Fixed.');
  const cause = thread('GitHub is signed out', { who: 'system', kind: 'cause', labels: ['cause'], product: 'desk' });
  for (const product of ['shop', 'shop', 'lab']) set(status(thread(`Stuck behind the cause, in ${nameOf(product)}`, { product }), 'blocked', 'agent'), 'blockedBy', [cause.id], 'agent', 40 * min);

  /* ---------- other people's words ---------- */
  answered(thread('Someone commented on its link: says Waiting on you', { linkComments: [{ name: 'Ines', file: 'plan.md', text: 'Is the second step needed?', ts: now - 5 * min }] }), 'Fixed.');
  status(answered(thread('Two comments on a closed thread: back', { linkComments: [{ name: 'Ines', file: 'plan.md', text: 'One.', ts: now - 9 * min }, { name: 'Rui', file: 'notes.md', text: 'Two.', ts: now - 5 * min }] }), 'Fixed.'), 'done', 'founder');

  /* ---------- what the supervisor says ---------- */
  const running = thread('Running on this Mac', { ago: 10 * min, status: 'claimed', claim: { holder: 'mcp-1' } });
  const runningCloud = thread('Running in the cloud', { ago: 12 * min, status: 'claimed', claim: { holder: 'mcp-cloud-2' } });
  const runningKept = thread('Running here while the cloud takes everything', { ago: 14 * min, status: 'claimed', claim: { holder: 'mcp-3' } });
  const runningAsks = thread('A question whose agent is still running: both lists', { who: 'agent', kind: 'question', status: 'claimed', claim: { holder: 'mcp-4' } });
  const runningDone = status(answered(thread('Done written, session not yet exited: in progress'), 'Fixed.'), 'done', 'agent');
  const starting = answered(thread('A run being started again: in progress'), 'Fixed.');
  const asking = thread('Its agent stopped on a permission request: says Asking', { ago: 15 * min, status: 'claimed', claim: { holder: 'mcp-5' } });
  thread('Waits on a sign-in link: says Asking', { ago: 16 * min, signIn: { id: 's-1', service: 'Vercel', url: 'https://example.com/login' } });
  const queued = thread('Queued', { ago: 17 * min });
  const next = thread('Queued and pushed to the front', { ago: 18 * min, priority: 9 });
  const asleep = thread('Queued, but this computer looks asleep', { ago: 19 * min });
  const limited = thread('Its tool is at its usage limit', { ago: 21 * min });
  const shipping = answered(thread('The app is shipping it'), 'Shipping.');
  const stalled = thread('Its worker died: says Stopped', { ago: 22 * min });
  const signedOut = thread('Its tool is signed out', { ago: 23 * min });
  const silent = thread('A run ended and wrote nothing', { ago: 24 * min, who: 'agent' });
  thread('Nothing is on it and nothing is coming', { ago: 25 * min });
  thread('Sent a moment from now: reads Queued', { ago: -10 * min, epoch: 0 });
  thread('Claimed by an agent outside this window', { ago: 26 * min, status: 'claimed', claim: { holder: 'mcp-9' } });
  thread('Claimed by a cloud run outside this window', { ago: 26 * min, status: 'claimed', claim: { holder: 'mcp-cloud-9' } });
  thread('Claimed, but the claim has run out', { ago: 26 * min, status: 'claimed', claim: { holder: 'mcp-8' }, claimExpired: true });
  const moving = thread('Running here, on its way to the cloud', { ago: 13 * min, status: 'claimed', claim: { holder: 'mcp-6' } });
  const stalledWithWords = thread('Its worker died and someone commented: the solid dot', { ago: 22 * min, linkComments: [{ name: 'Rui', file: 'notes.md', text: 'Any news?', ts: now - 5 * min }] });
  thread('Pinned and waiting its turn: first under In progress', { ago: 5 * hour, pinned: true });
  thread('Waiting its turn in the second project, urgent', { ago: 6 * hour, product: 'desk', priority: 9 });
  thread('Waiting its turn in a project never placed, urgent', { ago: 7 * hour, product: 'lab', priority: 9 });
  thread('Sent to Auto, which asks who takes it', { ago: 28 * min, labels: ['founder', 'auto'], routed: { ask: ['t-1', 't-2'] } });
  thread('In the practice project, where nothing starts', { ago: 27 * min, product: 'practice' });
  thread('Sent to Auto, being assigned', { ago: 28 * min, labels: ['founder', 'auto'] });
  thread('A chat just brought in, being written up', { ago: 29 * min, who: 'agent', labels: ['catch-up', 'thread-import'], kind: 'question' });
  answered(thread('An imported Codex conversation, not opened yet: on top', { product: 'lab', priority: 2, labels: ['founder', 'codex', 'codex:def456'], engine: 'codex' }), 'Imported.');
  set(answered(thread('Its agent runs it again tomorrow: Later'), 'Checked. I will look again tomorrow.'), 'runAt', now + day, 'agent', 30 * min);
  set(answered(thread('Its agent set a moment that has come: in progress'), 'Checked.', 3 * hour), 'runAt', now - hour, 'agent', 2 * hour);

  /* ---------- a task asked for in a chat ---------- */
  const chat = thread('The chat a task was asked in', { ago: 2 * hour });
  answered(thread('A task asked for in that chat: the chat is listed instead', { who: 'agent', labels: [`asked-in:${chat.product}/${chat.id}`], ago: hour }), 'Here is the answer.', 30 * min);

  return {
    products, items,
    approvals: [
      { id: 'a-1', at: now - 4 * min, product: asking.product, item: asking.id, tool: 'Bash', input: { command: 'npm test', description: 'Run the tests' } },
      { id: 'a-2', at: now - 3 * min, product: null, item: null, tool: 'Bash', input: { command: 'ls', description: 'List the folder' } },
    ],
    engines: { workspace: 'claude', byItem: { [routed.id]: 'antigravity' } },
    supervisor: {
      paused: false, pausedProducts: [], scheduled: [], hiddenProducts: [], personalProducts: [], capacity: 4,
      productOrder: ['shop', 'desk'],
      running: [
        { itemId: running.id, product: running.product, startedAt: now - 5 * min, engine: 'claude' },
        { itemId: runningCloud.id, product: runningCloud.product, startedAt: now - 5 * min, where: 'cloud', engine: 'codex' },
        { itemId: runningKept.id, product: runningKept.product, startedAt: now - 5 * min },
        { itemId: runningAsks.id, product: runningAsks.product, startedAt: now - 5 * min },
        { itemId: runningDone.id, product: runningDone.product, startedAt: now - 5 * min },
        { itemId: asking.id, product: asking.product, startedAt: now - 5 * min },
        { itemId: moving.id, product: moving.product, startedAt: now - 5 * min },
      ],
      starting: [starting.id],
      queued: [queued.id, next.id, asleep.id, putOffQueued.id],
      runNow: [next.id],
      cannotStart: { [asleep.id]: { why: 'asleep', tool: 'Claude Code' } },
      atLimit: { [limited.id]: { tool: 'Claude Code', until: now + 2 * hour } },
      shipping: [shipping.id],
      stalled: [stalled.id, signedOut.id, stalledWithWords.id, laterStalled.id],
      signInNeeded: { [signedOut.id]: 'Claude Code' },
      silent: { [silent.id]: { runs: 1, endedAt: now - 20 * min } },
      whyNotCloud: { [runningKept.id]: { at: 'mac', why: 'needs-mac' }, [moving.id]: { at: 'mac', why: 'moving' } },
      spawnTrouble: {
        since: now - 3 * hour, cause: 'mixed', message: '', remedy: '', retryAt: now + hour, count: 2,
        byCause: [{ cause: 'signed-out', count: 1, what: 'Claude Code is signed out', next: 'Sign in again.', act: null, hers: true }, { cause: 'interrupted', count: 1, what: 'A run was cut off', next: 'It retries.', act: null, hers: false }],
        ids: [stalled.id, signedOut.id],
      },
    },
    restartNeeded: null, config: {},
  };
}
