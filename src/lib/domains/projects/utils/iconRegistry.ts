/**
 * Icon registry for resolving framework, package manager, and language icons.
 * Each entity type has its own ID namespace in the database.
 */

import { ideService } from "$lib/domains/ide";
import { packageManagerService } from "$lib/domains/package_managers";
import { languageService } from "$lib/domains/languages";
import type { Project } from "$lib/domains/projects/types";
import {
  getProjectFramework,
  getProjectPackageManager,
} from "$lib/domains/projects/utils/display";

export interface ResolvedTechIcon {
  name: string;
  icon: string;
  icon_type: "devicon" | "file";
}

const FALLBACK_ICONS: Record<string, ResolvedTechIcon> = {
  react: { name: "React", icon: "logos:react", icon_type: "devicon" },
  vue: { name: "Vue", icon: "logos:vue", icon_type: "devicon" },
  angular: { name: "Angular", icon: "logos:angular-icon", icon_type: "devicon" },
  svelte: { name: "Svelte", icon: "logos:svelte-icon", icon_type: "devicon" },
  "next.js": { name: "Next.js", icon: "logos:nextjs-icon", icon_type: "devicon" },
  nuxt: { name: "Nuxt", icon: "logos:nuxt-icon", icon_type: "devicon" },
  "node.js": { name: "Node.js", icon: "logos:nodejs-icon", icon_type: "devicon" },
  express: { name: "Express", icon: "logos:express", icon_type: "devicon" },
  fastapi: { name: "FastAPI", icon: "logos:fastapi-icon", icon_type: "devicon" },
  django: { name: "Django", icon: "logos:django-icon", icon_type: "devicon" },
  flask: { name: "Flask", icon: "logos:flask", icon_type: "devicon" },
  laravel: { name: "Laravel", icon: "logos:laravel", icon_type: "devicon" },
  spring: { name: "Spring", icon: "logos:spring-icon", icon_type: "devicon" },
  rails: { name: "Rails", icon: "logos:rails", icon_type: "devicon" },
  flutter: { name: "Flutter", icon: "logos:flutter", icon_type: "devicon" },
  electron: { name: "Electron", icon: "logos:electron", icon_type: "devicon" },
  tauri: { name: "Tauri", icon: "logos:tauri", icon_type: "devicon" },
  python: { name: "Python", icon: "logos:python", icon_type: "devicon" },
  java: { name: "Java", icon: "logos:java", icon_type: "devicon" },
  go: { name: "Go", icon: "logos:go", icon_type: "devicon" },
  rust: { name: "Rust", icon: "logos:rust", icon_type: "devicon" },
  php: { name: "PHP", icon: "logos:php", icon_type: "devicon" },
  ruby: { name: "Ruby", icon: "logos:ruby", icon_type: "devicon" },
  swift: { name: "Swift", icon: "logos:swift", icon_type: "devicon" },
  html: { name: "HTML", icon: "logos:html-5", icon_type: "devicon" },
  css: { name: "CSS", icon: "logos:css-3", icon_type: "devicon" },
  "c#": { name: "C#", icon: "logos:c-sharp", icon_type: "devicon" },
  typescript: {
    name: "TypeScript",
    icon: "logos:typescript-icon",
    icon_type: "devicon",
  },
  javascript: {
    name: "JavaScript",
    icon: "logos:javascript",
    icon_type: "devicon",
  },
  npm: { name: "npm", icon: "logos:npm-icon", icon_type: "devicon" },
  yarn: { name: "yarn", icon: "logos:yarn", icon_type: "devicon" },
  pnpm: { name: "pnpm", icon: "logos:pnpm", icon_type: "devicon" },
  bun: { name: "bun", icon: "logos:bun", icon_type: "devicon" },
  pip: { name: "pip", icon: "logos:python", icon_type: "devicon" },
  cargo: { name: "cargo", icon: "logos:rust", icon_type: "devicon" },
  composer: { name: "composer", icon: "logos:composer", icon_type: "devicon" },
  maven: { name: "maven", icon: "logos:maven", icon_type: "devicon" },
  gradle: { name: "gradle", icon: "logos:gradle", icon_type: "devicon" },
};

type EntityKind = "framework" | "packageManager" | "language";

class ProjectIconRegistry {
  private frameworksById = new Map<number, ResolvedTechIcon>();
  private frameworksByName = new Map<string, ResolvedTechIcon>();
  private packageManagersById = new Map<number, ResolvedTechIcon>();
  private packageManagersByName = new Map<string, ResolvedTechIcon>();
  private languagesById = new Map<number, ResolvedTechIcon>();
  private languagesByName = new Map<string, ResolvedTechIcon>();
  private loaded = false;
  private loading: Promise<void> | null = null;

