import { createContext } from 'react';
import type { DemandRecord } from './demand-rules';

/** Each thread's demand, by `project:id` (./demand-rules.ts `demandByThread`). */
export const DemandsContext = createContext<Map<string, DemandRecord>>(new Map());

/** Every demand, and the way a person puts a thread in one or in none. Null where demands are not shown. */
export const DemandPlacingContext = createContext<{
  demands: readonly DemandRecord[];
  place: (item: { product: string; id: string }, demand: string | null) => void;
} | null>(null);

export const threadKey = (i: { product: string; id: string }) => `${i.product}:${i.id}`;
