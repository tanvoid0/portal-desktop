/**
 * Documents Domain Types
 */

export enum DocumentType {
	MARKDOWN = 'markdown',
	TEXT = 'text',
	CODE = 'code',
	HTML = 'html',
	JSON = 'json',
	YAML = 'yaml',
	XML = 'xml',
	OTHER = 'other'
}

export enum DocumentStatus {
	DRAFT = 'draft',
	PUBLISHED = 'published',
	ARCHIVED = 'archived',
	DELETED = 'deleted'
}

export interface Document {
	id: string;
	title: string;
	content: string;
	type: DocumentType;
	status: DocumentStatus;
	description?: string;
	tags: string[];
	folderId?: string;
	createdAt: Date;
	updatedAt: Date;
	lastViewedAt?: Date;
	viewCount: number;
	wordCount: number;
	characterCount: number;
	metadata: DocumentMetadata;
	version: DocumentVersion;
	permissions: DocumentPermissions;
}

export interface DocumentMetadata {
	author: string;
	language?: string;
	template?: string;
	project?: string;
	repository?: string;
	branch?: string;
	commit?: string;
	lastModifiedBy?: string;
	lastModifiedAt?: Date;
	checksum?: string;
	fileSize?: number;
	mimeType?: string;
	encoding?: string;
	notes?: string;
	customFields?: Record<string, any>;
}

export interface DocumentVersion {
	current: number;
	total: number;
	versions: VersionInfo[];
}

export interface VersionInfo {
	number: number;
	title: string;
	description?: string;
	createdAt: Date;
	createdBy: string;
	changes: string[];
	size: number;
	checksum: string;
}

export interface DocumentPermissions {
	owner: string;
	read: string[];
	write: string[];
	admin: string[];
	public: boolean;
	encrypted: boolean;
}

export interface DocumentFolder {
	id: string;
	name: string;
	description?: string;
	parentId?: string;
	path: string;
	createdAt: Date;
	updatedAt: Date;
	documentCount: number;
	subfolderCount: number;
	permissions: DocumentPermissions;
	metadata: DocumentMetadata;
}

export interface DocumentTemplate {
	id: string;
	name: string;
	description: string;
	type: DocumentType;
	content: string;
	tags: string[];
	category: string;
	author: string;
	createdAt: Date;
	updatedAt: Date;
	usageCount: number;
	metadata: DocumentMetadata;
}

export interface DocumentSearchRequest {
	query: string;
	type?: DocumentType;
	status?: DocumentStatus;
	tags?: string[];
	folderId?: string;
	author?: string;
	dateRange?: {
		start: Date;
		end: Date;
	};
	includeContent?: boolean;
	limit?: number;
	offset?: number;
	sortBy?: 'relevance' | 'title' | 'createdAt' | 'updatedAt' | 'viewCount';
	sortOrder?: 'asc' | 'desc';
}

export interface DocumentSearchResult {
	documents: Document[];
	total: number;
	page: number;
	limit: number;
	query: string;
	facets: SearchFacets;
	highlightedContent?: Record<string, string>;
}

export interface SearchFacets {
	types: Record<DocumentType, number>;
	statuses: Record<DocumentStatus, number>;
	tags: Record<string, number>;
	authors: Record<string, number>;
	folders: Record<string, number>;
}

export interface DocumentCreateRequest {
	title: string;
	content: string;
	type: DocumentType;
	description?: string;
	tags?: string[];
	folderId?: string;
	template?: string;
	metadata?: Partial<DocumentMetadata>;
	permissions?: Partial<DocumentPermissions>;
}

export interface DocumentUpdateRequest {
	title?: string;
	content?: string;
	description?: string;
	tags?: string[];
	folderId?: string;
	status?: DocumentStatus;
	metadata?: Partial<DocumentMetadata>;
	permissions?: Partial<DocumentPermissions>;
}

export interface DocumentExportRequest {
	documentId: string;
	format: 'pdf' | 'html' | 'markdown' | 'text' | 'docx';
	includeMetadata?: boolean;
	includeVersionHistory?: boolean;
	template?: string;
	options?: ExportOptions;
}

