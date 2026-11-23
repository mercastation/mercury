// WHAT AN AGENT DOES IN ITS PROJECT'S BROWSER IS FILMED, AND ONLY THAT.
//
// Why this exists (2026-11-23): the project's browser is hidden and signed in
// as its owner, so what an agent presses there is done in their name and was
// only ever a line of text. The owner asked to watch it work, as Claude's web
// app shows its cursor in a sheet, with performance first: the native window
// is being rewritten for it. So the host keeps a film per thread
// (main/project-browser/film.mts) that the native window draws, and it films
// only while a step runs.
//
// The real-browser half drives a real Chrome for Testing through the host and
// reads the film off the disk; it is skipped on a machine without one.
import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import type { AddressInfo } from 'node:net';
import type { CallToolResult } from '@modelcontextprotocol/sdk/types.js';
import { serveBrowser, testBrowser } from '../main/project-browser/host.mts';
import { askHost, writeProject } from '../main/project-browser/place.mts';
import { FILM_CAPS, LIVE_FILE, STEPS_FILE, filmDir, filmRoot, kindOf, locatorFor, shortAddress, targetOf, trimFilms, whatOf } from '../main/project-browser/film.mts';

describe('the words a step is filmed under', () => {
  it('name what it acted on, as the agent described it', () => {
    expect(whatOf('browser_click', { element: 'Save button', target: 'e12' })).toBe('Save button');
    expect(whatOf('browser_navigate', { url: 'https://www.gitlab.example.com/group/issues?state=open' })).toBe('gitlab.example.com/group/issues?state=open');
    expect(whatOf('browser_navigate', { url: 'https://acme.dev/' })).toBe('acme.dev');
    expect(whatOf('browser_press_key', { key: 'Enter' })).toBe('Enter');
    expect(whatOf('browser_fill_form', { fields: [{ name: 'Title', target: 'e1', value: 'x' }, { name: 'Due', target: 'e2', value: 'y' }] })).toBe('Title, Due');
    expect(whatOf('browser_tabs', { action: 'select', index: 1 })).toBe('tab 2');
    expect(whatOf('browser_handle_dialog', { accept: false })).toBe('dismissed');
  });

  it('never carry what was typed, which may be anything', () => {
    const words = JSON.stringify(['browser_type', 'browser_fill_form'].map((tool) => whatOf(tool, { element: 'Password', target: 'e3', text: 'hunter2', fields: [{ name: 'Password', target: 'e3', value: 'hunter2' }] })));
    expect(words).not.toContain('hunter2');
  });

  it('are cut short when long', () => {
    expect(whatOf('browser_click', { element: 'x'.repeat(200), target: 'e1' }).length).toBeLessThanOrEqual(60);
    expect(shortAddress('not an address')).toBe('not an address');
  });

  it('film the steps that change the page, name the reading ones, and leave out tab lists', () => {
    expect(kindOf('browser_click', {})).toBe('acts');
    expect(kindOf('browser_navigate', {})).toBe('acts');
    expect(kindOf('browser_snapshot', {})).toBe('reads');
    expect(kindOf('browser_find', {})).toBe('reads');
    expect(kindOf('browser_tabs', { action: 'list' })).toBe(null);
    expect(kindOf('browser_tabs', { action: 'select', index: 0 })).toBe('acts');
    expect(kindOf('browser_close', {})).toBe(null);
  });

  it('find the element pressed by its snapshot reference, or by a selector', () => {
    expect(locatorFor('e12')).toBe('aria-ref=e12');
    expect(locatorFor('f1e3')).toBe('aria-ref=f1e3');
    expect(locatorFor('#save')).toBe('#save');
    expect(targetOf('browser_click', { target: 'e5' })).toBe('e5');
    expect(targetOf('browser_fill_form', { fields: [{ target: 'e7' }] })).toBe('e7');
    expect(targetOf('browser_navigate', { url: 'https://acme.dev' })).toBe(null);
  });
});

describe('the project\'s films on the disk', () => {
  it('let go of the threads used longest ago past their caps, but never the one being filmed', () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'films-'));
    const make = (name: string, bytes: number, age: number) => {
      const dir = path.join(root, name);
      fs.mkdirSync(dir);
      fs.writeFileSync(path.join(dir, '1a.jpg'), Buffer.alloc(bytes));
      fs.writeFileSync(path.join(dir, STEPS_FILE), '{}\n');
      const when = new Date(Date.now() - age * 60_000);
      fs.utimesSync(path.join(dir, STEPS_FILE), when, when);
      return dir;
    };
    const newest = make('w-new', 600, 1);
    const middle = make('w-mid', 600, 2);
    const oldest = make('w-old', 600, 3);
    const caps = { ...FILM_CAPS, projectBytes: 1_300, threads: 40 };
    expect(trimFilms(root, { caps })).toEqual([oldest]);
    expect(fs.existsSync(newest) && fs.existsSync(middle)).toBe(true);
    expect(trimFilms(root, { caps: { ...caps, threads: 1 }, keep: middle })).toEqual([]);
    fs.rmSync(root, { recursive: true, force: true });
  });
});

const chrome = testBrowser();

