import { invoke } from '@tauri-apps/api/core';
import type {
	GeneratedDocumentStructure,
	GenerateDocumentRequest,
	ConversationMessage,
	DocumentContext,
} from '../types';

// Re-export types for convenience
export type { GeneratedDocumentStructure, ConversationMessage, DocumentContext } from '../types';

export interface AIErrorInfo {
	message: string;
	code?: string;
	details?: string;
}

export function parseError(error: unknown): AIErrorInfo {
	if (error instanceof Error) {
		// Try to extract structured error info if available
		const message = error.message;
		return {
			message,
			code: 'GENERATION_ERROR',
			details: message,
		};
	}

	return {
		message: 'An unexpected error occurred during document generation',
		code: 'UNKNOWN_ERROR',
	};
}

export class AIDocumentService {
	/**
	 * Generate document from prompt/description text using AI
	 * @param request - Prompt text and optional provider type
	 * @returns Generated document structure with title, content, and suggestions
	 */
	async generateDocumentFromPrompt(
		request: GenerateDocumentRequest
	): Promise<GeneratedDocumentStructure> {
		try {
			const command = {
				prompt: request.prompt,
				provider_type: request.providerType || null,
				history: request.history || null,
				context: request.context || null,
				instruction: request.instruction || null,
			};

			const response = await invoke<GeneratedDocumentStructure>(
				'generate_document_with_ai',
				{ command }
			);

			return response;
		} catch (error) {
			const errorInfo = parseError(error);
			console.error('AI document generation error:', error);

			// Create a custom error with the parsed information
			const customError = new Error(errorInfo.message) as Error & { errorInfo: AIErrorInfo };
			customError.errorInfo = errorInfo;
			throw customError;
		}
	}
}

export const aiDocumentService = new AIDocumentService();

