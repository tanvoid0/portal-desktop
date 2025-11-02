/**
 * Settings Service - Frontend business logic for settings management
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '$lib/domains/shared';
import type { 
	AppSettings, 
	EditorSettings, 
	TerminalSettings, 
	ThemeSettings
} from '../types';

// Define missing types
export interface Settings {
	id: string;
	app: AppSettings;
	editor: EditorSettings;
	terminal: TerminalSettings;
	theme: ThemeSettings;
}

export interface SettingsUpdate {
	app?: Partial<AppSettings>;
	editor?: Partial<EditorSettings>;
	terminal?: Partial<TerminalSettings>;
	theme?: Partial<ThemeSettings>;
}

export class SettingsService {
	private static instance: SettingsService;

	static getInstance(): SettingsService {
		if (!SettingsService.instance) {
			SettingsService.instance = new SettingsService();
		}
		return SettingsService.instance;
	}

	/**
	 * Get current settings
	 */
	async getSettings(): Promise<Settings> {
		try {
			logger.info('Getting settings', { context: 'SettingsService' });
			
			const settings = await invoke<Settings>('get_settings_command');
			
			logger.info('Settings retrieved successfully', { 
				context: 'SettingsService', 
				data: { settingsId: settings.id } 
			});
			
			return settings;
		} catch (error) {
			logger.error('Failed to get settings', {
				context: 'SettingsService',
				error
			});
			throw error;
		}
	}

	/**
	 * Save settings
	 */
	async saveSettings(settings: Settings): Promise<void> {
		try {
			logger.info('Saving settings', { 
				context: 'SettingsService', 
				data: { settingsId: settings.id } 
			});
			
			await invoke('save_settings_command', { settings });
			
			logger.info('Settings saved successfully', { 
				context: 'SettingsService', 
				data: { settingsId: settings.id } 
			});
		} catch (error) {
			logger.error('Failed to save settings', {
				context: 'SettingsService',
				error,
				data: { settingsId: settings.id }
			});
			throw error;
		}
	}

	/**
	 * Update settings
	 */
	async updateSettings(currentSettings: Settings, updates: SettingsUpdate): Promise<Settings> {
		try {
			logger.info('Updating settings', { 
				context: 'SettingsService', 
				data: { settingsId: currentSettings.id, updates } 
			});
			
			const updatedSettings = await invoke<Settings>('update_settings_command', {
				settings: currentSettings,
				updates
			});
			
			logger.info('Settings updated successfully', { 
				context: 'SettingsService', 
				data: { settingsId: updatedSettings.id } 
			});
			
			return updatedSettings;
		} catch (error) {
			logger.error('Failed to update settings', {
				context: 'SettingsService',
				error,
				data: { settingsId: currentSettings.id }
			});
			throw error;
		}
	}

	/**
	 * Reset settings to defaults
	 */
	async resetSettings(): Promise<Settings> {
		try {
			logger.info('Resetting settings', { context: 'SettingsService' });
			
			const defaultSettings = await invoke<Settings>('reset_settings_command');
			
			logger.info('Settings reset successfully', { 
				context: 'SettingsService', 
				data: { settingsId: defaultSettings.id } 
			});
			
			return defaultSettings;
		} catch (error) {
			logger.error('Failed to reset settings', {
				context: 'SettingsService',
				error
			});
			throw error;
		}
	}

	/**
	 * Export settings
	 */
	async exportSettings(settings: Settings): Promise<string> {
		try {
			logger.info('Exporting settings', { 
				context: 'SettingsService', 
				data: { settingsId: settings.id } 
			});
			
			const exportedSettings = await invoke<string>('export_settings_command', { settings });
			
			logger.info('Settings exported successfully', { 
				context: 'SettingsService', 
				data: { settingsId: settings.id } 
			});
			
			return exportedSettings;
		} catch (error) {
			logger.error('Failed to export settings', {
				context: 'SettingsService',
				error,
				data: { settingsId: settings.id }
			});
			throw error;
		}
	}

	/**
	 * Import settings
	 */
	async importSettings(settingsJson: string): Promise<Settings> {
		try {
			logger.info('Importing settings', { context: 'SettingsService' });
			
			const importedSettings = await invoke<Settings>('import_settings_command', { 
				settingsJson 
			});
			
			logger.info('Settings imported successfully', { 
				context: 'SettingsService', 
				data: { settingsId: importedSettings.id } 
			});
			
			return importedSettings;
		} catch (error) {
			logger.error('Failed to import settings', {
				context: 'SettingsService',
				error
			});
			throw error;
		}
	}

	/**
	 * Validate settings
	 */
	validateSettings(settings: Partial<Settings>): string[] {
		const errors: string[] = [];

		// Validate app settings
		if (settings.app) {
			if (!settings.app.theme || !['light', 'dark', 'system'].includes(settings.app.theme)) {
				errors.push('Invalid theme value');
			}
			if (!settings.app.language || settings.app.language.length !== 2) {
				errors.push('Invalid language code');
			}
			if (settings.app.windowState && settings.app.windowState.width < 400) {
				errors.push('Window width must be at least 400px');
			}
			if (settings.app.windowState && settings.app.windowState.height < 300) {
				errors.push('Window height must be at least 300px');
			}
		}

		// Validate editor settings
		if (settings.editor) {
			if (settings.editor.fontSize && (settings.editor.fontSize < 8 || settings.editor.fontSize > 72)) {
				errors.push('Font size must be between 8 and 72');
			}
			if (settings.editor.tabSize && (settings.editor.tabSize < 1 || settings.editor.tabSize > 8)) {
				errors.push('Tab size must be between 1 and 8');
			}
		}

		// Validate terminal settings
		if (settings.terminal) {
			if (settings.terminal.fontSize && (settings.terminal.fontSize < 8 || settings.terminal.fontSize > 72)) {
				errors.push('Terminal font size must be between 8 and 72');
			}
			if (settings.terminal.scrollback && (settings.terminal.scrollback < 100 || settings.terminal.scrollback > 10000)) {
				errors.push('Terminal scrollback must be between 100 and 10000');
			}
		}

		return errors;
	}

	/**
	 * Get theme colors
	 */
	getThemeColors(theme: string): Record<string, string> {
		const themes: Record<string, Record<string, string>> = {
			light: {
				primary: '#3b82f6',
				secondary: '#64748b',
				accent: '#f59e0b',
				background: '#ffffff',
				surface: '#f8fafc',
				text: '#1e293b',
				border: '#e2e8f0'
			},
			dark: {
				primary: '#3b82f6',
				secondary: '#64748b',
				accent: '#f59e0b',
				background: '#0f172a',
				surface: '#1e293b',
				text: '#f1f5f9',
				border: '#334155'
			},
			system: {
				primary: '#3b82f6',
				secondary: '#64748b',
				accent: '#f59e0b',
				background: 'var(--background)',
				surface: 'var(--surface)',
				text: 'var(--text)',
				border: 'var(--border)'
			}
		};

		return themes[theme] || themes.system;
	}

	/**
	 * Get available themes
	 */
	getAvailableThemes(): Array<{ id: string; name: string; description: string }> {
		return [
			{ id: 'light', name: 'Light', description: 'Light theme with bright colors' },
			{ id: 'dark', name: 'Dark', description: 'Dark theme with dark colors' },
			{ id: 'system', name: 'System', description: 'Follow system theme preference' }
		];
	}

	/**
	 * Get available languages
	 */
	getAvailableLanguages(): Array<{ id: string; name: string; native: string }> {
		return [
			{ id: 'en', name: 'English', native: 'English' },
			{ id: 'es', name: 'Spanish', native: 'Español' },
			{ id: 'fr', name: 'French', native: 'Français' },
			{ id: 'de', name: 'German', native: 'Deutsch' },
			{ id: 'it', name: 'Italian', native: 'Italiano' },
			{ id: 'pt', name: 'Portuguese', native: 'Português' },
			{ id: 'ru', name: 'Russian', native: 'Русский' },
			{ id: 'ja', name: 'Japanese', native: '日本語' },
			{ id: 'ko', name: 'Korean', native: '한국어' },
			{ id: 'zh', name: 'Chinese', native: '中文' }
		];
	}

	/**
	 * Get available fonts
	 */
	getAvailableFonts(): Array<{ id: string; name: string; category: string }> {
		return [
			{ id: 'monaco', name: 'Monaco', category: 'Monospace' },
			{ id: 'consolas', name: 'Consolas', category: 'Monospace' },
			{ id: 'courier-new', name: 'Courier New', category: 'Monospace' },
			{ id: 'fira-code', name: 'Fira Code', category: 'Monospace' },
			{ id: 'jetbrains-mono', name: 'JetBrains Mono', category: 'Monospace' },
			{ id: 'source-code-pro', name: 'Source Code Pro', category: 'Monospace' },
			{ id: 'roboto-mono', name: 'Roboto Mono', category: 'Monospace' },
			{ id: 'ubuntu-mono', name: 'Ubuntu Mono', category: 'Monospace' }
		];
	}

	/**
	 * Get available terminal themes
	 */
	getAvailableTerminalThemes(): Array<{ id: string; name: string; colors: Record<string, string> }> {
		return [
			{
				id: 'default',
				name: 'Default',
				colors: {
					background: '#1e1e1e',
					foreground: '#d4d4d4',
					cursor: '#ffffff',
					selection: '#264f78'
				}
			},
			{
				id: 'solarized-dark',
				name: 'Solarized Dark',
				colors: {
					background: '#002b36',
					foreground: '#839496',
					cursor: '#93a1a1',
					selection: '#073642'
				}
			},
			{
				id: 'solarized-light',
				name: 'Solarized Light',
				colors: {
					background: '#fdf6e3',
					foreground: '#586e75',
					cursor: '#93a1a1',
					selection: '#eee8d5'
				}
			},
			{
				id: 'monokai',
				name: 'Monokai',
				colors: {
					background: '#272822',
					foreground: '#f8f8f2',
					cursor: '#f8f8f0',
					selection: '#49483e'
				}
			}
		];
	}
}

export const settingsService = SettingsService.getInstance();
