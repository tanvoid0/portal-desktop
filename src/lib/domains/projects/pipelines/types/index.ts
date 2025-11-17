/**
 * Pipeline Automation System Types
 */

export interface Pipeline {
	id: string;
	name: string;
	description?: string;
	projectId: string;
	steps: PipelineStep[];
	variables: PipelineVariable[];
	secrets: string[]; // References to secret IDs from credential vault
	executionContext: ExecutionContext;
	enabled: boolean;
	createdAt: Date;
	updatedAt: Date;
}

export interface PipelineStep {
	id: string;
	blockId: string; // Reference to block library
	name: string;
	config: Record<string, any>; // Block-specific configuration
	dependsOn: string[]; // Step IDs that must complete before this step
	condition?: string; // Conditional execution expression
	retries?: number;
	retryDelay?: number; // Delay in seconds between retries
	timeout?: number; // Timeout in seconds
	parallel?: boolean; // Can run in parallel with other steps
	onSuccess?: string[]; // Step IDs to run on success
	onFailure?: string[]; // Step IDs to run on failure
}

export interface ExecutionContext {
	type: 'sdk' | 'docker';
	sdkType?: string; // node, python, rust, go, etc.
	sdkVersion?: string;
	dockerImage?: string;
	dockerfile?: string;
	dockerContext?: string;
	workingDirectory: string;
	environment?: Record<string, string>; // Additional environment variables
}

export interface PipelineVariable {
	name: string;
	value: string;
	type: 'string' | 'number' | 'boolean';
	description?: string;
	scope: 'project' | 'pipeline'; // Project-level or pipeline-specific
}

export interface Block {
	id: string;
	name: string;
	description: string;
	category: 'build' | 'test' | 'deploy' | 'utility' | 'custom';
	version: string;
	parameters: BlockParameter[];
	command: string; // Template with ${param} placeholders
	executionType: 'command' | 'script' | 'docker';
	defaultConfig: Record<string, any>;
	tags: string[];
	icon?: string;
	author?: string;
	createdAt?: Date;
	updatedAt?: Date;
}

export interface BlockParameter {
	name: string;
	type: 'string' | 'number' | 'boolean' | 'select' | 'file' | 'directory';
	description: string;
	required: boolean;
	defaultValue?: any;
	options?: string[]; // For select type
	validation?: {
		pattern?: string; // Regex pattern
		min?: number;
		max?: number;
		message?: string;
	};
}

export interface PipelineExecution {
	id: string;
	pipelineId: string;
	projectId: string;
	status: ExecutionStatus;
	startedAt: Date;
	finishedAt?: Date;
	triggeredBy: string; // User ID or 'system'
	stepExecutions: StepExecution[];
	variables: Record<string, string>; // Resolved variables at execution time
	error?: string;
}

export type ExecutionStatus = 
	| 'pending'
	| 'running'
	| 'success'
	| 'failed'
	| 'cancelled'
	| 'skipped';

export interface StepExecution {
	id: string;
	stepId: string;
	stepName: string;
	status: ExecutionStatus;
	startedAt: Date;
	finishedAt?: Date;
	output: string;
	error?: string;
	exitCode?: number;
	duration?: number; // Duration in milliseconds
	retryCount: number;
	logs: string[];
}

export interface CreatePipelineRequest {
	name: string;
	description?: string;
	projectId: string;
	steps: Omit<PipelineStep, 'id'>[];
	variables?: PipelineVariable[];
	secrets?: string[];
	executionContext: ExecutionContext;
	enabled?: boolean;
}

export interface UpdatePipelineRequest {
	name?: string;
	description?: string;
	steps?: PipelineStep[];
	variables?: PipelineVariable[];
	secrets?: string[];
	executionContext?: ExecutionContext;
	enabled?: boolean;
}

export interface CreateBlockRequest {
	name: string;
	description: string;
	category: Block['category'];
	parameters: BlockParameter[];
	command: string;
	executionType: Block['executionType'];
	defaultConfig?: Record<string, any>;
	tags?: string[];
}

export interface ExecutePipelineRequest {
	pipelineId: string;
	variables?: Record<string, string>; // Override variables
	secrets?: Record<string, string>; // Override secrets (temporary)
}

export enum PipelineStepType {
	COMMAND = 'command',
	SDK_COMMAND = 'sdk_command',
	DOCKER_COMMAND = 'docker_command',
}

export interface PipelineTemplate {
	key: string; // Unique, machine-readable identifier for hardcoded templates (e.g., 'react-build', 'nextjs-full')
	id?: string; // Database ID if template is persisted (optional)
	name: string; // Human-readable display name (e.g., 'React Build Pipeline')
	description: string;
	framework?: string;
	category?: 'build' | 'test' | 'deploy' | 'ci-cd' | 'full-stack';
	packageManager?: string;
	steps: Array<{
		key: string; // Unique key for this step within the template (e.g., 'install-deps', 'build')
		name: string;
		type: PipelineStepType;
		config: Record<string, any>;
		dependsOn?: string[]; // Array of step keys this step depends on
		enabled?: boolean;
	}>;
	variables?: Array<{
		name: string;
		type: 'string' | 'number' | 'boolean';
		defaultValue?: string | number | boolean;
		description?: string;
	}>;
	executionContext: ExecutionContext;
	tags: string[];
}

