// 2026-11-19: Antigravity becomes the third engine a thread can run on, beside
// Claude Code and Codex. Measured on agy 1.5.13 on a Mac: the installer puts
// `agy` in ~/.local/bin; its Google sign-in is in the Keychain, and a run
// started with `--gemini-dir` on a folder of the app's own kept that sign-in
// while reading none of the person's own agy settings, MCP servers, plugins or
// rules. It also reads a Gemini key or Google Cloud credentials from its
// environment (`gemini-api-key`, `agent-platform`), which would bill something
// other than the person's plan, so a worker is handed neither.
import { describe, it, expect, afterAll } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { harnessFor, HARNESS_OPERATIONS } from '../main/harnesses.mts';
import { ENGINES, engineFor, availableEngines, ENGINE_CHOICE_ENABLED, isEngine } from '../shared/engines.mts';
import { findEngine } from '../main/engine-bin.mts';
import { signInCommand, statusCommand, installCommand } from '../main/engine-setup.mts';
import { Supervisor } from '../main/supervisor.mts';

const dirs: string[] = [];
afterAll(() => { for (const d of dirs) fs.rmSync(d, { recursive: true, force: true }); });
const tmp = () => { const d = fs.mkdtempSync(path.join(os.tmpdir(), 'agy-engine-')); dirs.push(d); return d; };

const fakeSupervisor = (over: Record<string, unknown> = {}) => Object.assign(Object.create(Supervisor.prototype), {
  appDir: path.resolve(import.meta.dirname, '..'), config: { storeRoot: '/tmp/store', accountId: 'acct' }, storeMcpCommand: () => null, ...over,
});

