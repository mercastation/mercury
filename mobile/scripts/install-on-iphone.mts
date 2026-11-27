// PUTS THE IPHONE APP ON YOUR OWN IPHONE WITH A FREE APPLE ACCOUNT
// (w-dcdf1ccfec, 2026-11-25). TestFlight needs Apple's paid program. A free
// Apple account signed in to Xcode (its Personal Team) signs the app for its
// owner's own iPhone for seven days at a time, without push or Sign in with
// Apple (MOBILE_PERSONAL_TEAM in app.config.ts). Run it again within the week
// to carry on; what the app keeps (the sign-in, the last inbox) stays.
//
// Once, by hand: Xcode › Settings › Accounts › + your Apple Account; the iPhone
// on the cable, unlocked, Trust; then on the iPhone, Settings › Privacy &
// Security › Developer Mode on. Xcode 27 keeps an account's teams nowhere a
// script can read them, so the first time also open mobile/ios/Mercury.xcworkspace
// and pick your team under Mercury › Signing & Capabilities › Team; after that
// the certificate Xcode made names it. After the first install, Settings ›
// General › VPN & Device Management › your Apple Account › Trust.
//
//   node mobile/scripts/install-on-iphone.mts [--team <id>] [--device <name or udid>] [--fresh]
//
// --fresh makes the native project again (after a package or app.config.ts change).
import { spawnSync, type SpawnSyncOptions } from 'node:child_process';
import { X509Certificate } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const mobile = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BUNDLE_ID = process.env.MOBILE_BUNDLE_ID || 'io.github.matheusmercadante.mercury';

export interface Device { identifier: string; udid: string; name: string; developerMode: string | null; paired: boolean }

/** The physical iPhones in `devicectl list devices` JSON (Xcode 27 lists simulators there too). */
export function iphonesIn(json: unknown): Device[] {
  const list = (json as { result?: { devices?: unknown[] } } | null)?.result?.devices ?? [];
  return list.flatMap((d) => {
    const x = d as { identifier?: string; hardwareProperties?: Record<string, string>; deviceProperties?: Record<string, string>; connectionProperties?: Record<string, string> };
    const hw = x.hardwareProperties ?? {};
    if (hw.reality !== 'physical' || hw.deviceType !== 'iPhone' || !x.identifier || !hw.udid) return [];
    return [{ identifier: x.identifier, udid: hw.udid, name: x.deviceProperties?.name ?? 'iPhone', developerMode: x.deviceProperties?.developerModeStatus ?? null, paired: x.connectionProperties?.pairingState === 'paired' }];
  });
}

export function pickIphone(phones: Device[], wanted?: string): Device {
  const found = wanted ? phones.filter((p) => p.name === wanted || p.udid === wanted || p.identifier === wanted) : phones;
  if (found.length === 1) return found[0];
  if (!phones.length) throw new Error('No iPhone found. Plug it into this Mac, unlock it and tap Trust.');
  throw new Error(`${found.length ? 'More than one' : 'No'} iPhone matches${wanted ? ` "${wanted}"` : ''}; pass --device with one of: ${phones.map((p) => p.name).join(', ')}`);
}

/** The team picked once in Xcode (Signing & Capabilities › Team), as the project file keeps it. */
export function teamInProject(pbxproj: string): string | null {
  const ids = new Set([...pbxproj.matchAll(/DEVELOPMENT_TEAM = "?([A-Z0-9]{10})"?;/g)].map((m) => m[1]));
  return ids.size === 1 ? [...ids][0] : null;
}

/**
 * The teams of the Apple Development certificates in the keychain (each
 * subject's OU). Xcode 27 keeps an account's teams nowhere on disk, but the
 * certificate it makes at the first signing names the team, so a project made
 * again (--fresh) still finds it.
 */
export function teamsInCertificates(pems: string): string[] {
  const teams = new Set<string>();
  for (const pem of pems.match(/-----BEGIN CERTIFICATE-----[\s\S]+?-----END CERTIFICATE-----/g) ?? []) {
    try {
      const ou = /^OU=([A-Z0-9]{10})$/m.exec(new X509Certificate(pem).subject)?.[1];
      if (ou) teams.add(ou);
    } catch { /* not one we can read */ }
  }
  return [...teams];
}

/** The free team's id from `defaults read com.apple.dt.Xcode` text (Xcode before 27); a lone team is taken as it is. */
export function freeTeamIn(defaults: string): string | null {
  const teams = [...defaults.matchAll(/\{[^{}]*?teamID = "?([A-Z0-9]{10})"?;[^{}]*?\}/g)].map((m) => ({ id: m[1], free: /isFreeProvisioningTeam = 1;/.test(m[0]) }));
  const free = teams.filter((t) => t.free);
  if (free.length === 1) return free[0].id;
  const ids = [...new Set(teams.map((t) => t.id))];
  return ids.length === 1 ? ids[0] : null;
}

function run(cmd: string, args: string[], opts: SpawnSyncOptions = {}) {
  const r = spawnSync(cmd, args, { stdio: 'inherit', ...opts });
  if (r.status !== 0) throw new Error(`${cmd} ${args.slice(0, 3).join(' ')} … stopped (${r.status ?? r.signal})`);
}

