/**
 * SDK Configuration Service
 * 
 * Frontend service for fetching SDK configurations from backend.
 * Backend processes configs and returns formatted data.
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '../../shared';

export interface ProcessedSDKConfig {
	id: string;
	name: string;
	display_name: string;
	description: string;
	icon: string;
	category: string;
	tabs: Array<{ id: string; label: string }>;
	supported_sources: string[];
	default_source: string;
	sdk_managers: Array<{
		id: string;
		name: string;
		display_name: string;
		installed: boolean;
		version: string | null;
		supports_installation: boolean;
		supports_version_switching: boolean;
		install_command: string | null;
		website: string | null;
	}>;
	package_managers: Array<{
		id: string;
		name: string;
		display_name: string;
		installed: boolean;
		version: string | null;
		install_command: string | null;
		website: string | null;
	}>;
	detection: {
		binary_names: string[];
		version_command: string | null;
		path_patterns: string[];
		version_file_patterns: string[];
	};
	category_features: any;
	environment_variables: any;
	service_config: any;
	sdk_installed: boolean;
	sdk_version: string | null;
	service_running: boolean | null;
	service_port: number | null;
}

export class SDKConfigService {
	private static instance: SDKConfigService;

	static getInstance(): SDKConfigService {
		if (!SDKConfigService.instance) {
			SDKConfigService.instance = new SDKConfigService();
		}
		return SDKConfigService.instance;
	}

	/**
	 * Get SDK configuration by ID
	 */
	async getSDKConfig(sdkId: string): Promise<ProcessedSDKConfig | null> {
		try {
			logger.info('Getting SDK config', { context: 'SDKConfigService', data: { sdkId } });
			
			const config = await invoke<ProcessedSDKConfig | null>('get_sdk_config', { sdkId });
			
			logger.info('SDK config retrieved', { 
				context: 'SDKConfigService', 
				data: { sdkId, found: !!config } 
			});
			
			return config;
		} catch (error) {
			logger.error('Failed to get SDK config', {
				context: 'SDKConfigService',
				error,
				data: { sdkId }
			});
			throw error;
		}
	}

	/**
	 * Get all SDK configurations
	 */
	async getAllSDKConfigs(): Promise<ProcessedSDKConfig[]> {
		try {
			logger.info('Getting all SDK configs', { context: 'SDKConfigService' });
			
			const configs = await invoke<ProcessedSDKConfig[]>('get_all_sdk_configs');
			
			logger.info('All SDK configs retrieved', { 
				context: 'SDKConfigService', 
				data: { count: configs.length } 
			});
			
			return configs;
		} catch (error) {
			logger.error('Failed to get all SDK configs', {
				context: 'SDKConfigService',
				error
			});
			throw error;
		}
	}

	/**
	 * Get SDKs by category
	 */
	async getSDKsByCategory(category: string): Promise<ProcessedSDKConfig[]> {
		try {
			logger.info('Getting SDKs by category', { 
				context: 'SDKConfigService', 
				data: { category } 
			});
			
			const configs = await invoke<ProcessedSDKConfig[]>('get_sdks_by_category', { category });
			
			logger.info('SDKs by category retrieved', { 
				context: 'SDKConfigService', 
				data: { category, count: configs.length } 
			});
			
			return configs;
		} catch (error) {
			logger.error('Failed to get SDKs by category', {
				context: 'SDKConfigService',
				error,
				data: { category }
			});
			throw error;
		}
	}
}

export const sdkConfigService = SDKConfigService.getInstance();

