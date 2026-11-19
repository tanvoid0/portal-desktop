import { invoke } from '@tauri-apps/api/core';
import type {
  ModelProject,
  ModelRegistryEntry,
  ModelBuildJob,
  OllamaModel,
} from '../types/index.js';

/** Build pipeline stages, in execution order. Mirrors `model_ops::BUILD_STAGES`. */
export const BUILD_STAGES = ['prepare', 'train', 'merge', 'quantize', 'register'] as const;

/** Model-ops proxy — every call lands on agent-platformd `/api/v1/model-ops/*` via Rust. */
export const aiModelOpsService = {
  projects: () => invoke<ModelProject[]>('ai_model_projects'),

  createProject: (name: string, description: string | null, baseModel: string | null) =>
    invoke<ModelProject>('ai_create_model_project', { name, description, baseModel }),

  ollamaModels: () => invoke<OllamaModel[]>('ai_ollama_models'),

  pullModel: (name: string) => invoke<ModelBuildJob>('ai_pull_ollama_model', { name }),

  registry: () => invoke<ModelRegistryEntry[]>('ai_model_registry'),

  activateEntry: (entryId: number) =>
    invoke<ModelRegistryEntry>('ai_activate_registry_entry', { entryId }),

  startBuild: (project: string, stages: string[], registerAlias: string | null) =>
    invoke<ModelBuildJob>('ai_start_model_build', { project, stages, registerAlias }),

  job: (jobId: number) => invoke<ModelBuildJob>('ai_model_build_job', { jobId }),

  cancelJob: (jobId: number) => invoke<ModelBuildJob>('ai_cancel_model_build_job', { jobId }),
};

/** A job the server still considers in flight. */
export function isJobRunning(status: string): boolean {
  return !['completed', 'failed', 'cancelled'].includes(status);
}
