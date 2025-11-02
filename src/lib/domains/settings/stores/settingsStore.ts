/**
 * Settings Store - State management for application settings
 */

import { writable, derived } from 'svelte/store';
import { loadingState, loadingActions } from '$lib/domains/shared/stores/loadingState';
import { settingsService } from '../services/settingsService';
import { logger } from '$lib/domains/shared';
import type { 
	AppSettings, 
	EditorSettings, 
	TerminalSettings, 
	ThemeSettings
} from '../types';
import type { Settings, SettingsUpdate } from '../services/settingsService';

// Core stores
export const settings = writable<Settings | null>(null);
export const settingsLoadingState = loadingState;

// Derived stores
export const isLoadingSettings = derived(settingsLoadingState, ($state) => $state.isLoading);
export const settingsError = derived(settingsLoadingState, ($state) => $state.error);

// Individual settings stores for easier access
export const appSettings = derived(settings, ($settings) => $settings?.app || null);
export const editorSettings = derived(settings, ($settings) => $settings?.editor || null);
export const terminalSettings = derived(settings, ($settings) => $settings?.terminal || null);
export const themeSettings = derived(settings, ($settings) => $settings?.theme || null);

// Settings actions
export const settingsActions = {
	/**
	 * Load settings from backend
	 */
	async loadSettings() {
		loadingActions.setLoading(true);
		try {
			logger.info('Loading settings', { context: 'settingsStore' });
			
			const settingsData = await settingsService.getSettings();
			settings.set(settingsData);
			
			logger.info('Settings loaded successfully', { 
				context: 'settingsStore', 
				settingsId: settingsData.id 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to load settings';
			logger.error(errorMessage, { context: 'settingsStore.loadSettings', error });
			loadingActions.setError(errorMessage);
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Save settings to backend
	 */
	async saveSettings(settingsData: Settings) {
		loadingActions.setLoading(true);
		try {
			logger.info('Saving settings', { 
				context: 'settingsStore', 
				settingsId: settingsData.id 
			});
			
			await settingsService.saveSettings(settingsData);
			settings.set(settingsData);
			
			logger.info('Settings saved successfully', { 
				context: 'settingsStore', 
				settingsId: settingsData.id 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to save settings';
			logger.error(errorMessage, { 
				context: 'settingsStore.saveSettings', 
				error,
				settingsId: settingsData.id
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Update specific settings sections
	 */
	async updateSettings(updates: SettingsUpdate) {
		loadingActions.setLoading(true);
		try {
			logger.info('Updating settings', { 
				context: 'settingsStore', 
				updates 
			});
			
			const currentSettings = await settingsService.getSettings();
			const updatedSettings = await settingsService.updateSettings(currentSettings, updates);
			settings.set(updatedSettings);
			
			logger.info('Settings updated successfully', { 
				context: 'settingsStore', 
				settingsId: updatedSettings.id 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to update settings';
			logger.error(errorMessage, { 
				context: 'settingsStore.updateSettings', 
				error,
				updates
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Reset settings to defaults
	 */
	async resetSettings() {
		loadingActions.setLoading(true);
		try {
			logger.info('Resetting settings', { context: 'settingsStore' });
			
			const defaultSettings = await settingsService.resetSettings();
			settings.set(defaultSettings);
			
			logger.info('Settings reset successfully', { 
				context: 'settingsStore', 
				settingsId: defaultSettings.id 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to reset settings';
			logger.error(errorMessage, { 
				context: 'settingsStore.resetSettings', 
				error 
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Export settings
	 */
	async exportSettings(): Promise<string> {
		try {
			logger.info('Exporting settings', { context: 'settingsStore' });
			
			const currentSettings = await settingsService.getSettings();
			const exportedSettings = await settingsService.exportSettings(currentSettings);
			
			logger.info('Settings exported successfully', { 
				context: 'settingsStore', 
				settingsId: currentSettings.id 
			});
			
			return exportedSettings;
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to export settings';
			logger.error(errorMessage, { 
				context: 'settingsStore.exportSettings', 
				error 
			});
			throw error;
		}
	},

	/**
	 * Import settings
	 */
	async importSettings(settingsJson: string) {
		loadingActions.setLoading(true);
		try {
			logger.info('Importing settings', { context: 'settingsStore' });
			
			const importedSettings = await settingsService.importSettings(settingsJson);
			settings.set(importedSettings);
			
			logger.info('Settings imported successfully', { 
				context: 'settingsStore', 
				settingsId: importedSettings.id 
			});
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Failed to import settings';
			logger.error(errorMessage, { 
				context: 'settingsStore.importSettings', 
				error 
			});
			loadingActions.setError(errorMessage);
			throw error;
		} finally {
			loadingActions.setLoading(false);
		}
	},

	/**
	 * Update app settings
	 */
	async updateAppSettings(appSettings: AppSettings) {
		await this.updateSettings({ app: appSettings });
	},

	/**
	 * Update editor settings
	 */
	async updateEditorSettings(editorSettings: EditorSettings) {
		await this.updateSettings({ editor: editorSettings });
	},

	/**
	 * Update terminal settings
	 */
	async updateTerminalSettings(terminalSettings: TerminalSettings) {
		await this.updateSettings({ terminal: terminalSettings });
	},

	/**
	 * Update theme settings
	 */
	async updateThemeSettings(themeSettings: ThemeSettings) {
		await this.updateSettings({ theme: themeSettings });
	},

	/**
	 * Get current settings
	 */
	getCurrentSettings(): Settings | null {
		let currentSettings: Settings | null = null;
		settings.update(settingsData => {
			currentSettings = settingsData;
			return settingsData;
		});
		return currentSettings;
	},

	/**
	 * Clear error state
	 */
	clearError() {
		loadingActions.setError(null);
	}
};