describe('Antigravity as an engine', () => {
  it('is registered with every operation and offered after the other two', () => {
    expect(isEngine('antigravity')).toBe(true);
    expect(ENGINES.map(e => e.id)).toEqual(['claude', 'codex', 'antigravity']);
    const harness = harnessFor('antigravity');
    for (const name of HARNESS_OPERATIONS) expect(typeof harness[name as keyof typeof harness], name).toBe('function');
    expect(harness.label).toBe('Antigravity');
    expect(harness.accountKey('default')).toBe('antigravity:default');
    expect(harness.instructions({})).toEqual({ by: 'thread' });
  });

  it('never hands a worker a key that would bill something other than the plan', () => {
    const input = { PATH: '/bin', HOME: '/h', GEMINI_API_KEY: 'k', GOOGLE_API_KEY: 'k', GOOGLE_APPLICATION_CREDENTIALS: '/c.json', GOOGLE_CLOUD_PROJECT: 'p', CLOUDSDK_CONFIG: '/g', ANTHROPIC_API_KEY: 'k', OPENAI_API_KEY: 'k', CLAUDE_CONFIG_DIR: '/c', CODEX_HOME: '/x' };
    expect(harnessFor('antigravity').workerEnv(input)).toEqual({ PATH: '/bin', HOME: '/h' });
  });

  // docs/harnesses.md: an ACP worker gets every vendor's provider keys removed,
  // not only its own engine's. agy takes custom models with keys of their own.
  it('takes every vendor’s key out, as OpenCode’s worker does', () => {
    const input = {
      PATH: '/bin', HOME: '/h', OPENROUTER_API_KEY: 'k', GROQ_API_KEY: 'k', MISTRAL_API_KEY: 'k', DEEPSEEK_API_KEY: 'k',
      XAI_API_KEY: 'k', CEREBRAS_API_KEY: 'k', TOGETHER_API_KEY: 'k', FIREWORKS_API_KEY: 'k', DEEPINFRA_API_KEY: 'k',
      BASETEN_API_KEY: 'k', NVIDIA_API_KEY: 'k', GITLAB_TOKEN: 'k', AWS_ACCESS_KEY_ID: 'k', AWS_SECRET_ACCESS_KEY: 'k',
      AZURE_RESOURCE_NAME: 'k', SOME_OTHER_API_KEY: 'k', HUGGINGFACE_ACCESS_TOKEN: 'k',
    };
    expect(harnessFor('antigravity').workerEnv(input)).toEqual({ PATH: '/bin', HOME: '/h' });
  });

  it('signs in, checks the sign-in and installs with agy’s own commands', () => {
    expect(signInCommand('antigravity', '/u/agy')).toEqual({ file: '/u/agy', args: ['acp', '--login'] });
    expect(statusCommand('antigravity', '/u/agy')).toEqual({ file: '/u/agy', args: ['models'] });
    const install = installCommand('antigravity');
    expect(install.file).toBe('/bin/bash');
    expect(install.args[1]).toContain('https://antigravity.google/cli/install.sh');
  });

  it('is found where its installer puts it, and not where it is not', () => {
    const home = tmp();
    const bin = path.join(home, '.local/bin/agy');
    fs.mkdirSync(path.dirname(bin), { recursive: true });
    fs.writeFileSync(bin, '#!/bin/sh\n', { mode: 0o755 });
    expect(findEngine('antigravity', { home, env: { PATH: '/nowhere' }, shellLookup: () => null }).path).toBe(bin);
    expect(findEngine('antigravity', { home: tmp(), env: { PATH: '/nowhere' }, shellLookup: () => null }).found).toBe(false);
  });

  it('runs a row that names it, and Claude Code on a Mac without agy', () => {
    const config = { engineChoice: '2026-10-29T00:00:00.000Z' };
    const row = { engine: 'antigravity', wrote: { engine: { ts: Date.parse('2026-11-19T00:00:00Z') } } };
    expect(engineFor(row, { config, found: { claude: true, codex: true, antigravity: true }, enabled: ENGINE_CHOICE_ENABLED })).toBe('antigravity');
    expect(engineFor(row, { config, found: { claude: true, codex: true, antigravity: false }, enabled: ENGINE_CHOICE_ENABLED })).toBe('claude');
    expect(engineFor(row, { config, found: { claude: true, antigravity: true } })).toBe('claude');
    expect(availableEngines({ claude: true, antigravity: true }).map(e => e.id)).toEqual(['claude', 'antigravity']);
  });

  it('counts as here only where agy was found', () => {
    expect(fakeSupervisor({ config: { antigravityBin: '/u/agy' } })._enginesFound()).toMatchObject({ antigravity: true });
    expect(fakeSupervisor({ config: {} })._enginesFound()).toMatchObject({ antigravity: false });
    expect(fakeSupervisor({ config: { antigravityBin: '/u/agy' } })._engineCanRun('antigravity')).toBe(true);
    expect(fakeSupervisor({ config: { codexBin: '/u/codex' } })._engineCanRun('antigravity')).toBe(false);
  });

  it('starts agy in the app’s own home with the store and the connections in ACP’s shape', () => {
    const userDir = tmp();
    const sup = fakeSupervisor({ userDir, config: { storeRoot: '/tmp/store', accountId: 'acct', antigravityBin: '/u/agy' }, storeMcpCommand: () => '/app/mercury-mcp.sh' });
    const plan = { args: [], connectionServers: { 'zero-tracker': { command: '/app/node', args: ['/app/proxy.mts'], env: { AGENTBOX_CONNECTION: 'tracker' } } } };
    const launch = sup._antigravityLaunch(plan, { id: 'w-1', product: 'p' });
    expect(launch.bin).toBe('/u/agy');
    expect(launch.args).toEqual(['acp', '--disable-telemetry', '--gemini-dir', path.join(userDir, 'antigravity')]);
    const [store, tracker] = launch.mcpServers;
    expect(store.name).toBe('mercury');
    expect(store.command).toContain('mercury-mcp.sh');
    expect(store.env).toEqual(expect.arrayContaining([{ name: 'ZERO_PRODUCT', value: 'p' }, { name: 'ZERO_ITEM', value: 'w-1' }, { name: 'STORE_ACCOUNT_ID', value: 'acct' }]));
    expect(tracker).toEqual({ name: 'zero-tracker', command: '/app/node', args: ['/app/proxy.mts'], env: [{ name: 'AGENTBOX_CONNECTION', value: 'tracker' }] });
    expect(launch.storeServer).toBe('mercury');
    expect(launch.connectionServers).toEqual(['zero-tracker']);
  });

  it('says it runs on the Antigravity sign-in, never on another engine’s account', () => {
    expect(fakeSupervisor({ config: { antigravityBin: '/u/agy', antigravitySignedIn: true } }).runsOnAccount('antigravity')).toEqual({ engine: 'antigravity' });
    expect(fakeSupervisor({ config: { antigravityBin: '/u/agy' } }).runsOnAccount('antigravity')).toEqual({ engine: 'antigravity' });
    expect(fakeSupervisor({ config: { antigravityBin: '/u/agy', antigravitySignedIn: false } }).runsOnAccount('antigravity')).toBeNull();
    expect(fakeSupervisor({ config: { antigravitySignedIn: true } }).runsOnAccount('antigravity')).toBeNull();
  });

  it('tells a run which model it is on in agy’s own name for it, and never borrows Codex’s', async () => {
    const { readAntigravityModels } = await import('../main/antigravity-models.mts');
    const { EventEmitter } = await import('node:events');
    await readAntigravityModels('/u/agy', { spawn: () => {
      const child = Object.assign(new EventEmitter(), { stdout: new EventEmitter(), kill: () => {} });
      setImmediate(() => { child.stdout.emit('data', Buffer.from('gemini-4-neon-high\tGemini 4 Neon (High)\ngemini-4-neon-low\tGemini 4 Neon (Low)\n')); child.emit('close', 0); });
      return child;
    } });
    const sup = fakeSupervisor({ _codexModelWord: () => 'gpt-5.6-sol' });
    expect(sup._whatThisRunIsOn('antigravity', 'gemini-4-neon', 'high')).toMatch(/^This run is on Antigravity, Gemini 4 Neon, at high effort\./);
    expect(sup._whatThisRunIsOn('antigravity', 'claude-opus-5-5', null)).toMatch(/^This run is on Antigravity, claude-opus-5-5\./);
    expect(sup._whatThisRunIsOn('antigravity', null, null)).toBeNull();
  });

  it('has no usage reading, so nothing looks back on its room', () => {
    const sup = fakeSupervisor({ _engineFor: () => 'antigravity', codexUsage: () => { throw Error('must not read Codex'); } });
    expect(sup._roomToLookBack('antigravity')).toBe(false);
  });

  it('shows Settings where agy is and whether it last said it is signed in', async () => {
    const { antigravityState } = await import('../main/settings.mts');
    expect(antigravityState({ antigravityBin: '/u/agy', antigravitySignedIn: true }, { search: false })).toMatchObject({ bin: '/u/agy', found: true, signedIn: true, chosen: false });
    expect(antigravityState({ antigravityBin: null }, { search: false })).toMatchObject({ bin: '', found: false, signedIn: null });
    expect(antigravityState({ antigravityBin: '/u/agy', antigravityBinConfigured: '/u/agy', antigravitySignedIn: false }, { search: false })).toMatchObject({ chosen: true, signedIn: false });
  });

  it('has no cloud account until one is signed in for it, so a Mac with Claude logins up there still runs it here', async () => {
    const { cloudLoginInUse, cloudLoginsPath } = await import('../main/cloud/logins.mts');
    const home = tmp();
    fs.mkdirSync(path.dirname(cloudLoginsPath(home)), { recursive: true });
    fs.writeFileSync(cloudLoginsPath(home), JSON.stringify({ 'claude:acct-1': { id: 'acct-1', key: 'acct-1', engine: 'claude', savedAt: 1, expiresAt: Date.now() + 86_400_000 } }));
    expect(cloudLoginInUse('claude', { home })).not.toBeNull();
    expect(cloudLoginInUse('antigravity', { home })).toBeNull();
  });

  it('finds a conversation again after a restart where agy keeps it', () => {
    const userDir = tmp();
    const sup = fakeSupervisor({ userDir });
    const kept = path.join(userDir, 'antigravity', 'antigravity-cli', 'conversations', 'sess-1.db');
    fs.mkdirSync(path.dirname(kept), { recursive: true });
    fs.writeFileSync(kept, '');
    expect(harnessFor('antigravity').transcript(sup, { sessionId: 'sess-1' })).toBe(kept);
    expect(harnessFor('antigravity').transcript(sup, { sessionId: 'sess-2' })).toBeNull();
    expect(harnessFor('antigravity').transcript(sup, { sessionId: '../escape' })).toBeNull();
  });
});
