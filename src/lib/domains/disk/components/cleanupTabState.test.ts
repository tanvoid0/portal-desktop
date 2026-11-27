import { expect, it, vi } from 'vitest';

const gates: ((summary: unknown) => void)[] = [];
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string) =>
    cmd === 'scan_directory' ? new Promise((r) => gates.push(r)) : Promise.resolve(null)
  ),
}));
vi.mock('$lib/utils/confirm', () => ({ confirmAction: vi.fn() }));

import { cleanupTab } from './cleanupTabState.svelte';

const summary = (root: string) => ({
  root,
  totalBytes: 0,
  scannedFiles: 0,
  elapsedMs: 0,
  proposals: [],
});

// The state outlives the page, so a second scan can start while one still runs.
it('keeps the latest scan when an older one finishes last', async () => {
  const first = cleanupTab.scan('a');
  const second = cleanupTab.scan('b');
  gates[1](summary('b'));
  await second;
  gates[0](summary('a'));
  await first;
  expect(cleanupTab.summary?.root).toBe('b');
  expect(cleanupTab.busy).toBe(false);
});
