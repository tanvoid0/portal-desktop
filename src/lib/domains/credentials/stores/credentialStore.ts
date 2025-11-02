/**
 * Credentials Store - State management for encrypted credentials
 */

import { writable, derived } from 'svelte/store';
import { logger } from '$lib/domains/shared';
import type { 
	Credential, 
	CredentialType, 
	SecureVault, 
} from '../types';

// Core state
export const credentials = writable<Credential[]>([]);
export const vaults = writable<SecureVault[]>([]);
export const activeVault = writable<SecureVault | null>(null);
export const isVaultUnlocked = writable<boolean>(false);
export const searchQuery = writable<string>('');
export const selectedCredentialType = writable<CredentialType | null>(null);
export const selectedTags = writable<string[]>([]);

// Loading states
export const isLoading = writable<boolean>(false);
export const isDecrypting = writable<boolean>(false);
export const error = writable<string | null>(null);

// Derived stores
export const filteredCredentials = derived(
	[credentials, searchQuery, selectedCredentialType, selectedTags],
	([$credentials, $searchQuery, $selectedType, $selectedTags]) => {
		return $credentials.filter(credential => {
			// Search query filter
			if ($searchQuery && !credential.name.toLowerCase().includes($searchQuery.toLowerCase()) &&
				!credential.description?.toLowerCase().includes($searchQuery.toLowerCase())) {
				return false;
			}

			// Type filter
			if ($selectedType && credential.type !== $selectedType) {
				return false;
			}

			// Tags filter
			if ($selectedTags.length > 0 && !$selectedTags.some(tag => credential.tags.includes(tag))) {
				return false;
			}

			return true;
		});
	}
);

export const credentialsByType = derived(
	credentials,
	($credentials) => {
		const grouped: Partial<Record<CredentialType, Credential[]>> = {};
		$credentials.forEach(credential => {
			if (!grouped[credential.type]) {
				grouped[credential.type] = [];
			}
			grouped[credential.type]!.push(credential);
		});
		return grouped;
	}
);

export const allTags = derived(
	credentials,
	($credentials) => {
		const tagSet = new Set<string>();
		$credentials.forEach(credential => {
			credential.tags.forEach(tag => tagSet.add(tag));
		});
		return Array.from(tagSet).sort();
	}
);

export const credentialStats = derived(
	credentials,
	($credentials) => {
		const stats = {
			total: $credentials.length,
			active: $credentials.filter(c => c.status === 'active').length,
			expired: $credentials.filter(c => c.status === 'expired').length,
			byType: {} as Record<CredentialType, number>
		};

		$credentials.forEach(credential => {
			stats.byType[credential.type] = (stats.byType[credential.type] || 0) + 1;
		});

		return stats;
	}
);

// Store actions
export const credentialActions = {
	/**
	 * Set credentials list
	 */
	setCredentials(credentialList: Credential[]): void {
		logger.info('CredentialStore', `Setting credentials: ${credentialList.length} items`);
		credentials.set(credentialList);
	},

	/**
	 * Add new credential
	 */
	addCredential(credential: Credential): void {
		logger.info('CredentialStore', `Adding credential: ${credential.id} (${credential.type})`);
		credentials.update(list => [...list, credential]);
	},

	/**
	 * Update existing credential
	 */
	updateCredential(credentialId: string, updates: Partial<Credential>): void {
		logger.info('CredentialStore', `Updating credential: ${credentialId}`);
		credentials.update(list => 
			list.map(credential => 
				credential.id === credentialId 
					? { ...credential, ...updates }
					: credential
			)
		);
	},

	/**
	 * Remove credential
	 */
	removeCredential(credentialId: string): void {
		logger.info('CredentialStore', `Removing credential: ${credentialId}`);
		credentials.update(list => list.filter(credential => credential.id !== credentialId));
	},

	/**
	 * Set vaults
	 */
	setVaults(vaultList: SecureVault[]): void {
		logger.info('CredentialStore', `Setting vaults: ${vaultList.length} items`);
		vaults.set(vaultList);
	},

	/**
	 * Set active vault
	 */
	setActiveVault(vault: SecureVault | null): void {
		logger.info('CredentialStore', `Setting active vault: ${vault?.id || 'none'}`);
		activeVault.set(vault);
	},

	/**
	 * Set vault unlock state
	 */
	setVaultUnlocked(unlocked: boolean): void {
		logger.info('CredentialStore', `Setting vault unlock state: ${unlocked ? 'unlocked' : 'locked'}`);
		isVaultUnlocked.set(unlocked);
	},

	/**
	 * Set search query
	 */
	setSearchQuery(query: string): void {
		searchQuery.set(query);
	},

	/**
	 * Set selected credential type
	 */
	setSelectedType(type: CredentialType | null): void {
		selectedCredentialType.set(type);
	},

	/**
	 * Set selected tags
	 */
	setSelectedTags(tags: string[]): void {
		selectedTags.set(tags);
	},

	/**
	 * Set loading state
	 */
	setLoading(loading: boolean): void {
		isLoading.set(loading);
	},

	/**
	 * Set decrypting state
	 */
	setDecrypting(decrypting: boolean): void {
		isDecrypting.set(decrypting);
	},

	/**
	 * Set error
	 */
	setError(errorMessage: string | null): void {
		if (errorMessage) {
			logger.error('Credential store error', { context: 'CredentialStore', error: errorMessage });
		}
		error.set(errorMessage);
	},

	/**
	 * Clear all data
	 */
	clear(): void {
		logger.info('CredentialStore', 'Clearing credential store');
		credentials.set([]);
		vaults.set([]);
		activeVault.set(null);
		isVaultUnlocked.set(false);
		searchQuery.set('');
		selectedCredentialType.set(null);
		selectedTags.set([]);
		isLoading.set(false);
		isDecrypting.set(false);
		error.set(null);
	}
};

// Utility functions
export const getCredentialById = (id: string) => {
	return derived(credentials, $credentials => 
		$credentials.find(credential => credential.id === id)
	);
};

export const getCredentialsByType = (type: CredentialType) => {
	return derived(credentials, $credentials => 
		$credentials.filter(credential => credential.type === type)
	);
};

export const getCredentialsByTag = (tag: string) => {
	return derived(credentials, $credentials => 
		$credentials.filter(credential => credential.tags.includes(tag))
	);
};
