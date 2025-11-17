/**
 * Credential Service - Frontend business logic for credential management
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '$lib/domains/shared';
import type { 
	Credential, 
	CredentialCreateRequest,
	CredentialUpdateRequest,
	SecureVault,
	VaultCreateRequest,
	VaultUnlockRequest,
	VaultUnlockResult
} from '../types';

export class CredentialService {
	private static instance: CredentialService;

	static getInstance(): CredentialService {
		if (!CredentialService.instance) {
			CredentialService.instance = new CredentialService();
		}
		return CredentialService.instance;
	}

	/**
	 * Create a new credential
	 */
	async createCredential(request: CredentialCreateRequest): Promise<Credential> {
		try {
			logger.info('CredentialService', `Creating credential: ${request.name} (${request.type})`);
			
			const credential = await invoke<Credential>('create_credential', {
				name: request.name,
				credential_type: request.type,
				description: request.description,
				tags: request.tags,
				value: request.value,
				fields: request.fields,
				metadata: request.metadata,
				expires_at: request.expiresAt
			});
			
			logger.info('CredentialService', `Credential created successfully: ${credential.id}`);
			
			return credential;
		} catch (error) {
			logger.error('Failed to create credential', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Get all credentials
	 */
	async getCredentials(): Promise<Credential[]> {
		try {
			logger.info('CredentialService', 'Getting credentials');
			
			const credentials = await invoke<Credential[]>('get_credentials');
			
			logger.info('CredentialService', `Credentials retrieved: ${credentials.length} items`);
			
			return credentials;
		} catch (error) {
			logger.error('Failed to get credentials', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Get credential by ID
	 */
	async getCredential(id: string): Promise<Credential> {
		try {
			logger.info('CredentialService', `Getting credential: ${id}`);
			
			const credential = await invoke<Credential>('get_credential', { id });
			
			logger.info('CredentialService', `Credential retrieved: ${id}`);
			
			return credential;
		} catch (error) {
			logger.error('Failed to get credential', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Update credential
	 */
	async updateCredential(id: string, request: CredentialUpdateRequest): Promise<Credential> {
		try {
			logger.info('CredentialService', `Updating credential: ${id}`);
			
			const credential = await invoke<Credential>('update_credential', {
				id,
				name: request.name,
				description: request.description,
				tags: request.tags,
				value: request.value,
				fields: request.fields,
				metadata: request.metadata,
				status: request.status,
				expires_at: request.expiresAt
			});
			
			logger.info('CredentialService', `Credential updated successfully: ${id}`);
			
			return credential;
		} catch (error) {
			logger.error('Failed to update credential', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Delete credential
	 */
	async deleteCredential(id: string): Promise<void> {
		try {
			logger.info('CredentialService', `Deleting credential: ${id}`);
			
			await invoke('delete_credential', { id });
			
			logger.info('CredentialService', `Credential deleted successfully: ${id}`);
		} catch (error) {
			logger.error('Failed to delete credential', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Decrypt credential value
	 */
	async decryptCredential(id: string): Promise<string> {
		try {
			logger.info('CredentialService', `Decrypting credential: ${id}`);
			
			const decryptedValue = await invoke<string>('decrypt_credential', { id });
			
			logger.info('CredentialService', `Credential decrypted successfully: ${id}`);
			
			return decryptedValue;
		} catch (error) {
			logger.error('Failed to decrypt credential', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Search credentials
	 */
	async searchCredentials(query: string): Promise<Credential[]> {
		try {
			logger.info('CredentialService', `Searching credentials: ${query}`);
			
			const credentials = await invoke<Credential[]>('search_credentials', { query });
			
			logger.info('CredentialService', `Credentials search completed: ${query} (${credentials.length} results)`);
			
			return credentials;
		} catch (error) {
			logger.error('Failed to search credentials', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Create secure vault
	 */
	async createVault(request: VaultCreateRequest): Promise<SecureVault> {
		try {
			logger.info('CredentialService', `Creating secure vault: ${request.name}`);
			
			// This would need to be implemented in the backend
			const vault: SecureVault = {
				id: crypto.randomUUID(),
				name: request.name,
				description: request.description,
				credentials: [],
				createdAt: new Date(),
				updatedAt: new Date(),
				encryptionKey: '', // Would be derived from master password
				keyDerivation: {
					algorithm: 'pbkdf2',
					iterations: 100000,
					salt: crypto.randomUUID(),
					keyLength: 32
				}
			};
			
			logger.info('CredentialService', `Secure vault created: ${vault.id}`);
			
			return vault;
		} catch (error) {
			logger.error('Failed to create secure vault', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Unlock vault
	 */
	async unlockVault(request: VaultUnlockRequest): Promise<VaultUnlockResult> {
		try {
			logger.info('CredentialService', `Unlocking vault: ${request.vaultId}`);
			
			// This would need to be implemented in the backend
			const result: VaultUnlockResult = {
				success: true,
				sessionToken: crypto.randomUUID(),
				expiresAt: new Date(Date.now() + (request.sessionDuration || 30) * 60 * 1000)
			};
			
			logger.info('CredentialService', `Vault unlocked successfully: ${request.vaultId}`);
			
			return result;
		} catch (error) {
			logger.error('Failed to unlock vault', { context: 'CredentialService', error });
			throw error;
		}
	}

	/**
	 * Get credential type icon
	 */
	getCredentialTypeIcon(type: string): string {
		const icons: Record<string, string> = {
			ssh_key: '🔑',
			api_token: '🎫',
			env_var: '🌍',
			database: '🗄️',
			cloud_provider: '☁️',
			registry: '📦',
			other: '🔐'
		};
		return icons[type] || '🔐';
	}

	/**
	 * Get credential type color
	 */
	getCredentialTypeColor(type: string): string {
		const colors: Record<string, string> = {
			ssh_key: 'text-orange-500',
			api_token: 'text-blue-500',
			env_var: 'text-green-500',
			database: 'text-purple-500',
			cloud_provider: 'text-cyan-500',
			registry: 'text-pink-500',
			other: 'text-gray-500'
		};
		return colors[type] || 'text-gray-500';
	}

	/**
	 * Mask sensitive value for display
	 */
	maskValue(value: string, type: string): string {
		if (type === 'ssh_key') {
			// Show first and last few characters for SSH keys
			if (value.length > 20) {
				return `${value.substring(0, 10)}...${value.substring(value.length - 10)}`;
			}
			return value;
		}
		
		// For other types, show first few characters only
		if (value.length > 8) {
			return `${value.substring(0, 4)}****`;
		}
		return '****';
	}

	/**
	 * Validate credential data
	 */
	validateCredential(data: Partial<CredentialCreateRequest>): string[] {
		const errors: string[] = [];

		if (!data.name?.trim()) {
			errors.push('Name is required');
		}

		if (!data.type) {
			errors.push('Type is required');
		}

		if (!data.value?.trim()) {
			errors.push('Value is required');
		}

		if (data.expiresAt && data.expiresAt < new Date()) {
			errors.push('Expiration date must be in the future');
		}

		return errors;
	}
}

export const credentialService = CredentialService.getInstance();
