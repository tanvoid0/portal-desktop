/**
 * Service adapters to normalize different group structures to BaseItemGroup
 */

import type {
  BaseItem,
  BaseSuggestedItem,
  BaseItemGroup,
  ItemService,
} from "./types";
import type {
  Language,
  LanguageGroup,
  SuggestedLanguage,
} from "$lib/domains/languages/types";
import type {
  PackageManager,
  PackageManagerGroup,
  SuggestedPackageManager,
} from "$lib/domains/package_managers/types";
import type {
  Framework,
  FrameworkGroup,
  SuggestedFramework,
} from "$lib/domains/ide/services/ideService";
import { languageService } from "$lib/domains/languages/services/languageService";
import { packageManagerService } from "$lib/domains/package_managers/services/packageManagerService";
import { ideService } from "$lib/domains/ide/services/ideService";

/**
 * Normalize LanguageGroup to BaseItemGroup
 */
function normalizeLanguageGroup(
  group: LanguageGroup,
): BaseItemGroup<SuggestedLanguage> {
  return {
    category: group.category,
    items: group.languages,
  };
}

/**
 * Normalize PackageManagerGroup to BaseItemGroup
 */
function normalizePackageManagerGroup(
  group: PackageManagerGroup,
): BaseItemGroup<SuggestedPackageManager> {
  return {
    category: group.category,
    items: group.package_managers,
  };
}

/**
 * Normalize FrameworkGroup to BaseItemGroup
 */
function normalizeFrameworkGroup(
  group: FrameworkGroup,
): BaseItemGroup<SuggestedFramework> {
  return {
    category: group.category,
    items: group.frameworks,
  };
}

/**
 * Language Service Adapter
 */
export const languageServiceAdapter: ItemService<Language, SuggestedLanguage> =
  {
    async getAll() {
      return await languageService.getAllLanguages();
    },
    async getSuggested() {
      const groups = await languageService.getSuggestedLanguages();
      return groups.map(normalizeLanguageGroup);
    },
    async create(name, icon, iconType, category) {
      return await languageService.createLanguage(
        name,
        icon,
        iconType,
        category,
      );
    },
    async update(id, name, icon, iconType, category) {
      return await languageService.updateLanguage(
        id,
        name,
        icon,
        iconType,
        category,
      );
    },
    async delete(id) {
      return await languageService.deleteLanguage(id);
    },
    async createBatch(items) {
      const result = await languageService.createLanguagesBatch(items);
      return {
        success: result.success,
        failed: result.failed.map((f) => ({
          item: f.language,
          error: f.error,
        })),
      };
    },
  };

/**
 * Package Manager Service Adapter
 */
export const packageManagerServiceAdapter: ItemService<
  PackageManager,
  SuggestedPackageManager
> = {
  async getAll() {
    return await packageManagerService.getAllPackageManagers();
  },
  async getSuggested() {
    const groups = await packageManagerService.getSuggestedPackageManagers();
    return groups.map(normalizePackageManagerGroup);
  },
  async create(name, icon, iconType, category) {
    return await packageManagerService.createPackageManager(
      name,
      icon,
      iconType,
      category,
    );
  },
  async update(id, name, icon, iconType, category) {
    return await packageManagerService.updatePackageManager(
      id,
      name,
      icon,
      iconType,
      category,
    );
  },
  async delete(id) {
    return await packageManagerService.deletePackageManager(id);
  },
  async createBatch(items) {
    const result =
      await packageManagerService.createPackageManagersBatch(items);
    return {
      success: result.success,
      failed: result.failed.map((f) => ({
        item: f.packageManager,
        error: f.error,
      })),
    };
  },
};

/**
 * Framework Service Adapter
 */
export const frameworkServiceAdapter: ItemService<
  Framework,
  SuggestedFramework
> = {
  async getAll() {
    return await ideService.getAllFrameworks();
  },
  async getSuggested() {
    const groups = await ideService.getSuggestedFrameworks();
    return groups.map(normalizeFrameworkGroup);
  },
  async create(name, icon, iconType, category) {
    return await ideService.createFramework(name, icon, iconType, category);
  },
  async update(id, name, icon, iconType, category) {
    return await ideService.updateFramework(id, name, icon, iconType, category);
  },
  async delete(id) {
    return await ideService.deleteFramework(id);
  },
  async createBatch(items) {
    const result = await ideService.createFrameworksBatch(items);
    return {
      success: result.success,
      failed: result.failed.map((f) => ({ item: f.framework, error: f.error })),
    };
  },
};
