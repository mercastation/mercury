// Settings, as the web window draws it, for a Mac set up the way the person who uses this most
// has theirs: Full access everywhere, nine agents at once on Automatic, the memory check on, the
// leftovers swept, two Claude Code accounts, Codex on Auto, Antigravity signed in on Full access,
// a cloud of their own (plan "self") that takes threads when this Mac is full, the phone seeing
// every project, three projects and two archived, no instructions, every alpha off. Every name,
// address and folder here is invented.
//
// The page reads its settings with its own calls, not off the snapshot (`zero:settings-read`,
// `zero:account-usage`, `zero:cloud-account-plan` and the rest: `components/Settings.tsx`), so
// this answers them in the page and hands the same answers on in the fixture (`settings`,
// `answers`), so the native window is told the same. Every write is refused: this is a picture.
//
// A scene names the page to open (`open(h, 'running')`), and may type into the search, press a
// control, or leave the pointer on one (`then`).

let at = Date.now();
const MIN = 60_000;
const HOUR = 60 * MIN;
const DAY = 24 * HOUR;

const HOME = '/Users/sam';
const STORE = `${HOME}/Library/Application Support/Mercury/accounts/3f2a8c1e-7b4d-4e9a-9c11-5d0e6f7a8b9c`;

export const PROJECTS = [
  { slug: 'storefront', name: 'Storefront', repo: `${HOME}/dev/storefront`, running: 2 },
  { slug: 'mobile', name: 'Mobile', repo: `${HOME}/dev/mobile`, running: 0 },
  { slug: 'docs', name: 'Docs site', repo: `${HOME}/dev/docs-site`, running: 0 },
];

const limits = (session, week, model) => [
  { span: 'session', qualifier: null, name: 'Current session', percent: session, resetsAt: at + 3 * HOUR + 12 * MIN, resetsText: null, resetsOn: null, zone: null },
  { span: 'week', qualifier: 'all models', name: 'Week (all models)', percent: week, resetsAt: at + 3 * DAY + 5 * HOUR, resetsText: null, resetsOn: null, zone: null },
  { span: 'week', qualifier: 'Fable', name: 'Week (Fable)', percent: model, resetsAt: at + 3 * DAY + 5 * HOUR, resetsText: null, resetsOn: null, zone: null },
];

