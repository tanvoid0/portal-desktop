export interface Document {
	id: number;
	title: string;
	content: string;
	isArchived: boolean;
	contentDraft?: string;
	isDraft: boolean;
	tags?: string[];
	createdAt: Date;
	updatedAt: Date;
	lastEditedAt?: Date;
}

export interface CreateDocumentRequest {
	title: string;
	content: string;
	isArchived?: boolean;
	tags?: string[];
}

export interface UpdateDocumentRequest {
	title?: string;
	content?: string;
	isArchived?: boolean;
	tags?: string[];
}

export interface DocumentFilters {
	search?: string;
	tags?: string[];
}

export interface GeneratedDocumentStructure {
	title: string;
	content: string;
	suggestedTags: string[];
	confidence: number;
	modelUsed: string;
}

export interface DocumentContext {
	linkedTask?: {
		id: number;
		title: string;
		description?: string;
		status?: string;
	};
}

export interface ConversationMessage {
	role: 'user' | 'assistant';
	content: string;
}

export interface GenerateDocumentRequest {
	prompt: string;
	providerType?: 'Ollama' | 'Gemini' | 'OpenAI' | 'Anthropic';
	history?: ConversationMessage[];
	context?: DocumentContext;
	instruction?: string;
}
