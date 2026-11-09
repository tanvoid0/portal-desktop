import type { PipelineTemplate } from '../types';

const STORAGE_KEY = 'portal.pipelines.customTemplates';

/**
 * Imported/custom pipeline templates are a pure frontend concept — there is no backend table
 * for them, so `PipelineTemplateService` lives entirely in memory and loses every imported
 * template on reload. Persist them here, the same way chat folders and catalog prefs persist
 * client-only collections.
 */
export function loadCustomTemplates(): PipelineTemplate[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function saveCustomTemplates(templates: PipelineTemplate[]): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
  } catch {
    // ignore quota / private mode
  }
}
