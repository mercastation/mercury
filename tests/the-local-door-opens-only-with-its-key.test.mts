// THE LOCAL DOOR LETS IN WHOEVER HOLDS ITS KEY, AND NOBODY ELSE.
//
// main/local-door.mts puts the desktop app's own handlers behind a loopback
// port for the native window (desktop/). Whoever gets through it can run
// agents on this computer, so the things that keep it shut are what this file
// holds still: the key is asked on every call, the note with the key is this
// user's alone, no web screen is served through it, and it stays closed unless
// the config asks for it.

import { describe, it, expect, afterEach } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { LOCAL_DOOR_NOTE, openLocalDoor } from '../main/local-door.mts';
import type { LocalDoor, LocalDoorNote } from '../main/local-door.mts';
import type { IpcHandler, PushListener } from '../main/serve.mts';

const open: LocalDoor[] = [];
const folders: string[] = [];

afterEach(async () => {
  for (const door of open.splice(0)) await door.close();
  for (const dir of folders.splice(0)) fs.rmSync(dir, { recursive: true, force: true });
});

async function aDoor({ listeners }: { listeners?: Set<PushListener> } = {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'local-door-'));
  folders.push(dir);
  const note = path.join(dir, LOCAL_DOOR_NOTE);
  const channels = new Map<string, IpcHandler>([
    ['zero:snapshot', () => ({ items: [{ id: 'w-1' }] })],
    ['zero:echo', (_event, payload) => ({ said: payload })],
  ]);
  const door = await openLocalDoor({ channels, note, ...(listeners ? { listeners } : {}) });
  open.push(door);
  return { door, note, dir };
}

const ask = (door: LocalDoor, channel: string, { key, body = null }: { key?: string; body?: unknown } = {}) =>
  fetch(`http://127.0.0.1:${door.port}/api/${encodeURIComponent(channel)}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...(key ? { 'x-agentbox-token': key } : {}) },
    body: JSON.stringify(body),
  });

describe('the local door', () => {
  it('answers a channel for whoever holds the key', async () => {
    const { door } = await aDoor();
    const res = await ask(door, 'zero:snapshot', { key: door.token });
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ items: [{ id: 'w-1' }] });
    const echoed = await ask(door, 'zero:echo', { key: door.token, body: { hello: 'there' } });
    expect(await echoed.json()).toEqual({ said: { hello: 'there' } });
  });

  it('turns away a call with no key, and one with the wrong key', async () => {
    const { door } = await aDoor();
    expect((await ask(door, 'zero:snapshot')).status).toBe(403);
    expect((await ask(door, 'zero:snapshot', { key: `${door.token}x` })).status).toBe(403);
  });

  it('leaves a note with the port and the key that only this user can read', async () => {
    const { door, note } = await aDoor();
    const words = JSON.parse(fs.readFileSync(note, 'utf8')) as LocalDoorNote;
    expect(words).toMatchObject({ port: door.port, token: door.token, pid: process.pid });
    expect(words.token.length).toBeGreaterThanOrEqual(32);
    if (process.platform !== 'win32') expect(fs.statSync(note).mode & 0o777).toBe(0o600);
  });

  it('makes a new key for every door', async () => {
    const [a, b] = [await aDoor(), await aDoor()];
    expect(a.door.token).not.toBe(b.door.token);
  });

  it('serves no web screen: any other path is a plain 404, key or no key', async () => {
    const { door } = await aDoor();
    for (const where of ['/', '/index.html', `/?token=${door.token}`]) {
      const res = await fetch(`http://127.0.0.1:${door.port}${where}`);
      expect(res.status).toBe(404);
      expect(await res.text()).not.toMatch(/<html/i);
    }
  });

  it('listens on the loopback address only', async () => {
    const { door } = await aDoor();
    // Every other address this machine has refuses the connection outright.
    const others = Object.values(os.networkInterfaces()).flat()
      .filter((n): n is os.NetworkInterfaceInfo => !!n && n.family === 'IPv4' && !n.internal);
    for (const other of others) {
      await expect(fetch(`http://${other.address}:${door.port}/api/zero%3Asnapshot`, { method: 'POST', signal: AbortSignal.timeout(1500) }))
        .rejects.toThrow();
    }
  });

  it('passes on what the app pushes to a window that is listening', async () => {
    const listeners = new Set<PushListener>();
    const { door } = await aDoor({ listeners });
    const res = await fetch(`http://127.0.0.1:${door.port}/events?token=${door.token}`);
    const reader = res.body!.getReader();
    // The stream opens with its retry line; a push comes after it as a line of data.
    await reader.read();
    for (const send of listeners) send({ channel: 'zero:changed', args: [] });
    const { value } = await reader.read();
    expect(new TextDecoder().decode(value)).toContain('"channel":"zero:changed"');
    await reader.cancel();
  });

  it('takes its own note away when it closes, and leaves a newer door\'s alone', async () => {
    const { door, note } = await aDoor();
    // Another copy of the app opened after this one and wrote over the note.
    const newer: LocalDoorNote = { port: 1, token: 'a-newer-door', pid: 1, since: Date.now() };
    fs.writeFileSync(note, JSON.stringify(newer));
    await door.close();
    expect(JSON.parse(fs.readFileSync(note, 'utf8'))).toEqual(newer);

    const second = await aDoor();
    await second.door.close();
    expect(fs.existsSync(second.note)).toBe(false);
  });
});

describe('main/main.mts', () => {
  const source = fs.readFileSync(path.join(import.meta.dirname, '..', 'main', 'main.mts'), 'utf8');

  it('opens the local door only for a copy whose config asks for it', () => {
    const call = source.indexOf('openLocalDoor({');
    expect(call).toBeGreaterThan(0);
    // The line that guards it is the one just above the call.
    const guard = source.slice(source.lastIndexOf('\n  if (', call), call);
    expect(guard).toMatch(/config\.localDoor === true/);
  });

  it('gives it the handlers the window itself is answered by', () => {
    expect(source).toMatch(/openLocalDoor\(\{ channels: phoneChannels,/);
  });
});