  private toResolved(item: {
    name: string;
    icon: string;
    icon_type?: "devicon" | "file";
  }): ResolvedTechIcon {
    return {
      name: item.name,
      icon: item.icon,
      icon_type: item.icon_type ?? (item.icon.includes(":") ? "devicon" : "file"),
    };
  }

  private registerItem(
    kind: EntityKind,
    item: {
      id?: number;
      name: string;
      icon: string;
      icon_type?: "devicon" | "file";
    },
    overwrite = false,
  ) {
    const resolved = this.toResolved(item);
    const nameKey = item.name.toLowerCase();

    const { byId, byName } = this.mapsFor(kind);

    if (overwrite || !byName.has(nameKey)) {
      byName.set(nameKey, resolved);
    }
    if (item.id !== undefined && (overwrite || !byId.has(item.id))) {
      byId.set(item.id, resolved);
    }
  }

  private mapsFor(kind: EntityKind): {
    byId: Map<number, ResolvedTechIcon>;
    byName: Map<string, ResolvedTechIcon>;
  } {
    switch (kind) {
      case "framework":
        return {
          byId: this.frameworksById,
          byName: this.frameworksByName,
        };
      case "packageManager":
        return {
          byId: this.packageManagersById,
          byName: this.packageManagersByName,
        };
      case "language":
        return {
          byId: this.languagesById,
          byName: this.languagesByName,
        };
    }
  }

  async ensureLoaded(): Promise<void> {
    if (this.loaded) return;
    if (this.loading) {
      await this.loading;
      return;
    }

    this.loading = this.load();
    await this.loading;
  }

  private async load(): Promise<void> {
    try {
      const [
        frameworks,
        frameworkGroups,
        packageManagers,
        packageManagerGroups,
        languages,
        languageGroups,
      ] = await Promise.all([
        ideService.getAllFrameworks().catch(() => []),
        ideService.getSuggestedFrameworks().catch(() => []),
        packageManagerService.getAllPackageManagers().catch(() => []),
        packageManagerService.getSuggestedPackageManagers().catch(() => []),
        languageService.getAllLanguages().catch(() => []),
        languageService.getSuggestedLanguages().catch(() => []),
      ]);

      for (const framework of frameworks) {
        this.registerItem("framework", framework, true);
      }
      for (const group of frameworkGroups) {
        for (const framework of group.frameworks) {
          this.registerItem("framework", framework);
        }
      }

      for (const packageManager of packageManagers) {
        this.registerItem("packageManager", packageManager, true);
      }
      for (const group of packageManagerGroups) {
        for (const packageManager of group.package_managers) {
          this.registerItem("packageManager", packageManager);
        }
      }

      for (const language of languages) {
        this.registerItem("language", language, true);
      }
      for (const group of languageGroups) {
        for (const language of group.languages) {
          this.registerItem("language", language);
        }
      }

      this.loaded = true;
    } finally {
      this.loading = null;
    }
  }

  private resolveByIds(
    kind: EntityKind,
    ids: number[] | undefined,
  ): ResolvedTechIcon[] {
    if (!ids?.length) return [];

    const { byId } = this.mapsFor(kind);
    const seen = new Set<string>();
    const resolved: ResolvedTechIcon[] = [];

    for (const rawId of ids) {
      const id = Number(rawId);
      if (!Number.isFinite(id)) continue;

      const icon = byId.get(id);
      if (icon && !seen.has(icon.name.toLowerCase())) {
        seen.add(icon.name.toLowerCase());
        resolved.push(icon);
      }
    }

    return resolved;
  }

  private resolveByName(
    kind: EntityKind,
    name: string | undefined,
  ): ResolvedTechIcon | undefined {
    if (!name) return undefined;

    const { byName } = this.mapsFor(kind);
    return byName.get(name.toLowerCase()) ?? FALLBACK_ICONS[name.toLowerCase()];
  }

  resolvePrimaryFramework(project: Project): ResolvedTechIcon | undefined {
    const fromIds = this.resolveByIds("framework", project.framework_ids);
    if (fromIds.length > 0) return fromIds[0];

    return this.resolveByName("framework", getProjectFramework(project));
  }

  resolveFrameworks(project: Project): ResolvedTechIcon[] {
    const fromIds = this.resolveByIds("framework", project.framework_ids);
    if (fromIds.length > 0) return fromIds;

    const resolved = this.resolveByName("framework", getProjectFramework(project));
    return resolved ? [resolved] : [];
  }

  resolvePackageManagers(project: Project): ResolvedTechIcon[] {
    const fromIds = this.resolveByIds(
      "packageManager",
      project.package_manager_ids,
    );
    if (fromIds.length > 0) return fromIds;

    const resolved = this.resolveByName(
      "packageManager",
      getProjectPackageManager(project),
    );
    return resolved ? [resolved] : [];
  }

  resolveLanguages(project: Project): ResolvedTechIcon[] {
    return this.resolveByIds("language", project.language_ids);
  }
}

export const projectIconRegistry = new ProjectIconRegistry();
