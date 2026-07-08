/** URL-backed tab selection helpers for SvelteKit pages. */

export function resolveUrlTab<T extends string>(
  searchParams: URLSearchParams,
  allowed: readonly T[],
  defaultTab: T,
  param = "tab",
): T {
  const value = searchParams.get(param);
  if (value && (allowed as readonly string[]).includes(value)) {
    return value as T;
  }
  return defaultTab;
}

export function buildTabUrl(
  pathname: string,
  searchParams: URLSearchParams,
  tab: string,
  param = "tab",
): string {
  const params = new URLSearchParams(searchParams);
  params.set(param, tab);
  const query = params.toString();
  return query ? `${pathname}?${query}` : pathname;
}