describe.skipIf(!chrome)('a thread\'s film, on a real Chrome for Testing', { timeout: 90_000 }, () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'project-film-'));
  const dir = path.join(root, 'browser');
  const pages = path.join(root, 'pages');
  const sock = path.join(root, 'host.sock');
  let site: http.Server;
  let on: string;
  let host: ReturnType<typeof serveBrowser>;

  const call = (thread: string, name: string, args: Record<string, unknown> = {}) =>
    askHost<CallToolResult>(sock, { method: 'tools/call', thread, params: { name, arguments: args } }, { ms: 60_000 });
  const text = (r: CallToolResult | null): string => (r?.content ?? []).map((c) => (c.type === 'text' ? c.text : '')).join('\n');
  const steps = (thread: string) => fs.readFileSync(path.join(filmDir(pages, thread), STEPS_FILE), 'utf8').trim().split('\n').map((l) => JSON.parse(l));
  const until = async (what: () => boolean, ms = 10_000) => {
    const end = Date.now() + ms;
    while (Date.now() < end) { if (what()) return; await new Promise((r) => setTimeout(r, 50)); }
    throw new Error('waited too long');
  };

  beforeAll(async () => {
    site = http.createServer((req, res) => {
      res.writeHead(200, { 'content-type': 'text/html' });
      if (req.url === '/done') return res.end('<h1>Saved</h1>');
      // The button sits low on a long page, so pressing it means bringing it into view first.
      res.end(`<title>Form</title><h1>An issue</h1><div style="height:1400px"></div><label>Title <input id="t"></label> <button onclick="location.href='/done'">Save</button>`);
    });
    await new Promise<void>((resolve) => site.listen(0, '127.0.0.1', resolve));
    on = `http://localhost:${(site.address() as AddressInfo).port}`;
    writeProject(dir, { slug: 'oms', sites: ['localhost'], pages });
    host = serveBrowser({ dir, sock, appPid: process.pid, film: true, settle: { everyMs: 50, enough: 2, shortAfterMs: 100, mostMs: 2_000 } });
    await host.started;
  });

  afterAll(async () => {
    await host?.close();
    await new Promise((resolve) => site?.close(resolve));
    fs.rmSync(root, { recursive: true, force: true });
  });

  it('keeps each step, where a press landed, and the page before and after it', async () => {
    expect(text(await call('w-1', 'browser_navigate', { url: `${on}/form` }))).toContain('Page Title: Form');
    // A snapshot comes inline, or as a file the agent reads when it is long.
    const snap = text(await call('w-1', 'browser_snapshot'));
    const file = snap.match(/\((\/[^)]+\.yml)\)/)?.[1];
    const page = file ? fs.readFileSync(file, 'utf8') : snap;
    const ref = (name: string, role: string) => page.match(new RegExp(`${role} "?${name}"?[^\\n]*\\[ref=(e\\d+)\\]`))?.[1];
    const field = ref('Title', 'textbox');
    const save = ref('Save', 'button');
    expect(field && save).toBeTruthy();
    await call('w-1', 'browser_type', { element: 'Title field', target: field, text: 'secret words' });
    await call('w-1', 'browser_click', { element: 'Save button', target: save });
    await until(() => steps('w-1').filter((s) => s.end).length === 4);

    const all = steps('w-1');
    const started = all.filter((s) => s.tool);
    expect(started.map((s) => [s.n, s.tool, s.what])).toEqual([
      [1, 'browser_navigate', 'localhost/form'],
      [2, 'browser_snapshot', ''],
      [3, 'browser_type', 'Title field'],
      [4, 'browser_click', 'Save button'],
    ]);

    const click = started.find((s) => s.tool === 'browser_click');
    // Where it pressed is on the page as it was drawn, brought into view first.
    const [x, y, w, h] = click.box;
    expect(w).toBeGreaterThan(10);
    expect(h).toBeGreaterThan(10);
    expect(x).toBeGreaterThanOrEqual(0);
    expect(y + h).toBeLessThanOrEqual(click.view[1]);
    const film = filmDir(pages, 'w-1');
    expect(fs.readFileSync(path.join(film, click.before)).subarray(0, 2)).toEqual(Buffer.from([0xff, 0xd8]));
    const clickEnd = all.find((s) => s.n === click.n && s.end);
    expect(clickEnd.url).toBe(`${on}/done`);
    expect(fs.existsSync(path.join(film, clickEnd.after))).toBe(true);

    // A reading step takes no picture: the page did not change.
    const snapshot = all.filter((s) => s.n === 2);
    expect(snapshot.some((s) => s.before || s.after)).toBe(false);
    // The newest picture is there whole, for the live view.
    expect(fs.statSync(path.join(film, LIVE_FILE)).size).toBeGreaterThan(1_000);
    // What was typed is nowhere in the film's words.
    expect(fs.readFileSync(path.join(film, STEPS_FILE), 'utf8')).not.toContain('secret words');
  });

  it('keeps each thread\'s film apart', async () => {
    await call('w-2', 'browser_navigate', { url: `${on}/done` });
    await until(() => fs.existsSync(path.join(filmDir(pages, 'w-2'), STEPS_FILE)) && steps('w-2').some((s) => s.end));
    expect(steps('w-2').filter((s) => s.tool).map((s) => s.n)).toEqual([1]);
    expect(fs.readdirSync(filmRoot(pages)).sort()).toEqual(['w-1', 'w-2']);
  });

  it('films nothing on a host that was not asked to, as a copy in the cloud is not', async () => {
    const other = path.join(root, 'other');
    const otherPages = path.join(root, 'other-pages');
    const otherSock = path.join(root, 'other.sock');
    writeProject(other, { slug: 'oms', sites: ['localhost'], pages: otherPages });
    const plain = serveBrowser({ dir: other, sock: otherSock, appPid: process.pid, settle: { everyMs: 50, enough: 2, shortAfterMs: 100, mostMs: 2_000 } });
    try {
      await plain.started;
      await askHost(otherSock, { method: 'tools/call', thread: 'w-3', params: { name: 'browser_navigate', arguments: { url: `${on}/done` } } }, { ms: 60_000 });
      expect(fs.existsSync(filmRoot(otherPages))).toBe(false);
    } finally { await plain.close(); }
  });
});
