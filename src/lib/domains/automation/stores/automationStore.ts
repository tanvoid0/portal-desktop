import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { 
	WorkflowExecution, 
	WorkflowResult, 
	AvailableWorkflow 
} from '../types';

interface AutomationState {
	availableWorkflows: AvailableWorkflow[];
	suggestedWorkflows: AvailableWorkflow[];
	activeExecutions: Map<string, WorkflowExecution>;
	executionResults: Map<string, WorkflowResult>;
	isN8nHealthy: boolean;
	loading: boolean;
	error: string | null;
}

const initialState: AutomationState = {
	availableWorkflows: [],
	suggestedWorkflows: [],
	activeExecutions: new Map(),
	executionResults: new Map(),
	isN8nHealthy: false,
	loading: false,
	error: null
};

function createAutomationStore() {
	const { subscribe, set, update } = writable<AutomationState>(initialState);

	return {
		subscribe,

		// Load available workflows
		async loadWorkflows() {
			update(state => ({ ...state, loading: true, error: null }));
			
			try {
				const workflows = await invoke<AvailableWorkflow[]>('list_available_workflows');
				update(state => ({ 
					...state, 
					availableWorkflows: workflows,
					loading: false 
				}));
			} catch (error) {
				update(state => ({ 
					...state, 
					error: `Failed to load workflows: ${error}`,
					loading: false 
				}));
			}
		},

		// Get suggested workflows for a project
		async getSuggestedWorkflows(framework?: string, packageManager?: string) {
			update(state => ({ ...state, loading: true, error: null }));
			
			try {
				const workflows = await invoke<AvailableWorkflow[]>('get_suggested_workflows', {
					framework,
					packageManager
				});
				update(state => ({ 
					...state, 
					suggestedWorkflows: workflows,
					loading: false 
				}));
			} catch (error) {
				update(state => ({ 
					...state, 
					error: `Failed to get suggested workflows: ${error}`,
					loading: false 
				}));
			}
		},

		// Trigger a workflow
		async triggerWorkflow(workflowId: string, projectData: { id: string; name: string; path: string; framework?: string; package_manager?: string; build_command?: string; start_command?: string; test_command?: string; output_directory?: string; dev_port?: number; prod_port?: number; }) {
			update(state => ({ ...state, loading: true, error: null }));
			
			try {
				const result = await invoke<WorkflowResult>('trigger_n8n_workflow', {
					workflowId,
					projectData
				});
				
				update(state => {
					const newResults = new Map(state.executionResults);
					newResults.set(result.execution_id, result);
					return { 
						...state, 
						executionResults: newResults,
						loading: false 
					};
				});
				
				return result;
			} catch (error) {
				update(state => ({ 
					...state, 
					error: `Failed to trigger workflow: ${error}`,
					loading: false 
				}));
				throw error;
			}
		},

		// Check workflow status
		async checkWorkflowStatus(executionId: string) {
			try {
				const execution = await invoke<WorkflowExecution>('get_workflow_status', {
					executionId
				});
				
				update(state => {
					const newExecutions = new Map(state.activeExecutions);
					newExecutions.set(executionId, execution);
					return { 
						...state, 
						activeExecutions: newExecutions
					};
				});
				
				return execution;
			} catch (error) {
				update(state => ({ 
					...state, 
					error: `Failed to check workflow status: ${error}`
				}));
				throw error;
			}
		},

		// Check n8n health
		async checkHealth() {
			try {
				const isHealthy = await invoke<boolean>('check_n8n_health');
				update(state => ({ ...state, isN8nHealthy: isHealthy }));
				return isHealthy;
			} catch (error) {
				update(state => ({ 
					...state, 
					isN8nHealthy: false,
					error: `n8n is not accessible: ${error}`
				}));
				return false;
			}
		},

		// Clear error
		clearError() {
			update(state => ({ ...state, error: null }));
		},

		// Reset store
		reset() {
			set(initialState);
		}
	};
}

export const automationStore = createAutomationStore();
