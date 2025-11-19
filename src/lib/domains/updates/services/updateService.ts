import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { invoke } from '@tauri-apps/api/core';
import { logger } from '$lib/domains/shared';

export interface UpdateInfo {
	version: string;
	date?: string;
	body?: string;
	available: boolean;
}

export interface UpdateStatus {
	checking: boolean;
	available: boolean;
	installing: boolean;
	error: string | null;
	info: UpdateInfo | null;
}

/**
 * Check for available updates
 */
export async function checkForUpdates(): Promise<UpdateInfo> {
	try {
		logger.info('Checking for updates...');
		const update = await check();
		
		if (update !== null) {
			logger.info('Update available', { version: update.version });
			
			return {
				version: update.version || 'unknown',
				date: update.date,
				body: update.body,
				available: true
			};
		} else {
			logger.info('No updates available');
			const currentVersion = await getCurrentVersion();
			return {
				version: currentVersion,
				available: false
			};
		}
	} catch (error) {
		logger.error('Failed to check for updates', { error });
		throw error;
	}
}

/**
 * Install the available update
 */
export async function installUpdateAndRelaunch(): Promise<void> {
	try {
		logger.info('Installing update...');
		
		// Check for update first
		const update = await check();
		
		if (update === null) {
			throw new Error('No update available to install');
		}
		
		// Download and install the update
		await update.downloadAndInstall((progress) => {
			// Progress callback
			if (progress.event === 'Started') {
				logger.info('Download started');
			} else if (progress.event === 'Progress') {
				const chunkLength = progress.data?.chunkLength || 0;
				logger.info(`Download progress: ${chunkLength} bytes downloaded`);
			} else if (progress.event === 'Finished') {
				logger.info('Download finished, installing...');
			}
		});
		
		logger.info('Update installed, relaunching...');
		
		// Relaunch the application
		await relaunch();
	} catch (error) {
		logger.error('Failed to install update', { error });
		throw error;
	}
}

/**
 * Get the current application version
 */
export async function getCurrentVersion(): Promise<string> {
	try {
		const version = await invoke<string>('get_app_version_command');
		return version;
	} catch (error) {
		logger.error('Failed to get app version', { error });
		throw error;
	}
}

