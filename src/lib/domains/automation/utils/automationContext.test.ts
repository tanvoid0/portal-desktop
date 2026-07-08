import { describe, expect, it } from "vitest";
import {
  contextFromDirectory,
  contextFromProject,
  createAutomationContext,
} from "./automationContext";
import { normalizeStepRefs, presets } from "./stepRefs";

describe("automationContext", () => {
  it("sets cwd builtins for any directory", () => {
    const ctx = createAutomationContext({ cwd: "D:/apps/my-project" });
    expect(ctx.variables?.CWD).toBe("D:/apps/my-project");
    expect(ctx.variables?.PROJECT_PATH).toBe("D:/apps/my-project");
    expect(ctx.variables?.WORKING_DIR).toBe("D:/apps/my-project");
  });

  it("merges project metadata when provided", () => {
    const ctx = contextFromProject({
      id: "1",
      name: "Portal",
      path: "D:/portal",
      package_manager: "pnpm",
    });
    expect(ctx.variables?.PROJECT_NAME).toBe("Portal");
    expect(ctx.variables?.PACKAGE_MANAGER).toBe("pnpm");
    expect(ctx.variables?.PROJECT_ID).toBe("1");
  });

  it("allows extra variables from any source", () => {
    const ctx = contextFromDirectory("/tmp", { port: 3000, debug: true });
    expect(ctx.variables?.port).toBe("3000");
    expect(ctx.variables?.debug).toBe("true");
  });
});

describe("stepRefs", () => {
  it("normalizes string refs to step inputs", () => {
    const steps = normalizeStepRefs(["install-npm", { blockId: "test-npm" }]);
    expect(steps[0].blockId).toBe("install-npm");
    expect(steps[1].blockId).toBe("test-npm");
  });

  it("exposes common presets", () => {
    expect(presets.ci("pnpm")).toHaveLength(3);
    expect(presets.install()).toHaveLength(1);
  });
});
