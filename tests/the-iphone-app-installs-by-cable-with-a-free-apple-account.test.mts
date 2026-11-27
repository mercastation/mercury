// THE IPHONE APP INSTALLS BY CABLE WITH A FREE APPLE ACCOUNT (w-dcdf1ccfec,
// 2026-11-25). The owner would not pay Apple's yearly program for TestFlight,
// so mobile/scripts/install-on-iphone.mts signs the app with the free team
// Xcode holds for his Apple Account and puts it on his iPhone, again each
// week. These are the parts that decide which iPhone and which team.
import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { freeTeamIn, iphonesIn, pickIphone, teamInProject, teamsInCertificates } from '../mobile/scripts/install-on-iphone.mts';

const sim = (name) => ({ identifier: `sim-${name}`, hardwareProperties: { reality: 'simulated', deviceType: 'iPhone', udid: `sim-${name}` }, deviceProperties: { name }, connectionProperties: { pairingState: 'paired' } });
const phone = (name, udid, extra = {}) => ({ identifier: `core-${udid}`, hardwareProperties: { reality: 'physical', deviceType: 'iPhone', udid }, deviceProperties: { name, developerModeStatus: 'enabled' }, connectionProperties: { pairingState: 'paired' }, ...extra });
const ipad = { identifier: 'core-ipad', hardwareProperties: { reality: 'physical', deviceType: 'iPad', udid: 'ipad' }, deviceProperties: { name: 'iPad' } };

describe('which iPhone', () => {
  it('Xcode 27 lists simulators beside phones; only a physical iPhone counts', () => {
    const found = iphonesIn({ result: { devices: [sim('iPhone 17 Pro'), ipad, phone('Meu iPhone', '00008110-0001')] } });
    expect(found).toEqual([{ identifier: 'core-00008110-0001', udid: '00008110-0001', name: 'Meu iPhone', developerMode: 'enabled', paired: true }]);
  });
  it('one iPhone is the one', () => {
    expect(pickIphone(iphonesIn({ result: { devices: [phone('Meu iPhone', 'a')] } })).udid).toBe('a');
  });
  it('none says to plug it in and trust', () => {
    expect(() => pickIphone([])).toThrow(/Plug it into this Mac, unlock it and tap Trust/);
  });
  it('two ask which, by name', () => {
    const two = iphonesIn({ result: { devices: [phone('Um', 'a'), phone('Dois', 'b')] } });
    expect(() => pickIphone(two)).toThrow(/pass --device with one of: Um, Dois/);
    expect(pickIphone(two, 'Dois').udid).toBe('b');
  });
  it('an answer with no list is no iPhone', () => {
    expect(iphonesIn(null)).toEqual([]);
  });
});

describe('which team signs it', () => {
  const xcode = (teams) => `{\n    IDEProvisioningTeamByIdentifier =     {\n        "acct" =         (\n${teams.join(',\n')}\n        );\n    };\n}`;
  const team = (id, free) => `            {\n                isFreeProvisioningTeam = ${free ? 1 : 0};\n                teamID = ${id};\n                teamName = "Someone";\n            }`;
  it('the free team, when there is a paid one too', () => {
    expect(freeTeamIn(xcode([team('PAID000001', false), team('FREE000001', true)]))).toBe('FREE000001');
  });
  it('a lone team as it is', () => {
    expect(freeTeamIn(xcode([team('ONLY000001', false)]))).toBe('ONLY000001');
  });
  it('no account in Xcode is no team', () => {
    expect(freeTeamIn('{\n    DVTDeveloperAccountManagerAppleIDLists = { "IDE.Identifiers.Prod" = ( ); };\n}')).toBeNull();
  });
});

// Xcode 27 keeps an account's teams nowhere on disk (looked for on 2026-11-27:
// not in its settings, caches or logs), so the team comes from the project file
// once it was picked in Xcode, else from the certificate Xcode made with it.
describe('the team, where Xcode 27 leaves it', () => {
  it('the project file, once it was picked in Xcode', () => {
    expect(teamInProject('\t\t\t\tDEVELOPMENT_TEAM = ABCDE12345;\n\t\t\t\tDEVELOPMENT_TEAM = ABCDE12345;')).toBe('ABCDE12345');
    expect(teamInProject('\t\t\t\tDEVELOPMENT_TEAM = "ABCDE12345";')).toBe('ABCDE12345');
  });
  it('no team, or two, is not one', () => {
    expect(teamInProject('PRODUCT_NAME = Mercury;')).toBeNull();
    expect(teamInProject('DEVELOPMENT_TEAM = ABCDE12345;\nDEVELOPMENT_TEAM = FGHIJ67890;')).toBeNull();
  });
  const openssl = spawnSync('openssl', ['version']).status === 0;
  it.skipIf(!openssl)('an Apple Development certificate names its team in its OU', () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'cert-'));
    const made = spawnSync('openssl', ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1', '-keyout', path.join(dir, 'k.pem'), '-out', path.join(dir, 'c.pem'),
      '-subj', '/UID=ABCDEFGHIJ/CN=Apple Development: someone (ZZZZZZZZZZ)/OU=TEAM123456/O=Someone/C=BR']);
    expect(made.status).toBe(0);
    const pem = fs.readFileSync(path.join(dir, 'c.pem'), 'utf8');
    expect(teamsInCertificates(pem + 'not a certificate\n' + pem)).toEqual(['TEAM123456']);
  });
  it('no certificate is no team', () => {
    expect(teamsInCertificates('')).toEqual([]);
  });
});