function read(cmd: string, args: string[]) {
  const r = spawnSync(cmd, args, { encoding: 'utf8' });
  return r.status === 0 ? r.stdout : '';
}

function main(argv: string[]) {
  const arg = (name: string) => { const i = argv.indexOf(name); return i >= 0 ? argv[i + 1] : undefined; };

  const pbxproj = path.join(mobile, 'ios', 'Mercury.xcodeproj', 'project.pbxproj');
  const certified = teamsInCertificates(read('security', ['find-certificate', '-a', '-c', 'Apple Development', '-p']));
  const team = arg('--team')
    ?? (fs.existsSync(pbxproj) ? teamInProject(fs.readFileSync(pbxproj, 'utf8')) : null)
    ?? (certified.length === 1 ? certified[0] : null)
    ?? freeTeamIn(read('defaults', ['read', 'com.apple.dt.Xcode']));
  if (!team) throw new Error('No team to sign with. Add your Apple Account in Xcode › Settings › Accounts, then open mobile/ios/Mercury.xcworkspace once and pick it under Mercury › Signing & Capabilities › Team (or pass --team <id>).');

  const listed = path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'iphone-')), 'devices.json');
  run('xcrun', ['devicectl', 'list', 'devices', '--json-output', listed], { stdio: 'ignore' });
  const phone = pickIphone(iphonesIn(JSON.parse(fs.readFileSync(listed, 'utf8'))), arg('--device'));
  if (!phone.paired) throw new Error(`${phone.name} has not trusted this Mac yet: unlock it on the cable and tap Trust.`);
  if (phone.developerMode === 'disabled') throw new Error(`Turn on Developer Mode on ${phone.name}: Settings › Privacy & Security › Developer Mode, then restart it.`);

  // Without the team's config (not in git) the app is built for Agentbox Cloud,
  // where a Mercury account cannot sign in (app.config.ts cloudConfig).
  const teamCloud = path.join(mobile, '..', 'cloud', 'team.config.json');
  if (!process.env.EXPO_PUBLIC_CLOUD_URL && !fs.existsSync(teamCloud)) throw new Error(`No ${path.relative(process.cwd(), teamCloud)}, so the app would sign in to Agentbox Cloud, not yours. Copy it from the app folder; git does not carry it.`);
  console.log(`Signs in to ${process.env.EXPO_PUBLIC_CLOUD_URL ?? JSON.parse(fs.readFileSync(teamCloud, 'utf8')).url}.`);

  const env = { ...process.env, MOBILE_PERSONAL_TEAM: '1', LANG: 'en_US.UTF-8', LC_ALL: 'en_US.UTF-8', CI: '1' };
  const entitlements = path.join(mobile, 'ios', 'Mercury', 'Mercury.entitlements');
  const madeForPaid = fs.existsSync(entitlements) && /aps-environment|applesignin/.test(fs.readFileSync(entitlements, 'utf8'));
  if (argv.includes('--fresh') || !fs.existsSync(path.join(mobile, 'ios')) || madeForPaid) {
    // Prebuild rewrites package.json's ios and android scripts; they are put back as they were.
    const pkg = fs.readFileSync(path.join(mobile, 'package.json'), 'utf8');
    try { run('npx', ['expo', 'prebuild', '--platform', 'ios', '--clean'], { cwd: mobile, env }); } finally { fs.writeFileSync(path.join(mobile, 'package.json'), pkg); }
  }

  const derived = path.join(mobile, 'ios', 'build-iphone');
  console.log(`Building for ${phone.name}, signed by team ${team}…`);
  run('xcodebuild', ['-workspace', 'ios/Mercury.xcworkspace', '-scheme', 'Mercury', '-configuration', 'Release', '-destination', `id=${phone.udid}`,
    '-derivedDataPath', derived, '-allowProvisioningUpdates', '-allowProvisioningDeviceRegistration', '-quiet',
    `DEVELOPMENT_TEAM=${team}`, 'CODE_SIGN_STYLE=Automatic', 'build'], { cwd: mobile, env });

  const app = path.join(derived, 'Build', 'Products', 'Release-iphoneos', 'Mercury.app');
  run('xcrun', ['devicectl', 'device', 'install', 'app', '--device', phone.identifier, app]);
  const profile = read('security', ['cms', '-D', '-i', path.join(app, 'embedded.mobileprovision')]);
  const until = /<key>ExpirationDate<\/key>\s*<date>([^<]+)<\/date>/.exec(profile)?.[1];
  console.log(`Installed on ${phone.name}${until ? `; it opens until ${new Date(until).toLocaleString()}` : ''}.`);
  const opened = spawnSync('xcrun', ['devicectl', 'device', 'process', 'launch', '--device', phone.identifier, BUNDLE_ID], { stdio: 'ignore' });
  if (opened.status !== 0) console.log('If it does not open: Settings › General › VPN & Device Management › your Apple Account › Trust, then tap Mercury.');
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  try { main(process.argv.slice(2)); } catch (err) { console.error((err as Error).message); process.exit(1); }
}
