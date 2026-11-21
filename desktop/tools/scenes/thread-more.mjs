// The same conversation with the markdown its first answer does not have: the other headings,
// a quote, lists inside lists and with paragraphs, struck words, a rule, a table whose cells
// wrap, and a code line longer than the column. For a tall window (1280 by 1700 shows it all).
import { open } from './thread.mjs';

const MORE = `# What the logs say

The failures are ~~random~~ tied to one tax table, and they started with the 08:10 deploy.

### Where it breaks

> Rounding happens after the total is added up, so a cart can be a cent short of what the
> card was charged.

- Carts over $1,000
  - paid by card
  - with more than one tax line
- Carts with a coupon

1. Read the tax service's changes.

   They moved from two decimal places to four.

2. Run the checkout tests against yesterday's rates.

#### What each fix costs

| Fix | What it touches | Risk |
| --- | --- | --- |
| Round before adding | One function and its three callers in the checkout service | Low |
| Pin the old rates | The tax client's configuration, until the tax team answers | Medium |

---

\`\`\`sh
curl -s https://tax.example.com/v2/rates?region=us-ca&cart=1049.99&lines=3&round=after | jq '.lines[] | {rate, amount, rounded}'
\`\`\`
`;

export const go = (h) => open(h, { unfold: true, found: MORE });
