/**
 * SDK Service - Frontend business logic for SDK management
 */

import { invoke } from '@tauri-apps/api/core';
import { logger } from '../../shared';
import type { 
	SDK, 
	SDKManagerInfo, 
	SDKInstallation, 
	SDKDetectionResult,
	SDKInstallRequest,
	SDKSwitchRequest,
	SDKCommandResult,
	SDKVersionSource
} from '../types';

export class SDKService {
	private static instance: SDKService;

	static getInstance(): SDKService {
		if (!SDKService.instance) {
			SDKService.instance = new SDKService();
		}
		return SDKService.instance;
	}

	/**
	 * Detect all installed SDK managers
	 */
	async detectManagers(): Promise<SDKManagerInfo[]> {
		try {
			logger.info('Detecting SDK managers', { context: 'SDKService' });
			
			const managers = await invoke<SDKManagerInfo[]>('detect_sdk_managers');
			
			logger.info('SDK managers detected', { 
				context: 'SDKService', 
				data: { count: managers.length } 
			});
			
			return managers;
		} catch (error) {
			logger.error('Failed to detect SDK managers', {
				context: 'SDKService',
				error
			});
			throw error;
		}
	}

	/**
	 * Get all available SDKs and languages (both installed and not installed)
	 */
	async getAllAvailableSDKs(): Promise<SDKManagerInfo[]> {
		try {
			logger.info('Getting all available SDKs', { context: 'SDKService' });
			
			const sdks = await invoke<SDKManagerInfo[]>('get_all_available_sdks');
			
			logger.info('All available SDKs retrieved', { 
				context: 'SDKService', 
				data: { 
					count: sdks.length,
					firstFew: sdks.slice(0, 3),
					allSDKs: sdks
				} 
			});
			
			return sdks;
		} catch (error) {
			logger.error('Failed to get all available SDKs', {
				context: 'SDKService',
				error
			});
			throw error;
		}
	}

	/**
	 * List available versions for a specific SDK
	 */
	async listVersions(sdkType: string): Promise<string[]> {
		try {
			logger.info('Listing SDK versions', { 
				context: 'SDKService', 
				data: { sdkType } 
			});
			
			const versions = await invoke<string[]>('list_sdk_versions', { sdkType });
			
			logger.info('SDK versions listed', { 
				context: 'SDKService', 
				data: { sdkType, count: versions.length } 
			});
			
			return versions;
		} catch (error) {
			logger.error('Failed to list SDK versions', {
				context: 'SDKService',
				error,
				data: { sdkType }
			});
			throw error;
		}
	}

	/**
	 * Get active version for a specific SDK
	 */
	async getActiveVersion(sdkType: string): Promise<string | null> {
		try {
			logger.info('Getting active SDK version', { 
				context: 'SDKService', 
				data: { sdkType } 
			});
			
			const version = await invoke<string | null>('get_active_sdk_version', { sdkType });
			
			logger.info('Active SDK version retrieved', { 
				context: 'SDKService', 
				data: { sdkType, version } 
			});
			
			return version;
		} catch (error) {
			logger.error('Failed to get active SDK version', {
				context: 'SDKService',
				error,
				data: { sdkType }
			});
			throw error;
		}
	}

	/**
	 * Switch to a specific SDK version
	 */
	async switchVersion(request: SDKSwitchRequest): Promise<void> {
		try {
			logger.info('Switching SDK version', { 
				context: 'SDKService', 
				data: request 
			});
			
			await invoke('switch_sdk_version', {
				sdkType: request.type,
				version: request.version,
				projectPath: request.projectPath
			});
			
			logger.info('SDK version switched successfully', { 
				context: 'SDKService', 
				data: request 
			});
		} catch (error) {
			logger.error('Failed to switch SDK version', {
				context: 'SDKService',
				error,
				data: request
			});
			throw error;
		}
	}

