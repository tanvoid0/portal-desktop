import type { Project } from "$lib/domains/projects/types";

export interface GitHubAccount {
  id: number;
  login: string;
  name?: string;
  avatarUrl?: string;
  htmlUrl: string;
  scopes: string[];
}

export interface GitHubConnectionStatus {
  connected: boolean;
  clientIdConfigured: boolean;
  account?: GitHubAccount | null;
}

export interface GitHubDeviceFlowStart {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  verificationUriComplete?: string;
  expiresIn: number;
  interval: number;
}

export interface GitHubDeviceFlowPollResult {
  status: "pending" | "connected" | "expired" | "denied" | "error";
  message?: string | null;
  retryAfterSeconds?: number | null;
  account?: GitHubAccount | null;
}

export interface GitHubDeviceFlowCallbacks {
  onStarted?: (start: GitHubDeviceFlowStart) => void | Promise<void>;
  onPolling?: () => void | Promise<void>;
}

export interface GitHubRepoOwner {
  login: string;
  avatarUrl?: string;
  htmlUrl?: string;
}

export interface GitHubRepository {
  id: number;
  name: string;
  fullName: string;
  owner: GitHubRepoOwner;
  description?: string;
  private: boolean;
  htmlUrl: string;
  cloneUrl: string;
  sshUrl?: string;
  defaultBranch: string;
  language?: string;
  stargazersCount: number;
  forksCount: number;
  openIssuesCount: number;
  updatedAt?: string;
}

export interface GitHubIssue {
  id: number;
  number: number;
  title: string;
  body?: string;
  state: string;
  htmlUrl: string;
  repoFullName?: string;
  authorLogin?: string;
  labels: string[];
  assignees: string[];
  createdAt?: string;
  updatedAt?: string;
  closedAt?: string;
  isPullRequest: boolean;
}

export interface GitHubListIssuesRequest {
  owner?: string;
  repo?: string;
  state?: string;
  filter?: string;
  page?: number;
  perPage?: number;
}

export interface GitHubCreateIssueRequest {
  owner: string;
  repo: string;
  title: string;
  body?: string;
  labels?: string[];
}

export interface GitHubUpdateIssueRequest {
  owner: string;
  repo: string;
  number: number;
  title?: string;
  body?: string;
  state?: string;
  labels?: string[];
}

export interface GitHubProjectLink {
  projectId: number;
  repoOwner: string;
  repoName: string;
  repoFullName: string;
  repoHtmlUrl?: string;
  defaultBranch?: string;
  cloneUrl?: string;
  sshUrl?: string;
}

export interface GitHubRepoProjects {
  repository: GitHubRepository;
  linkedProjects: Project[];
}

export interface GitHubCloneRepositoryRequest {
  owner: string;
  repo: string;
  destinationPath: string;
}

export interface GitHubLinkExistingRepositoryRequest {
  path: string;
  owner?: string;
  repo?: string;
}

export interface GitHubLocalRepositoryDetection {
  path: string;
  isGitRepository: boolean;
  owner?: string;
  repo?: string;
  repoFullName?: string;
  remoteUrl?: string;
}

export interface GitHubProjectLinkResult {
  project: Project;
  link: GitHubProjectLink;
  localPath: string;
}
