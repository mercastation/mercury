// A CLOUD RUN CARRIES ITS PROJECT'S BROWSER, WITH THE SIGN-INS MADE ON THE MAC.
//
// Until 2026-11-15 a run in the cloud had no project browser at all: the app
// handed it to local runs only, and every server in a cloud run's MCP config
// was a stub calling back to the Mac, which stops answering once the Mac
// sleeps. The owner asked for the cloud to use the same sign-ins. Now, when the
// cloud image says it can run the browser (`browser` in its caps) and the Mac's
// host hands over its sign-ins, the run takes them along and its
// `project-browser` server is the copy in the sandbox
// (main/project-browser/cloud-host.mts). What the sites renewed comes back and
// goes to the host here, which takes only cookies its profile already holds on
// its own sites. Anything else leaves the stub, which still works while the
// Mac is awake.
import { describe, expect, it } from 'vitest';
import { browserForTheCloud, CLOUD_BROWSER_HOST, takeBrowserHome } from '../main/cloud/browser.mts';
import { mcpConfigForTheCloud } from '../main/cloud/run.mts';
import { cookiesToTake, renewedCookies, type Cookie } from '../main/project-browser/host.mts';
import type { HostRequest } from '../main/project-browser/place.mts';
import { BROWSER_SERVER } from '../shared/project-browser.mts';

const servers = { [BROWSER_SERVER]: { command: '/app/electron', args: ['mcp/project-browser.mts'], env: { PROJECT_BROWSER_SOCK: '/tmp/host.sock', PROJECT_BROWSER_THREAD: 'w-1a2b3c4d5e' } }, mercury: { command: 'node', args: ['mcp/server.mts'], env: {} } };
const signedIn = { cookies: [{ name: 'remember_user_token', value: 'v1', domain: 'gitlab.example.gov', path: '/', expires: 1_900_000_000, httpOnly: true, secure: true, sameSite: 'Lax' }], origins: [] };

const hostSaying = (answer: unknown) => {
  const asked: HostRequest[] = [];
  const ask = async <T,>(_sock: string, request: HostRequest): Promise<T | null> => { asked.push(request); return answer as T; };
  return { ask, asked };
};

describe('a cloud run on a project with a browser', () => {
  it('carries the sign-ins and runs the browser in the sandbox when the image can', async () => {
    const host = hostSaying({ ok: true, sites: ['gitlab.example.gov'], state: signedIn });
    const browser = await browserForTheCloud(servers, { runId: 'run-1', caps: ['offline', 'browser'], ask: host.ask });
    expect(host.asked).toEqual([{ method: 'state' }]);
    expect(browser?.state).toEqual(signedIn);
    expect(browser?.given).toMatch(/^\/tmp\/[a-z]+-browser-run-1\.json$/);
    expect(browser?.out).toMatch(/^\/tmp\/[a-z]+-browser-run-1-renewed\.json$/);
    expect(browser?.server).toEqual({
      command: 'node',
      args: [CLOUD_BROWSER_HOST],
      env: { PROJECT_BROWSER_SITES: 'gitlab.example.gov', PROJECT_BROWSER_STATE: browser!.given, PROJECT_BROWSER_OUT: browser!.out, PROJECT_BROWSER_THREAD: 'w-1a2b3c4d5e' },
    });
  });

  it('keeps every other server a stub calling back to the Mac', () => {
    const cloud = mcpConfigForTheCloud({ mcpServers: servers }, '/opt/agentbox/mcp-stub.mjs', { [BROWSER_SERVER]: { command: 'node', args: [CLOUD_BROWSER_HOST], env: {} } });
    expect(cloud.mcpServers[BROWSER_SERVER]).toEqual({ command: 'node', args: [CLOUD_BROWSER_HOST], env: {} });
    expect(cloud.mcpServers.mercury).toEqual({ command: 'node', args: ['/opt/agentbox/mcp-stub.mjs', 'mercury'], env: {} });
    expect(mcpConfigForTheCloud({ mcpServers: servers }, '/stub').mcpServers[BROWSER_SERVER]).toEqual({ command: 'node', args: ['/stub', BROWSER_SERVER], env: {} });
  });

  it('leaves the stub when the image cannot run the browser, and asks the host nothing', async () => {
    const host = hostSaying({ ok: true, sites: ['gitlab.example.gov'], state: signedIn });
    expect(await browserForTheCloud(servers, { runId: 'run-1', caps: ['offline'], ask: host.ask })).toBeNull();
    expect(host.asked).toEqual([]);
  });

  it('leaves the stub when the host does not answer, is being signed in to, or lists no site', async () => {
    for (const answer of [null, { ok: false, error: 'signing-in' }, { ok: true, sites: [], state: signedIn }, { ok: true, sites: ['a.example'] }]) {
      expect(await browserForTheCloud(servers, { runId: 'run-1', caps: ['browser'], ask: hostSaying(answer).ask })).toBeNull();
    }
  });

  it('is nothing on a project with no browser', async () => {
    expect(await browserForTheCloud({ mercury: servers.mercury }, { runId: 'run-1', caps: ['browser'], ask: hostSaying({ ok: true }).ask })).toBeNull();
  });
});

