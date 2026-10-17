/**
 * Per-project deploy config, remembered in localStorage.
 *
 * The deploy form writes this on a successful deploy; the project's "Deploy"
 * button reads it to run a one-click deploy (skip the form) next time. Keyed by
 * project path so each project keeps its own choices — including which `.env`
 * file seeded the environment variables.
 */

import { DeploymentType, ProjectType, type DeploymentCreateRequest } from '../types';

export interface SavedDeployConfig {
  deploymentType: DeploymentType;
  projectType: ProjectType;
  description?: string;
  exposedPort?: number;
  dockerImageName?: string;
  dockerfilePath?: string;
  command?: string;
  workingDirectory?: string;
  /** Flattened env vars that get injected at run time. */
  envVars: Record<string, string>;
  /** Name of the `.env*` file the user last loaded, e.g. `.env.local`. */
  selectedEnvFile?: string;
}

const KEY_PREFIX = 'portal:deploy-config:';

const keyFor = (projectPath: string) => `${KEY_PREFIX}${projectPath}`;

export function loadDeployConfig(projectPath: string): SavedDeployConfig | null {
  if (!projectPath) return null;
  try {
    const raw = localStorage.getItem(keyFor(projectPath));
    return raw ? (JSON.parse(raw) as SavedDeployConfig) : null;
  } catch {
    return null;
  }
}

export function saveDeployConfig(projectPath: string, config: SavedDeployConfig): void {
  if (!projectPath) return;
  try {
    localStorage.setItem(keyFor(projectPath), JSON.stringify(config));
  } catch {
    // Persistence is a convenience; a storage failure must not block deploying.
  }
}

export function hasDeployConfig(projectPath: string): boolean {
  return loadDeployConfig(projectPath) !== null;
}

/** Build the backend create request from a saved config. */
export function buildCreateRequest(
  name: string,
  projectPath: string,
  config: SavedDeployConfig
): DeploymentCreateRequest {
  const isDocker = config.deploymentType === DeploymentType.DOCKER;
  return {
    name,
    description: config.description || undefined,
    type: config.deploymentType,
    projectPath,
    projectType: config.projectType,
    environment: {
      variables: config.envVars ?? {},
      secrets: [],
      volumes: [],
      ports: [],
      networks: [],
    },
    dockerImageName: isDocker ? config.dockerImageName || undefined : undefined,
    dockerfilePath: isDocker ? config.dockerfilePath || undefined : undefined,
    exposedPort: isDocker ? config.exposedPort : undefined,
    command: isDocker ? undefined : config.command,
    workingDirectory: isDocker ? undefined : config.workingDirectory || projectPath,
  };
}
