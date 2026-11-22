// A thread that named more files than its list shows at once, with the list open: fourteen,
// so it stands 440 points tall and scrolls, one with a name too long for its row, cut short
// with dots, and two with notes on them, which say how many. Notes, decks, reports and sheets
// only: a picture or a film is drawn in the conversation, and these need not be planted.
import { open } from './thread.mjs';

const FILES = [
  'notes/checkout-rounding-rules-for-carts-over-a-thousand-dollars.md',
  'reports/week-32.pdf',
  'exports/refunds.csv',
  'exports/failed-orders.csv',
  'notes/support-reply.md',
  'decks/rounding-review.pptx',
  'reports/tax-service-release.pdf',
  'exports/orders-by-cart-size.tsv',
  'notes/refund-steps.md',
  'notes/what-changed.md',
  'reports/week-33.pdf',
  'exports/retries.csv',
  'decks/support-briefing.pptx',
  'notes/follow-ups.md',
];

const SHIPPED = `**Every file from the rounding work, in one place.**

${FILES.map((file) => `- ${file}`).join('\n')}`;

// Two notes on the week's report and one on the refunds, each on the version it was left on.
const note = (id, file, text, ts) => ({ id, file, v: 1, place: { kind: 'whole' }, text, by: null, ts, mentions: [], owner: null, uid: null });
const MARKS = {
  notes: [note('n1', 'week-32.pdf', 'Is the total on page two before the refunds?', 1), note('n2', 'week-32.pdf', 'Add the store credit line.', 2), note('n3', 'refunds.csv', 'Order 1007 was refunded twice.', 3)],
  approvals: [],
  seen: {},
};

export const go = (h) => open(h, {
  unfold: false,
  shipped: SHIPPED,
  marks: MARKS,
  then: async (h) => {
    await h.evaluate(`document.querySelector('.ts-files')?.click()`);
    await h.wait(300);
    return { filesOpen: true };
  },
});
