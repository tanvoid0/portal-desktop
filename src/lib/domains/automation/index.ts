export { default as QuickRunPage } from "./components/QuickRunPage.svelte";
export { default as BlocksPage } from "./components/BlocksPage.svelte";
export { default as ScriptsPage } from "./components/ScriptsPage.svelte";
export { default as UtilitiesPage } from "./components/UtilitiesPage.svelte";
export { default as WorkflowTrigger } from "./components/WorkflowTrigger.svelte";
export { default as WorkflowResults } from "./components/WorkflowResults.svelte";

/** Primary API — use this for integrations */
export { automation, automationExecutionService } from "./services/automationExecutionService";
export type {
  RunAutomationOptions,
  ResolveAutomationOptions,
  AutomationRunResult,
  AutomationStepResult,
  AutomationRunner,
  AutomationPlanPreview,
} from "./services/automationExecutionService";

export * from "./utils/blockResolver";
export * from "./utils/automationContext";
export * from "./utils/stepRefs";
export * from "./types";
export { automationStore } from "./stores/automationStore";
