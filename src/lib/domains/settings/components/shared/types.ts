/**
 * Shared types for settings components (Languages, Package Managers, Frameworks)
 */

import type { PatternType } from '@/lib/domains/learning/types';

/**
 * Base interface for all item types (Language, PackageManager, Framework)
 */
export interface BaseItem {
	id: number;
	name: string;
	icon: string;
	icon_type: 'devicon' | 'file';
	category: string;
	created_at?: string;
	updated_at?: string;
}

/**
 * Base interface for suggested items
 */
export interface BaseSuggestedItem {
	name: string;
	icon: string;
	category: string;
}

/**
 * Base interface for item groups
 */
export interface BaseItemGroup<T extends BaseSuggestedItem> {
	category: string;
	items: T[];
}

/**
 * Service interface for CRUD operations
 */
export interface ItemService<T extends BaseItem, TSuggested extends BaseSuggestedItem> {
	getAll(): Promise<T[]>;
	getSuggested(): Promise<BaseItemGroup<TSuggested>[]>;
	create(name: string, icon: string, iconType: 'devicon' | 'file', category: string): Promise<T>;
	update(id: number, name?: string, icon?: string, iconType?: 'devicon' | 'file', category?: string): Promise<T>;
	delete(id: number): Promise<void>;
	createBatch(items: TSuggested[]): Promise<{ success: T[]; failed: { item: TSuggested; error: string }[] }>;
}

/**
 * Configuration for the reusable settings component
 */
export interface SettingsComponentConfig<T extends BaseItem, TSuggested extends BaseSuggestedItem> {
	// Display labels
	itemName: string; // e.g., "Language", "Package Manager", "Framework"
	itemNamePlural: string; // e.g., "Languages", "Package Managers", "Frameworks"
	emptyIcon: any; // Lucide icon component
	emptyMessage: string;
	emptySearchMessage: string;
	
	// Service
	service: ItemService<T, TSuggested>;
	
	// Learning/Recommendation
	recommendationPatternType: PatternType; // e.g., 'config' for languages, 'framework' for frameworks
	recommendationDataKey: string; // e.g., "language", "package_manager", "framework" - the key in pattern_data
	
	// Optional callbacks
	onItemAdded?: (item: T) => Promise<void>;
	onItemsBatchAdded?: (items: T[]) => Promise<void>;
}

