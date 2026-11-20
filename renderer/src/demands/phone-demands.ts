// DEMANDS ON A PHONE (cloud-bridge.ts): each computer's demands from its
// settings copy (main/phone-inbox.mts), with what the phone sent and the
// computer has not taken drawn as if it had, as phone-repeats.ts does.
import { phoneSlug, type PhoneRows } from '../phone-rows';
import type { DemandRecord } from './demand-rules';

type Command = PhoneRows['commands'][number];
export interface DemandSettingsRow { device: string; settings?: { demands?: unknown } | null; updated_at?: string }

// As long as phone-rows.ts shows a reply that landed before its thread's new copy.
const LANDING_SHOWN_MS = 2 * 3600_000;
type DemandSetting = Command & { payload: Command['payload'] & { product: string } };
const isDemandSetting = (c: Command): c is DemandSetting => c.kind === 'setting' && c.payload?.scope === 'demand' && typeof c.payload?.product === 'string';

export function phoneDemands(settings: DemandSettingsRow[], commands: Command[], projects: PhoneRows['projects'], now = Date.now()): DemandRecord[] {
  const names = new Map(projects.map((p) => [phoneSlug(p.device, p.project), p.name]));
  const copyAt = new Map<string, number>();
  let out: DemandRecord[] = [];
  for (const row of settings) {
    // Newest first: an older row of the same computer is not read again.
    if (copyAt.has(row.device)) continue;
    copyAt.set(row.device, Date.parse(row.updated_at ?? '') || 0);
    const list = Array.isArray(row.settings?.demands) ? row.settings!.demands as Partial<DemandRecord>[] : [];
    for (const d of list) {
      const product = phoneSlug(row.device, String(d?.product ?? ''));
      if (typeof d?.id !== 'string' || typeof d.name !== 'string' || !names.has(product)) continue;
      out.push({ ...d, id: d.id, name: d.name, product, productName: names.get(product)!, threads: Array.isArray(d.threads) ? [...d.threads] : [], createdAt: Number(d.createdAt) || 0, updatedAt: Number(d.updatedAt) || 0 });
    }
  }
  const sent = commands.filter(isDemandSetting).sort((a, b) => String(a.created_at).localeCompare(String(b.created_at)));
  for (const c of sent) {
    const takenAt = c.taken_at ? Date.parse(c.taken_at) || 0 : null;
    const standing = takenAt === null ? !c.outcome : c.outcome === 'done' && (copyAt.get(c.device) ?? 0) < takenAt && now - takenAt < LANDING_SHOWN_MS;
    if (!standing) continue;
    const p = c.payload;
    const product = phoneSlug(c.device, p.product);
    if (!names.has(product)) continue;
    if (p.key === 'place' && typeof p.item === 'string') {
      const thread = p.item;
      out = out.map((d) => (d.product !== product ? d : { ...d, threads: [...d.threads.filter((t) => t !== thread), ...(d.id === p.value ? [thread] : [])] }));
    } else if (p.key === 'change') {
      const patch = { ...(p.value as Partial<DemandRecord>) };
      out = out.flatMap((d) => {
        if (d.product !== product || d.id !== p.id) return [d];
        if (patch.endedAt) return [];
        const next = { ...d, ...patch };
        if (!next.today) delete next.today;
        return [next];
      });
    } else if (p.key === 'new') {
      const created = Date.parse(c.created_at) || now;
      out.push({ ...(p.value as Pick<DemandRecord, 'name'> & Partial<DemandRecord>), id: `phone-${c.id}`, product, productName: names.get(product)!, threads: [], createdAt: created, updatedAt: created });
    }
  }
  return out;
}
