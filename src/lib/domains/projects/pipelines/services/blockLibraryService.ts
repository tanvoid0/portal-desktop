/**
 * Block Library Service - Management of reusable command blocks
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '@/lib/domains/shared';
import type { Block, CreateBlockRequest } from '../types';

const log = logger.createScoped('BlockLibraryService');

export class BlockLibraryService {
	private static instance: BlockLibraryService;
	private defaultBlocks: Block[] = [];

	static getInstance(): BlockLibraryService {
		if (!BlockLibraryService.instance) {
			BlockLibraryService.instance = new BlockLibraryService();
			BlockLibraryService.instance.initializeDefaultBlocks();
		}
		return BlockLibraryService.instance;
	}

	/**
	 * Initialize default block library
	 */
	private initializeDefaultBlocks(): void {
		this.defaultBlocks = [
			// Install Dependencies
			{
				id: 'install-npm',
				name: 'Install NPM Dependencies',
				description: 'Install Node.js dependencies using npm',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'packageManager',
						type: 'select',
						description: 'Package manager to use',
						required: true,
						defaultValue: 'npm',
						options: ['npm', 'yarn', 'pnpm'],
					},
					{
						name: 'installCommand',
						type: 'select',
						description: 'Install command variant',
						required: false,
						defaultValue: 'install',
						options: ['install', 'ci'],
					},
				],
				command: '${packageManager} ${installCommand}',
				executionType: 'command',
				defaultConfig: { packageManager: 'npm', installCommand: 'install' },
				tags: ['node', 'npm', 'dependencies', 'install'],
			},
			{
				id: 'install-pip',
				name: 'Install Python Dependencies',
				description: 'Install Python dependencies using pip',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'requirementsFile',
						type: 'string',
						description: 'Requirements file path',
						required: false,
						defaultValue: 'requirements.txt',
					},
					{
						name: 'upgrade',
						type: 'boolean',
						description: 'Upgrade existing packages',
						required: false,
						defaultValue: false,
					},
				],
				command: 'pip install -r ${requirementsFile}${upgrade ? " --upgrade" : ""}',
				executionType: 'command',
				defaultConfig: { requirementsFile: 'requirements.txt', upgrade: false },
				tags: ['python', 'pip', 'dependencies', 'install'],
			},
			{
				id: 'install-cargo',
				name: 'Install Rust Dependencies',
				description: 'Install Rust dependencies using Cargo',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildDeps',
						type: 'boolean',
						description: 'Build dependencies',
						required: false,
						defaultValue: true,
					},
				],
				command: 'cargo build${buildDeps ? " --deps" : ""}',
				executionType: 'command',
				defaultConfig: { buildDeps: true },
				tags: ['rust', 'cargo', 'dependencies', 'install'],
			},
			// Run Tests
			{
				id: 'test-npm',
				name: 'Run NPM Tests',
				description: 'Execute test suite using npm',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'testCommand',
						type: 'string',
						description: 'Test command',
						required: false,
						defaultValue: 'test',
					},
					{
						name: 'coverage',
						type: 'boolean',
						description: 'Generate coverage report',
						required: false,
						defaultValue: false,
					},
				],
				command: 'npm run ${testCommand}${coverage ? " -- --coverage" : ""}',
				executionType: 'command',
				defaultConfig: { testCommand: 'test', coverage: false },
				tags: ['node', 'npm', 'test'],
			},
			{
				id: 'test-pytest',
				name: 'Run Pytest',
				description: 'Execute Python tests using pytest',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'testPath',
						type: 'string',
						description: 'Test path or file',
						required: false,
						defaultValue: '.',
					},
					{
						name: 'verbose',
						type: 'boolean',
						description: 'Verbose output',
						required: false,
						defaultValue: false,
					},
				],
				command: 'pytest ${testPath}${verbose ? " -v" : ""}',
				executionType: 'command',
				defaultConfig: { testPath: '.', verbose: false },
				tags: ['python', 'pytest', 'test'],
			},
			{
				id: 'test-cargo',
				name: 'Run Cargo Tests',
				description: 'Execute Rust tests using Cargo',
				category: 'test',
				version: '1.0.0',
				parameters: [
					{
						name: 'release',
						type: 'boolean',
						description: 'Run in release mode',
						required: false,
						defaultValue: false,
					},
				],
				command: 'cargo test${release ? " --release" : ""}',
				executionType: 'command',
				defaultConfig: { release: false },
				tags: ['rust', 'cargo', 'test'],
			},
			// Lint
			{
				id: 'lint-eslint',
				name: 'Lint with ESLint',
				description: 'Run ESLint to check code quality',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'fix',
						type: 'boolean',
						description: 'Auto-fix issues',
						required: false,
						defaultValue: false,
					},
				],
				command: 'eslint .${fix ? " --fix" : ""}',
				executionType: 'command',
				defaultConfig: { fix: false },
				tags: ['node', 'eslint', 'lint'],
			},
			{
				id: 'lint-pylint',
				name: 'Lint with Pylint',
				description: 'Run Pylint to check Python code quality',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'target',
						type: 'string',
						description: 'Target file or directory',
						required: false,
						defaultValue: '.',
					},
				],
				command: 'pylint ${target}',
				executionType: 'command',
				defaultConfig: { target: '.' },
				tags: ['python', 'pylint', 'lint'],
			},
			{
				id: 'lint-clippy',
				name: 'Lint with Clippy',
				description: 'Run Clippy to check Rust code quality',
				category: 'utility',
				version: '1.0.0',
				parameters: [],
				command: 'cargo clippy -- -D warnings',
				executionType: 'command',
				defaultConfig: {},
				tags: ['rust', 'clippy', 'lint'],
			},
			// Build
			{
				id: 'build-npm',
				name: 'Build NPM Project',
				description: 'Build Node.js project',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'buildCommand',
						type: 'string',
						description: 'Build command',
						required: false,
						defaultValue: 'build',
					},
					{
						name: 'environment',
						type: 'select',
						description: 'Build environment',
						required: false,
						defaultValue: 'production',
						options: ['development', 'staging', 'production'],
					},
				],
				command: 'npm run ${buildCommand}',
				executionType: 'command',
				defaultConfig: { buildCommand: 'build', environment: 'production' },
				tags: ['node', 'npm', 'build'],
			},
			{
				id: 'build-cargo',
				name: 'Build Cargo Project',
				description: 'Build Rust project using Cargo',
				category: 'build',
				version: '1.0.0',
				parameters: [
					{
						name: 'release',
						type: 'boolean',
						description: 'Build in release mode',
						required: false,
						defaultValue: false,
					},
				],
				command: 'cargo build${release ? " --release" : ""}',
				executionType: 'command',
				defaultConfig: { release: false },
				tags: ['rust', 'cargo', 'build'],
			},
			// Docker
			{
				id: 'docker-build',
				name: 'Build Docker Image',
				description: 'Build a Docker image',
				category: 'deploy',
				version: '1.0.0',
				parameters: [
					{
						name: 'imageName',
						type: 'string',
						description: 'Docker image name',
						required: true,
					},
					{
						name: 'dockerfile',
						type: 'file',
						description: 'Dockerfile path',
						required: false,
						defaultValue: 'Dockerfile',
					},
					{
						name: 'context',
						type: 'directory',
						description: 'Build context',
						required: false,
						defaultValue: '.',
					},
				],
				command: 'docker build -t ${imageName} -f ${dockerfile} ${context}',
				executionType: 'docker',
				defaultConfig: { dockerfile: 'Dockerfile', context: '.' },
				tags: ['docker', 'build', 'deploy'],
			},
			{
				id: 'docker-push',
				name: 'Push Docker Image',
				description: 'Push Docker image to registry',
				category: 'deploy',
				version: '1.0.0',
				parameters: [
					{
						name: 'imageName',
						type: 'string',
						description: 'Docker image name',
						required: true,
					},
				],
				command: 'docker push ${imageName}',
				executionType: 'docker',
				defaultConfig: {},
				tags: ['docker', 'push', 'deploy'],
			},
			// Git
			{
				id: 'git-commit',
				name: 'Git Commit',
				description: 'Commit changes to Git',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'message',
						type: 'string',
						description: 'Commit message',
						required: true,
					},
				],
				command: 'git commit -m "${message}"',
				executionType: 'command',
				defaultConfig: {},
				tags: ['git', 'commit'],
			},
			{
				id: 'git-push',
				name: 'Git Push',
				description: 'Push changes to remote repository',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'branch',
						type: 'string',
						description: 'Branch to push',
						required: false,
						defaultValue: 'main',
					},
				],
				command: 'git push origin ${branch}',
				executionType: 'command',
				defaultConfig: { branch: 'main' },
				tags: ['git', 'push'],
			},
			// Custom Script
			{
				id: 'run-script',
				name: 'Run Custom Script',
				description: 'Execute a custom script or command',
				category: 'utility',
				version: '1.0.0',
				parameters: [
					{
						name: 'command',
						type: 'string',
						description: 'Command or script to run',
						required: true,
					},
				],
				command: '${command}',
				executionType: 'command',
				defaultConfig: {},
				tags: ['custom', 'script'],
			},
		];
	}

	/**
	 * Get all blocks (default + custom)
	 */
	async getBlocks(): Promise<Block[]> {
		try {
			log.info('Loading blocks');
			const customBlocks = await invoke<Block[]>('get_blocks');
			return [...this.defaultBlocks, ...customBlocks];
		} catch (error) {
			log.warn('Failed to load custom blocks, using defaults only', error);
			return this.defaultBlocks;
		}
	}

	/**
	 * Get default blocks only
	 */
	getDefaultBlocks(): Block[] {
		return [...this.defaultBlocks];
	}

	/**
	 * Get blocks by category
	 */
	async getBlocksByCategory(category: Block['category']): Promise<Block[]> {
		const blocks = await this.getBlocks();
		return blocks.filter((b) => b.category === category);
	}

	/**
	 * Get blocks by tag
	 */
	async getBlocksByTag(tag: string): Promise<Block[]> {
		const blocks = await this.getBlocks();
		return blocks.filter((b) => b.tags.includes(tag));
	}

	/**
	 * Search blocks
	 */
	async searchBlocks(query: string): Promise<Block[]> {
		const blocks = await this.getBlocks();
		const lowerQuery = query.toLowerCase();
		return blocks.filter(
			(b) =>
				b.name.toLowerCase().includes(lowerQuery) ||
				b.description.toLowerCase().includes(lowerQuery) ||
				b.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))
		);
	}

	/**
	 * Get a specific block by ID
	 */
	async getBlock(blockId: string): Promise<Block | null> {
		const blocks = await this.getBlocks();
		return blocks.find((b) => b.id === blockId) || null;
	}

	/**
	 * Create a custom block
	 */
	async createBlock(request: CreateBlockRequest): Promise<Block> {
		try {
			log.info('Creating block', { name: request.name });
			const block = await invoke<Block>('create_block', { request });
			log.info('Block created', { id: block.id });
			return block;
		} catch (error) {
			log.error('Failed to create block', error);
			throw error;
		}
	}

	/**
	 * Update a custom block
	 */
	async updateBlock(blockId: string, request: Partial<CreateBlockRequest>): Promise<Block> {
		try {
			log.info('Updating block', { blockId });
			const block = await invoke<Block>('update_block', { blockId, request });
			log.info('Block updated', { blockId });
			return block;
		} catch (error) {
			log.error('Failed to update block', error);
			throw error;
		}
	}

	/**
	 * Delete a custom block
	 */
	async deleteBlock(blockId: string): Promise<void> {
		try {
			log.info('Deleting block', { blockId });
			await invoke('delete_block', { blockId });
			log.info('Block deleted', { blockId });
		} catch (error) {
			log.error('Failed to delete block', error);
			throw error;
		}
	}
}

export const blockLibraryService = BlockLibraryService.getInstance();

