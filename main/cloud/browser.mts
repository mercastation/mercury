// A CLOUD RUN CARRIES ITS PROJECT'S BROWSER (main/project-browser/cloud-host.mts).
//
// The run's MCP config names the project's browser as a server on this Mac,
// which in the cloud would be a stub calling back here and stop answering
// once the Mac sleeps. When the image can run the browser itself (its caps say
// `browser`) and the Mac's host hands over the sign-ins, the run takes them
// with it and the server becomes the copy in the sandbox. Whatever its sites
// renewed comes back with the results and goes to the host here.
import { nameSlug } from '../../shared/product-name.mjs';
import { BROWSER_SERVER } from '../../shared/project-browser.mts';
import { askHost, type HostRequest } from '../project-browser/place.mts';

export const CLOUD_BROWSER_HOST = '/opt/agentbox/browser/main/project-browser/cloud-host.mts';
export const BROWSER_RESULT = 'browser.json';
const STATE_MS = 60_000;
const TAKE_MS = 60_000;

interface Server { command?: string; args?: string[]; env?: Record<string, string | undefined> }
type Ask = <T>(sock: string, request: HostRequest, opts: { ms: number }) => Promise<T | null>;

export interface CloudBrowser {
  /** Where the sign-ins are laid in the sandbox, and where the renewed ones are written there. */
  given: string;
  out: string;
  /** The host here, which takes back what the sites renewed. */
  sock: string;
  state: unknown;
  server: { command: string; args: string[]; env: Record<string, string> };
}

/** The project's browser for a cloud run to carry, or null to leave the server a stub calling back here. */
export async function browserForTheCloud(servers: Record<string, Server> | null | undefined, { runId, caps, ask = askHost as Ask }: { runId: string; caps: readonly string[]; ask?: Ask }): Promise<CloudBrowser | null> {
  const here = servers?.[BROWSER_SERVER];
  const sock = here?.env?.PROJECT_BROWSER_SOCK;
  if (!sock || !caps.includes('browser')) return null;
  const got = await ask<{ ok?: boolean; sites?: string[]; state?: unknown }>(sock, { method: 'state' }, { ms: STATE_MS });
  if (!got?.ok || !got.state || !got.sites?.length) return null;
  // The sandbox's own /tmp: the sign-ins are laid there by its server, and the
  // copy, running as the agent, writes its own file beside them.
  const given = `/tmp/${nameSlug}-browser-${runId}.json`;
  const out = `/tmp/${nameSlug}-browser-${runId}-renewed.json`;
  return {
    given, out, sock, state: got.state,
    server: {
      command: 'node',
      args: [CLOUD_BROWSER_HOST],
      env: { PROJECT_BROWSER_SITES: got.sites.join(','), PROJECT_BROWSER_STATE: given, PROJECT_BROWSER_OUT: out, PROJECT_BROWSER_THREAD: here?.env?.PROJECT_BROWSER_THREAD ?? '' },
    },
  };
}

/** Hands the host here what a cloud run's sites renewed. Says why when it could not. */
export async function takeBrowserHome(sock: string, body: Buffer | null, { ask = askHost as Ask }: { ask?: Ask } = {}): Promise<string | null> {
  if (!body) return null;
  let cookies: unknown;
  try { cookies = JSON.parse(body.toString('utf8'))?.cookies; } catch { return 'the browser sign-ins it renewed could not be read'; }
  if (!Array.isArray(cookies) || !cookies.length) return null;
  const got = await ask<{ ok?: boolean; error?: string }>(sock, { method: 'take-cookies', cookies }, { ms: TAKE_MS });
  return got?.ok ? null : `the browser sign-ins it renewed were not taken here (${got?.error ?? 'the project\'s browser did not answer'})`;
}
