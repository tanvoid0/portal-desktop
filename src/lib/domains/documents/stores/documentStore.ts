import { writable, derived, type Writable } from 'svelte/store';
import type { Document, DocumentFilters } from '../types';
import { documentService } from '../services/documentService';

// Core stores
export const documents: Writable<Document[]> = writable([]);
export const selectedDocument: Writable<Document | null> = writable(null);
export const isLoading: Writable<boolean> = writable(false);
export const error: Writable<string | null> = writable(null);

// Filter stores
export const documentFilters: Writable<DocumentFilters> = writable({});

// Derived stores
export const filteredDocuments = derived(
	[documents, documentFilters],
	([$documents, $filters]) => {
		let filtered = [...$documents];

		// Apply search filter
		if ($filters.search?.trim()) {
			const query = $filters.search.toLowerCase();
			filtered = filtered.filter(
				(doc) =>
					doc.title.toLowerCase().includes(query) ||
					doc.content.toLowerCase().includes(query)
			);
		}

		// Apply tag filter
		if ($filters.tags && $filters.tags.length > 0) {
			filtered = filtered.filter((doc) => {
				if (!doc.tags || doc.tags.length === 0) return false;
				return $filters.tags!.some((tag) => doc.tags!.includes(tag));
			});
		}

		return filtered;
	}
);

// Document actions
export const documentActions = {
	async loadDocuments() {
		isLoading.set(true);
		error.set(null);
		try {
			const docs = await documentService.getDocuments();
			documents.set(docs);
		} catch (err) {
			const errorMessage = err instanceof Error ? err.message : 'Failed to load documents';
			error.set(errorMessage);
			console.error('❌ Failed to load documents:', err);
		} finally {
			isLoading.set(false);
		}
	},

	async createDocument(title: string, content: string, tags?: string[], isArchived?: boolean): Promise<Document> {
		const newDoc = await documentService.createDocument({ title, content, tags, isArchived });
		documents.update((docs) => [newDoc, ...docs]);
		return newDoc;
	},

	async updateDocument(id: number, updates: { title?: string; content?: string; tags?: string[]; isArchived?: boolean }): Promise<Document> {
		const updatedDoc = await documentService.updateDocument(id, updates);
		documents.update((docs) => docs.map((doc) => (doc.id === id ? updatedDoc : doc)));
		if (selectedDocument) {
			selectedDocument.update((doc) => (doc?.id === id ? updatedDoc : doc));
		}
		return updatedDoc;
	},

	async updateDraft(id: number, contentDraft: string): Promise<Document> {
		const updatedDoc = await documentService.updateDraft(id, contentDraft);
		documents.update((docs) => docs.map((doc) => (doc.id === id ? updatedDoc : doc)));
		if (selectedDocument) {
			selectedDocument.update((doc) => (doc?.id === id ? updatedDoc : doc));
		}
		return updatedDoc;
	},

	async saveDocument(id: number, title?: string, content?: string, tags?: string[], isArchived?: boolean): Promise<Document> {
		const savedDoc = await documentService.saveDocument(id, title, content, tags, isArchived);
		documents.update((docs) => docs.map((doc) => (doc.id === id ? savedDoc : doc)));
		if (selectedDocument) {
			selectedDocument.update((doc) => (doc?.id === id ? savedDoc : doc));
		}
		return savedDoc;
	},

	async deleteDocument(id: number): Promise<void> {
		await documentService.deleteDocument(id);
		documents.update((docs) => docs.filter((doc) => doc.id !== id));
		if (selectedDocument) {
			selectedDocument.update((doc) => (doc?.id === id ? null : doc));
		}
	},

	async getDocument(id: number): Promise<Document | null> {
		return await documentService.getDocument(id);
	},

	async searchDocuments(query: string): Promise<Document[]> {
		const results = await documentService.searchDocuments(query);
		return results;
	},

	selectDocument(doc: Document | null) {
		selectedDocument.set(doc);
	},
};

