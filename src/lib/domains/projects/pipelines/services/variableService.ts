/**
 * Variable Service - Management of pipeline variables and secrets
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '@/lib/domains/shared';
import { credentialService } from '@/lib/domains/credentials/services/credentialService';
import type { PipelineVariable } from '../types';

const log = logger.createScoped('VariableService');

export interface VariableScope {
	projectId?: string;
	pipelineId?: string;
}

export interface SecretReference {
	id: string;
	name: string;
	scope: 'project' | 'pipeline';
}

export class VariableService {
	private static instance: VariableService;

	static getInstance(): VariableService {
		if (!VariableService.instance) {
			VariableService.instance = new VariableService();
		}
		return VariableService.instance;
	}

	/**
	 * Get variables for a project or pipeline
	 */
	async getVariables(scope: VariableScope): Promise<PipelineVariable[]> {
		try {
			log.info('Loading variables', { scope });
			const variables = await invoke<PipelineVariable[]>('get_pipeline_variables', { scope });
			log.info('Variables loaded', { scope, count: variables.length });
			return variables;
		} catch (error) {
			log.error('Failed to load variables', error);
			throw error;
		}
	}

	/**
	 * Set a variable
	 */
	async setVariable(
		scope: VariableScope,
		variable: PipelineVariable
	): Promise<PipelineVariable> {
		try {
			log.info('Setting variable', { scope, name: variable.name });
			const result = await invoke<PipelineVariable>('set_pipeline_variable', {
				scope,
				variable,
			});
			log.info('Variable set', { scope, name: variable.name });
			return result;
		} catch (error) {
			log.error('Failed to set variable', error);
			throw error;
		}
	}

	/**
	 * Delete a variable
	 */
	async deleteVariable(scope: VariableScope, variableName: string): Promise<void> {
		try {
			log.info('Deleting variable', { scope, variableName });
			await invoke('delete_pipeline_variable', { scope, variableName });
			log.info('Variable deleted', { scope, variableName });
		} catch (error) {
			log.error('Failed to delete variable', error);
			throw error;
		}
	}

	/**
	 * Get secrets for a project or pipeline
	 */
	async getSecrets(scope: VariableScope): Promise<SecretReference[]> {
		try {
			log.info('Loading secrets', { scope });
			const secretIds = await invoke<string[]>('get_pipeline_secrets', { scope });
			
			// Fetch secret details from credential vault
			const secrets: SecretReference[] = [];
			for (const secretId of secretIds) {
				try {
					const credential = await credentialService.getCredential(secretId);
					secrets.push({
						id: secretId,
						name: credential.name,
						scope: scope.pipelineId ? 'pipeline' : 'project',
					});
				} catch (error) {
					log.warn('Failed to load secret', { secretId, error });
				}
			}
			
			log.info('Secrets loaded', { scope, count: secrets.length });
			return secrets;
		} catch (error) {
			log.error('Failed to load secrets', error);
			throw error;
		}
	}

	/**
	 * Add a secret reference
	 */
	async addSecret(scope: VariableScope, secretId: string): Promise<void> {
		try {
			log.info('Adding secret', { scope, secretId });
			await invoke('add_pipeline_secret', { scope, secretId });
			log.info('Secret added', { scope, secretId });
		} catch (error) {
			log.error('Failed to add secret', error);
			throw error;
		}
	}

	/**
	 * Remove a secret reference
	 */
	async removeSecret(scope: VariableScope, secretId: string): Promise<void> {
		try {
			log.info('Removing secret', { scope, secretId });
			await invoke('remove_pipeline_secret', { scope, secretId });
			log.info('Secret removed', { scope, secretId });
		} catch (error) {
			log.error('Failed to remove secret', error);
			throw error;
		}
	}

	/**
	 * Resolve variable value (handles both variables and secrets)
	 */
	async resolveVariable(
		scope: VariableScope,
		variableName: string
	): Promise<string | null> {
		try {
			// First check regular variables
			const variables = await this.getVariables(scope);
			const variable = variables.find((v) => v.name === variableName);
			if (variable) {
				return variable.value;
			}

			// Then check if it's a secret reference
			const secrets = await this.getSecrets(scope);
			const secret = secrets.find((s) => s.name === variableName);
			if (secret) {
				try {
					const decryptedValue = await credentialService.decryptCredential(secret.id);
					return decryptedValue || null;
				} catch (error) {
					log.warn('Failed to resolve secret', { secretId: secret.id, error });
					return null;
				}
			}

			return null;
		} catch (error) {
			log.error('Failed to resolve variable', { scope, variableName, error });
			return null;
		}
	}

	/**
	 * Resolve all variables for a scope
	 */
	async resolveAllVariables(scope: VariableScope): Promise<Record<string, string>> {
		try {
			const resolved: Record<string, string> = {};

			// Resolve regular variables
			const variables = await this.getVariables(scope);
			for (const variable of variables) {
				resolved[variable.name] = variable.value;
			}

			// Resolve secrets
			const secrets = await this.getSecrets(scope);
			for (const secret of secrets) {
				try {
					const decryptedValue = await credentialService.decryptCredential(secret.id);
					if (decryptedValue) {
						resolved[secret.name] = decryptedValue;
					}
				} catch (error) {
					log.warn('Failed to resolve secret', { secretId: secret.id, error });
				}
			}

			return resolved;
		} catch (error) {
			log.error('Failed to resolve all variables', { scope, error });
			return {};
		}
	}

	/**
	 * Validate variable name
	 */
	validateVariableName(name: string): { valid: boolean; error?: string } {
		if (!name || name.trim().length === 0) {
			return { valid: false, error: 'Variable name cannot be empty' };
		}

		// Must start with letter or underscore
		if (!/^[a-zA-Z_]/.test(name)) {
			return {
				valid: false,
				error: 'Variable name must start with a letter or underscore',
			};
		}

		// Only alphanumeric, underscore, and dash
		if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
			return {
				valid: false,
				error: 'Variable name can only contain letters, numbers, underscores, and dashes',
			};
		}

		return { valid: true };
	}
}

export const variableService = VariableService.getInstance();

