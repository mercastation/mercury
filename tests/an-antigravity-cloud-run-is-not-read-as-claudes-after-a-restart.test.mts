// 2026-11-21 review (Codex's finding, measured before it was fixed). A cloud
// run the app left going is read back after a restart, and every engine but
// Codex was read with Claude Code's stream readers, so an Antigravity run's ACP
// lines went through them as if they were Claude's. Measured on agy 1.5.13: when
// its input closes mid-turn, agy exits at once without finishing the turn, so a
// quit ends an Antigravity cloud turn, as it ends a Codex one, and the thread is
// picked up again after the restart. Its stream is not followed at all.
import { describe, it, expect } from 'vitest';
import { EventEmitter } from 'node:events';
import { Supervisor } from '../main/supervisor.mts';

const child = () => Object.assign(new EventEmitter(), { stdout: new EventEmitter(), kill: () => true }) as never;
const sup = () => Object.assign(Object.create(Supervisor.prototype), { store: { listProducts: () => [], readItem: () => null }, onChange: () => {} });

describe('a cloud run read back after a restart', () => {
  it('is not followed when it is Antigravity’s', () => {
    expect(sup()._followComingHome({ itemId: 'w-1', rec: { engine: 'antigravity', product: 'p', cwd: '/w', cloud: true } }, child())).toBeNull();
  });

  it('is followed when it is Claude Code’s or Codex’s, as before', () => {
    expect(sup()._followComingHome({ itemId: 'w-2', rec: { engine: 'claude', product: 'p', cwd: '/w', cloud: true } }, child())).not.toBeNull();
    expect(sup()._followComingHome({ itemId: 'w-3', rec: { engine: 'codex', product: 'p', cwd: '/w', cloud: true } }, child())).not.toBeNull();
  });
});
