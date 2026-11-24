// `thread-filed` with three proposals waiting and one finished: the count line says how many,
// and answers for the lot with Dismiss all and Start all (`.made-all`).
import { filed, PROPOSAL, FINISHED } from './thread-filed.mjs';

export const MANY = [
  PROPOSAL,
  { id: 'w-f11ed00003', title: 'Add a test for a cart of exactly $1,000' },
  { id: 'w-f11ed00004', title: 'Tell the tax service team their release notes missed the change' },
  FINISHED,
];

export const go = (h) => filed(h, MANY);
