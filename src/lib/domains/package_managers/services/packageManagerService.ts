/**
 * Package Manager Service - Frontend service for package manager configuration
 */

import { invokeClient } from "$lib/utils/invokeClient";
import { logger } from "$lib/domains/shared";
import type {
  PackageManager,
  PackageManagerGroup,
  SuggestedPackageManager,
} from "../types";

export class PackageManagerService {
  private static instance: PackageManagerService;
  private log = logger.createScoped("PackageManagerService");

  private constructor() {}

  static getInstance(): PackageManagerService {
    if (!PackageManagerService.instance) {
      PackageManagerService.instance = new PackageManagerService();
    }
    return PackageManagerService.instance;
  }

  /**
   * Get all user-defined package managers from database
   */
  async getAllPackageManagers(): Promise<PackageManager[]> {
    try {
      this.log.info("Getting all package managers");
      const packageManagers = await invokeClient.post<PackageManager[]>(
        "get_all_package_managers",
      );
      const safePackageManagers = packageManagers ?? [];
      this.log.info("Package managers retrieved", {
        count: safePackageManagers.length,
      });
      return safePackageManagers;
    } catch (error) {
      this.log.error("Failed to get package managers", { error });
      throw error;
    }
  }

  /**
   * Get suggested package managers from backend
   */
  async getSuggestedPackageManagers(): Promise<PackageManagerGroup[]> {
    try {
      this.log.info("Getting suggested package managers");
      const groups = await invokeClient.post<PackageManagerGroup[]>(
        "get_suggested_package_managers",
      );
      const safeGroups = groups ?? [];
      this.log.info("Suggested package managers retrieved", {
        count: safeGroups.length,
      });
      return safeGroups;
    } catch (error) {
      this.log.error("Failed to get suggested package managers", { error });
      throw error;
    }
  }

  /**
   * Create a new package manager
   */
  async createPackageManager(
    name: string,
    icon: string,
    iconType: "devicon" | "file",
    category: string,
  ): Promise<PackageManager> {
    try {
      this.log.info("Creating package manager", { name });
      const packageManager = await invokeClient.post<PackageManager>(
        "create_package_manager",
        {
          name,
          icon,
          iconType, // Tauri v2 converts camelCase to snake_case automatically
          category,
        },
      );
      if (!packageManager) {
        throw new Error("Failed to create package manager: no response");
      }
      this.log.info("Package manager created successfully", {
        id: packageManager.id,
      });
      return packageManager;
    } catch (error) {
      this.log.error("Failed to create package manager", { error });
      throw error;
    }
  }

  /**
   * Update an existing package manager
   */
  async updatePackageManager(
    id: number,
    name?: string,
    icon?: string,
    iconType?: "devicon" | "file",
    category?: string,
  ): Promise<PackageManager> {
    try {
      this.log.info("Updating package manager", { id });
      const packageManager = await invokeClient.post<PackageManager>(
        "update_package_manager",
        {
          id,
          name,
          icon,
          iconType, // Tauri v2 converts camelCase to snake_case automatically
          category,
        },
      );
      if (!packageManager) {
        throw new Error("Failed to update package manager: no response");
      }
      this.log.info("Package manager updated successfully", { id });
      return packageManager;
    } catch (error) {
      this.log.error("Failed to update package manager", { error });
      throw error;
    }
  }

  /**
   * Create multiple package managers in batch
   */
  async createPackageManagersBatch(
    packageManagers: SuggestedPackageManager[],
  ): Promise<{
    success: PackageManager[];
    failed: { packageManager: SuggestedPackageManager; error: string }[];
  }> {
    const success: PackageManager[] = [];
    const failed: { packageManager: SuggestedPackageManager; error: string }[] =
      [];

    for (const pm of packageManagers) {
      try {
        const created = await this.createPackageManager(
          pm.name,
          pm.icon,
          "devicon", // Suggested package managers always use devicon
          pm.category,
        );
        success.push(created);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        failed.push({ packageManager: pm, error: errorMessage });
        this.log.warn("Failed to create package manager in batch", {
          name: pm.name,
          error,
        });
      }
    }

    return { success, failed };
  }

  /**
   * Delete a package manager
   */
  async deletePackageManager(id: number): Promise<void> {
    try {
      this.log.info("Deleting package manager", { id });
      await invokeClient.post("delete_package_manager", { id });
      this.log.info("Package manager deleted successfully", { id });
    } catch (error) {
      this.log.error("Failed to delete package manager", { error });
      throw error;
    }
  }
}

export const packageManagerService = PackageManagerService.getInstance();
