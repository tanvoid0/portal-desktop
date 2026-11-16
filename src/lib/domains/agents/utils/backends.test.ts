import { describe, expect, it } from 'vitest';
import type { BackendInfo } from '../generated';
import { hasBackendChoice } from './backends';

const backend = (id: string): BackendInfo => ({
  id,
  display_name: id,
  available: true,
  models: [],
});

describe('hasBackendChoice', () => {
  it('is false for a single backend', () => {
    expect(hasBackendChoice([backend('claude-code')])).toBe(false);
  });

  it('is true once a second backend is detected', () => {
    expect(hasBackendChoice([backend('claude-code'), backend('codex')])).toBe(true);
  });

  it('is false with no backends detected', () => {
    expect(hasBackendChoice([])).toBe(false);
  });
});
