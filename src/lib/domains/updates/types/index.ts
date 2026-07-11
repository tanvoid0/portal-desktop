export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
  available: boolean;
}

export interface UpdateStatus {
  checking: boolean;
  available: boolean;
  installing: boolean;
  error: UpdateErrorInfo | null;
  info: UpdateInfo | null;
}

export type UpdateErrorCategory =
  | "network"
  | "manifest"
  | "signature"
  | "download"
  | "install"
  | "disabled"
  | "unknown";

export interface UpdateErrorInfo {
  category: UpdateErrorCategory;
  title: string;
  message: string;
  hint?: string;
  recoverable: boolean;
  /** Original error message for logging only */
  technical?: string;
}

export type UpdateCheckResult =
  | { status: "available"; info: UpdateInfo }
  | { status: "current"; info: UpdateInfo }
  | { status: "error"; error: UpdateErrorInfo };

export type UpdateInstallResult =
  | { status: "error"; error: UpdateErrorInfo };
