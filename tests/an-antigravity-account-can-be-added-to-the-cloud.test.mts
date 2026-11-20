// 2026-11-20: the cloud setup screen adds an Antigravity account the way it
// adds a Claude one: Google's page opens, its code is copied back, and the
// account joins the list with its mark. It is offered only on a Mac that runs
// Antigravity, since a cloud account of an engine this Mac never starts would
// never be used; Claude and Codex stay offered as before.
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { SETUP, addableKinds, kindName, planName, signInView } from '../renderer/src/cloud-setup.ts';
import { CloudSetup } from '../renderer/src/components/CloudSetup.tsx';
import { accountLabel, engineWord, type CloudAccount, type CloudSignIn, type CloudStatus } from '../renderer/src/cloud-menu.ts';
import { Supervisor } from '../main/supervisor.mts';
import { recordCloudLogin } from '../main/cloud/logins.mts';

const text = (html: string) => html.replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ');
const month = { runs: 0, agentHours: 0, estimatedCost: 0, costPerAgentHour: null };
const status = (over: Partial<CloudStatus> = {}) => ({ setUp: true, managed: false, mode: 'mac', running: 0, month, accounts: [], signIns: [], ...over }) as CloudStatus;
const draw = (cloud: CloudStatus) => renderToStaticMarkup(React.createElement(CloudSetup, { cloud, onMode: () => {}, onClose: () => {} }));

describe('the cloud setup on a Mac that runs Antigravity', () => {
  it('offers an Antigravity account beside Claude and Codex, signed in with Google', () => {
    const html = draw(status({ kinds: ['claude', 'codex', 'antigravity'] }));
    expect(text(html)).toContain('Antigravity Google account');
    expect(html).toContain('engine-logo antigravity');
    expect(text(html)).toContain(SETUP.homeLedeThree);
  });

  it('says it in one short sentence, as the line it replaces does', () => {
    expect(SETUP.homeLedeThree.split(' ').length).toBeLessThanOrEqual(10);
    expect(SETUP.homeLedeThree).toMatch(/^[A-Z][^.:;—–]*\.$/);
  });

  it('offers only Claude and Codex where this Mac does not run it, and where the app is older than the list', () => {
    for (const cloud of [status({ kinds: ['claude', 'codex'] }), status()]) {
      const html = draw(cloud);
      expect(html).not.toContain('engine-logo antigravity');
      expect(text(html)).toContain(SETUP.homeLede);
    }
    expect(addableKinds({})).toEqual(['claude', 'codex']);
  });
});

describe('an Antigravity sign-in on that screen', () => {
  const s = (state: CloudSignIn['state']) => ({ engine: 'antigravity', key: 'acct-g1', state, url: 'https://accounts.google.com/o/oauth2/auth?x=1', code: null, message: null }) as CloudSignIn;

  it('names Google’s page and asks for the code it shows, to paste or copy back', () => {
    expect(signInView(s('starting'))).toMatchObject({ lead: 'Opening Google’s sign-in…', busy: true });
    expect(signInView(s('needs-code'))).toMatchObject({ lead: 'Approve on Google’s page, then copy the code it shows.', paste: true, codeToType: null });
    expect(SETUP.signInHead('antigravity')).toBe('Sign in to Antigravity');
    const html = draw(status({ kinds: ['claude', 'codex', 'antigravity'], signIns: [s('needs-code')] }));
    expect(html).toContain('aria-label="Antigravity’s sign-in code"');
  });

  it('names the account by its kind while it is added and after', () => {
    expect(kindName('antigravity')).toBe('Antigravity');
    expect(planName('antigravity')).toBe('Google account');
    expect(SETUP.menuAdding('antigravity')).toBe('Adding your Antigravity account…');
    const a = { engine: 'antigravity', key: 'acct-g1', name: null, ready: true, state: 'ok' } as unknown as CloudAccount;
    expect(accountLabel(a, [a])).toBe('Antigravity account');
    expect(engineWord(a)).toBe('Antigravity');
  });
});

describe('the app’s side of that screen', () => {
  let home: string;
  beforeEach(() => { home = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'cloud-agy-kinds-'))); });
  afterEach(() => { fs.rmSync(home, { recursive: true, force: true }); });
  const supervisor = (engines: string[]) => Object.assign(new Supervisor({ home, storeRoot: home, maxConcurrentSessions: 1, authProfiles: ['default'] }, { listItems: () => [], listProducts: () => [] }, '/nonexistent-app'), {
    engineChoices: () => engines.map((id) => ({ id })),
  });

  it('offers Antigravity only where this Mac runs it', () => {
    expect(supervisor(['claude', 'codex', 'antigravity'])._cloudKinds()).toEqual(['claude', 'codex', 'antigravity']);
    expect(supervisor(['claude'])._cloudKinds()).toEqual(['claude', 'codex']);
  });

  it('lists the Antigravity accounts added for the cloud, by the email Google gave', () => {
    recordCloudLogin('antigravity', 'acct-g1', { id: 'id-g1', email: 'me@example.test' }, { home });
    const accounts = supervisor(['claude', 'antigravity'])._cloudAccounts() as { engine: string; key: string; name: string | null }[];
    expect(accounts.map((a) => [a.engine, a.key, a.name])).toEqual([['antigravity', 'acct-g1', 'me@example.test']]);
  });
});