	/**
	 * Install a new SDK version
	 */
	async installVersion(request: SDKInstallRequest): Promise<void> {
		try {
			logger.info('Installing SDK version', { 
				context: 'SDKService', 
				data: request 
			});
			
			await invoke('install_sdk_version', {
				sdkType: request.type,
				version: request.version
			});
			
			logger.info('SDK version installed successfully', { 
				context: 'SDKService', 
				data: request 
			});
		} catch (error) {
			logger.error('Failed to install SDK version', {
				context: 'SDKService',
				error,
				data: request
			});
			throw error;
		}
	}

	/**
	 * Get SDK installations from database
	 */
	async getInstallations(): Promise<SDKInstallation[]> {
		try {
			logger.info('Getting SDK installations', { context: 'SDKService' });
			
			const installations = await invoke<SDKInstallation[]>('get_sdk_installations');
			
			logger.info('SDK installations retrieved', { 
				context: 'SDKService', 
				data: { count: installations.length } 
			});
			
			return installations;
		} catch (error) {
			logger.warn('No SDK installations found in database, returning empty array', {
				context: 'SDKService',
				error
			});
			// Return empty array instead of throwing error for initial setup
			return [];
		}
	}

	/**
	 * Execute a custom SDK command
	 */
	async executeCommand(command: string, args: string[], workingDirectory?: string): Promise<SDKCommandResult> {
		try {
			logger.info('Executing SDK command', { 
				context: 'SDKService', 
				data: { command, args, workingDirectory } 
			});
			
			// This would need to be implemented in the backend
			// For now, we'll use a placeholder
			const result: SDKCommandResult = {
				success: true,
				output: `Command executed: ${command} ${args.join(' ')}`,
				exitCode: 0,
				duration: 0
			};
			
			logger.info('SDK command executed', { 
				context: 'SDKService', 
				data: { command, success: result.success } 
			});
			
			return result;
		} catch (error) {
			logger.error('Failed to execute SDK command', {
				context: 'SDKService',
				error,
				data: { command, args }
			});
			throw error;
		}
	}

	/**
	 * Detect all SDKs and managers
	 */
	async detectAll(): Promise<SDKDetectionResult> {
		try {
			logger.info('Detecting all SDKs and managers', { context: 'SDKService' });
			
			const managers = await this.detectManagers();
			const installations = await this.getInstallations();
			
			// Convert installations to SDKs
			const sdks: SDK[] = installations.map(installation => ({
				id: installation.id,
				name: this.getSDKName(installation.type),
				type: installation.type,
				manager: installation.manager,
				description: this.getSDKDescription(installation.type),
				website: this.getSDKWebsite(installation.type),
				icon: this.getSDKIconClass(installation.type),
				installation: {
					...installation,
					versions: installation.versions,
					activeVersion: installation.activeVersion
				}
			}));
			
			const result: SDKDetectionResult = {
				managers,
				sdks,
				errors: []
			};
			
			logger.info('SDK detection completed', { 
				context: 'SDKService', 
				data: { 
					managersCount: managers.length,
					sdksCount: sdks.length 
				} 
			});
			
			return result;
		} catch (error) {
			logger.error('Failed to detect all SDKs', {
				context: 'SDKService',
				error
			});
			throw error;
		}
	}

	/**
	 * Get SDK name by type (FlyEnv-style comprehensive support)
	 */
	private getSDKName(type: string): string {
		const names: Record<string, string> = {
			node: 'Node.js',
			rust: 'Rust',
			python: 'Python',
			java: 'Java',
			go: 'Go',
			php: 'PHP',
			ruby: 'Ruby',
			bun: 'Bun',
			deno: 'Deno',
			gradle: 'Gradle',
			kotlin: 'Kotlin',
			scala: 'Scala',
			nginx: 'Nginx',
			apache2: 'Apache',
			caddy: 'Caddy',
			tomcat: 'Tomcat',
			mysql: 'MySQL',
			postgres: 'PostgreSQL',
			mongodb: 'MongoDB',
			'redis-server': 'Redis',
			docker: 'Docker',
			composer: 'Composer',
			npm: 'NPM',
			pip: 'Pip',
			cargo: 'Cargo'
		};
		return names[type] || type;
	}

