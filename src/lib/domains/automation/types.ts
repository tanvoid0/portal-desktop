export interface WorkflowExecution {
	id: string;
	workflow_id: string;
	status: WorkflowStatus;
	started_at: string;
	finished_at?: string;
	data?: Record<string, unknown>;
}

export type WorkflowStatus = 'running' | 'success' | 'error' | 'waiting' | 'canceled';

export interface WorkflowResult {
	success: boolean;
	execution_id: string;
	results: WorkflowResults;
	errors: string[];
	suggestions: string[];
}

export interface WorkflowResults {
	commands_executed: string[];
	output: string;
	duration: number;
	files_created: string[];
}

export interface AvailableWorkflow {
	id: string;
	name: string;
	description: string;
	framework?: string;
	package_manager?: string;
	active: boolean;
}

export interface WorkflowTrigger {
	workflow_id: string;
	project_data: { id: string; name: string; path: string };
}
