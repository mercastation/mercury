// The Inbox on a Mac set up for the cloud, for the head strip's cloud button: here with the
// cloud on and nothing running there, which is the plain square with an orange cloud.
//
// `reference.mjs`'s invented inbox has no cloud at all, and then there is no button. A scene
// names one of the states below and the page is fed the same inbox with that `cloud` in its
// snapshot; the fixture file carries it, so the native window is told the same. The accounts
// and their addresses are invented.
import { withSnapshot } from './inbox.mjs';

const month = { runs: 0, agentHours: 0, estimatedCost: 0, costPerAgentHour: null };
const account = { engine: 'claude', key: 'a-claude', name: 'you@example.com', ready: true, state: 'ready', inUse: true };
const setUp = { setUp: true, managed: true, mode: 'full', whenOn: 'full', running: 0, month, accounts: [account], signIns: [], keys: [], projects: ['storefront', 'mobile'] };

export const CLOUDS = {
  // On, and idle.
  on: setUp,
  // Off: the same square, with nothing orange on it.
  off: { ...setUp, mode: 'mac' },
  // Three threads run there: the button widens to carry the number.
  count: { ...setUp, mode: 'all', whenOn: 'all', running: 3 },
  // Offered and never set up: it says what it is, with the dot that asks for a look.
  named: { setUp: false, offered: true, mode: 'mac', running: 0, month, accounts: [], signIns: [] },
  // An account stopped working and its menu has not been opened since.
  again: { ...setUp, running: 2, accounts: [account, { engine: 'codex', key: 'a-codex', name: null, ready: false, state: 'broken' }] },
  // An account is being added in the background: the square, and the dot.
  adding: { ...setUp, signIns: [{ engine: 'codex', key: 'a-codex', state: 'checking', url: null, code: null, message: null }] },
};

export const withCloud = (h, cloud) => withSnapshot(h, (snapshot) => ({ ...snapshot, cloud }));

export const go = (h) => withCloud(h, CLOUDS.on);
