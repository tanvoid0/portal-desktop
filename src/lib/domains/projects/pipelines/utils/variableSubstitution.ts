/**
 * Variable Substitution Engine - Replace variables in command templates
 */

import { logger } from '@/lib/domains/shared';

const log = logger.createScoped('VariableSubstitution');

export interface SubstitutionContext {
	variables: Record<string, string>;
	secrets?: Record<string, string>;
	projectVariables?: Record<string, string>;
}

/**
 * Substitute variables in a command template
 * Supports ${variable} and ${variable:default} syntax
 */
export function substituteVariables(
	template: string,
	context: SubstitutionContext
): string {
	let result = template;

	// Merge all variable sources (secrets override variables, project variables are fallback)
	const allVariables: Record<string, string> = {
		...(context.projectVariables || {}),
		...(context.variables || {}),
		...(context.secrets || {}),
	};

	// Match ${variable} or ${variable:default}
	const variablePattern = /\$\{([^}:]+)(?::([^}]*))?\}/g;

	result = result.replace(variablePattern, (match, varName, defaultValue) => {
		const trimmedName = varName.trim();
		const value = allVariables[trimmedName];

		if (value !== undefined) {
			return value;
		}

		if (defaultValue !== undefined) {
			return defaultValue.trim();
		}

		log.warn('Variable not found and no default provided', {
			variable: trimmedName,
			template,
		});
		return match; // Keep original if not found and no default
	});

	return result;
}

/**
 * Extract all variable references from a template
 */
export function extractVariables(template: string): string[] {
	const variables: string[] = [];
	const variablePattern = /\$\{([^}:]+)(?::[^}]*)?\}/g;
	let match;

	while ((match = variablePattern.exec(template)) !== null) {
		const varName = match[1].trim();
		if (!variables.includes(varName)) {
			variables.push(varName);
		}
	}

	return variables;
}

/**
 * Validate that all required variables are provided
 */
export function validateVariables(
	template: string,
	context: SubstitutionContext
): { valid: boolean; missing: string[] } {
	const required = extractVariables(template);
	const allVariables: Record<string, string> = {
		...(context.projectVariables || {}),
		...(context.variables || {}),
		...(context.secrets || {}),
	};

	const missing: string[] = [];

	for (const varName of required) {
		// Check if variable has a default value in template
		const hasDefault = new RegExp(`\\$\\{${varName}:[^}]+\\}`).test(template);
		
		if (!hasDefault && !(varName in allVariables)) {
			missing.push(varName);
		}
	}

	return {
		valid: missing.length === 0,
		missing,
	};
}

/**
 * Sanitize variable value to prevent command injection
 */
export function sanitizeVariable(value: string): string {
	// Remove potentially dangerous characters
	return value
		.replace(/[;&|`$(){}[\]<>]/g, '')
		.replace(/\n/g, ' ')
		.replace(/\r/g, '')
		.trim();
}

/**
 * Escape special characters for shell commands
 */
export function escapeForShell(value: string): string {
	// Escape single quotes by replacing ' with '\''
	return `'${value.replace(/'/g, "'\\''")}'`;
}

