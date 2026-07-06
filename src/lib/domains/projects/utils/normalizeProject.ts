/**
 * Normalize raw Tauri project payloads into frontend Project objects.
 * Tauri IPC uses camelCase on the wire; our domain types use snake_case fields.
 */

import type { Project, ProjectMetadata } from "$lib/domains/projects/types";
import { ProjectStatus } from "$lib/domains/projects/types";

type RawProject = Record<string, unknown>;

function toNumberArray(value: unknown): number[] {
  if (!Array.isArray(value)) return [];
  return value
    .map((item) => Number(item))
    .filter((item) => Number.isFinite(item));
}

function toOptionalDate(value: unknown): Date | undefined {
  if (!value) return undefined;
  const date = new Date(String(value));
  return Number.isNaN(date.getTime()) ? undefined : date;
}

function toStatus(value: unknown): Project["status"] {
  if (value === ProjectStatus.ARCHIVED || value === ProjectStatus.DELETED) {
    return value;
  }
  return ProjectStatus.ACTIVE;
}

export function normalizeProject(raw: RawProject): Project {
  const openCount = Number(raw.open_count ?? raw.openCount ?? 0);
  const size = Number(raw.size ?? 0);
  const fileCount = Number(raw.file_count ?? raw.fileCount ?? 0);
  const createdAt =
    toOptionalDate(raw.created_at ?? raw.createdAt) ?? new Date();
  const updatedAt =
    toOptionalDate(raw.updated_at ?? raw.updatedAt) ?? createdAt;

  const metadata: ProjectMetadata = {
    openCount,
    size,
    fileCount,
    lastOpened: toOptionalDate(raw.last_opened ?? raw.lastOpened),
    gitInfo:
      raw.git_repository || raw.git_branch || raw.gitBranch
        ? {
            repository: (raw.git_repository ?? raw.gitRepository) as
              | string
              | undefined,
            branch: (raw.git_branch ?? raw.gitBranch) as string | undefined,
            commit: (raw.git_commit ?? raw.gitCommit) as string | undefined,
            hasUncommittedChanges: Boolean(
              raw.has_uncommitted_changes ?? raw.hasUncommittedChanges,
            ),
            lastCommit: toOptionalDate(raw.last_commit ?? raw.lastCommit),
          }
        : undefined,
  };

  return {
    id: String(raw.id ?? ""),
    name: String(raw.name ?? ""),
    description: (raw.description as string | undefined) ?? undefined,
    path: String(raw.path ?? ""),
    status: toStatus(raw.status),
    framework_ids: toNumberArray(raw.framework_ids ?? raw.frameworkIds),
    package_manager_ids: toNumberArray(
      raw.package_manager_ids ?? raw.packageManagerIds,
    ),
    language_ids: toNumberArray(raw.language_ids ?? raw.languageIds),
    build_command: (raw.build_command ?? raw.buildCommand) as string | undefined,
    start_command: (raw.start_command ?? raw.startCommand) as string | undefined,
    test_command: (raw.test_command ?? raw.testCommand) as string | undefined,
    output_directory: (raw.output_directory ?? raw.outputDirectory) as
      | string
      | undefined,
    dev_port: (raw.dev_port ?? raw.devPort) as number | undefined,
    prod_port: (raw.prod_port ?? raw.prodPort) as number | undefined,
    starred: Boolean(raw.starred),
    open_count: openCount,
    last_opened: toOptionalDate(raw.last_opened ?? raw.lastOpened),
    size,
    file_count: fileCount,
    git_repository: (raw.git_repository ?? raw.gitRepository) as
      | string
      | undefined,
    git_branch: (raw.git_branch ?? raw.gitBranch) as string | undefined,
    git_commit: (raw.git_commit ?? raw.gitCommit) as string | undefined,
    has_uncommitted_changes: Boolean(
      raw.has_uncommitted_changes ?? raw.hasUncommittedChanges,
    ),
    last_commit: toOptionalDate(raw.last_commit ?? raw.lastCommit),
    created_at: createdAt,
    updated_at: updatedAt,
    metadata,
    createdAt,
    updatedAt,
  };
}

export function normalizeProjects(raw: unknown): Project[] {
  if (!Array.isArray(raw)) return [];
  return raw.map((item) => normalizeProject(item as RawProject));
}

/** Build Tauri invoke payload from domain request fields. */
export function toProjectInvokePayload(
  request: Partial<CreateProjectRequestLike> & { id?: number },
): Record<string, unknown> {
  const payload: Record<string, unknown> = {};

  if (request.id !== undefined) payload.id = request.id;
  if (request.name !== undefined) payload.name = request.name;
  if (request.description !== undefined) payload.description = request.description;
  if (request.path !== undefined) payload.path = request.path;
  if (request.status !== undefined) payload.status = request.status;
  if (request.framework_ids !== undefined) {
    payload.frameworkIds = request.framework_ids;
  }
  if (request.package_manager_ids !== undefined) {
    payload.packageManagerIds = request.package_manager_ids;
  }
  if (request.language_ids !== undefined) {
    payload.languageIds = request.language_ids;
  }
  if (request.build_command !== undefined) {
    payload.buildCommand = request.build_command;
  }
  if (request.start_command !== undefined) {
    payload.startCommand = request.start_command;
  }
  if (request.test_command !== undefined) {
    payload.testCommand = request.test_command;
  }
  if (request.output_directory !== undefined) {
    payload.outputDirectory = request.output_directory;
  }
  if (request.dev_port !== undefined) payload.devPort = request.dev_port;
  if (request.prod_port !== undefined) payload.prodPort = request.prod_port;

  return payload;
}

interface CreateProjectRequestLike {
  name?: string;
  description?: string;
  path?: string;
  status?: Project["status"];
  framework_ids?: number[];
  package_manager_ids?: number[];
  language_ids?: number[];
  build_command?: string;
  start_command?: string;
  test_command?: string;
  output_directory?: string;
  dev_port?: number;
  prod_port?: number;
}
