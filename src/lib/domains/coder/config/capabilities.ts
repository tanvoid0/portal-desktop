/** What a coder provider offers; the shared UI hides what a provider lacks. */
export interface CoderCapabilities {
  agentModes: boolean;
  permissionModes: boolean;
  /** `catalog`: switch model any time from the provider catalog; `launch-only`: chosen once at launch. */
  modelPicker: 'catalog' | 'launch-only';
  queue: boolean;
  contextUsage: boolean;
  cost: boolean;
  agentChanges: boolean;
  editRerun: boolean;
  retry: boolean;
  subAgents: boolean;
  timeline: boolean;
  endRun: boolean;
  rememberRule: boolean;
}

export const PORTAL_CAPABILITIES: CoderCapabilities = {
  agentModes: true,
  permissionModes: true,
  modelPicker: 'catalog',
  queue: true,
  contextUsage: true,
  cost: false,
  agentChanges: true,
  editRerun: true,
  retry: true,
  subAgents: true,
  timeline: false,
  endRun: false,
  rememberRule: true,
};

// ponytail: identical for every external backend; branch on `backend` here when one differs.
export function agentCapabilities(_backend: string): CoderCapabilities {
  return {
    agentModes: false,
    permissionModes: false,
    modelPicker: 'launch-only',
    queue: true,
    contextUsage: false,
    cost: true,
    agentChanges: false,
    editRerun: false,
    retry: false,
    subAgents: false,
    timeline: true,
    endRun: true,
    rememberRule: false,
  };
}
