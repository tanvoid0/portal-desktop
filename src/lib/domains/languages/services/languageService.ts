/**
 * Language Service - Frontend service for language configuration
 */

import { invokeClient } from '$lib/utils/invokeClient';
import { logger } from '$lib/domains/shared';
import type { Language, LanguageGroup, SuggestedLanguage } from '../types';

export class LanguageService {
	private static instance: LanguageService;
	private log = logger.createScoped('LanguageService');

	private constructor() {}

	static getInstance(): LanguageService {
		if (!LanguageService.instance) {
			LanguageService.instance = new LanguageService();
		}
		return LanguageService.instance;
	}

	/**
	 * Get all user-defined languages from database
	 */
	async getAllLanguages(): Promise<Language[]> {
		try {
			this.log.info('Getting all languages');
			const languages = await invokeClient.post<Language[]>('get_all_languages');
			const safeLanguages = languages ?? [];
			this.log.info('Languages retrieved', { count: safeLanguages.length });
			return safeLanguages;
		} catch (error) {
			this.log.error('Failed to get languages', { error });
			throw error;
		}
	}

	/**
	 * Get suggested languages from backend
	 */
	async getSuggestedLanguages(): Promise<LanguageGroup[]> {
		try {
			this.log.info('Getting suggested languages');
			const groups = await invokeClient.post<LanguageGroup[]>('get_suggested_languages');
			const safeGroups = groups ?? [];
			this.log.info('Suggested languages retrieved', { count: safeGroups.length });
			return safeGroups;
		} catch (error) {
			this.log.error('Failed to get suggested languages', { error });
			throw error;
		}
	}

	/**
	 * Create a new language
	 */
	async createLanguage(
		name: string,
		icon: string,
		iconType: 'devicon' | 'file',
		category: string
	): Promise<Language> {
		try {
			this.log.info('Creating language', { name });
			const language = await invokeClient.post<Language>('create_language', {
				name,
				icon,
				iconType, // Tauri v2 converts camelCase to snake_case automatically
				category
			});
			if (!language) {
				throw new Error('Failed to create language: no response');
			}
			this.log.info('Language created successfully', { id: language.id });
			return language;
		} catch (error) {
			this.log.error('Failed to create language', { error });
			throw error;
		}
	}

	/**
	 * Update an existing language
	 */
	async updateLanguage(
		id: number,
		name?: string,
		icon?: string,
		iconType?: 'devicon' | 'file',
		category?: string
	): Promise<Language> {
		try {
			this.log.info('Updating language', { id });
			const language = await invokeClient.post<Language>('update_language', {
				id,
				name,
				icon,
				iconType, // Tauri v2 converts camelCase to snake_case automatically
				category
			});
			if (!language) {
				throw new Error('Failed to update language: no response');
			}
			this.log.info('Language updated successfully', { id });
			return language;
		} catch (error) {
			this.log.error('Failed to update language', { error });
			throw error;
		}
	}

	/**
	 * Create multiple languages in batch
	 */
	async createLanguagesBatch(languages: SuggestedLanguage[]): Promise<{ success: Language[]; failed: { language: SuggestedLanguage; error: string }[] }> {
		const success: Language[] = [];
		const failed: { language: SuggestedLanguage; error: string }[] = [];

		for (const lang of languages) {
			try {
				const created = await this.createLanguage(
					lang.name,
					lang.icon,
					'devicon', // Suggested languages always use devicon
					lang.category
				);
				success.push(created);
			} catch (error) {
				const errorMessage = error instanceof Error ? error.message : String(error);
				failed.push({ language: lang, error: errorMessage });
				this.log.warn('Failed to create language in batch', { name: lang.name, error });
			}
		}

		return { success, failed };
	}

	/**
	 * Delete a language
	 */
	async deleteLanguage(id: number): Promise<void> {
		try {
			this.log.info('Deleting language', { id });
			await invokeClient.post('delete_language', { id });
			this.log.info('Language deleted successfully', { id });
		} catch (error) {
			this.log.error('Failed to delete language', { error });
			throw error;
		}
	}
}

export const languageService = LanguageService.getInstance();