/** The settings model the page reads, as main answers `zero:settings-read`. */
export function settings() {
  const engines = [
    { id: 'claude', label: 'Claude Code', word: 'Claude Code' },
    { id: 'codex', label: 'Codex', word: 'Codex' },
    { id: 'antigravity', label: 'Antigravity', word: 'Antigravity' },
  ];
  return {
    ok: true,
    archivedProjects: [
      { slug: 'old-landing', name: 'Old landing', dir: `${STORE}/old-landing` },
      { slug: 'zine', name: 'Zine', dir: `${STORE}/zine` },
    ],
    workspace: {
      phoneAll: true, cloudPlan: 'self', agentsRunning: true, productOrder: PROJECTS.map((p) => p.slug),
      sessionsAtOnce: 9, agentsAuto: true, agentsTotal: 9, agentsNudge: 0, agentsAutoTotal: 9, slotsSuggested: 6, slotsMax: 12, machineNote: null,
      memoryGate: {
        on: true, slots: null, slotsAuto: 3, slotsMax: 12, now: 'Nothing is waiting on memory now.',
        reading: { heavy: null, waiting: null, pressure: null, usedPct: null, asked: null, waited: null, refused: null, holding: false },
      },
      leftovers: { on: true, now: 'Nothing is left running now.' },
      capacity: 9, running: 2, model: null, engine: 'claude', engineChoices: engines,
      codex: {
        bin: '/opt/homebrew/bin/codex', found: true, certain: true, url: 'https://developers.openai.com/codex/cli', chosen: false, trouble: null,
        account: { email: 'sam@example.com', name: 'Sam Rivera', plan: 'plus', planNamed: true, mode: 'chatgpt', accountId: 'acct-0000' },
        accounts: [{ profile: 'default', email: 'sam@example.com', name: 'Sam Rivera', plan: 'plus', planNamed: true, accountId: 'acct-0000', signedIn: true, chosen: true }],
      },
      codexModels: [], codexModelDefault: 'gpt-6-astra', codexModel: null, codexMode: 'auto',
      antigravity: {
        bin: `${HOME}/.antigravity/bin/agy`, found: true, certain: true, url: 'https://antigravity.google/cli', chosen: false,
        signedIn: true, email: 'sam@example.com', mode: 'full-access', cloudMode: 'full-access',
      },
      antigravityModels: [],
      permission: 'bypassPermissions', permissionArgs: ['--permission-mode', 'bypassPermissions'], cloudPermission: 'bypassPermissions', cloudCodexMode: 'full-access',
      files: { on: true, offAfterDays: 30 },
      diagnostics: true, diagnosticsDestination: false, adhdMode: false, importNew: false,
      alpha: { rooms: false, teammates: false, multiplayer: false, glass: false },
      cornerTag: true, outsideAgents: 'off',
      accounts: [
        { profile: 'default', email: 'sam@example.com', accountUuid: '00000000-0000-4000-8000-000000000001', chosen: false, label: 'default', dir: null, cooldownUntil: 0, live: true, state: 'live', trouble: null, running: 1, plan: { label: 'Max', max: true } },
        { profile: 'studio', email: 'sam@studio.example', accountUuid: '00000000-0000-4000-8000-000000000002', chosen: true, label: 'studio', dir: `${HOME}/.claude-studio`, cooldownUntil: 0, live: true, state: 'live', trouble: null, running: 1, plan: { label: 'Max', max: true } },
      ],
      accountsNote: null,
      storePath: STORE, homePath: HOME,
      claudeBin: `${HOME}/.local/bin/claude`, claudeFound: true, claudeCertain: true, claudeInstallUrl: 'https://claude.com/claude-code', claudeChosen: false,
      codexFound: true, codexCertain: true, codexInstallUrl: 'https://developers.openai.com/codex/cli',
      claudeModels: [], standingLines: 0, messageRulesLines: 0, projectsWithInstructions: 0, projectCount: PROJECTS.length,
    },
    projects: PROJECTS.map((p, i) => ({
      slug: p.slug, name: p.name, dir: `${STORE}/${p.slug}`, repoPath: p.repo, logo: null, createdAt: at - (40 - i) * DAY,
      autonomous: false, onPhone: true, permission: 'workspace', permissionArgs: null, codexMode: 'workspace', baseBranch: null,
      browserSites: [], account: 'default', instructions: '', running: p.running,
    })),
  };
}

/** What the page's other reads are answered with, by the bridge's own names. */
export function answers() {
  return {
    accountUsage: { readings: [
      { profile: 'default', limits: limits(18, 46, 12), at: at - 4 * MIN },
      { profile: 'studio', limits: limits(64, 87, 31), at: at - 4 * MIN },
    ] },
    codexUsage: { readings: [{ profile: 'default', limits: [
      { span: 'session', qualifier: null, name: '5 hours', percent: 9, resetsAt: at + 2 * HOUR + 40 * MIN, resetsText: null, resetsOn: null, zone: null },
      { span: 'week', qualifier: null, name: 'Week', percent: 22, resetsAt: at + 5 * DAY + 2 * HOUR, resetsText: null, resetsOn: null, zone: null },
    ], at: at - 9 * MIN }] },
    refreshUsage: { ok: false },
    filesCloud: { cap: 0, bytes: 0, full: false, on: true, offAfterDays: 30, offBytes: 0 },
    cloudAccountPlan: { signedIn: true, plan: 'self', unlimited: false, agents: 2, hoursPerMonth: 1000000, running: 0, usedSeconds: 40715, billing: 'given', trial: null, trialOffer: null, team: null },
    fileLinks: { ok: true, site: 'https://phone.example.com', links: [] },
    projectBranches: { main: 'main', branches: ['main', 'release', 'design-pass'] },
    projectBrowser: { state: 'none' },
  };
}

