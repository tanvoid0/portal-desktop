// UI-only shapes for the github domain.
//
// Everything that also exists in Rust is generated from it — see `./generated/`,
// produced by ts-rs from `portal_core::domains::github` and refreshed by
// `pnpm gen:types`. What is left below has no Rust counterpart.

export type {
  GitHubAccount,
  GitHubCloneRepositoryRequest,
  GitHubConnectionStatus,
  GitHubCreateIssueRequest,
  GitHubDeviceFlowPollResult,
  GitHubDeviceFlowStart,
  GitHubDispatchWorkflowRequest,
  GitHubIssue,
  GitHubLinkExistingRepositoryRequest,
  GitHubListIssuesRequest,
  GitHubListWorkflowRunsRequest,
  GitHubListWorkflowsRequest,
  GitHubLocalRepositoryDetection,
  GitHubProjectLink,
  GitHubProjectLinkResult,
  GitHubRepoOwner,
  GitHubRepoProjects,
  GitHubRepository,
  GitHubUpdateIssueRequest,
  GitHubWorkflow,
  GitHubWorkflowJob,
  GitHubWorkflowJobStep,
  GitHubWorkflowRun,
  GitHubWorkflowRunDetail,
} from './generated';

import type { GitHubDeviceFlowStart } from './generated';

import type { Project } from '$lib/domains/projects/types';

export interface GitHubDeviceFlowCallbacks {
  onStarted?: (start: GitHubDeviceFlowStart) => void | Promise<void>;
  onPolling?: () => void | Promise<void>;
}
