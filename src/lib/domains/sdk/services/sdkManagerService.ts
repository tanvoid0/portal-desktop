import { invoke } from "@tauri-apps/api/core";

export interface SDKManagerInfo {
  id: string;
  name: string;
  display_name: string;
  installed: boolean;
  version: string | null;
  supports_installation: boolean;
  supports_version_switching: boolean;
  install_command: string | null;
  website: string | null;
  install_available: boolean;
  install_unavailable_reason: string | null;
  uninstall_available: boolean;
  uninstall_unavailable_reason: string | null;
}

export async function listSDKManagers(): Promise<SDKManagerInfo[]> {
  return invoke<SDKManagerInfo[]>("get_all_sdk_managers");
}

export async function installSDKManager(managerName: string): Promise<string> {
  return invoke<string>("install_sdk_manager", { managerName });
}

export async function uninstallSDKManager(managerName: string): Promise<string> {
  return invoke<string>("uninstall_sdk_manager", { managerName });
}
