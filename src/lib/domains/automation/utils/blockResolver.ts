/**
 * Block Resolver — single source of truth for turning block references
 * into executable commands with merged parameters and variables.
 */

import type {
  Block,
  BlockParameter,
  PipelineStep,
  PipelineVariable,
} from "$lib/domains/projects/pipelines/types";
import {
  substituteVariables,
  validateVariables,
  type SubstitutionContext,
} from "$lib/domains/projects/pipelines/utils/variableSubstitution";

/** A block reference with optional per-step parameter overrides */
export interface AutomationStepInput {
  blockId: string;
  name?: string;
  id?: string;
  parameters?: Record<string, string | number | boolean>;
  overrides?: {
    command?: string;
    longRunning?: boolean;
    workingDirectory?: string;
  };
  dependsOn?: string[];
}

/** Fully resolved step ready for execution */
export interface ResolvedExecutionStep {
  id: string;
  blockId: string;
  name: string;
  command: string;
  executionType: Block["executionType"];
  longRunning: boolean;
  workingDirectory?: string;
  dependsOn: string[];
  parameters: Record<string, string>;
}

export interface ResolveContext {
  variables?: Record<string, string>;
  secrets?: Record<string, string>;
  projectVariables?: Record<string, string>;
}

export interface ResolvePlanResult {
  steps: ResolvedExecutionStep[];
  errors: string[];
  warnings: string[];
}

function paramValueToString(value: unknown): string {
  if (value === undefined || value === null) return "";
  if (typeof value === "boolean") return value ? "true" : "false";
  return String(value);
}

/** Merge block defaults → step config → explicit parameters (later wins) */
export function mergeStepParameters(
  block: Block,
  stepConfig: Record<string, unknown> = {},
  explicitParams: Record<string, string | number | boolean> = {},
): Record<string, string> {
  const merged: Record<string, string> = {};

  for (const param of block.parameters) {
    if (param.defaultValue !== undefined) {
      merged[param.name] = paramValueToString(param.defaultValue);
    }
  }

  for (const [key, value] of Object.entries(block.defaultConfig ?? {})) {
    merged[key] = paramValueToString(value);
  }

  for (const [key, value] of Object.entries(stepConfig)) {
    if (key === "command" || key === "longRunning" || key === "workingDirectory") {
      continue;
    }
    merged[key] = paramValueToString(value);
  }

  for (const [key, value] of Object.entries(explicitParams)) {
    merged[key] = paramValueToString(value);
  }

  return merged;
}

/** Resolve the command template for a single block + step */
export function resolveBlockCommand(
  block: Block,
  stepConfig: Record<string, unknown> = {},
  explicitParams: Record<string, string | number | boolean> = {},
  context: ResolveContext = {},
): { command: string; parameters: Record<string, string>; missing: string[] } {
  const parameters = mergeStepParameters(block, stepConfig, explicitParams);

  const commandTemplate =
    (stepConfig.command as string | undefined) ??
    (explicitParams as Record<string, unknown>).command as string | undefined ??
    block.command;

  const substitutionContext: SubstitutionContext = {
    variables: { ...parameters, ...(context.variables ?? {}) },
    secrets: context.secrets,
    projectVariables: context.projectVariables,
  };

  const validation = validateVariables(commandTemplate, substitutionContext);
  const command = substituteVariables(commandTemplate, substitutionContext);

  return { command, parameters, missing: validation.missing };
}

/** Build a pipeline step from a block library entry */
export function createStepFromBlock(
  block: Block,
  overrides?: Partial<AutomationStepInput>,
): PipelineStep {
  const config: Record<string, unknown> = {
    ...block.defaultConfig,
    command: block.command,
    longRunning: false,
    ...overrides?.overrides,
  };

  return {
    id: overrides?.id ?? crypto.randomUUID(),
    blockId: block.id,
    name: overrides?.name ?? block.name,
    config,
    dependsOn: overrides?.dependsOn ?? [],
  };
}

