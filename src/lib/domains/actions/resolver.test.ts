import { describe, expect, it } from "vitest";
import { buildDefaultActions, buildDefaultWorkflows } from "./defaults";
import { profileFromDirectory } from "./profile";
import {
  mergeCatalog,
  resolveCatalogFromFileContent,
} from "./resolver";
import { parsePortalPipelineFile } from "./pipelineFile";
import type { ProjectAutomationProfile } from "./types";

function nodeProfile(
  overrides: Partial<ProjectAutomationProfile> = {},
): ProjectAutomationProfile {
  return {
    id: "1",
    name: "demo",
    path: "D:/demo",
    packageManager: "pnpm",
    frameworks: ["Next.js"],
    framework: "Next.js",
    ...overrides,
  };
}

describe("parsePortalPipelineFile", () => {
  it("parses YAML version 1", () => {
    const file = parsePortalPipelineFile(`
version: 1
actions:
  migrate:
    name: Migrate DB
    run: pnpm db:migrate
    category: utility
workflows:
  release:
    name: Release
    steps:
      - action: install
      - action: migrate
        needs: [install]
`);
    expect(file.version).toBe(1);
    expect(file.actions?.migrate?.run).toBe("pnpm db:migrate");
    expect(file.workflows?.release?.steps).toHaveLength(2);
  });

  it("parses JSON", () => {
    const file = parsePortalPipelineFile(
      JSON.stringify({
        version: 1,
        actions: { ping: { run: "echo ok", name: "Ping" } },
      }),
    );
    expect(file.actions?.ping?.run).toBe("echo ok");
  });

  it("rejects unsupported version", () => {
    expect(() => parsePortalPipelineFile("version: 99\n")).toThrow(/version/);
  });
});

describe("buildDefaultActions", () => {
  it("uses package manager and metadata commands", () => {
    const actions = buildDefaultActions(
      nodeProfile({
        buildCommand: "pnpm build",
        startCommand: "pnpm dev",
        testCommand: "pnpm test",
      }),
    );
    const byId = Object.fromEntries(actions.map((a) => [a.id, a]));
    expect(byId.install?.command).toBe("pnpm install");
    expect(byId.build?.command).toBe("pnpm build");
    expect(byId.dev?.command).toBe("pnpm dev");
    expect(byId.test?.command).toBe("pnpm test");
    expect(byId.dev?.longRunning).toBe(true);
  });

  it("builds rust defaults for cargo projects", () => {
    const actions = buildDefaultActions(
      nodeProfile({
        packageManager: "cargo",
        frameworks: ["Rust"],
        framework: "Rust",
        buildCommand: undefined,
        startCommand: undefined,
      }),
    );
    expect(actions.find((a) => a.id === "build")?.command).toBe("cargo build");
  });

  it("includes ci workflow", () => {
    const actions = buildDefaultActions(nodeProfile());
    const workflows = buildDefaultWorkflows(actions);
    expect(workflows.find((w) => w.id === "ci")?.steps.map((s) => s.action)).toEqual([
      "install",
      "test",
      "build",
    ]);
  });
});

describe("mergeCatalog priority", () => {
  it("file overrides custom overrides defaults", () => {
    const profile = nodeProfile();
    const catalog = resolveCatalogFromFileContent(
      profile,
      `
version: 1
actions:
  build:
    name: Custom Build
    run: echo from-file
workflows:
  ship:
    steps:
      - action: build
`,
      {
        customActions: [
          {
            id: "build",
            name: "DB Build",
            source: "custom",
            runner: "local",
            command: "echo from-custom",
          },
        ],
        includeLint: false,
      },
    );

    const build = catalog.actions.find((a) => a.id === "build");
    expect(build?.command).toBe("echo from-file");
    expect(build?.source).toBe("file");
    expect(catalog.workflows.find((w) => w.id === "ship")).toBeTruthy();
    expect(catalog.workflows.find((w) => w.id === "ci")).toBeTruthy();
  });

  it("warns on unknown workflow action refs", () => {
    const catalog = mergeCatalog(profileFromDirectory("/tmp/x"), {
      file: {
        version: 1,
        workflows: {
          broken: {
            steps: [{ action: "does-not-exist" }],
          },
        },
      },
      includeLint: false,
      skipFileLoad: true,
    });
    expect(catalog.warnings.some((w) => w.includes("does-not-exist"))).toBe(
      true,
    );
  });
});
