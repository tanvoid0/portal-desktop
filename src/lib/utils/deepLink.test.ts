import { describe, expect, it } from 'vitest';
import { routeForDeepLink } from './deepLink';

describe('routeForDeepLink', () => {
  it('maps entities and lists', () => {
    expect(routeForDeepLink('portal://project/12')).toBe('/projects/12');
    expect(routeForDeepLink('portal://task/34')).toBe('/tasks/34');
    expect(routeForDeepLink('portal://run/7')).toBe('/pipeline-runs/7');
    expect(routeForDeepLink('portal://tasks')).toBe('/tasks');
    expect(routeForDeepLink('portal://runs')).toBe('/pipeline-runs');
  });
  it('passes through only the filters each page reads', () => {
    expect(routeForDeepLink('portal://tasks?project=3&view=list&evil=1')).toBe(
      '/tasks?project=3&view=list'
    );
    expect(routeForDeepLink('portal://runs?status=failed&q=build')).toBe(
      '/pipeline-runs?q=build&status=failed'
    );
    expect(routeForDeepLink('portal://project/12?tab=actions')).toBe('/projects/12?tab=actions');
    expect(routeForDeepLink('portal://task/34?tab=x')).toBe('/tasks/34');
    expect(routeForDeepLink('portal://tasks?project=<script>')).toBe('/tasks');
  });
  it('focuses an agent session', () => {
    expect(routeForDeepLink('portal://agent/abc-1')).toMatch(/\?session=abc-1$/);
  });
  it('rejects unknown, foreign and malformed links', () => {
    expect(routeForDeepLink('https://project/12')).toBeNull();
    expect(routeForDeepLink('portal://nope/1')).toBeNull();
    expect(routeForDeepLink('portal://project/a/b')).toBeNull();
    expect(routeForDeepLink('garbage')).toBeNull();
  });
});