export interface ExportOptions {
	pageSize?: 'A4' | 'A3' | 'Letter';
	orientation?: 'portrait' | 'landscape';
	margins?: {
		top: number;
		right: number;
		bottom: number;
		left: number;
	};
	fontSize?: number;
	fontFamily?: string;
	lineHeight?: number;
	includeToc?: boolean;
	includePageNumbers?: boolean;
	includeHeader?: boolean;
	includeFooter?: boolean;
}

export interface DocumentImportRequest {
	file: File;
	type?: DocumentType;
	folderId?: string;
	tags?: string[];
	metadata?: Partial<DocumentMetadata>;
	options?: ImportOptions;
}

export interface ImportOptions {
	overwriteExisting?: boolean;
	extractMetadata?: boolean;
	detectLanguage?: boolean;
	convertFormat?: boolean;
	preserveFormatting?: boolean;
}

export interface DocumentCollaboration {
	documentId: string;
	participants: CollaborationParticipant[];
	activeUsers: ActiveUser[];
	changes: DocumentChange[];
	comments: DocumentComment[];
	conflicts: DocumentConflict[];
}

export interface CollaborationParticipant {
	userId: string;
	username: string;
	email: string;
	role: 'viewer' | 'editor' | 'admin';
	permissions: string[];
	joinedAt: Date;
	lastActiveAt: Date;
}

export interface ActiveUser {
	userId: string;
	username: string;
	email: string;
	cursorPosition: CursorPosition;
	selection?: TextSelection;
	lastSeen: Date;
	status: 'online' | 'away' | 'busy';
}

export interface CursorPosition {
	line: number;
	column: number;
	offset: number;
}

export interface TextSelection {
	start: CursorPosition;
	end: CursorPosition;
	text: string;
}

export interface DocumentChange {
	id: string;
	userId: string;
	username: string;
	type: 'insert' | 'delete' | 'format' | 'move';
	position: CursorPosition;
	content: string;
	length: number;
	timestamp: Date;
	version: number;
}

export interface DocumentComment {
	id: string;
	userId: string;
	username: string;
	content: string;
	position: CursorPosition;
	selection?: TextSelection;
	createdAt: Date;
	updatedAt: Date;
	resolved: boolean;
	replies: DocumentComment[];
}

export interface DocumentConflict {
	id: string;
	userId: string;
	username: string;
	change1: DocumentChange;
	change2: DocumentChange;
	position: CursorPosition;
	content: string;
	createdAt: Date;
	resolved: boolean;
	resolution?: ConflictResolution;
}

export interface ConflictResolution {
	resolution: 'keep1' | 'keep2' | 'merge' | 'manual';
	content: string;
	resolvedBy: string;
	resolvedAt: Date;
	notes?: string;
}

export interface DocumentAnalytics {
	documentId: string;
	views: number;
	uniqueViews: number;
	timeSpent: number;
	avgTimePerView: number;
	popularSections: DocumentSection[];
	searchQueries: string[];
	referrers: string[];
	lastViewedAt: Date;
	viewHistory: ViewEvent[];
}

export interface DocumentSection {
	title: string;
	level: number;
	position: number;
	views: number;
	timeSpent: number;
	popularity: number;
}

export interface ViewEvent {
	userId: string;
	viewedAt: Date;
	duration: number;
	scrollDepth: number;
	actions: string[];
}

export interface DocumentBackup {
	id: string;
	documentId: string;
	name: string;
	description?: string;
	content: string;
	metadata: DocumentMetadata;
	createdAt: Date;
	size: number;
	checksum: string;
	compressed: boolean;
}

export interface DocumentSync {
	documentId: string;
	lastSyncAt: Date;
	syncStatus: 'synced' | 'pending' | 'conflict' | 'error';
	conflicts: DocumentConflict[];
	changes: DocumentChange[];
	remoteVersion: number;
	localVersion: number;
	error?: string;
}
