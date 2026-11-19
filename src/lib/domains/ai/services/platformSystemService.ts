import { invoke } from '@tauri-apps/api/core';

/** One readiness probe from the daemon's `/system/status`. */
export interface ReadinessCheck {
  name: string;
  ok: boolean;
  detail: string;
}

export interface ReadinessBlock {
  ok?: boolean;
  status?: string;
  checks?: ReadinessCheck[];
}

/**
 * `GET /api/v1/system/status` — free-form aggregator (agent-platform's
 * `system.rs`). Every field is optional so a daemon that grows or drops one
 * never breaks the Status screen.
 */
export interface PlatformSystemStatus {
  service?: string;
  env?: string;
  uptime_seconds?: number;
  /** This crate's version (labelled "Server"). */
  server?: string;
  /** OS-arch, e.g. `windows-x86_64`. */
  platform?: string;
  listening_on?: { host?: string; port?: number };
  auth_required?: boolean;
  readiness?: ReadinessBlock;
  llm_proxy?: ReadinessBlock;
  processes?: { by_status?: Record<string, number>; active?: number; total?: number };
  paths?: Record<string, string | null>;
}

/** `GET /api/v1/system/logs` — `{ lines, next, dropped }`; `next` is the cursor. */
export interface PlatformLogSnapshot {
  lines: string[];
  next: number;
  dropped: number;
}

export class PlatformSystemService {
  /** Daemon status aggregate: version, uptime, readiness, process counts, paths. */
  getStatus(): Promise<PlatformSystemStatus> {
    return invoke<PlatformSystemStatus>('get_platform_system_status');
  }

  /** Recent daemon log lines. Pass the previous `next` cursor to tail only new lines. */
  getLogs(after = 0): Promise<PlatformLogSnapshot> {
    return invoke<PlatformLogSnapshot>('get_platform_system_logs', { after });
  }
}

export const platformSystemService = new PlatformSystemService();
