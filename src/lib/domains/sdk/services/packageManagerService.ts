/**
 * Package Manager Service
 * 
 * TypeScript service for interacting with package managers via Tauri commands
 */

import { invoke } from '@tauri-apps/api/core';

export interface Package {
	id: string;
	name: string;
	version?: string;
	publisher?: string;
	description?: string;
	homepage?: string;
	license?: string;
	source: string;
}

export interface InstalledPackage {
	id: string;
	name: string;
	version: string;
	installed_version?: string;
	available_version?: string;
	source: string;
}

export interface PackageDetails {
	id: string;
	name: string;
	version?: string;
	publisher?: string;
	description?: string;
	homepage?: string;
	license?: string;
	dependencies: string[];
	source: string;
}

export interface PackageUpdate {
	id: string;
	name: string;
	current_version: string;
	available_version: string;
	source: string;
}

export interface PackageManagerInfo {
	name: string;
	display_name: string;
	platform: string;
	available: boolean;
	version: string;
	supports_search: boolean;
	supports_updates: boolean;
	requires_elevation: boolean;
}

/**
 * Get list of available package managers on the system
 */
export async function getAvailablePackageManagers(): Promise<string[]> {
	try {
		return await invoke<string[]>('get_available_package_managers');
	} catch (error) {
		console.error('Failed to get available package managers:', error);
		throw error;
	}
}

/**
 * Search for packages using a specific package manager
 */
export async function searchPackages(
	managerName: string,
	query: string
): Promise<Package[]> {
	try {
		return await invoke<Package[]>('package_manager_search', {
			managerName,
			query,
		});
	} catch (error) {
		console.error('Failed to search packages:', error);
		throw error;
	}
}

/**
 * List installed packages from a specific package manager
 */
export async function listInstalledPackages(
	managerName: string
): Promise<InstalledPackage[]> {
	try {
		return await invoke<InstalledPackage[]>('package_manager_list_installed', {
			managerName,
		});
	} catch (error) {
		console.error('Failed to list installed packages:', error);
		throw error;
	}
}

/**
 * Get detailed information about a package
 */
export async function getPackageDetails(
	managerName: string,
	packageId: string
): Promise<PackageDetails> {
	try {
		return await invoke<PackageDetails>('package_manager_get_details', {
			managerName,
			packageId,
		});
	} catch (error) {
		console.error('Failed to get package details:', error);
		throw error;
	}
}

/**
 * Install a package using a specific package manager
 */
export async function installPackage(
	managerName: string,
	packageId: string,
	version?: string
): Promise<string> {
	try {
		return await invoke<string>('package_manager_install', {
			managerName,
			packageId,
			version,
		});
	} catch (error) {
		console.error('Failed to install package:', error);
		throw error;
	}
}

/**
 * Upgrade a package using a specific package manager
 */
export async function upgradePackage(
	managerName: string,
	packageId: string
): Promise<string> {
	try {
		return await invoke<string>('package_manager_upgrade', {
			managerName,
			packageId,
		});
	} catch (error) {
		console.error('Failed to upgrade package:', error);
		throw error;
	}
}

/**
 * Uninstall a package using a specific package manager
 */
export async function uninstallPackage(
	managerName: string,
	packageId: string
): Promise<string> {
	try {
		return await invoke<string>('package_manager_uninstall', {
			managerName,
			packageId,
		});
	} catch (error) {
		console.error('Failed to uninstall package:', error);
		throw error;
	}
}

/**
 * Check for available updates using a specific package manager
 */
export async function checkUpdates(
	managerName: string
): Promise<PackageUpdate[]> {
	try {
		return await invoke<PackageUpdate[]>('package_manager_check_updates', {
			managerName,
		});
	} catch (error) {
		console.error('Failed to check updates:', error);
		throw error;
	}
}

/**
 * Get information about a package manager
 */
export async function getPackageManagerInfo(
	managerName: string
): Promise<PackageManagerInfo> {
	try {
		return await invoke<PackageManagerInfo>('package_manager_info', {
			managerName,
		});
	} catch (error) {
		console.error('Failed to get package manager info:', error);
		throw error;
	}
}