/** The snapshot, with the cloud, the person signed in, the version and the usage on it. */
export function dress(snapshot, now) {
  at = now ?? Date.now();
  const products = PROJECTS.map((p) => ({ slug: p.slug, name: p.name, dir: `/fixtures/${p.slug}`, repoPath: p.repo }));
  const cloudAccount = (engine, key, name, plan, inUse, profile, reading) => ({
    engine, key, name, plan, ready: true, state: 'ready', savedAt: at - 9 * DAY, limitedUntil: null,
    limits: reading, readAt: reading ? at - 6 * MIN : null, duplicateOf: null, profile, readError: null, inUse,
  });
  return {
    ...snapshot,
    products,
    supervisor: { ...snapshot.supervisor, productOrder: products.map((p) => p.slug) },
    engines: { choices: settings().workspace.engineChoices, workspace: 'claude', byItem: {} },
    // Multiplayer off keeps the sign-in for the phone alone, so no page asks who sees a project.
    alpha: { rooms: false, teammates: false, multiplayer: false, glass: false },
    team: {
      configured: false, canSignIn: true, started: true, settling: false, signedIn: true, signingIn: null,
      me: { id: 'me-0000', email: 'sam@example.com', name: 'Sam Rivera', avatarUrl: null, status: null },
      team: null, invites: [], sent: [], people: [], cards: [], lastSyncAt: at - MIN, error: null,
      phoneSite: 'https://phone.example.com', phoneSiteQr: null, since: null, multiplayer: null, teamMates: [], seen: {},
      phoneError: null, phoneSentAt: at - MIN, sharedFileNews: [], threadTalk: [],
    },
    cloud: {
      setUp: true, offered: false, managed: true, mode: 'full', whenOn: 'full', running: 0,
      month: { runs: 32, agentHours: 10.9, measuredHours: 10.9, estimatedCost: 0.14, costPerAgentHour: 0.0128 },
      health: 'ok', lastError: null, restingUntil: null, switched: null, noRoom: null, versionGap: null,
      accounts: [
        cloudAccount('claude', 'c-claude-1', 'sam@example.com', 'Max', false, 'default', limits(18, 46, 12)),
        cloudAccount('claude', 'c-claude-2', 'sam@studio.example', 'Max', true, 'studio', limits(64, 87, 31)),
        cloudAccount('antigravity', 'c-agy-1', 'sam@example.com', null, true, null, null),
      ],
      keys: [], notInCloud: [], noCloudSignIn: ['codex'], signIns: [], kinds: ['claude', 'codex', 'antigravity'],
      projects: products.map((p) => p.slug), plan: 'self', movable: [],
    },
    update: {
      canMove: false, place: null, moveError: null, phase: 'current', currentVersion: '0.1.22', newVersion: null,
      percent: null, error: null, checkedAt: at - 20 * MIN, readyAt: null, ready: false, source: true, changes: [], behind: null, installing: false,
    },
    usage: { engine: 'claude', profile: 'studio', limits: limits(64, 87, 31), at: at - 4 * MIN },
    usageByEngine: [
      { engine: 'claude', profile: 'studio', limits: limits(64, 87, 31), at: at - 4 * MIN },
      { engine: 'codex', profile: 'default', limits: answers().codexUsage.readings[0].limits, at: at - 9 * MIN },
    ],
  };
}

/** The bridge the page's own reads go through, answered as above; every write refused. */
function bridge() {
  const a = answers();
  return `(() => {
    const settings = ${JSON.stringify(settings())};
    const a = ${JSON.stringify(a)};
    const no = async () => { throw new Error('read-only shot bridge'); };
    Object.assign(window.zero, {
      settingsRead: async () => settings,
      accountUsage: async (p) => (p && p.engine === 'codex' ? a.codexUsage : a.accountUsage),
      refreshUsage: async () => a.refreshUsage,
      filesCloud: async () => a.filesCloud,
      cloudAccountPlan: async () => a.cloudAccountPlan,
      cloudAccountTeamBilling: async () => ({ ok: false }),
      fileLinks: async () => a.fileLinks,
      projectBranches: async () => a.projectBranches,
      projectBrowser: async () => a.projectBrowser,
      instructionRead: async () => ({ ok: true, text: '' }),
      standingRead: async () => ({ ok: true, text: '' }),
      messageRulesRead: async () => ({ ok: true, text: '' }),
      setWorkspaceSetting: no, setProjectSetting: no, updateCheck: no, updateInstall: no,
      agentUpdate: no, claudeAddAccount: no, codexAddAccount: no, antigravitySignIn: no, antigravityRefresh: no,
      claudeRecheck: no, codexRecheck: no, antigravityRecheck: no, engineBin: no, terminal: no,
      projectRename: no, projectArchive: no, projectIcon: no, projectIconClear: no, setProductOrder: no,
    });
  })();`;
}

