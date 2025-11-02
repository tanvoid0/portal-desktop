/**
 * Projects domain types
 */

import type { BaseEntity } from '@/lib/domains/shared/types';

export interface Project extends BaseEntity {
	name: string;
	description?: string;
	path: string;
	status: ProjectStatus;
	framework?: string;
	package_manager?: string;
	build_command?: string;
	start_command?: string;
	test_command?: string;
	output_directory?: string;
	dev_port?: number;
	prod_port?: number;
	starred: boolean;
	open_count: number;
	last_opened?: Date;
	size: number;
	file_count: number;
	git_repository?: string;
	git_branch?: string;
	git_commit?: string;
	has_uncommitted_changes: boolean;
	last_commit?: Date;
	created_at?: Date;
	updated_at?: Date;
	metadata: ProjectMetadata;
}


export enum ProjectStatus {
	ACTIVE = 'active',
	ARCHIVED = 'archived',
	DELETED = 'deleted'
}

export interface ProjectSettings {
	buildCommand?: string;
	startCommand?: string;
	testCommand?: string;
	outputDirectory?: string;
	environment?: Record<string, string>;
	ports?: {
		dev?: number;
		prod?: number;
	};
	framework?: string;
	packageManager?: 'npm' | 'yarn' | 'pnpm';
}

export interface ProjectMetadata {
	lastOpened?: Date;
	openCount: number;
	size: number;
	fileCount: number;
	gitInfo?: GitInfo;
	dependencies?: DependencyInfo;
}

export interface GitInfo {
	repository?: string;
	branch?: string;
	commit?: string;
	hasUncommittedChanges: boolean;
	lastCommit?: Date;
}

export interface DependencyInfo {
	packageManager: string;
	dependencies: Record<string, string>;
	devDependencies: Record<string, string>;
	outdated: string[];
	vulnerabilities: string[];
}

export interface CreateProjectRequest {
	name: string;
	description?: string;
	path: string;
	framework?: string;
	package_manager?: string;
	build_command?: string;
	start_command?: string;
	test_command?: string;
	output_directory?: string;
	dev_port?: number;
	prod_port?: number;
	settings?: ProjectSettings;
}

export interface UpdateProjectRequest {
	name?: string;
	description?: string;
	path?: string;
	status?: ProjectStatus;
	framework?: string;
	package_manager?: string;
	build_command?: string;
	start_command?: string;
	test_command?: string;
	output_directory?: string;
	dev_port?: number;
	prod_port?: number;
}

export interface ProjectTemplate {
	id: string;
	name: string;
	description: string;
	framework?: string;
	repository?: string;
	commands: {
		create: string;
		install?: string;
		start?: string;
	};
}

export interface ProjectStats {
	total_projects: number;
	active_projects: number;
	archived_projects: number;
	total_size: number;
	most_used_type: string;
	recent_projects: Project[];
}
