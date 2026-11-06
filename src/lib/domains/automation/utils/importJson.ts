import type { CreateBlockRequest } from '$lib/domains/projects/pipelines';

export type ImportResult = { data: CreateBlockRequest } | { error: string };

/** Parse pasted/uploaded script or block JSON. Valid JSON that is not a definition is an error. */
export function parseImportJson(
  text: string,
  defaultExecutionType: CreateBlockRequest['executionType']
): ImportResult {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return { error: 'Invalid JSON format' };
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    return { error: 'Expected a JSON object with "name" and "command"' };
  }
  const p = parsed as Record<string, any>;
  const missing = ['name', 'command'].filter((k) => typeof p[k] !== 'string' || !p[k].trim());
  if (missing.length) {
    return {
      error: `Missing required field${missing.length > 1 ? 's' : ''}: ${missing.join(', ')}`,
    };
  }
  return {
    data: {
      name: p.name,
      description: p.description || '',
      category: p.category || 'utility',
      parameters: Array.isArray(p.parameters) ? p.parameters : [],
      command: p.command,
      executionType: p.executionType || defaultExecutionType,
      defaultConfig: p.defaultConfig || {},
      tags: Array.isArray(p.tags) ? p.tags : [],
    },
  };
}
