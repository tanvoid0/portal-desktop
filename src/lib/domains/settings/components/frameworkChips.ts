import type { Framework, FrameworkGroup } from '$lib/domains/ide';

export interface FrameworkChip {
  name: string;
  icon?: string;
  icon_type?: 'devicon' | 'file';
  category: string;
  isCustom: boolean;
  /** The underlying database row, present only when `isCustom`. */
  source?: Framework;
}

/**
 * Suggested (backend) + user-defined (database) frameworks, deduplicated by name — the two
 * sources overlap heavily (a popular framework is both "recommended" and seeded in the
 * database), so without this every name showed two or three times. The database row wins a
 * collision so the chip stays editable/deletable.
 */
export function dedupeFrameworks(
  frameworks: Framework[],
  groups: FrameworkGroup[]
): FrameworkChip[] {
  const seen = new Set<string>();
  const result: FrameworkChip[] = [];
  for (const fw of frameworks) {
    const key = fw.name.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    result.push({
      name: fw.name,
      icon: fw.icon,
      icon_type: fw.icon_type,
      category: fw.category,
      isCustom: true,
      source: fw,
    });
  }
  for (const group of groups) {
    for (const fw of group.frameworks) {
      const key = fw.name.toLowerCase();
      if (seen.has(key)) continue;
      seen.add(key);
      result.push({ name: fw.name, icon: fw.icon, category: fw.category, isCustom: false });
    }
  }
  return result;
}

export function groupByCategory(list: FrameworkChip[]): [string, FrameworkChip[]][] {
  const byCategory: Record<string, FrameworkChip[]> = {};
  for (const fw of list) (byCategory[fw.category || 'Other'] ??= []).push(fw);
  return Object.entries(byCategory);
}
