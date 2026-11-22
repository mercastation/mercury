// Home while an agent works on a thread: In progress lists it, with the agent's face and the
// green dot of a thread an agent is on (`.hm-face.hm-agent.live`), and under its title who is on
// it and where.
const MIN = 60_000;

export function dress(snapshot, now) {
  const [first, ...rest] = snapshot.items;
  const picked = now - 3 * MIN;
  const { result: _result, ...asked } = first;
  const working = {
    ...asked, status: 'claimed', epoch: 3, updatedAt: picked,
    claim: { holder: 'mcp-40131', leaseUntil: picked + 31 * MIN },
    wrote: { ...first.wrote, status: { ts: picked, source: 'agent' } },
  };
  const session = { itemId: working.id, product: working.product, startedAt: picked - 10_000, engine: 'claude-code', tail: [], activity: [] };
  return { ...snapshot, items: [working, ...rest], supervisor: { ...snapshot.supervisor, running: [session] } };
}
