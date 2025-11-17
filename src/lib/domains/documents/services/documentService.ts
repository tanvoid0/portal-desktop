import { invoke } from '@tauri-apps/api/core';
import type { Document, CreateDocumentRequest, UpdateDocumentRequest } from '../types';

export interface TauriDocumentResponse {
	id: number;
	title: string;
	content: string;
	is_archived: boolean;
	content_draft: string | null;
	is_draft: boolean;
	tags: string | null; // JSON array of strings
	created_at: string | null;
	updated_at: string | null;
	last_edited_at: string | null;
}

// Convert Tauri response to frontend Document type
function convertTauriDocumentToDocument(tauriDoc: TauriDocumentResponse): Document {
	return {
		id: tauriDoc.id,
		title: tauriDoc.title,
		content: tauriDoc.content,
		isArchived: tauriDoc.is_archived,
		contentDraft: tauriDoc.content_draft || undefined,
		isDraft: tauriDoc.is_draft,
		tags: tauriDoc.tags ? JSON.parse(tauriDoc.tags) : undefined,
		createdAt: tauriDoc.created_at ? new Date(tauriDoc.created_at) : new Date(),
		updatedAt: tauriDoc.updated_at ? new Date(tauriDoc.updated_at) : new Date(),
		lastEditedAt: tauriDoc.last_edited_at ? new Date(tauriDoc.last_edited_at) : undefined,
	};
}

export class DocumentService {
	async createDocument(request: CreateDocumentRequest): Promise<Document> {
		const command = {
			title: request.title,
			content: request.content,
			is_archived: request.isArchived || false,
			tags: request.tags || null,
		};
		const response = await invoke<TauriDocumentResponse>('create_document', { command });
		return convertTauriDocumentToDocument(response);
	}

	async updateDocument(id: number, request: UpdateDocumentRequest): Promise<Document> {
		const command = {
			title: request.title || null,
			content: request.content || null,
			is_archived: request.isArchived !== undefined ? request.isArchived : null,
			tags: request.tags || null,
		};
		const response = await invoke<TauriDocumentResponse>('update_document', {
			id,
			command,
		});
		return convertTauriDocumentToDocument(response);
	}

	async updateDraft(id: number, contentDraft: string): Promise<Document> {
		const response = await invoke<TauriDocumentResponse>('update_document_draft', {
			id,
			content_draft: contentDraft,
		});
		return convertTauriDocumentToDocument(response);
	}

	async saveDocument(
		id: number,
		title?: string,
		content?: string,
		tags?: string[],
		isArchived?: boolean
	): Promise<Document> {
		const response = await invoke<TauriDocumentResponse>('save_document', {
			id,
			title: title || null,
			content: content || null,
			tags: tags || null,
			is_archived: isArchived !== undefined ? isArchived : null,
		});
		return convertTauriDocumentToDocument(response);
	}

	async deleteDocument(id: number): Promise<void> {
		await invoke('delete_document', { id });
	}

	async getDocument(id: number): Promise<Document | null> {
		const response = await invoke<TauriDocumentResponse | null>('get_document', { id });
		return response ? convertTauriDocumentToDocument(response) : null;
	}

	async getDocuments(): Promise<Document[]> {
		const response = await invoke<TauriDocumentResponse[]>('get_documents');
		return response.map(convertTauriDocumentToDocument);
	}

	async searchDocuments(query: string): Promise<Document[]> {
		const response = await invoke<TauriDocumentResponse[]>('search_documents', { query });
		return response.map(convertTauriDocumentToDocument);
	}
}

export const documentService = new DocumentService();

