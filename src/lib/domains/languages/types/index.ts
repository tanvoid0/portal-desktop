/**
 * Languages domain types
 */

export interface Language {
	id: number;
	name: string;
	icon: string;
	icon_type: 'devicon' | 'file';
	category: string;
	created_at?: string;
	updated_at?: string;
}

export interface SuggestedLanguage {
	name: string;
	icon: string;
	category: string;
}

export interface LanguageGroup {
	category: string;
	languages: SuggestedLanguage[];
}

