// agy's model list for the picker, read from `agy models` and kept in memory:
// agy keeps no list on disk, and asking costs a network round trip. The same
// call is the sign-in check, since signed out it exits 1.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawn as nodeSpawn } from 'node:child_process';
import type { ChildProcess } from 'node:child_process';
import type { EventEmitter } from 'node:events';

export interface AntigravityModel { id: string; label: string; levels: string[] }

const LEVEL = /^(.*) \((Low|Medium|High|XHigh|Extra|Max)\)$/;
const LEVEL_WORD: Record<string, string> = { Low: 'low', Medium: 'medium', High: 'high', XHigh: 'xhigh', Extra: 'xhigh', Max: 'max' };

/**
 * One row per model, its levels in agy's order. A model is folded only when it
 * is listed at two levels or more: a lone `gpt-oss-120b-medium` is a model
 * whose id names its level, and agy takes it whole.
 */
export function antigravityModelRows(out: string): AntigravityModel[] {
  const lines = String(out ?? '').split('\n').map(l => l.split('\t')).filter(p => p.length === 2 && /^[a-z0-9][a-z0-9._-]*$/i.test(p[0].trim()) && p[1].trim());
  const folded = new Map<string, AntigravityModel>();
  const parts = lines.map(([rawId, rawLabel]) => {
    const id = rawId.trim(); const label = rawLabel.trim();
    const m = LEVEL.exec(label);
    const level = m ? LEVEL_WORD[m[2]] : null;
    return level && id.endsWith(`-${level}`) ? { id, label, base: id.slice(0, -level.length - 1), baseLabel: m![1], level } : { id, label, base: null, baseLabel: null, level: null };
  });
  const count = new Map<string, number>();
  for (const p of parts) if (p.base) count.set(p.base, (count.get(p.base) ?? 0) + 1);
  for (const p of parts) {
    const fold = p.base && (count.get(p.base) ?? 0) > 1;
    const key = fold ? p.base! : p.id;
    const row = folded.get(key) ?? { id: key, label: fold ? p.baseLabel! : p.label, levels: [] };
    if (fold && !row.levels.includes(p.level!)) row.levels.push(p.level!);
    folded.set(key, row);
  }
  return [...folded.values()];
}

let kept: AntigravityModel[] = [];

/** The last list agy gave, or none. */
export function antigravityModels(): AntigravityModel[] { return kept; }
export function forgetAntigravityModels() { kept = []; }

type Spawned = Pick<ChildProcess, 'kill'> & EventEmitter & { stdout: EventEmitter | null };

// WHOSE ACCOUNT IT IS. No agy command says so, but its log does, on every
// call that signs in: measured on agy 1.5.13, `agy --log-file <f> models`
// wrote "OAuth: authenticated successfully as <email>". The log goes to a file
// of the app's own, read once and deleted.
const SIGNED_IN_AS = /authenticated successfully as (\S+@\S+?)[\s,]/;
export function accountInAgyLog(log: string): string | null {
  return SIGNED_IN_AS.exec(String(log ?? ''))?.[1] ?? null;
}

/** Ask agy. `signedIn` is null when nothing was established; `email` is whose sign-in it is, when its log said. */
export function readAntigravityModels(bin: string | null | undefined, { spawn = (file: string, args: string[]) => nodeSpawn(file, args, { stdio: ['ignore', 'pipe', 'pipe'] }) as Spawned, timeoutMs = 20_000, logFile = path.join(os.tmpdir(), `mercury-agy-${process.pid}-${Date.now()}.log`) }: { spawn?: (file: string, args: string[]) => Spawned; timeoutMs?: number; logFile?: string } = {}): Promise<{ signedIn: boolean | null; email: string | null }> {
  if (!bin) return Promise.resolve({ signedIn: null, email: null });
  return new Promise((resolve) => {
    let out = '';
    let done = false;
    let child: Spawned;
    const finish = (signedIn: boolean | null) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      try { child?.kill(); } catch { /* already gone */ }
      let email: string | null = null;
      try { email = signedIn ? accountInAgyLog(fs.readFileSync(logFile, 'utf8')) : null; } catch { /* no log written */ }
      try { fs.rmSync(logFile, { force: true }); } catch { /* already gone */ }
      resolve({ signedIn, email });
    };
    const timer = setTimeout(() => finish(null), timeoutMs);
    try { child = spawn(bin, ['--log-file', logFile, 'models']); } catch { finish(null); return; }
    child.stdout?.on('data', (d: Buffer) => { out = (out + d.toString()).slice(-256 * 1024); });
    child.on('error', () => finish(null));
    child.on('close', (code: number | null) => {
      if (code === 0) {
        const rows = antigravityModelRows(out);
        if (rows.length) kept = rows;
        finish(true);
      } else finish(code === 1 ? false : null);
    });
  });
}