	/**
	 * Get SDK description by type (FlyEnv-style comprehensive support)
	 */
	private getSDKDescription(type: string): string {
		const descriptions: Record<string, string> = {
			node: 'JavaScript runtime built on Chrome\'s V8 JavaScript engine',
			rust: 'Systems programming language focused on safety and performance',
			python: 'High-level programming language with dynamic semantics',
			java: 'Object-oriented programming language and computing platform',
			go: 'Open source programming language that makes it easy to build software',
			php: 'Server-side scripting language designed for web development',
			ruby: 'Dynamic, open source programming language with a focus on simplicity',
			bun: 'Fast all-in-one JavaScript runtime, bundler, and package manager',
			deno: 'Modern runtime for JavaScript and TypeScript',
			gradle: 'Build automation tool for multi-language software development',
			kotlin: 'Statically typed programming language for modern multiplatform applications',
			scala: 'High-level language that combines object-oriented and functional programming',
			nginx: 'High-performance web server and reverse proxy',
			apache2: 'Popular open-source web server software',
			caddy: 'Modern web server with automatic HTTPS',
			tomcat: 'Apache Tomcat servlet container',
			mysql: 'Popular open-source relational database management system',
			postgres: 'Advanced open-source relational database',
			mongodb: 'NoSQL document database for modern applications',
			'redis-server': 'In-memory data structure store and cache',
			docker: 'Containerization platform for applications',
			composer: 'Dependency manager for PHP',
			npm: 'Package manager for Node.js',
			pip: 'Package installer for Python',
			cargo: 'Package manager for Rust'
		};
		return descriptions[type] || 'Development tool and platform';
	}

	/**
	 * Get SDK website by type
	 */
	private getSDKWebsite(type: string): string {
		const websites: Record<string, string> = {
			node: 'https://nodejs.org',
			rust: 'https://rust-lang.org',
			python: 'https://python.org',
			java: 'https://java.com',
			go: 'https://golang.org',
			php: 'https://php.net',
			ruby: 'https://ruby-lang.org',
			kotlin: 'https://kotlinlang.org',
			scala: 'https://scala-lang.org'
		};
		return websites[type] || '';
	}

	/**
	 * Get SDK icon class by type (Devicons support)
	 */
	private getSDKIconClass(type: string): string {
		const iconClasses: Record<string, string> = {
			node: 'devicon-nodejs-plain',
			rust: 'devicon-rust-plain',
			python: 'devicon-python-plain',
			java: 'devicon-java-plain',
			go: 'devicon-go-plain',
			php: 'devicon-php-plain',
			ruby: 'devicon-ruby-plain',
			bun: 'devicon-bun-plain',
			deno: 'devicon-deno-plain',
			gradle: 'devicon-gradle-plain',
			kotlin: 'devicon-kotlin-plain',
			scala: 'devicon-scala-plain',
			nginx: 'devicon-nginx-original',
			apache2: 'devicon-apache-plain',
			caddy: 'devicon-caddy-plain',
			tomcat: 'devicon-tomcat-plain',
			mysql: 'devicon-mysql-plain',
			postgres: 'devicon-postgresql-plain',
			mongodb: 'devicon-mongodb-plain',
			'redis-server': 'devicon-redis-plain',
			docker: 'devicon-docker-plain',
			composer: 'devicon-composer-plain',
			npm: 'devicon-npm-original-wordmark',
			pip: 'devicon-python-plain',
			cargo: 'devicon-cargo-plain'
		};
		return iconClasses[type] || 'devicon-devicon-plain';
	}

