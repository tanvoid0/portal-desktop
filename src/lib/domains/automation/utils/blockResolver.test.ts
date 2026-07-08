import { describe, expect, it } from "vitest";
import type { Block } from "$lib/domains/projects/pipelines/types";
import {
  buildExecutionPlan,
  createStepFromBlock,
  mergeStepParameters,
  resolveBlockCommand,
} from "./blockResolver";

const installBlock: Block = {
  id: "install-npm",
  name: "Install NPM Dependencies",
  description: "Install deps",
  category: "build",
  version: "1.0.0",
  parameters: [
    {
      name: "packageManager",
      type: "select",
      description: "PM",
      required: true,
      defaultValue: "npm",
      options: ["npm", "pnpm"],
    },
  ],
  command: "${packageManager} install",
  executionType: "command",
  defaultConfig: { packageManager: "npm" },
  tags: [],
};

describe("blockResolver", () => {
  it("merges defaults, config, and explicit parameters", () => {
    const merged = mergeStepParameters(
      installBlock,
      { packageManager: "yarn" },
      { extra: "value" },
    );
    expect(merged.packageManager).toBe("yarn");
    expect(merged.extra).toBe("value");
  });

  it("resolves command with pipeline variables", () => {
    const { command } = resolveBlockCommand(
      installBlock,
      {},
      { packageManager: "pnpm" },
      { variables: { PROJECT_PATH: "/app" } },
    );
    expect(command).toBe("pnpm install");
  });

  it("creates a materialized pipeline step from a block", () => {
    const step = createStepFromBlock(installBlock);
    expect(step.blockId).toBe("install-npm");
    expect(step.config.command).toBe("${packageManager} install");
    expect(step.config.packageManager).toBe("npm");
  });

  it("builds an execution plan from block references", () => {
    const result = buildExecutionPlan(
      [
        { blockId: "install-npm", parameters: { packageManager: "pnpm" } },
      ],
      [installBlock],
      { variables: { PROJECT_PATH: "/proj" } },
    );
    expect(result.errors).toHaveLength(0);
    expect(result.steps[0].command).toBe("pnpm install");
  });
});
