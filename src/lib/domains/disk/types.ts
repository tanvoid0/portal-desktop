// UI-only shapes for the disk domain.
//
// Everything that also exists in Rust is generated from it — see `./generated/`,
// produced by ts-rs from `portal_core::domains::disk` and refreshed by
// `pnpm gen:types`. What is left below has no Rust counterpart.

export type {
  AgentNote,
  AuditEntry,
  DevCleanerCleanProgress,
  DevCleanerCleanResult,
  DevCleanerGroup,
  DevCleanerItem,
  DevCleanerScan,
  DiskUsage,
  ItemVerdict,
  Location,
  MovedItem,
  Project,
  ProjectScan,
  ProjectTemp,
  Proposal,
  QuarantineProgress,
  QuarantineResult,
  Risk,
  VerificationResult,
  VerifyProgress,
  VerifyTask,
} from './generated';

import type { Proposal } from './generated';

// Mirrors the Rust structs returned across the Tauri bridge by the `disk`
// domain (src-tauri/src/domains/disk). Ported from portal_disk_utility.

export interface ScanSummary {
  root: string;
  totalBytes: number;
  scannedFiles: number;
  elapsedMs: number;
  proposals: Proposal[];
}

export interface ScanProgress {
  phase: 'counting' | 'scanning';
  scannedFiles: number;
  totalFiles: number;
  totalBytes: number;
  currentPath: string;
  etaMs: number;
  elapsedMs: number;
}

export interface CachedScan {
  ts: number;
  status: 'complete' | 'partial';
  scannedFiles: number;
  totalFiles: number;
  summary: ScanSummary;
}
