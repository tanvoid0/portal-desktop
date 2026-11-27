import { afterEach, expect, it, vi } from 'vitest';
import { history, record } from './history.svelte';

afterEach(() => vi.useRealTimers());

it('starts the trace over after a gap instead of joining across it', () => {
  vi.useFakeTimers();
  record('net', 1, 1);
  vi.advanceTimersByTime(2000);
  record('net', 2, 2);
  expect(history.net.map((s) => s.a)).toEqual([1, 2]);
  vi.advanceTimersByTime(11_000);
  record('net', 3, 3);
  expect(history.net.map((s) => s.a)).toEqual([3]);
});
