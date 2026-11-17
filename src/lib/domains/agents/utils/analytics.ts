import type { ChartConfig } from '$lib/components/ui/chart';
import type { AgentEventRow, AgentUsageCell } from '../generated';

/** Model colour follows the model family, never its rank. Slots 1-4 of the validated
 *  categorical palette (dataviz: CVD/normal-vision pass in both modes); the light
 *  aqua/yellow sit under 3:1, so every chart ships a table view. */
export const FAMILY_CONFIG = {
  opus: { label: 'Opus', theme: { light: '#2a78d6', dark: '#3987e5' } },
  sonnet: { label: 'Sonnet', theme: { light: '#eb6834', dark: '#d95926' } },
  haiku: { label: 'Haiku', theme: { light: '#1baf7a', dark: '#199e70' } },
  other: { label: 'Other', theme: { light: '#eda100', dark: '#c98500' } },
} satisfies ChartConfig;
export type Family = keyof typeof FAMILY_CONFIG;
const FAMILIES = Object.keys(FAMILY_CONFIG) as Family[];

export const familyOf = (model: string): Family =>
  FAMILIES.find((f) => model.toLowerCase().includes(f)) ?? 'other';

const DAY_MS = 86_400_000;

/** Pivot day x model cells into one row per day (gaps filled with 0) and one value
 *  column per model family present — the shape a stacked bar chart wants. */
export function pivotUsage(cells: AgentUsageCell[], metric: 'tokens' | 'cost_usd') {
  const byDay = new Map<string, Partial<Record<Family, number>>>();
  for (const c of cells) {
    const row = byDay.get(c.day) ?? {};
    const f = familyOf(c.model);
    row[f] = (row[f] ?? 0) + c[metric];
    byDay.set(c.day, row);
  }
  const days = [...byDay.keys()].sort();
  const families = FAMILIES.filter((f) => [...byDay.values()].some((r) => r[f]));
  const rows: Record<string, string | number>[] = [];
  if (days.length) {
    const end = Date.parse(days[days.length - 1]);
    for (let t = Date.parse(days[0]); t <= end; t += DAY_MS) {
      const day = new Date(t).toISOString().slice(0, 10);
      const src = byDay.get(day) ?? {};
      rows.push({ day, ...Object.fromEntries(families.map((f) => [f, src[f] ?? 0])) });
    }
  }
  return { rows, families };
}

export interface SubagentNode {
  id: string;
  name: string;
  /** Tool calls the subagent itself made (not its descendants'). */
  tools: number;
  done: boolean;
  children: SubagentNode[];
}

/** Subagent tree of one session: a subagent's parent is the `parent_tool_use_id` of the
 *  tool call that spawned it (null = the main agent). */
export function buildSubagentTree(events: AgentEventRow[]): SubagentNode[] {
  const spawnedBy = new Map<string, string | null>();
  const tools = new Map<string, number>();
  const done = new Set<string>();
  const nodes = new Map<string, SubagentNode>();
  for (const { event: e } of events) {
    if (e.type === 'tool_call') {
      spawnedBy.set(e.id, e.parent_tool_use_id ?? null);
      if (e.parent_tool_use_id)
        tools.set(e.parent_tool_use_id, (tools.get(e.parent_tool_use_id) ?? 0) + 1);
    } else if (e.type === 'subagent_started') {
      nodes.set(e.tool_use_id, {
        id: e.tool_use_id,
        name: e.name ?? 'subagent',
        tools: 0,
        done: false,
        children: [],
      });
    } else if (e.type === 'subagent_finished') done.add(e.tool_use_id);
  }
  const roots: SubagentNode[] = [];
  for (const n of nodes.values()) {
    n.tools = tools.get(n.id) ?? 0;
    n.done = done.has(n.id);
    const parent = nodes.get(spawnedBy.get(n.id) ?? '');
    (parent ? parent.children : roots).push(n);
  }
  return roots;
}
