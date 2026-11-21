// What an Antigravity worker may do, in agy's own terms (the sister of
// shared/codex-modes.mts). The first four are the modes agy offers in
// `session/new`, under its own names. Full access is not one of them but a
// flag on its server, `agy acp --dangerously-skip-permissions`: Auto with every
// question approved, as Codex's Full access and Claude Code's Bypass
// Permissions are. Without the flag, whether to ask is agy's own judgement.
// A worker starts in Auto (main/antigravity-worker.mts WORKER_MODE).

export type AntigravityModeId = 'auto' | 'accept-edits' | 'default' | 'plan' | 'full-access';

export const ANTIGRAVITY_MODE_ORDER: readonly AntigravityModeId[] = ['auto', 'accept-edits', 'default', 'plan', 'full-access'];

export const ANTIGRAVITY_MODES: Record<AntigravityModeId, { label: string; what: string }> = {
  auto: { label: 'Auto', what: 'Edits and the commands it judges safe go ahead. Antigravity decides what to ask about.' },
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
