// WHAT AN ANTIGRAVITY WORKER MAY DO, IN AGY'S OWN TERMS.
//
// The sister of shared/codex-modes.mts. The first four are agy's own modes,
// read off `session/new`'s `availableModes` on agy 1.5.13, 2026-11-20, under
// agy's own names. The sentences are ours, from its descriptions and from what
// was measured.
//
// FULL ACCESS IS NOT ONE OF AGY'S MODES but a flag on its server:
// `agy acp --dangerously-skip-permissions`, "Auto-approve all tool permission
// requests". It is Auto with that flag, and it is what Codex's Full access and
// Claude Code's Bypass Permissions are (asked 2026-11-20: "o agy não tem um
// modo de full access igual o codex e claude?"). Measured the same day with a
// real turn: in Auto, with and without the flag, a network call, a file written
// outside the folder and `rm -rf` of a folder outside it all ran without a
// question; without the flag, whether to ask is agy's own judgement, and with
// it nothing is ever asked.
//
// A worker starts in Auto unless this says otherwise (main/antigravity-worker.mts
// WORKER_MODE).

export type AntigravityModeId = 'auto' | 'accept-edits' | 'default' | 'plan' | 'full-access';

export const ANTIGRAVITY_MODE_ORDER: readonly AntigravityModeId[] = ['auto', 'accept-edits', 'default', 'plan', 'full-access'];

export const ANTIGRAVITY_MODES: Record<AntigravityModeId, { label: string; what: string }> = {
  auto: { label: 'Auto', what: 'Edits and safe commands go ahead in its sandbox. It asks before anything irreversible.' },
  'accept-edits': { label: 'Accept edits', what: 'Edits in the folder go ahead. It asks before running commands.' },
  default: { label: 'Default', what: 'It asks before every edit and every command.' },
  plan: { label: 'Plan', what: 'It plans the task carefully before doing it.' },
  'full-access': { label: 'Full access', what: 'Anything, anywhere on this computer, without asking.' },
};

export const ANTIGRAVITY_DEFAULT_MODE: AntigravityModeId = 'auto';

export function isAntigravityMode(value: unknown): value is AntigravityModeId {
  return typeof value === 'string' && (ANTIGRAVITY_MODE_ORDER as readonly string[]).includes(value);
}

/** What one of these asks of agy: the mode its session is set to, and whether its server skips every question. */
export function agyLaunchFor(mode: AntigravityModeId | null | undefined): { mode: string | null; skipQuestions: boolean } {
  if (mode === 'full-access') return { mode: 'auto', skipQuestions: true };
  return { mode: mode ?? null, skipQuestions: false };
}

/** The server flag behind Full access. */
export const AGY_SKIP_QUESTIONS = '--dangerously-skip-permissions';
