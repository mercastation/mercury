// A DEMAND IS ON THE PHONE, AND IS CHANGED FROM IT.
//
// Demands were asked for on the phone first: it is where their person looks at
// the day. A demand is not a thread, so it does not ride with the threads'
// copies. It rides with the computer's settings copy, as a repeating task
// does, and what the phone changes goes back as a setting the Mac checks and
// hands to the window's own handlers.
//
// Covered here: what the Mac copies and what it leaves, each change a phone may
// send and the ones refused, the handler each goes to, and the phone drawing a
// change before the Mac has taken it.
import { describe, it, expect } from 'vitest';
import { phoneDemandsOf } from '../shared/phone-snapshot.mts';
import { checkSetting, settingRoute } from '../shared/phone-settings.mts';
import { phoneDemands } from '../renderer/src/demands/phone-demands';

const NOW = 1_795_000_000_000;
const demand = (id: string, over: Record<string, unknown> = {}) => ({
  id, product: 'shop', productName: 'Shop', name: `Demand ${id}`, threads: ['w-1', 'w-2'], createdAt: NOW - 1000, updatedAt: NOW, ...over,
});

describe('what the Mac sends up', () => {
  it('is each demand of a project on the phone, with its threads and its report', () => {
    const [sent] = phoneDemandsOf([demand('d-1', { about: 'What it is.', delivered: ['One', 'Two'], outcome: 'Done.', today: '2026-11-20', wroteAt: NOW })], ['shop']);
    expect(sent).toEqual({
      product: 'shop', id: 'd-1', name: 'Demand d-1', about: 'What it is.', delivered: ['One', 'Two'], outcome: 'Done.',
      today: '2026-11-20', threads: ['w-1', 'w-2'], createdAt: NOW - 1000, updatedAt: NOW,
    });
  });

  it('leaves out a project that is not on the phone, an ended demand and a row that is not one', () => {
    const sent = phoneDemandsOf([demand('d-1'), demand('d-2', { product: 'studio' }), demand('d-3', { endedAt: NOW }), { product: 'shop', name: 'no id' } as never, null as never], ['shop']);
    expect(sent.map((d) => d.id)).toEqual(['d-1']);
  });
});

describe('a change sent from the phone', () => {
  const ok = (change: Record<string, unknown>) => expect(checkSetting({ scope: 'demand', product: 'shop', ...change } as never)).toBeNull();
  const refused = (change: Record<string, unknown>) => expect(checkSetting({ scope: 'demand', product: 'shop', ...change } as never)).toEqual(expect.any(String));

  it('may make a demand, rename it, pick it for a day, take the day back and remove it', () => {
    ok({ key: 'new', value: { name: 'Refunds a customer can follow' } });
    ok({ key: 'change', id: 'd-0123456789', value: { name: 'Refunds' } });
    ok({ key: 'change', id: 'd-0123456789', value: { today: '2026-11-20' } });
    ok({ key: 'change', id: 'd-0123456789', value: { today: '' } });
    ok({ key: 'change', id: 'd-0123456789', value: { endedAt: NOW } });
  });

  it('may put a thread in a demand, or in none', () => {
    ok({ key: 'place', item: 'w-1', value: 'd-0123456789' });
    ok({ key: 'place', item: 'w-1', value: null });
  });

  it('is refused when it is not one of those', () => {
    refused({ key: 'new', value: { about: 'no name' } });
    refused({ key: 'new', value: { name: '   ' } });
    refused({ key: 'change', value: { name: 'no id' } });
    refused({ key: 'change', id: 'd-0123456789', value: {} });
    refused({ key: 'change', id: 'd-0123456789', value: { today: 'tomorrow' } });
    refused({ key: 'change', id: 'd-0123456789', value: { wroteAt: NOW } });
    refused({ key: 'change', id: 'w-1', value: { name: 'a thread is not a demand' } });
    refused({ key: 'place', value: 'd-0123456789' });
    refused({ key: 'place', item: 'w-1', value: 'w-2' });
    refused({ key: 'delete', id: 'd-0123456789', value: true });
    expect(checkSetting({ scope: 'demand', key: 'new', value: { name: 'No project' } } as never)).toEqual(expect.any(String));
  });

  it('goes to the window\'s own handler for it', () => {
    expect(settingRoute({ scope: 'demand', key: 'new', product: 'shop', value: { name: 'Refunds', today: '2026-11-20' } }, null))
      .toEqual(['zero:compose-demand', { product: 'shop', name: 'Refunds', today: '2026-11-20' }]);
    expect(settingRoute({ scope: 'demand', key: 'change', product: 'shop', id: 'd-0123456789', value: { today: '' } }, null))
      .toEqual(['zero:set-demand', { product: 'shop', id: 'd-0123456789', patch: { today: '' } }]);
    expect(settingRoute({ scope: 'demand', key: 'place', product: 'shop', item: 'w-1', value: null }, null))
      .toEqual(['zero:place-in-demand', { product: 'shop', thread: 'w-1', demand: null }]);
  });
});

describe('what the phone draws', () => {
  const projects = [{ device: 'aaaaaaaaaaaa', project: 'shop', name: 'Shop' }];
  const settings = [{ device: 'aaaaaaaaaaaa', updated_at: new Date(NOW - 60_000).toISOString(), settings: { demands: [demand('d-1'), demand('d-2', { product: 'studio' })] } }];
  const command = (payload: Record<string, unknown>, over: Record<string, unknown> = {}) => ({
    id: 'c-1', device: 'aaaaaaaaaaaa', kind: 'setting', created_at: new Date(NOW - 5000).toISOString(), taken_at: null, outcome: null,
    payload: { scope: 'demand', product: 'shop', ...payload }, ...over,
  });
  const draw = (commands: unknown[] = []) => phoneDemands(settings as never, commands as never, projects as never, NOW);

  it('is each demand of a project it has, named as the phone names that project', () => {
    const drawn = draw();
    expect(drawn).toHaveLength(1);
    expect(drawn[0]).toMatchObject({ id: 'd-1', productName: 'Shop', threads: ['w-1', 'w-2'] });
    expect(drawn[0].product).toContain('shop');
    expect(drawn[0].product).not.toBe('shop');
  });

  it('shows a pick for today before the Mac has taken it', () => {
    expect(draw([command({ key: 'change', id: 'd-1', value: { today: '2026-11-20' } })])[0].today).toBe('2026-11-20');
    expect(draw([command({ key: 'change', id: 'd-1', value: { today: '' } })])[0].today).toBeUndefined();
  });

  it('shows a thread where it was just put, and a demand just made', () => {
    expect(draw([command({ key: 'place', item: 'w-1', value: null })])[0].threads).toEqual(['w-2']);
    expect(draw([command({ key: 'place', item: 'w-9', value: 'd-1' })])[0].threads).toEqual(['w-1', 'w-2', 'w-9']);
    expect(draw([command({ key: 'new', value: { name: 'Refunds' } })]).map((d) => d.name)).toEqual(['Demand d-1', 'Refunds']);
  });

  it('drops a demand just removed, and stops drawing a change the Mac refused', () => {
    expect(draw([command({ key: 'change', id: 'd-1', value: { endedAt: NOW } })])).toEqual([]);
    expect(draw([command({ key: 'change', id: 'd-1', value: { today: '2026-11-20' } }, { taken_at: new Date(NOW - 1000).toISOString(), outcome: 'refused' })])[0].today).toBeUndefined();
  });
});
