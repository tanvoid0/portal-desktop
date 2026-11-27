import { afterEach, expect, it, vi } from 'vitest';
import { generateId } from './index';

const UUID_V4 = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

afterEach(() => vi.unstubAllGlobals());

it('makes a v4 UUID without crypto.randomUUID (insecure context)', () => {
  vi.stubGlobal('crypto', { getRandomValues: crypto.getRandomValues.bind(crypto) });
  const a = generateId();
  expect(a).toMatch(UUID_V4);
  expect(generateId()).not.toBe(a);
});
