// THE IPHONE APP INSTALLS BY CABLE WITH A FREE APPLE ACCOUNT (w-dcdf1ccfec,
// 2026-11-25). The owner would not pay Apple's yearly program for TestFlight,
// so mobile/scripts/install-on-iphone.mts signs the app with the free team
// Xcode holds for his Apple Account and puts it on his iPhone, again each
// week. These are the parts that decide which iPhone and which team.
import { describe, it, expect } from 'vitest';
import { freeTeamIn, iphonesIn, pickIphone } from '../mobile/scripts/install-on-iphone.mts';

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
