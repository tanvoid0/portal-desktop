/**
 * Custom Script Service
 * Frontend service for managing custom scripts
 */

import { invoke } from '@tauri-apps/api/core';

export interface ScriptParameter {
	name: string;
	label: string;
	parameter_type: 'file' | 'folder' | 'string' | 'number' | 'boolean' | 'password';
	required: boolean;
	default_value?: string;
	description?: string;
	file_filters?: string[]; // For file type: e.g., ["*.ovpn", "*.txt"]
}

export interface CustomScript {
	id: number;
	name: string;
	description?: string;
	command: string;
	parameters_json: string;
	category?: string;
	icon?: string;
	requires_sudo: boolean;
	is_interactive: boolean;
	created_at?: string;
	updated_at?: string;
	last_run_at?: string;
	run_count: number;
}

export interface CreateScriptRequest {
	name: string;
	description?: string;
	command: string;
	parameters: ScriptParameter[];
	category?: string;
	icon?: string;
	requires_sudo?: boolean;
	is_interactive?: boolean;
}

export interface UpdateScriptRequest {
	name?: string;
	description?: string;
	command?: string;
	parameters?: ScriptParameter[];
	category?: string;
	icon?: string;
	requires_sudo?: boolean;
	is_interactive?: boolean;
}

export class CustomScriptService {
	static async getAllScripts(): Promise<CustomScript[]> {
		try {
			return await invoke<CustomScript[]>('get_all_custom_scripts');
		} catch (error) {
			console.error('Failed to get all scripts:', error);
			throw error;
		}
	}

	static async getScript(id: number): Promise<CustomScript | null> {
		try {
			return await invoke<CustomScript | null>('get_custom_script', { id });
		} catch (error) {
			console.error('Failed to get script:', error);
			throw error;
		}
	}

	static async createScript(request: CreateScriptRequest): Promise<CustomScript> {
		try {
			const parametersJson = JSON.stringify(request.parameters || []);
			return await invoke<CustomScript>('create_custom_script', {
				name: request.name,
				description: request.description,
				command: request.command,
				parametersJson: parametersJson, // Tauri v2 expects camelCase
				category: request.category,
				icon: request.icon,
				requiresSudo: request.requires_sudo ?? false, // Tauri v2 expects camelCase
				isInteractive: request.is_interactive ?? false, // Tauri v2 expects camelCase
			});
		} catch (error) {
			console.error('Failed to create script:', error);
			throw error;
		}
	}

	static async updateScript(id: number, request: UpdateScriptRequest): Promise<CustomScript> {
		try {
			const parametersJson = request.parameters ? JSON.stringify(request.parameters) : undefined;
			return await invoke<CustomScript>('update_custom_script', {
				id,
				name: request.name,
				description: request.description,
				command: request.command,
				parametersJson: parametersJson, // Tauri v2 expects camelCase
				category: request.category,
				icon: request.icon,
				requiresSudo: request.requires_sudo,
				isInteractive: request.is_interactive,
			});
		} catch (error) {
			console.error('Failed to update script:', error);
			throw error;
		}
	}

	static async deleteScript(id: number): Promise<void> {
		try {
			await invoke('delete_custom_script', { id });
		} catch (error) {
			console.error('Failed to delete script:', error);
			throw error;
		}
	}

	static async recordScriptRun(id: number): Promise<CustomScript> {
		try {
			return await invoke<CustomScript>('record_script_run', { id });
		} catch (error) {
			console.error('Failed to record script run:', error);
			throw error;
		}
	}

	static parseParameters(parametersJson: string): ScriptParameter[] {
		try {
			return JSON.parse(parametersJson);
		} catch (error) {
			console.error('Failed to parse parameters:', error);
			return [];
		}
	}

	/**
	 * Sanitize parameter values to prevent command injection
	 * Removes shell metacharacters and control characters
	 */
	private static sanitizeCommandValue(value: string): string {
		if (!value) return '';
		
		// Remove shell metacharacters that could be used for injection
		// This includes: ; & | ` $ ( ) { } [ ] < > \n \r
		return value
			.replace(/[;&|`$(){}[\]<>]/g, '')
			.replace(/\n/g, '')
			.replace(/\r/g, '')
			.replace(/\0/g, '') // Null bytes
			.trim();
	}

	static buildCommand(
		commandTemplate: string,
		parameters: ScriptParameter[],
		values: Record<string, string>,
		requiresSudo: boolean = false
	): string {
		let command = commandTemplate;
		
		// Replace placeholders like $VPN_DIR, ${param_name}, etc.
		for (const param of parameters) {
			const value = values[param.name];
			if (value) {
				// Sanitize value to prevent command injection
				const sanitizedValue = CustomScriptService.sanitizeCommandValue(value);
				
				// Replace ${param_name} or $param_name
				command = command.replace(new RegExp(`\\$\\{${param.name}\\}`, 'g'), sanitizedValue);
				command = command.replace(new RegExp(`\\$${param.name}\\b`, 'g'), sanitizedValue);
			}
		}
		
		// If requires_sudo is true, prepend appropriate privilege escalation command
		if (requiresSudo) {
			const isWindows = navigator.userAgent.includes('Windows');
			if (isWindows) {
				// Windows: Note that admin elevation typically requires UAC prompt
				// For terminal execution, we'll prepend a note and let the user handle elevation
				// or use runas command if available
				// For now, we'll use a PowerShell approach that works in terminal
				// The user will need to run the terminal as administrator for this to work
				command = command; // Keep command as-is, but note that terminal needs admin privileges
				// Alternative: Use runas if available (requires password input)
				// command = `runas /user:Administrator "${command}"`;
			} else {
				// Linux/Mac: Use sudo
				command = `sudo ${command}`;
			}
		}
		
		return command;
	}
}

