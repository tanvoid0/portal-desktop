/**
 * IDE Service - Frontend service for IDE configuration
 */

import { invokeClient } from "$lib/utils/invokeClient";
import { logger } from "$lib/domains/shared";

export interface IdeConfig {
  id?: number;
  name: string;
  executable: string;
  is_default?: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface FrameworkIdeMapping {
  id?: number;
  framework: string;
  ide_id: number;
  created_at?: string;
  updated_at?: string;
}

export interface SuggestedFramework {
  name: string;
  icon: string;
  category: string;
}

export interface FrameworkGroup {
  category: string;
  frameworks: SuggestedFramework[];
}

export interface Framework {
  id: number;
  name: string;
  icon: string;
  icon_type: "devicon" | "file";
  category: string;
  created_at?: string;
  updated_at?: string;
}

export class IdeService {
  private static instance: IdeService;
  private log = logger.createScoped("IdeService");

  private constructor() {}

  static getInstance(): IdeService {
    if (!IdeService.instance) {
      IdeService.instance = new IdeService();
    }
    return IdeService.instance;
  }

  /**
   * Get all configured IDEs
   */
  async getAllIdes(): Promise<IdeConfig[]> {
    try {
      this.log.info("Getting all IDEs");
      const ides = await invokeClient.post<IdeConfig[]>("get_all_ides");
      const safeIdes = ides ?? [];
      this.log.info("IDEs retrieved", { count: safeIdes.length });
      return safeIdes;
    } catch (error) {
      this.log.error("Failed to get IDEs", { error });
      throw error;
    }
  }

  /**
   * Add a new IDE
   */
  async addIde(name: string, executable: string): Promise<number> {
    try {
      this.log.info("Adding IDE", { name, executable });
      const id = await invokeClient.post<number>("add_ide", {
        name,
        executable,
      });
      this.log.info("IDE added successfully", { id });
      return id;
    } catch (error) {
      this.log.error("Failed to add IDE", { error });
      throw error;
    }
  }

  /**
   * Update an existing IDE
   */
  async updateIde(
    id: number,
    name: string,
    executable: string,
  ): Promise<number> {
    try {
      this.log.info("Updating IDE", { id, name, executable });
      const result = await invokeClient.post<number>("update_ide", {
        id,
        name,
        executable,
      });
      this.log.info("IDE updated successfully", { id });
      return result;
    } catch (error) {
      this.log.error("Failed to update IDE", { error });
      throw error;
    }
  }

  /**
   * Delete an IDE
   */
  async deleteIde(id: number): Promise<number> {
    try {
      this.log.info("Deleting IDE", { id });
      const result = await invokeClient.post<number>("delete_ide", { id });
      this.log.info("IDE deleted successfully", { id });
      return result;
    } catch (error) {
      this.log.error("Failed to delete IDE", { error });
      throw error;
    }
  }

  /**
   * Set default IDE
   */
  async setDefaultIde(id: number): Promise<number> {
    try {
      this.log.info("Setting default IDE", { id });
      const result = await invokeClient.post<number>("set_default_ide", { id });
      this.log.info("Default IDE set successfully", { id });
      return result;
    } catch (error) {
      this.log.error("Failed to set default IDE", { error });
      throw error;
    }
  }

  /**
   * Get default IDE
   */
  async getDefaultIde(): Promise<IdeConfig | null> {
    try {
      this.log.info("Getting default IDE");
      const ide = await invokeClient.post<IdeConfig | null>("get_default_ide");
      return ide ?? null;
    } catch (error) {
      this.log.error("Failed to get default IDE", { error });
      throw error;
    }
  }

  /**
   * Detect installed IDEs on the system
   */
  async detectInstalledIdes(): Promise<string[]> {
    try {
      this.log.info("Detecting installed IDEs");
      const ides = await invokeClient.post<string[]>("detect_installed_ides");
      const safeIdes = ides ?? [];
      this.log.info("IDEs detected", { count: safeIdes.length });
      return safeIdes;
    } catch (error) {
      this.log.error("Failed to detect IDEs", { error });
      throw error;
    }
  }

  /**
   * Get all framework IDE mappings
   */
  async getAllFrameworkIdeMappings(): Promise<FrameworkIdeMapping[]> {
    try {
      this.log.info("Getting all framework IDE mappings");
      const mappings = await invokeClient.post<FrameworkIdeMapping[]>(
        "get_all_framework_ide_mappings",
      );
      const safeMappings = mappings ?? [];
      this.log.info("Framework IDE mappings retrieved", {
        count: safeMappings.length,
      });
      return safeMappings;
    } catch (error) {
      this.log.error("Failed to get framework IDE mappings", { error });
      throw error;
    }
  }

  /**
   * Set framework IDE mapping
   */
  async setFrameworkIdeMapping(
    framework: string,
    ideId: number,
  ): Promise<number> {
    try {
      this.log.info("Setting framework IDE mapping", { framework, ideId });
      const result = await invokeClient.post<number>(
        "set_framework_ide_mapping",
        { framework, ide_id: ideId },
      );
      this.log.info("Framework IDE mapping set successfully", {
        framework,
        ideId,
      });
      return result;
    } catch (error) {
      this.log.error("Failed to set framework IDE mapping", { error });
      throw error;
    }
  }