/** Resolve a stored pipeline step against the block library */
export function resolvePipelineStep(
  step: PipelineStep,
  block: Block | null,
  context: ResolveContext = {},
): { resolved: ResolvedExecutionStep | null; error?: string } {
  const config = (step.config ?? {}) as Record<string, unknown>;

  if (!block && !config.command) {
    return {
      resolved: null,
      error: `Step "${step.name}" (${step.blockId}): block not found and no command override`,
    };
  }

  const commandTemplate = (config.command as string) ?? block?.command;
  if (!commandTemplate) {
    return {
      resolved: null,
      error: `Step "${step.name}" (${step.blockId}): no command template`,
    };
  }

  const parameters = block
    ? mergeStepParameters(block, config)
    : Object.fromEntries(
        Object.entries(config).filter(
          ([k]) => !["command", "longRunning", "workingDirectory"].includes(k),
        ).map(([k, v]) => [k, paramValueToString(v)]),
      );

  const substitutionContext: SubstitutionContext = {
    variables: { ...parameters, ...(context.variables ?? {}) },
    secrets: context.secrets,
    projectVariables: context.projectVariables,
  };

  const command = substituteVariables(commandTemplate, substitutionContext);

  return {
    resolved: {
      id: step.id,
      blockId: step.blockId,
      name: step.name,
      command,
      executionType: block?.executionType ?? "command",
      longRunning: Boolean(config.longRunning),
      workingDirectory: config.workingDirectory as string | undefined,
      dependsOn: step.dependsOn ?? [],
      parameters,
    },
  };
}

/** Resolve an ordered list of block references into an execution plan */
export function buildExecutionPlan(
  inputs: AutomationStepInput[],
  blocks: Block[],
  context: ResolveContext = {},
): ResolvePlanResult {
  const blockMap = new Map(blocks.map((b) => [b.id, b]));
  const steps: ResolvedExecutionStep[] = [];
  const errors: string[] = [];
  const warnings: string[] = [];

  for (const input of inputs) {
    const block = blockMap.get(input.blockId);
    if (!block) {
      errors.push(`Block not found: ${input.blockId}`);
      continue;
    }

    const stepConfig: Record<string, unknown> = {
      ...input.overrides,
    };

    const { command, parameters, missing } = resolveBlockCommand(
      block,
      stepConfig,
      input.parameters ?? {},
      context,
    );

    if (missing.length > 0) {
      const requiredMissing = missing.filter((name) =>
        block.parameters.some((p: BlockParameter) => p.name === name && p.required),
      );
      if (requiredMissing.length > 0) {
        errors.push(
          `Block "${block.name}": missing required parameters: ${requiredMissing.join(", ")}`,
        );
        continue;
      }
      warnings.push(
        `Block "${block.name}": unresolved variables: ${missing.join(", ")}`,
      );
    }

    steps.push({
      id: input.id ?? crypto.randomUUID(),
      blockId: input.blockId,
      name: input.name ?? block.name,
      command,
      executionType: block.executionType,
      longRunning: input.overrides?.longRunning ?? false,
      workingDirectory: input.overrides?.workingDirectory,
      dependsOn: input.dependsOn ?? [],
      parameters,
    });
  }

  return { steps, errors, warnings };
}

/** Resolve pipeline steps, materializing config.command where missing */
export function materializePipelineSteps(
  steps: PipelineStep[],
  blocks: Block[],
  context: ResolveContext = {},
): { steps: PipelineStep[]; errors: string[] } {
  const blockMap = new Map(blocks.map((b) => [b.id, b]));
  const errors: string[] = [];

  const materialized = steps.map((step) => {
    const block = blockMap.get(step.blockId) ?? null;
    const { resolved, error } = resolvePipelineStep(step, block, context);

    if (error) {
      errors.push(error);
      return step;
    }

    if (!resolved) return step;

    return {
      ...step,
      config: {
        ...step.config,
        command: resolved.command,
        longRunning: resolved.longRunning,
        ...resolved.parameters,
      },
    };
  });

  return { steps: materialized, errors };
}

/** Convert pipeline variables array to a flat record */
export function pipelineVariablesToRecord(
  variables: PipelineVariable[] = [],
): Record<string, string> {
  return Object.fromEntries(
    variables.map((v) => [v.name, String(v.value ?? "")]),
  );
}
