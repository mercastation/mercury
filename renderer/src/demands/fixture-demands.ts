// Demands over the canned threads (fixtures.ts), so the Demands pages can be
// looked at with no store behind them: two picked for today, one waiting on
// the person, one with an agent on it, one not begun and two delivered.
import { dayKey } from '../../../shared/demands.mts';
import type { DemandRecord } from './demand-rules';

const now = Date.now();
const day = 86_400_000;

export const fixtureDemands: DemandRecord[] = [
  {
    id: 'd-fx-cards', product: 'kestrel', productName: 'Kestrel', name: 'Cards ready for the first booster',
    about: 'Art, frames and the build menu, so a booster can be opened from start to end.',
    delivered: ['Card art finished in both sizes', 'Card frames built, and the art fits them'],
    threads: ['w-p0', 'w-p1', 'w-p2', 'w-p3', 'w-p4'], today: dayKey(now), createdAt: now - 12 * day, updatedAt: now - 3_600_000,
  },
  {
    id: 'd-fx-first-run', product: 'onboard', productName: 'Onboard', name: 'First run of a new project',
    about: 'What a person sees from the welcome email to their first row.',
    delivered: ['Three first-run flows drawn'],
    threads: ['w-m12', 'w-n13', 'w-e5', 'w-h9'], today: dayKey(now), createdAt: now - 9 * day, updatedAt: now - 2 * 3_600_000,
  },
  {
    id: 'd-fx-pricing', product: 'kestrel', productName: 'Kestrel', name: 'Pricing page',
    about: 'One page that says what each plan costs, with the annual price beside the monthly one.',
    threads: ['w-a1', 'w-h8'], createdAt: now - 6 * day, updatedAt: now - day,
  },
  {
    id: 'd-fx-name', product: 'thicket', productName: 'Thicket', name: 'A name for the product',
    threads: ['w-j10'], createdAt: now - 4 * day, updatedAt: now - 2 * day,
  },
  {
    id: 'd-fx-league', product: 'kestrel', productName: 'Kestrel', name: 'Weekly league table',
    threads: [], createdAt: now - 2 * day, updatedAt: now - 2 * day,
  },
  {
    id: 'd-fx-program', product: 'thicket', productName: 'Thicket', name: 'Deload weeks in the program builder',
    outcome: 'Every fourth week now drops the load by a third, and the builder says so on the plan.',
    delivered: ['Deload week rule written and tested', 'The plan shows which week is the light one'],
    threads: ['w-f6'], createdAt: now - 10 * day, updatedAt: now - 2 * day,
  },
  {
    id: 'd-fx-holo', product: 'kestrel', productName: 'Kestrel', name: 'Holo effect on the rare cards',
    outcome: 'Plain CSS won: it holds sixty frames on a phone and needs no WebGL.',
    threads: ['w-g7'], createdAt: now - 8 * day, updatedAt: now - 3 * day,
  },
];