describe('what the run\'s sites renewed', () => {
  const cookie = (name: string, value: string, more: Partial<Cookie> = {}): Cookie => ({ name, value, domain: 'gitlab.example.gov', path: '/', expires: 1_900_000_000, httpOnly: true, secure: true, sameSite: 'Lax', ...more });

  it('is only what the copy was given and holds differently now', () => {
    const given = [cookie('session', 'a'), cookie('remember', 'r')];
    const now = [cookie('session', 'b'), cookie('remember', 'r'), cookie('tracker', 'x')];
    expect(renewedCookies(given, now)).toEqual([cookie('session', 'b')]);
    expect(renewedCookies(given, [cookie('remember', 'r', { expires: 1_950_000_000 })])).toEqual([cookie('remember', 'r', { expires: 1_950_000_000 })]);
    expect(renewedCookies(given, given)).toEqual([]);
  });

  it('is taken here only for cookies the profile holds, on the project\'s sites', () => {
    const held = [cookie('session', 'a'), cookie('sso', 's', { domain: '.example.gov' })];
    const came = [cookie('session', 'b'), cookie('new-one', 'n'), cookie('session', 'z', { domain: 'elsewhere.example' }), cookie('sso', 't', { domain: '.example.gov' }), { name: 'broken' }];
    expect(cookiesToTake(came, held, ['gitlab.example.gov'])).toEqual([cookie('session', 'b')]);
    expect(cookiesToTake(came, held, ['example.gov'])).toEqual([cookie('session', 'b'), cookie('sso', 't', { domain: '.example.gov' })]);
    expect(cookiesToTake('not a list', held, ['gitlab.example.gov'])).toEqual([]);
  });

  it('goes to the host here when the run comes home', async () => {
    const host = hostSaying({ ok: true, taken: 1 });
    const renewed = [cookie('session', 'b')];
    expect(await takeBrowserHome('/tmp/host.sock', Buffer.from(JSON.stringify({ cookies: renewed })), { ask: host.ask })).toBeNull();
    expect(host.asked).toEqual([{ method: 'take-cookies', cookies: renewed }]);
  });

  it('asks nothing when nothing came back or nothing was renewed, and says so when it could not be taken', async () => {
    const host = hostSaying({ ok: true });
    expect(await takeBrowserHome('/tmp/host.sock', null, { ask: host.ask })).toBeNull();
    expect(await takeBrowserHome('/tmp/host.sock', Buffer.from('{"cookies":[]}'), { ask: host.ask })).toBeNull();
    expect(host.asked).toEqual([]);
    expect(await takeBrowserHome('/tmp/host.sock', Buffer.from('not json'), { ask: host.ask })).toMatch(/could not be read/);
    expect(await takeBrowserHome('/tmp/host.sock', Buffer.from(JSON.stringify({ cookies: [cookie('s', 'b')] })), { ask: hostSaying(null).ask })).toMatch(/did not answer/);
    expect(await takeBrowserHome('/tmp/host.sock', Buffer.from(JSON.stringify({ cookies: [cookie('s', 'b')] })), { ask: hostSaying({ ok: false, error: 'signing-in' }).ask })).toMatch(/signing-in/);
  });
});