  /**
   * Get framework IDE mapping
   */
  async getFrameworkIdeMapping(framework: string): Promise<IdeConfig | null> {
    try {
      this.log.info("Getting framework IDE mapping", { framework });
      const ide = await invokeClient.post<IdeConfig | null>(
        "get_framework_ide_mapping",
        { framework },
      );
      return ide ?? null;
    } catch (error) {
      this.log.error("Failed to get framework IDE mapping", { error });
      throw error;
    }
  }

  /**
   * Delete framework IDE mapping
   */
  async deleteFrameworkIdeMapping(framework: string): Promise<number> {
    try {
      this.log.info("Deleting framework IDE mapping", { framework });
      const result = await invokeClient.post<number>(
        "delete_framework_ide_mapping",
        { framework },
      );
      this.log.info("Framework IDE mapping deleted successfully", {
        framework,
      });
      return result;
    } catch (error) {
      this.log.error("Failed to delete framework IDE mapping", { error });
      throw error;
    }
  }

  /**
   * Get suggested frameworks from backend
   */
  async getSuggestedFrameworks(): Promise<FrameworkGroup[]> {
    try {
      this.log.info("Getting suggested frameworks");
      const groups = await invokeClient.post<FrameworkGroup[]>(
        "get_suggested_frameworks",
      );
      const safeGroups = groups ?? [];
      this.log.info("Suggested frameworks retrieved", {
        count: safeGroups.length,
      });
      return safeGroups;
    } catch (error) {
      this.log.error("Failed to get suggested frameworks", { error });
      throw error;
    }
  }

  /**
   * Get all user-defined frameworks from database
   */
  async getAllFrameworks(): Promise<Framework[]> {
    try {
      this.log.info("Getting all frameworks");
      const frameworks =
        await invokeClient.post<Framework[]>("get_all_frameworks");
      const safeFrameworks = frameworks ?? [];
      this.log.info("Frameworks retrieved", { count: safeFrameworks.length });
      return safeFrameworks;
    } catch (error) {
      this.log.error("Failed to get frameworks", { error });
      throw error;
    }
  }

  /**
   * Create a custom framework
   */
  async createFramework(
    name: string,
    icon: string,
    iconType: "devicon" | "file",
    category: string,
  ): Promise<Framework> {
    try {
      this.log.info("Creating framework", { name, icon, iconType, category });
      const framework = await invokeClient.post<Framework>("create_framework", {
        name,
        icon,
        iconType, // Tauri v2 converts camelCase to snake_case automatically
        category,
      });
      if (!framework) {
        throw new Error("Failed to create framework: no response");
      }
      this.log.info("Framework created successfully", { id: framework.id });
      return framework;
    } catch (error) {
      this.log.error("Failed to create framework", { error });
      throw error;
    }
  }

  /**
   * Update a framework
   */
  async updateFramework(
    id: number,
    name?: string,
    icon?: string,
    iconType?: "devicon" | "file",
    category?: string,
  ): Promise<Framework> {
    try {
      this.log.info("Updating framework", {
        id,
        name,
        icon,
        iconType,
        category,
      });
      const framework = await invokeClient.post<Framework>("update_framework", {
        id,
        name,
        icon,
        iconType, // Tauri v2 converts camelCase to snake_case automatically
        category,
      });
      if (!framework) {
        throw new Error("Failed to update framework: no response");
      }
      this.log.info("Framework updated successfully", { id });
      return framework;
    } catch (error) {
      this.log.error("Failed to update framework", { error });
      throw error;
    }
  }

  /**
   * Create multiple frameworks in batch
   */
  async createFrameworksBatch(frameworks: SuggestedFramework[]): Promise<{
    success: Framework[];
    failed: { framework: SuggestedFramework; error: string }[];
  }> {
    const success: Framework[] = [];
    const failed: { framework: SuggestedFramework; error: string }[] = [];

    for (const framework of frameworks) {
      try {
        const created = await this.createFramework(
          framework.name,
          framework.icon,
          "devicon", // Suggested frameworks always use devicon
          framework.category,
        );
        success.push(created);
      } catch (error) {
        const errorMessage =
          error instanceof Error ? error.message : String(error);
        failed.push({ framework, error: errorMessage });
        this.log.warn("Failed to create framework in batch", {
          name: framework.name,
          error,
        });
      }
    }

    return { success, failed };
  }

  /**
   * Delete a framework
   */
  async deleteFramework(id: number): Promise<void> {
    try {
      this.log.info("Deleting framework", { id });
      await invokeClient.post("delete_framework", { id });
      this.log.info("Framework deleted successfully", { id });
    } catch (error) {
      this.log.error("Failed to delete framework", { error });
      throw error;
    }
  }
}

export const ideService = IdeService.getInstance();
