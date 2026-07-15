export type {
  ActionSource,
  ActionRunner,
  ActionCategory,
  UnifiedAction,
  UnifiedWorkflow,
  UnifiedWorkflowStep,
  ActionRunOptions,
  ActionStepResult,
  ActionRunResult,
  ActionPlanPreview,
  PortalPipelineFile,
  ProjectAutomationProfile,
} from "./types";

export {
  profileFromProject,
  profileFromDirectory,
  profileVariables,
  resolveFrameworkNames,
  resolvePackageManagerName,
  substituteVars,
} from "./profile";

export { buildDefaultActions, buildDefaultWorkflows } from "./defaults";

export {
  parsePortalPipelineFile,
  validatePortalPipelineFile,
  loadPortalPipelineFile,
} from "./pipelineFile";

export {
  mergeCatalog,
  resolveProjectCatalog,
  resolveDirectoryCatalog,
  resolveCatalogFromFileContent,
} from "./resolver";
export type { ResolveCatalogOptions, ActionCatalog } from "./resolver";

export { actions, actionsService } from "./actionsService";
export type { ProjectActionsHandle } from "./actionsService";
