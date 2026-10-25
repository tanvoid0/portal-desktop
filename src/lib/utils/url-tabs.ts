/** URL-backed tab selection helpers for SvelteKit pages. */

type ReadableSearchParams = Pick<
  URLSearchParams,
  | 'get'
  | 'getAll'
  | 'has'
  | 'keys'
  | 'values'
  | 'entries'
  | 'forEach'
  | 'toString'
  | typeof Symbol.iterator
>;

export function resolveUrlTab<T extends string>(
  searchParams: ReadableSearchParams,
  allowed: readonly T[],
  defaultTab: T,
  param = 'tab'
): T {
  const value = searchParams.get(param);
  if (value && (allowed as readonly string[]).includes(value)) {
    return value as T;
  }
  return defaultTab;
}

export function buildTabUrl(
  pathname: string,
  searchParams: ReadableSearchParams,
  tab: string,
  param = 'tab'
): string {
  const params = new URLSearchParams(searchParams.toString());
  params.set(param, tab);
  const query = params.toString();
  return query ? `${pathname}?${query}` : pathname;
}
