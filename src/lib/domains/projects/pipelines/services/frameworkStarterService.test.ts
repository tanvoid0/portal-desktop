import { describe, expect, it } from "vitest";
import type { Project } from "$lib/domains/projects/types";
import { ProjectStatus } from "$lib/domains/projects/types";
import {
  frameworkToSlug,
  getStarterPack,
  normalizePackageScriptCommand,
  resolveCommands,
} from "./frameworkStarterService";

// Access private helper via duplicate logic for priority test
function pickPack(names: string[]) {
  const slugs = names
    .map((n) => frameworkToSlug(n))
    .filter((s): s is string => !!s);
  const priority = [
    "nextjs",
    "nuxt",
    "react",
    "vue",
    "svelte",
    "angular",
    "django",
    "rust",
  ];
  for (const p of priority) {
    if (slugs.includes(p)) return getStarterPack(p);
  }
  for (const s of slugs) {
    const pack = getStarterPack(s);
    if (pack) return pack;
  }
  return null;
}

describe("frameworkStarterService", () => {
  it("maps Next.js to nextjs slug", () => {
    expect(frameworkToSlug("Next.js")).toBe("nextjs");
  });

  it("prefers Next.js starter pack over Node.js", () => {
    const pack = pickPack(["Node.js", "Next.js"]);
    expect(pack?.frameworkSlug).toBe("nextjs");
    expect(pack?.pipelines).toHaveLength(3);
  });

  it("returns nextjs install/dev/build pipeline defs", () => {
    const pack = getStarterPack("nextjs");
    expect(pack?.pipelines.map((p) => p.category)).toEqual([
      "install",
      "dev",
      "build",
    ]);
  });

  it("keeps pnpm build shorthand", () => {
    expect(normalizePackageScriptCommand("pnpm", "pnpm build", "build")).toBe(
      "pnpm build",
    );
  });

  it("normalizes script bodies to pnpm build", () => {
    expect(
      normalizePackageScriptCommand("pnpm", "next build --webpack", "build"),
    ).toBe("pnpm build");
  });

  it("uses npm run build for npm projects", () => {
    expect(normalizePackageScriptCommand("npm", "next build", "build")).toBe(
      "npm run build",
    );
  });

  it("resolveCommands preserves pnpm build build_command", () => {
    const project = {
      id: "1",
      name: "Devstrail",
      path: "D:\\devstrail\\devstrail",
      status: ProjectStatus.ACTIVE,
      build_command: "pnpm build",
      package_manager_ids: [3],
      framework_ids: [],
      language_ids: [],
      metadata: { dependencies: { packageManager: "pnpm" } },
    } as Project;

    expect(
      resolveCommands(project, {
        presetKey: "nextjs-build",
        name: "Build Production",
        description: "Build",
        category: "build",
        command: "${PACKAGE_MANAGER} run build",
      }),
    ).toBe("pnpm build");
  });
});
