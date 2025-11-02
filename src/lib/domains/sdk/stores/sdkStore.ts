/**
 * SDK Manager Store
 */

import { writable, derived } from 'svelte/store';
import { logger } from '$lib/domains/shared';
import type { SDK, SDKManagerInfo, SDKInstallation, SDKDetectionResult } from '../types';

// Core state
export const sdkManagers = writable<SDKManagerInfo[]>([]);
export const installedSDKs = writable<SDK[]>([]);
export const detectionResult = writable<SDKDetectionResult | null>(null);
export const isDetecting = writable<boolean>(false);
export const detectionError = writable<string | null>(null);

// Derived stores
export const installedManagers = derived(
	sdkManagers,
	($managers) => {
		const filtered = $managers.filter(manager => {
			const isInstalled = manager.installed === true || manager.installed === "true";
			logger.info('Filtering manager', { 
				context: 'SDKStore', 
				data: { 
					manager: manager,
					installed: manager.installed,
					installedType: typeof manager.installed,
					isInstalled: isInstalled
				} 
			});
			return isInstalled;
		});
		
		logger.info('Installed managers filtered', { 
			context: 'SDKStore', 
			data: { 
				originalCount: $managers.length,
				filteredCount: filtered.length,
				filtered: filtered
			} 
		});
		
		return filtered;
	}
);

export const availableSDKs = derived(
	installedSDKs,
	($sdks) => $sdks.filter(sdk => sdk.installation.installed)
);

export const sdkByType = derived(
	installedSDKs,
	($sdks) => {
		const grouped: Record<string, SDK[]> = {};
		$sdks.forEach(sdk => {
			if (!grouped[sdk.type]) {
				grouped[sdk.type] = [];
			}
			grouped[sdk.type].push(sdk);
		});
		return grouped;
	}
);

export const activeSDKs = derived(
	installedSDKs,
	($sdks) => $sdks.filter(sdk => sdk.installation.activeVersion)
);

// Store actions
export const sdkActions = {
	/**
	 * Set SDK managers from detection result
	 */
	setManagers(managers: SDKManagerInfo[]): void {
		logger.info('Setting SDK managers', { 
			context: 'SDKStore', 
			data: { count: managers.length } 
		});
		sdkManagers.set(managers);
	},

	/**
	 * Set installed SDKs
	 */
	setSDKs(sdks: SDK[]): void {
		logger.info('Setting installed SDKs', { 
			context: 'SDKStore', 
			data: { count: sdks.length } 
		});
		installedSDKs.set(sdks);
	},

	/**
	 * Set detection result
	 */
	setDetectionResult(result: SDKDetectionResult): void {
		logger.info('Setting SDK detection result', { 
			context: 'SDKStore', 
			data: { 
				managersCount: result.managers.length,
				sdksCount: result.sdks.length,
				errorsCount: result.errors.length,
				managers: result.managers,
				firstManager: result.managers[0]
			} 
		});
		detectionResult.set(result);
		sdkManagers.set(result.managers);
		installedSDKs.set(result.sdks);
		
		// Debug: Log what was actually set
		logger.info('SDK managers set in store', { 
			context: 'SDKStore', 
			data: { 
				managersSet: result.managers,
				managersLength: result.managers.length
			} 
		});
	},

	/**
	 * Set detection state
	 */
	setDetecting(detecting: boolean): void {
		isDetecting.set(detecting);
		if (detecting) {
			detectionError.set(null);
		}
	},

	/**
	 * Set detection error
	 */
	setDetectionError(error: string): void {
		logger.error('SDK detection error', { 
			context: 'SDKStore', 
			error: new Error(error) 
		});
		detectionError.set(error);
		isDetecting.set(false);
	},

	/**
	 * Update SDK installation status
	 */
	updateSDKInstallation(sdkId: string, installation: Partial<SDKInstallation>): void {
		installedSDKs.update(sdks => 
			sdks.map(sdk => 
				sdk.id === sdkId 
					? { ...sdk, installation: { ...sdk.installation, ...installation } }
					: sdk
			)
		);
	},

	/**
	 * Add new SDK
	 */
	addSDK(sdk: SDK): void {
		logger.info('Adding new SDK', { 
			context: 'SDKStore', 
			data: { sdkId: sdk.id, type: sdk.type } 
		});
		installedSDKs.update(sdks => [...sdks, sdk]);
	},

	/**
	 * Remove SDK
	 */
	removeSDK(sdkId: string): void {
		logger.info('Removing SDK', { 
			context: 'SDKStore', 
			data: { sdkId } 
		});
		installedSDKs.update(sdks => sdks.filter(sdk => sdk.id !== sdkId));
	},

	/**
	 * Clear all data
	 */
	clear(): void {
		logger.info('Clearing SDK store', { context: 'SDKStore' });
		sdkManagers.set([]);
		installedSDKs.set([]);
		detectionResult.set(null);
		isDetecting.set(false);
		detectionError.set(null);
	}
};

// Utility functions
export const getSDKByType = (type: string) => {
	return derived(installedSDKs, $sdks => 
		$sdks.find(sdk => sdk.type === type)
	);
};

export const getManagerByType = (type: string) => {
	return derived(sdkManagers, $managers => 
		$managers.find(manager => manager.sdk_type === type || manager.type === type)
	);
};
