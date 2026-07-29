import { describe, expect, it } from 'vitest';
import { getModelRow } from './catalog.js';
import type { CatalogModel } from '../types/index.js';

describe('getModelRow', () => {
  it('maps local backend metadata onto columns', () => {
    const model: CatalogModel = {
      id: 'gemma3:4b',
      provider: 'ollama',
      source: 'live',
      metadata: {
        display_name: 'Gemma 3 4B',
        parameter_size: '4.3B',
        family: 'gemma3',
        quantization_level: 'Q4_K_M',
        size: 3_338_801_804,
      },
      capabilities: { tools: true, vision_input: true },
    };

    expect(getModelRow(model)).toEqual({
      id: 'gemma3:4b',
      name: 'Gemma 3 4B',
      quant: 'Q4_K_M',
      params: '4.3B',
      family: 'gemma3',
      publisher: 'ollama',
      sizeLabel: '3.11 GB',
      source: 'live',
      backendId: undefined,
      tools: true,
      vision: true,
      embeddings: undefined,
    });
  });

  it('falls back to the id and leaves cloud columns empty', () => {
    const row = getModelRow({
      id: 'claude-opus-4',
      provider: 'anthropic',
      source: 'alias',
      backend_id: 'claude-opus-4-20250514',
    });

    expect(row.name).toBe('claude-opus-4');
    expect(row.backendId).toBe('claude-opus-4-20250514');
    expect(row.params).toBeUndefined();
    expect(row.sizeLabel).toBeUndefined();
  });
});
