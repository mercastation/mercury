// AN AGENT ON THE MAC KEEPS IT FROM FALLING ASLEEP WHILE IT RUNS (2026-11-15).
//
// An agent started at 02:28 on a Mac left open stopped when the Mac slept on
// its own, and stayed stopped until morning: its transcript holds 27
// "Connection lost while your computer was asleep" retries and 535 minutes.
// Nothing told macOS a run was going. Now the run host, which holds every
// local Claude Code worker with the app open or shut, starts `caffeinate -i -w
// <pid>` beside each one: idle sleep waits until that worker ends, the display
// still sleeps, and a closed lid still sleeps the Mac. A run moved to the
// cloud ends here, so it lets go too. Measured with the real run host and a
// stand-in worker, and on a Mac with macOS's own list of sleep assertions.
import { afterEach, describe, expect, it } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawn, execFileSync } from 'node:child_process';
import { serveRuns, keepAwakeWhile } from '../main/run-host.mts';
import { HostedRun } from '../main/hosted-run.mts';
import { attachClaudeInput } from '../main/claude-input.mts';

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));
async function until<T>(fn: () => T | Promise<T>, ms = 7000): Promise<T | null> {
  const end = Date.now() + ms;
  for (;;) { const v = await fn(); if (v) return v; if (Date.now() > end) return null; await wait(25); }
}
const alive = (pid: number) => { try { process.kill(pid, 0); return true; } catch { return false; } };

let dir: string | null = null;
let host: ReturnType<typeof serveRuns> | null = null;
afterEach(async () => {
  await host?.close();
  host = null;
  if (dir) fs.rmSync(dir, { recursive: true, force: true });
  dir = null;
});

describe('the run host', () => {
  it('holds the Mac awake for each worker it starts, by that worker\'s pid', async () => {
    dir = fs.mkdtempSync(path.join(fs.existsSync('/tmp') ? '/tmp' : os.tmpdir(), 'awake-'));
    fs.writeFileSync(path.join(dir, 'worker.mjs'), "process.stdin.on('data', () => {}); process.stdin.on('end', () => process.exit(0));");
    const held: number[] = [];
    const sockPath = path.join(dir, 'runs.sock');
    host = serveRuns({ sockPath, spoolDir: path.join(dir, 'spool'), keepAwake: (pid) => { held.push(pid); } });
    expect(await host.started).toBe('serving');
    const child = new HostedRun({ sockPath, runId: crypto.randomUUID(), start: { bin: process.execPath, argv: [path.join(dir, 'worker.mjs')], env: { PATH: process.env.PATH }, cwd: dir } });
    attachClaudeInput(child);
    const pid = await until(() => child.pid);
    expect(pid).toBeTruthy();
    expect(await until(() => held.length)).toBe(1);
    expect(held).toEqual([pid]);
    child.stdin.end();
  });
});

describe('keeping the Mac awake', () => {
  type Spawned = { bin: string; args: string[]; opts: Record<string, unknown> };
  const recorder = () => {
    const made: Spawned[] = [];
    const spawnImpl = (bin: string, args: string[], opts: Record<string, unknown>) => {
      made.push({ bin, args, opts });
      return { on: () => {}, unref: () => {} };
    };
    return { made, spawnImpl };
  };

  it('asks macOS to keep from idle sleep until that pid ends, and nothing more', () => {
    const { made, spawnImpl } = recorder();
    keepAwakeWhile(4242, { platform: 'darwin', spawnImpl });
    expect(made).toEqual([{ bin: '/usr/bin/caffeinate', args: ['-i', '-w', '4242'], opts: { stdio: 'ignore' } }]);
  });

  it('does nothing off a Mac, or with no pid', () => {
    const { made, spawnImpl } = recorder();
    keepAwakeWhile(4242, { platform: 'linux', spawnImpl });
    keepAwakeWhile(0, { platform: 'darwin', spawnImpl });
    expect(made).toEqual([]);
  });

  it('says so when it cannot, and the run goes on', () => {
    const said: string[] = [];
    const spawnImpl = () => { throw new Error('ENOENT'); };
    expect(() => keepAwakeWhile(4242, { platform: 'darwin', spawnImpl, log: (m) => said.push(m) })).not.toThrow();
    expect(said.join('\n')).toMatch(/cannot keep the Mac awake.*ENOENT/);
  });

  it.runIf(process.platform === 'darwin' && fs.existsSync('/usr/bin/caffeinate'))('on this Mac, is in macOS\'s list while the worker runs and gone once it ends', async () => {
    const worker = spawn(process.execPath, ['-e', 'setTimeout(() => {}, 60000)'], { stdio: 'ignore' });
    const held = () => execFileSync('/usr/bin/pmset', ['-g', 'assertions'], { encoding: 'utf8' })
      .includes(`caffeinate asserting on behalf of Process ID ${worker.pid}\n`);
    try {
      keepAwakeWhile(worker.pid!);
      expect(await until(held, 5000)).toBe(true);
    } finally {
      worker.kill();
    }
    await until(() => !alive(worker.pid!));
    expect(await until(() => !held(), 5000)).toBe(true);
  });
});
