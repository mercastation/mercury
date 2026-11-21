// Antigravity (`agy acp`), the first engine on the ACP transport. Declared
// missing rather than faked: plan usage (agy reports quota only inside its own
// screen, `/usage`), compaction, its slash commands, forking, remote input, and
// the app's small-model questions (`agy -p` takes no home of the app's own, so
// they would land in the person's own agy history).
import fs from 'node:fs';
import path from 'node:path';
import type { EventEmitter } from 'node:events';
import { harnessDefinition } from '../../shared/harness-definitions.mts';
import { resolveEngine } from '../engine-bin.mts';
import {
  captureAntigravityEvent, antigravityStreamingText, summarizeAntigravityEvent,
  traceAntigravityEvent, rememberAntigravityChange, antigravityTookTheWords,
} from '../antigravity.mts';
import type { AntigravitySession } from '../antigravity.mts';
import { antigravityActivity } from '../agent-activity.mts';
import { definitionMethods } from './common.mts';
import type { HarnessConfig, HarnessPlan, HarnessRecord, HarnessSession, HarnessSpawnContext, HarnessSupervisor } from './common.mts';

const definition = harnessDefinition('antigravity')!;

/** Where agy keeps one conversation under a home (`--gemini-dir`). */
export function antigravityConversation(home: string, sessionId: unknown): string | null {
  if (typeof sessionId !== 'string' || !/^[A-Za-z0-9-]+$/.test(sessionId)) return null;
  const file = path.join(home, 'antigravity-cli', 'conversations', `${sessionId}.db`);
  return fs.existsSync(file) ? file : null;
}

export const antigravityHarness = {
  ...definitionMethods(definition),
  discover: (configured: string | null = null) => resolveEngine('antigravity', configured),
  binary: (config: HarnessConfig | null | undefined) => config?.antigravityBin ?? null,
  // One sign-in, the one agy itself holds; the launch check writes whether it answered.
  profiles: () => ['default'],
  signedIn: (sup: HarnessSupervisor) => sup.config.antigravitySignedIn !== false,
  profileHome: (sup: HarnessSupervisor) => sup._antigravityHome(),
  profileEnv: (env: NodeJS.ProcessEnv) => env,
  signInFiles: () => [],
  defaultModel: () => null,
  workspaceModel: () => null,
  tooling: () => null,
  // On the session itself, as `_meta.antigravity.systemPrompt.append`.
  instructions: () => ({ by: 'thread' as const }),
  spawn: (sup: HarnessSupervisor, plan: HarnessPlan, context: HarnessSpawnContext) => sup._spawnAntigravityWorker(plan, context),
  readers: () => ({
    capture: captureAntigravityEvent,
    stream: antigravityStreamingText,
    summarize: summarizeAntigravityEvent,
    trace: traceAntigravityEvent,
    activity: antigravityActivity,
    took: antigravityTookTheWords,
  }),
  subscribe(child: EventEmitter, session: HarnessSession, absorb: (method: string, params: unknown) => void): void {
    child.on('event', (method: string, params: unknown) => {
      rememberAntigravityChange(session as AntigravitySession, method, params);
      absorb(method, params);
    });
  },
  attachInput: () => {},
  transcript: (sup: HarnessSupervisor, rec: HarnessRecord | null | undefined) => antigravityConversation(sup._antigravityHome(), rec?.sessionId),
  change: (session: HarnessSession) => {
    const files = (session as AntigravitySession).antigravityChange;
    return files?.length ? { files: [...files] } : null;
  },
  release: () => {},
  updateSettings: () => ({}),
  memoryGate: () => ({}),
  updatePlan: () => null,
};
