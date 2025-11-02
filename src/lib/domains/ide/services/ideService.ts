/**
 * IDE Service - Frontend service for IDE configuration
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '$lib/domains/shared';

export interface IdeConfig {
	id?: number;
	name: string;
	executable: string;
	is_default?: boolean;
	created_at?: string;
	updated_at?: string;
}

export interface FrameworkIdeMapping {
	id?: number;
	framework: string;
	ide_id: number;
	created_at?: string;
	updated_at?: string;
}

export interface SuggestedFramework {
	name: string;
	icon: string;
	category: string;
}

export interface FrameworkGroup {
	category: string;
	frameworks: SuggestedFramework[];
}

export interface Framework {
	id: number;
	name: string;
	icon: string;
	icon_type: 'devicon' | 'file';
	category: string;
	created_at?: string;
	updated_at?: string;
}

export class IdeService {
	private static instance: IdeService;
	private log = logger.createScoped('IdeService');

	private constructor() {}

	static getInstance(): IdeService {
		if (!IdeService.instance) {
			IdeService.instance = new IdeService();
		}
		return IdeService.instance;
	}

	/**
	 * Get all configured IDEs
	 */
	async getAllIdes(): Promise<IdeConfig[]> {
		try {
			this.log.info('Getting all IDEs');
			const ides = await invoke<IdeConfig[]>('get_all_ides');
			this.log.info('IDEs retrieved', { count: ides.length });
			return ides;
		} catch (error) {
			this.log.error('Failed to get IDEs', error);
			throw error;
		}
	}

	/**
	 * Add a new IDE
	 */
	async addIde(name: string, executable: string): Promise<number> {
		try {
			this.log.info('Adding IDE', { name, executable });
			const id = await invoke<number>('add_ide', { name, executable });
			this.log.info('IDE added successfully', { id });
			return id;
		} catch (error) {
			this.log.error('Failed to add IDE', error);
			throw error;
		}
	}

	/**
	 * Update an existing IDE
	 */
	async updateIde(id: number, name: string, executable: string): Promise<number> {
		try {
			this.log.info('Updating IDE', { id, name, executable });
			const result = await invoke<number>('update_ide', { id, name, executable });
			this.log.info('IDE updated successfully', { id });
			return result;
		} catch (error) {
			this.log.error('Failed to update IDE', error);
			throw error;
		}
	}

	/**
	 * Delete an IDE
	 */
	async deleteIde(id: number): Promise<number> {
		try {
			this.log.info('Deleting IDE', { id });
			const result = await invoke<number>('delete_ide', { id });
			this.log.info('IDE deleted successfully', { id });
			return result;
		} catch (error) {
			this.log.error('Failed to delete IDE', error);
			throw error;
		}
	}

	/**
	 * Set default IDE
	 */
	async setDefaultIde(id: number): Promise<number> {
		try {
			this.log.info('Setting default IDE', { id });
			const result = await invoke<number>('set_default_ide', { id });
			this.log.info('Default IDE set successfully', { id });
			return result;
		} catch (error) {
			this.log.error('Failed to set default IDE', error);
			throw error;
		}
	}

	/**
	 * Get default IDE
	 */
	async getDefaultIde(): Promise<IdeConfig | null> {
		try {
			this.log.info('Getting default IDE');
			const ide = await invoke<IdeConfig | null>('get_default_ide');
			return ide;
		} catch (error) {
			this.log.error('Failed to get default IDE', error);
			throw error;
		}
	}

	/**
	 * Detect installed IDEs on the system
	 */
	async detectInstalledIdes(): Promise<string[]> {
		try {
			this.log.info('Detecting installed IDEs');
			const ides = await invoke<string[]>('detect_installed_ides');
			this.log.info('IDEs detected', { count: ides.length });
			return ides;
		} catch (error) {
			this.log.error('Failed to detect IDEs', error);
			throw error;
		}
	}

	/**
	 * Get all framework IDE mappings
	 */
	async getAllFrameworkIdeMappings(): Promise<FrameworkIdeMapping[]> {
		try {
			this.log.info('Getting all framework IDE mappings');
			const mappings = await invoke<FrameworkIdeMapping[]>('get_all_framework_ide_mappings');
			this.log.info('Framework IDE mappings retrieved', { count: mappings.length });
			return mappings;
		} catch (error) {
			this.log.error('Failed to get framework IDE mappings', error);
			throw error;
		}
	}

	/**
	 * Set framework IDE mapping
	 */
	async setFrameworkIdeMapping(framework: string, ideId: number): Promise<number> {
		try {
			this.log.info('Setting framework IDE mapping', { framework, ideId });
			const result = await invoke<number>('set_framework_ide_mapping', { framework, ide_id: ideId });
			this.log.info('Framework IDE mapping set successfully', { framework, ideId });
			return result;
		} catch (error) {
			this.log.error('Failed to set framework IDE mapping', error);
			throw error;
		}
	}

	/**
	 * Get framework IDE mapping
	 */
	async getFrameworkIdeMapping(framework: string): Promise<IdeConfig | null> {
		try {
			this.log.info('Getting framework IDE mapping', { framework });
			const ide = await invoke<IdeConfig | null>('get_framework_ide_mapping', { framework });
			return ide;
		} catch (error) {
			this.log.error('Failed to get framework IDE mapping', error);
			throw error;
		}
	}

	/**
	 * Delete framework IDE mapping
	 */
	async deleteFrameworkIdeMapping(framework: string): Promise<number> {
		try {
			this.log.info('Deleting framework IDE mapping', { framework });
			const result = await invoke<number>('delete_framework_ide_mapping', { framework });
			this.log.info('Framework IDE mapping deleted successfully', { framework });
			return result;
		} catch (error) {
			this.log.error('Failed to delete framework IDE mapping', error);
			throw error;
		}
	}

	/**
	 * Get suggested frameworks from backend
	 */
	async getSuggestedFrameworks(): Promise<FrameworkGroup[]> {
		try {
			this.log.info('Getting suggested frameworks');
			const groups = await invoke<FrameworkGroup[]>('get_suggested_frameworks');
			this.log.info('Suggested frameworks retrieved', { count: groups.length });
			return groups;
		} catch (error) {
			this.log.error('Failed to get suggested frameworks', error);
			throw error;
		}
	}

	/**
	 * Get all user-defined frameworks from database
	 */
	async getAllFrameworks(): Promise<Framework[]> {
		try {
			this.log.info('Getting all frameworks');
			const frameworks = await invoke<Framework[]>('get_all_frameworks');
			this.log.info('Frameworks retrieved', { count: frameworks.length });
			return frameworks;
		} catch (error) {
			this.log.error('Failed to get frameworks', error);
			throw error;
		}
	}

	/**
	 * Create a custom framework
	 */
	async createFramework(
		name: string,
		icon: string,
		iconType: 'devicon' | 'file',
		category: string
	): Promise<Framework> {
		try {
			this.log.info('Creating framework', { name, icon, iconType, category });
			const framework = await invoke<Framework>('create_framework', {
				name,
				icon,
				icon_type: iconType,
				category
			});
			this.log.info('Framework created successfully', { id: framework.id });
			return framework;
		} catch (error) {
			this.log.error('Failed to create framework', error);
			throw error;
		}
	}

	/**
	 * Update a framework
	 */
	async updateFramework(
		id: number,
		name?: string,
		icon?: string,
		iconType?: 'devicon' | 'file',
		category?: string
	): Promise<Framework> {
		try {
			this.log.info('Updating framework', { id, name, icon, iconType, category });
			const framework = await invoke<Framework>('update_framework', {
				id,
				name,
				icon,
				icon_type: iconType,
				category
			});
			this.log.info('Framework updated successfully', { id });
			return framework;
		} catch (error) {
			this.log.error('Failed to update framework', error);
			throw error;
		}
	}

	/**
	 * Delete a framework
	 */
	async deleteFramework(id: number): Promise<void> {
		try {
			this.log.info('Deleting framework', { id });
			await invoke('delete_framework', { id });
			this.log.info('Framework deleted successfully', { id });
		} catch (error) {
			this.log.error('Failed to delete framework', error);
			throw error;
		}
	}
}

export const ideService = IdeService.getInstance();

