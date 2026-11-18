import type { BaseItemGroup, BaseSuggestedItem } from './types';

interface Suggestion {
  pattern_data?: unknown;
  frequency: number;
  success_rate: number;
}

/** Top 15 names by summed frequency * success_rate for the given pattern_data key. */
export function rankRecommendedNames(suggestions: Suggestion[], dataKey: string): string[] {
  const scores = new Map<string, number>();
  for (const suggestion of suggestions) {
    if (suggestion.pattern_data && typeof suggestion.pattern_data === 'object') {
      const value = (suggestion.pattern_data as Record<string, unknown>)[dataKey];
      if (value && typeof value === 'string') {
        scores.set(
          value,
          (scores.get(value) || 0) + suggestion.frequency * suggestion.success_rate
        );
      }
    }
  }
  return Array.from(scores.entries())
    .sort((a, b) => b[1] - a[1])
    .map(([name]) => name)
    .slice(0, 15);
}

/** Prepend a "Recommended" group (resolved against known suggestions) to the suggested groups. */
export function mergeRecommendedGroup<TSuggested extends BaseSuggestedItem>(
  suggestedGroups: BaseItemGroup<TSuggested>[],
  allSuggestedItems: TSuggested[],
  recommendedNames: string[]
): BaseItemGroup<TSuggested>[] {
  const groups = [...suggestedGroups];
  if (recommendedNames.length > 0) {
    const items: TSuggested[] = recommendedNames.map(
      (name) =>
        allSuggestedItems.find((si) => si.name.toLowerCase() === name.toLowerCase()) ??
        ({
          name,
          icon: `logos:${name.toLowerCase().replace(/\s+/g, '-')}`,
          category: 'Recommended',
        } as TSuggested)
    );
    groups.unshift({ category: 'Recommended (Based on Your Usage)', items });
  }
  return groups;
}