	/**
	 * Setup project environment (FlyEnv-style auto-switching)
	 */
	async setupProjectEnvironment(projectPath: string): Promise<void> {
		try {
			logger.info('Setting up project environment', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
			
			await invoke('setup_project_environment', { projectPath });
			
			logger.info('Project environment setup completed', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
		} catch (error) {
			logger.error('Failed to setup project environment', {
				context: 'SDKService',
				error,
				data: { projectPath }
			});
			throw error;
		}
	}

	/**
	 * Create project-specific SDK configuration
	 */
	async createProjectConfig(projectPath: string, sdkType: string, version: string): Promise<void> {
		try {
			logger.info('Creating project config', { 
				context: 'SDKService', 
				data: { projectPath, sdkType, version } 
			});
			
			await invoke('create_project_config', {
				projectPath,
				sdkType,
				version
			});
			
			logger.info('Project config created successfully', { 
				context: 'SDKService', 
				data: { projectPath, sdkType, version } 
			});
		} catch (error) {
			logger.error('Failed to create project config', {
				context: 'SDKService',
				error,
				data: { projectPath, sdkType, version }
			});
			throw error;
		}
	}

	/**
	 * Setup terminal integration for a project (FlyEnv-style)
	 */
	async setupTerminalIntegration(projectPath: string): Promise<void> {
		try {
			logger.info('Setting up terminal integration', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
			
			await invoke('setup_terminal_integration', { projectPath });
			
			logger.info('Terminal integration setup completed', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
		} catch (error) {
			logger.error('Failed to setup terminal integration', {
				context: 'SDKService',
				error,
				data: { projectPath }
			});
			throw error;
		}
	}

	/**
	 * Get terminal integration status for a project
	 */
	async getTerminalIntegrationStatus(projectPath: string): Promise<Record<string, boolean>> {
		try {
			logger.info('Getting terminal integration status', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
			
			const status = await invoke<Record<string, boolean>>('get_terminal_integration_status', { projectPath });
			
			logger.info('Terminal integration status retrieved', { 
				context: 'SDKService', 
				data: { projectPath, status } 
			});
			
			return status;
		} catch (error) {
			logger.error('Failed to get terminal integration status', {
				context: 'SDKService',
				error,
				data: { projectPath }
			});
			throw error;
		}
	}

	/**
	 * Remove terminal integration from a project
	 */
	async removeTerminalIntegration(projectPath: string): Promise<void> {
		try {
			logger.info('Removing terminal integration', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
			
			await invoke('remove_terminal_integration', { projectPath });
			
			logger.info('Terminal integration removed', { 
				context: 'SDKService', 
				data: { projectPath } 
			});
		} catch (error) {
			logger.error('Failed to remove terminal integration', {
				context: 'SDKService',
				error,
				data: { projectPath }
			});
			throw error;
		}
	}

	/**
	 * Get version sources for a specific SDK type (FlyEnv-style)
	 */
	async getVersionSources(sdkType: string): Promise<SDKVersionSource[]> {
		try {
			logger.info('Getting version sources', { 
				context: 'SDKService', 
				data: { sdkType } 
			});
			
			const sources = await invoke<SDKVersionSource[]>('get_version_sources', { sdkType });
			
			logger.info('Version sources retrieved', { 
				context: 'SDKService', 
				data: { sdkType, count: sources.length } 
			});
			
			return sources;
		} catch (error) {
			// Don't treat missing sources as an error - return empty array
			logger.info('No version sources available', {
				context: 'SDKService',
				data: { sdkType, message: error instanceof Error ? error.message : 'No sources' }
			});
			return [];
		}
	}

	/**
	 * Refresh version status for a specific SDK type
	 */
	async refreshVersionStatus(sdkType: string): Promise<void> {
		try {
			logger.info('Refreshing version status', { 
				context: 'SDKService', 
				data: { sdkType } 
			});
			
			await invoke('refresh_version_status', { sdkType });
			
			logger.info('Version status refreshed', { 
				context: 'SDKService', 
				data: { sdkType } 
			});
		} catch (error) {
			logger.error('Failed to refresh version status', {
				context: 'SDKService',
				error,
				data: { sdkType }
			});
			throw error;
		}
	}

	/**
	 * Uninstall a specific SDK version
	 */
	async uninstallVersion(request: SDKInstallRequest): Promise<void> {
		try {
			logger.info('Uninstalling SDK version', { 
				context: 'SDKService', 
				data: request 
			});
			
			await invoke('uninstall_sdk_version', {
				type: request.type,
				version: request.version,
				manager: request.manager
			});
			
			logger.info('SDK version uninstalled successfully', { 
				context: 'SDKService', 
				data: request 
			});
		} catch (error) {
			logger.error('Failed to uninstall SDK version', {
				context: 'SDKService',
				error,
				data: request
			});
			throw error;
		}
	}
}

export const sdkService = SDKService.getInstance();
