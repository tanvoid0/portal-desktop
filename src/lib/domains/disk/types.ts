// Mirrors the Rust structs returned across the Tauri bridge by the `disk`
// domain (src-tauri/src/domains/disk). Ported from portal_disk_utility.

export type Risk = "Safe" | "Review" | "Danger";

export interface Proposal {
  id: string;
  path: string;
  kind: string;
  reason: string;
  sizeBytes: number;
  fileCount: number;
  risk: Risk;
}

export interface ScanSummary {
  root: string;
  totalBytes: number;
  scannedFiles: number;
  elapsedMs: number;
  proposals: Proposal[];
}

export interface ScanProgress {
  phase: "counting" | "scanning";
  scannedFiles: number;
  totalFiles: number;
  totalBytes: number;
  currentPath: string;
  etaMs: number;
  elapsedMs: number;
}

export interface QuarantineProgress {
  done: number;
  total: number;
  currentPath: string;
}

export interface ProjectTemp {
  id: string;
  path: string;
  tempKind: string;
  sizeBytes: number;
  fileCount: number;
  risk: Risk;
}

export interface Project {
  root: string;
  kind: string;
  totalBytes: number;
  fileCount: number;
  temps: ProjectTemp[];
}

export interface ProjectScan {
  root: string;
  projectCount: number;
  totalBytes: number;
  projects: Project[];
}

export interface CachedScan {
  ts: number;
  status: "complete" | "partial";
  scannedFiles: number;
  totalFiles: number;
  summary: ScanSummary;
}

export interface MovedItem {
  path: string;
  kind: string;
  sizeBytes: number;
}

export interface QuarantineResult {
  moved: MovedItem[];
  failed: { path: string; kind: string; error: string }[];
  reclaimedBytes: number;
}

export interface Location {
  path: string;
  label: string;
  kind: "drive" | "folder";
}

export interface DiskUsage {
  mountPoint: string;
  name: string;
  fsKind: string;
  totalBytes: number;
  availableBytes: number;
  isRemovable: boolean;
}

export interface AuditEntry {
  id: number;
  ts: number;
  action: string;
  path: string;
  sizeBytes: number;
  kind: string;
  status: string;
}

export interface AiConfig {
  baseUrl?: string;
  apiToken?: string;
  teamTemplateId?: number;
}

export interface TeamOption {
  id: number;
  name: string;
  description: string;
  isAppTeam: boolean;
}

export interface AgentNote {
  taskId: number;
  role: string;
  status: string;
  output: string;
}

export interface ItemVerdict {
  path: string;
  verdict: "safe" | "review" | "dangerous";
  reason: string;
}

export interface VerificationResult {
  processId: number;
  status: string;
  notes: AgentNote[];
  verdicts: ItemVerdict[];
  gated: boolean;
}

export interface VerifyTask {
  role: string;
  status: string;
}

export interface VerifyProgress {
  processId: number;
  status: string;
  tasks: VerifyTask[];
}
