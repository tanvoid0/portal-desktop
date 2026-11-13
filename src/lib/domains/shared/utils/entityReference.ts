export interface EntityRef {
  /** e.g. "task", "project", "coder session", "script", "deployment". */
  kind: string;
  id: string | number;
  title: string;
  /** Rendered as "k: v · k: v"; falsy values are dropped. */
  fields?: Record<string, string | undefined>;
  /** A trailing block (description, command, …). Omitted when undefined. */
  body?: string;
}

/**
 * A compact, paste-ready reference to an entity — the thing you drop into Claude
 * (or any chat) to say "this is what I mean". Self-contained: carries the id so
 * the portal board can resolve it, plus the human-readable summary.
 */
export function buildReference(ref: EntityRef): string {
  const lines = [`Portal ${ref.kind} #${ref.id} — ${ref.title}`];
  if (ref.fields) {
    const parts = Object.entries(ref.fields)
      .filter(([, v]) => v)
      .map(([k, v]) => `${k}: ${v}`);
    if (parts.length) lines.push(parts.join(' · '));
  }
  if (ref.body != null) lines.push('', ref.body);
  return lines.join('\n');
}
