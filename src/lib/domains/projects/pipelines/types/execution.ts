/**
 * Pipeline Execution Types
 */

import type { PipelineExecution, StepExecution, ExecutionStatus } from './index';

export interface ExecutionProgress {
	executionId: string;
	pipelineId: string;
	status: ExecutionStatus;
	currentStep?: string;
	completedSteps: number;
	totalSteps: number;
	progress: number; // 0-100
	startedAt: Date;
	estimatedTimeRemaining?: number; // in seconds
}

export interface ExecutionLog {
	executionId: string;
	stepId?: string;
	timestamp: Date;
	level: 'info' | 'warn' | 'error' | 'debug';
	message: string;
	data?: Record<string, any>;
}

export interface ExecutionMetrics {
	executionId: string;
	totalDuration: number;
	stepsExecuted: number;
	stepsSucceeded: number;
	stepsFailed: number;
	stepsSkipped: number;
	averageStepDuration: number;
	longestStep: {
		stepId: string;
		stepName: string;
		duration: number;
	};
	shortestStep: {
		stepId: string;
		stepName: string;
		duration: number;
	};
}

export interface ExecutionFilter {
	pipelineId?: string;
	projectId?: string;
	status?: ExecutionStatus;
	startedAfter?: Date;
	startedBefore?: Date;
	triggeredBy?: string;
}

export interface ExecutionSummary {
	execution: PipelineExecution;
	progress: ExecutionProgress;
	metrics?: ExecutionMetrics;
}

