// Home as someone with more switched on sees it: teammates (so the second card is the team's
// week, Team is a third door, the rail has Chats and the composer offers To), a meeting recorder,
// a reading of usage, an account signed in, and a build that takes no feedback. Everyone here is
// invented.
const HOUR = 3_600_000;

export function dress(snapshot, now) {
  const reading = (engine, ago) => ({
    engine, at: now - ago, profile: 'default',
    limits: [{ span: 'week', qualifier: 'all models', name: 'This week', percent: 41, resetsAt: now + 50 * HOUR, resetsText: '1am', resetsOn: 'Tue', zone: 'UTC' }],
  });
  return {
    ...snapshot,
    alpha: { rooms: false, teammates: true, multiplayer: false, glass: false },
    teammates: { list: [], chat: [], runs: [], playbooks: [], sending: [] },
    team: {
      configured: false, canSignIn: true, started: true, settling: false, signedIn: true, signingIn: null,
      me: { id: 'p-ada', email: 'ada@example.com', name: 'ada.lovelace', avatarUrl: null, status: null },
      team: null, invites: [], sent: [], people: [], cards: [], teamMates: [], seen: {}, error: null,
    },
    meeting: { hex: true, phase: 'idle', title: null, startedAt: null, product: snapshot.products[0].slug, problem: null, model: null, preparing: null },
    feedback: false,
    usage: reading('claude', 60_000),
    usageByEngine: [reading('claude', 60_000), reading('codex', 5 * HOUR)],
  };
}
