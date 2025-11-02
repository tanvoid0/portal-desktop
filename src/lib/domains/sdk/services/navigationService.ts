/**
 * SDK Navigation Service
 * 
 * Frontend service for managing SDK navigation items
 */

import { invoke } from '@tauri-apps/api/core';

export interface NavigationItem {
	id: string;
	title: string;
	description: string;
	icon: string;
	category: string;
	url: string;
	badge: number | null;
	installed: boolean;
	version: string | null;
	latest_version: string | null;
	manager_type: string | null;
}

export interface NavigationSection {
	title: string;
	items: NavigationItem[];
}

export interface NavigationResponse {
	sections: NavigationSection[];
	total_installed: number;
	total_available: number;
}

export class SDKNavigationService {
	/**
	 * Get SDK navigation items from backend
	 */
	async getNavigationItems(): Promise<NavigationResponse> {
		try {
			const response = await invoke<NavigationResponse>('get_sdk_navigation_items');
			return response;
		} catch (error) {
			console.error('Failed to fetch SDK navigation items:', error);
			throw new Error('Failed to load SDK navigation items');
		}
	}

	/**
	 * Get navigation items grouped by category
	 */
	async getNavigationByCategory(): Promise<Record<string, NavigationItem[]>> {
		const response = await this.getNavigationItems();
		const grouped: Record<string, NavigationItem[]> = {};

		response.sections.forEach(section => {
			grouped[section.title] = section.items;
		});

		return grouped;
	}

	/**
	 * Get only installed SDKs
	 */
	async getInstalledSDKs(): Promise<NavigationItem[]> {
		const response = await this.getNavigationItems();
		return response.sections
			.flatMap(section => section.items)
			.filter(item => item.installed);
	}

	/**
	 * Get SDKs by category
	 */
	async getSDKsByCategory(category: string): Promise<NavigationItem[]> {
		const response = await this.getNavigationItems();
		const section = response.sections.find(s => s.title === category);
		return section ? section.items : [];
	}

	/**
	 * Get navigation statistics
	 */
	async getNavigationStats(): Promise<{
		total_installed: number;
		total_available: number;
		installed_percentage: number;
	}> {
		const response = await this.getNavigationItems();
		const installed_percentage = response.total_available > 0 
			? Math.round((response.total_installed / response.total_available) * 100)
			: 0;

		return {
			total_installed: response.total_installed,
			total_available: response.total_available,
			installed_percentage
		};
	}
}

// Export singleton instance
export const sdkNavigationService = new SDKNavigationService();
