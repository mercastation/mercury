import { createContext } from 'react';

/** Each thread's demand, by `project:id` (./demand-rules.ts `demandByThread`). */
export const DemandsContext = createContext<Map<string, { name: string }>>(new Map());

export const threadKey = (i: { product: string; id: string }) => `${i.product}:${i.id}`;
