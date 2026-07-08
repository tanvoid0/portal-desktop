export type {
  GitHubAccount,
  GitHubCloneRepositoryRequest,
  GitHubConnectionStatus,
  GitHubCreateIssueRequest,
  GitHubDeviceFlowCallbacks,
  GitHubDeviceFlowPollResult,
  GitHubDeviceFlowStart,
  GitHubIssue,
  GitHubListIssuesRequest,
  GitHubLocalRepositoryDetection,
  GitHubProjectLink,
  GitHubProjectLinkResult,
  GitHubRepoProjects,
  GitHubRepository,
  GitHubUpdateIssueRequest,
} from "./types";

export { default as GitHubConnectPrompt } from "./components/GitHubConnectPrompt.svelte";
export { githubService } from "./service";
export {
  createGitHubIssuesQuery,
  createGitHubRepositoriesQuery,
  createGitHubRepositoryQuery,
  createGitHubStatusQuery,
} from "./queries";
