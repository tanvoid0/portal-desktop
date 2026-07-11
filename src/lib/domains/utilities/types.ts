export interface EnvVariable {
  name: string;
  value: string;
  scope: "user" | "system" | "session";
}

export interface EnvPermissions {
  canEditUser: boolean;
  canEditSystem: boolean;
  isElevated: boolean;
  platform: string;
}

export interface EnvChange {
  action: "set" | "delete";
  name: string;
  value?: string;
  scope: "user" | "system" | "session";
}

export interface EnvApplyResult {
  success: boolean;
  message: string;
  elevated: boolean;
}

export interface EnvRow {
  id: string;
  name: string;
  value: string;
  scope: EnvVariable["scope"];
  isNew?: boolean;
  isDeleted?: boolean;
  isDirty?: boolean;
}
