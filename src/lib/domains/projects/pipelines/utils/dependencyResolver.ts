/**
 * Dependency Resolver - Resolve step dependencies and execution order
 */

import type { PipelineStep } from '../types';
import { logger } from '@/lib/domains/shared';

const log = logger.createScoped('DependencyResolver');

export interface ExecutionGroup {
	steps: PipelineStep[];
	canRunInParallel: boolean;
}

/**
 * Resolve step dependencies and create execution groups
 */
export function resolveDependencies(steps: PipelineStep[]): ExecutionGroup[] {
	const groups: ExecutionGroup[] = [];
	const executed = new Set<string>();
	const stepMap = new Map<string, PipelineStep>();

	// Create step map for quick lookup
	for (const step of steps) {
		stepMap.set(step.id, step);
	}

	// Build dependency graph
	const dependencies = new Map<string, Set<string>>();
	for (const step of steps) {
		dependencies.set(step.id, new Set(step.dependsOn || []));
	}

	// Resolve execution order using topological sort
	while (executed.size < steps.length) {
		const readySteps: PipelineStep[] = [];

		for (const step of steps) {
			if (executed.has(step.id)) {
				continue;
			}

			// Check if all dependencies are satisfied
			const deps = dependencies.get(step.id) || new Set();
			const allDepsSatisfied = Array.from(deps).every((depId) => executed.has(depId));

			if (allDepsSatisfied) {
				readySteps.push(step);
			}
		}

		if (readySteps.length === 0) {
			// Circular dependency or missing dependency
			const remaining = steps.filter((s) => !executed.has(s.id));
			log.error('Circular or missing dependencies detected', {
				remaining: remaining.map((s) => s.id),
			});
			break;
		}

		// Group steps that can run in parallel
		const parallelSteps = readySteps.filter((s) => s.parallel !== false);
		const sequentialSteps = readySteps.filter((s) => s.parallel === false);

		// Add parallel group if any
		if (parallelSteps.length > 0) {
			groups.push({
				steps: parallelSteps,
				canRunInParallel: true,
			});
		}

		// Add sequential steps
		for (const step of sequentialSteps) {
			groups.push({
				steps: [step],
				canRunInParallel: false,
			});
		}

		// Mark all ready steps as executed
		for (const step of readySteps) {
			executed.add(step.id);
		}
	}

	return groups;
}

/**
 * Validate pipeline dependencies
 */
export function validateDependencies(steps: PipelineStep[]): {
	valid: boolean;
	errors: string[];
} {
	const errors: string[] = [];
	const stepIds = new Set(steps.map((s) => s.id));

	// Check for duplicate step IDs
	const duplicates = steps.filter(
		(s, index) => steps.findIndex((s2) => s2.id === s.id) !== index
	);
	if (duplicates.length > 0) {
		errors.push(`Duplicate step IDs: ${duplicates.map((s) => s.id).join(', ')}`);
	}

	// Check for missing dependencies
	for (const step of steps) {
		for (const depId of step.dependsOn || []) {
			if (!stepIds.has(depId)) {
				errors.push(`Step ${step.id} depends on missing step: ${depId}`);
			}
		}
	}

	// Check for circular dependencies (simple check)
	const visited = new Set<string>();
	const recStack = new Set<string>();

	function hasCycle(stepId: string): boolean {
		if (recStack.has(stepId)) {
			return true;
		}
		if (visited.has(stepId)) {
			return false;
		}

		visited.add(stepId);
		recStack.add(stepId);

		const step = steps.find((s) => s.id === stepId);
		if (step) {
			for (const depId of step.dependsOn || []) {
				if (hasCycle(depId)) {
					return true;
				}
			}
		}

		recStack.delete(stepId);
		return false;
	}

	for (const step of steps) {
		if (!visited.has(step.id) && hasCycle(step.id)) {
			errors.push(`Circular dependency detected involving step: ${step.id}`);
			break;
		}
	}

	return {
		valid: errors.length === 0,
		errors,
	};
}

