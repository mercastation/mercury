// WHAT AN ANTIGRAVITY WORKER MAY DO, IN AGY'S OWN TERMS.
//
// The sister of shared/codex-modes.mts. Every id is one agy offers, read off
// `session/new`'s `availableModes` on agy 1.5.13, 2026-11-20, and the labels
// are agy's own names for them. The sentences are ours, from its descriptions
// and from what was measured: in Auto, commands ran in agy's sandbox (in the
// folder, no network) and it asked before leaving it.
//
// A worker starts in Auto unless this says otherwise (main/antigravity-worker.mts
// WORKER_MODE). agy has no mode that skips every question, so neither does this.

export type AntigravityModeId = 'auto' | 'accept-edits' | 'default' | 'plan';

export const ANTIGRAVITY_MODE_ORDER: readonly AntigravityModeId[] = ['auto', 'accept-edits', 'default', 'plan'];

export const ANTIGRAVITY_MODES: Record<AntigravityModeId, { label: string; what: string }> = {
  auto: { label: 'Auto', what: 'Edits and safe commands go ahead in its sandbox. It asks before anything irreversible.' },
  'accept-edits': { label: 'Accept edits', what: 'Edits in the folder go ahead. It asks before running commands.' },
  default: { label: 'Default', what: 'It asks before every edit and every command.' },
  plan: { label: 'Plan', what: 'It plans the task carefully before doing it.' },
};

export const ANTIGRAVITY_DEFAULT_MODE: AntigravityModeId = 'auto';

export function isAntigravityMode(value: unknown): value is AntigravityModeId {
  return typeof value === 'string' && (ANTIGRAVITY_MODE_ORDER as readonly string[]).includes(value);
}
