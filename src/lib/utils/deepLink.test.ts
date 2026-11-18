import { describe, expect, it } from 'vitest';
import { routeForDeepLink } from './deepLink';

describe('routeForDeepLink', () => {
  it('maps entities and lists', () => {
    expect(routeForDeepLink('portal://project/12')).toBe('/projects/12');
    expect(routeForDeepLink('portal://task/34')).toBe('/tasks/34');
    expect(routeForDeepLink('portal://tasks')).toBe('/tasks');
  });
  it('rejects unknown, foreign and traversal links', () => {
    expect(routeForDeepLink('https://project/12')).toBeNull();
    expect(routeForDeepLink('portal://nope/1')).toBeNull();
    expect(routeForDeepLink('portal://project/a/b')).toBeNull();
    expect(routeForDeepLink('garbage')).toBeNull();
  });
});
