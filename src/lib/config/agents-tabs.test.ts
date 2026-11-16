import { describe, expect, it } from 'vitest';
import { DEFAULT_AGENT_BACKEND, agentTabPath, engineTabsFor, isAgentTabRoute } from './agents-tabs';
import { getActiveAiTab } from './ai-tabs';

describe('agentTabPath', () => {
  it('defaults to the primary backend', () => {
    expect(DEFAULT_AGENT_BACKEND).toBe('claude-code');
    expect(agentTabPath()).toBe('/ai/coder/claude-code');
  });

  it('builds a path for any backend id', () => {
    expect(agentTabPath('codex')).toBe('/ai/coder/codex');
  });
});

describe('isAgentTabRoute', () => {
  it('is true for a backend tab route', () => {
    expect(isAgentTabRoute('/ai/coder/claude-code')).toBe(true);
  });

  it('is false for the bare Portal tab', () => {
    expect(isAgentTabRoute('/ai/coder')).toBe(false);
  });

  it('is false outside /ai/coder', () => {
    expect(isAgentTabRoute('/ai/agents')).toBe(false);
  });
});

describe('engineTabsFor', () => {
  const backends = [
    { id: 'claude-code', display_name: 'Claude Code' },
    { id: 'codex', display_name: 'Codex' },
  ];

  it('desktop shows Portal plus every backend', () => {
    expect(engineTabsFor(backends, null).map((t) => [t.label, t.url])).toEqual([
      ['Portal', '/ai/coder'],
      ['Claude Code', '/ai/coder/claude-code'],
      ['Codex', '/ai/coder/codex'],
    ]);
  });

  it('mobile gates Portal on coder and backends on agents', () => {
    expect(engineTabsFor(backends, ['coder']).map((t) => t.id)).toEqual(['portal']);
    expect(engineTabsFor(backends, ['agents']).map((t) => t.id)).toEqual(['claude-code', 'codex']);
    expect(engineTabsFor(backends, [])).toEqual([]);
  });
});

describe('getActiveAiTab', () => {
  it('treats a backend tab route as the Code tab', () => {
    expect(getActiveAiTab(agentTabPath())).toBe('code');
  });
});