/** The labels of the menu's rows, by the page ids the native window opens them by. */
export const LABELS = {
  general: 'General', appearance: 'Appearance', shortcuts: 'Shortcuts', phone: 'Phone', alpha: 'Alpha features',
  claude: 'Claude Code', codex: 'Codex', antigravity: 'Antigravity', running: 'Running', instructions: 'Instructions',
  projects: 'Projects', files: 'Files', cloud: 'Plan and Billing',
};

/**
 * Settings opened from the rail, on `page` (a page id, or `project:<slug>` for a project's own
 * page), the page loaded again on the bridge above. `then(h)` does the rest and may hand back
 * more for the fixture.
 */
export async function open(h, page = 'general', { then } = {}) {
  await h.call('Page.addScriptToEvaluateOnNewDocument', { source: bridge() });
  await h.goto(h.origin);
  await h.evaluate('(async () => { await document.fonts.ready; })()');
  await h.wait(1600);
  await h.evaluate(`document.querySelector('.workspace-navigation button[aria-label="Settings"]').click()`);
  await h.wait(700);
  const [first, slug] = page.split(':');
  await h.evaluate(`(() => {
    const want = ${JSON.stringify(LABELS[first] ?? first)};
    const row = [...document.querySelectorAll('.set-nav-item')].find((b) => b.textContent.trim() === want);
    row?.click();
  })()`);
  await h.wait(700);
  if (slug) {
    await h.evaluate(`(() => {
      const row = [...document.querySelectorAll('.pp-row')].find((r) => r.querySelector('.pp-name')?.textContent.trim() === ${JSON.stringify(PROJECTS.find((p) => p.slug === slug)?.name ?? slug)});
      row?.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerId: 1 }));
      row?.dispatchEvent(new PointerEvent('pointerup', { bubbles: true, button: 0, pointerId: 1 }));
    })()`);
    await h.wait(700);
  }
  const more = then ? (await then(h)) ?? {} : {};
  return { settingsPage: page, settings: settings(), answers: answers(), ...more };
}

/** The key the native window names each picker by, from the label the web window gives it. */
const PICKERS = {
  'How many run at once': 'at-once', 'Agents you started yourself': 'outside',
  'Permission mode': 'mode', 'Permission mode in the cloud': 'cloud-mode',
  'Claude account': 'project-account', 'New threads start from': 'project-branch',
  'Take big files off this Mac': 'files-off-after',
};

/**
 * Presses the `nth` picker whose name is `label`: its own (`aria-label` on a bare one), the word
 * over it, or its row's. The fixture says which, so the native window opens the same one.
 */
export async function pressPicker(h, label, nth = 0) {
  await h.evaluate(`(() => {
    const named = (b) => b.getAttribute('aria-label') === ${JSON.stringify(label)}
      || b.closest('.set-picker')?.querySelector('.set-picker-label')?.textContent === ${JSON.stringify(label)}
      || b.closest('.set-row')?.querySelector('.set-row-label')?.textContent === ${JSON.stringify(label)};
    [...document.querySelectorAll('.set-picker-now')].filter(named)[${nth}]?.click();
  })()`);
  await h.wait(400);
  return { picker: PICKERS[label] ?? label };
}

/** Types `words` into the menu's search, as a hand would. */
export async function typeIntoSearch(h, words) {
  await h.evaluate(`document.querySelector('.set-search input').focus()`);
  await h.call('Input.insertText', { text: words });
  await h.wait(400);
  return { search: words };
}

export const go = (h) => open(h, 'general');
