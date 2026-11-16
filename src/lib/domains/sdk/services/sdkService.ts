/**
 * SDK Service - Frontend business logic for SDK management
 */

import { invoke } from '@tauri-apps/api/core';

export class SDKService {
  private static instance: SDKService;

  static getInstance(): SDKService {
    if (!SDKService.instance) {
      SDKService.instance = new SDKService();
    }
    return SDKService.instance;
  }

  /**
   * Fetch installable versions for an SDK type.
   */
  async fetchAvailableVersions(sdkType: string): Promise<unknown[]> {
    const versions = await invoke<unknown[]>('fetch_available_versions', {
      sdkType,
    });
    return Array.isArray(versions) ? versions : [];
  }

  /**
   * Download + install a version (optionally through its manager).
   */
  async downloadAndInstallVersion(
    sdkType: string,
    version: string,
    useManager = false
  ): Promise<void> {
    await invoke('download_and_install_version', {
      sdkType,
      version,
      use_manager: useManager,
    });
  }

  /**
   * Switch the active version through the SDK manager that owns it (nvm, pyenv, ...).
   */
  async switchVersionViaManager(managerName: string, version: string): Promise<string> {
    return invoke<string>('switch_version_via_manager', { managerName, version });
  }

  /**
   * Uninstall a version through its SDK manager.
   */
  async uninstallVersionViaManager(managerName: string, version: string): Promise<string> {
    return invoke<string>('uninstall_version_via_manager', { managerName, version });
  }
}

export const sdkService = SDKService.getInstance();
