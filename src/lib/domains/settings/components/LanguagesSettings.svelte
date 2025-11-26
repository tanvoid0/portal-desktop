<!--
	Languages Settings - Manage languages with intelligent recommendations
-->

<script lang="ts">
	import { Code } from '@lucide/svelte';
	import { learningService } from '@/lib/domains/learning';
	import ItemSettings from './shared/ItemSettings.svelte';
	import { languageServiceAdapter } from './shared/serviceAdapters';
	import type { Language, SuggestedLanguage } from '@/lib/domains/languages/types';

	const config = {
		itemName: 'Language',
		itemNamePlural: 'Languages',
		emptyIcon: Code,
		emptyMessage: 'No languages added yet',
		emptySearchMessage: 'No languages match your search',
		service: languageServiceAdapter,
		recommendationPatternType: 'config' as const,
		recommendationDataKey: 'language',
		onItemAdded: async (item: Language) => {
			try {
				await learningService.learnPattern({
					pattern_type: 'config',
					pattern_data: {
						language: item.name,
						category: item.category
					},
					context: `category_${item.category.toLowerCase().replace(/\s+/g, '_')}`
				});
			} catch (error) {
				console.warn('Failed to learn language pattern', error);
			}
		},
		onItemsBatchAdded: async (items: Language[]) => {
			for (const item of items) {
				try {
					await learningService.learnPattern({
						pattern_type: 'config',
						pattern_data: {
							language: item.name,
							category: item.category
						},
						context: `category_${item.category.toLowerCase().replace(/\s+/g, '_')}`
					});
				} catch (error) {
					console.warn('Failed to learn language pattern', error);
				}
			}
		}
	};
</script>

<ItemSettings config={config} />
