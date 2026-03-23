/**
 * SDK Manager Domain Types
 */

export enum SDKManager {
	NVM = 'nvm',
	RUSTUP = 'rustup',
	PYENV = 'pyenv',
	SDKMAN = 'sdkman',
	GOENV = 'goenv',
	MANUAL = 'manual'
}

export enum SDKType {
	NODE = 'node',
	RUST = 'rust',
	PYTHON = 'python',
	JAVA = 'java',
	GO = 'go',
	PHP = 'php',
	RUBY = 'ruby',
	KOTLIN = 'kotlin',
	SCALA = 'scala'
}

export interface SDKVersion {
	version: string;
	installed: boolean;
	active: boolean;
	path?: string;
	installedAt?: Date;
}

export interface SDKInstallation {
	id: string;
	type: SDKType;
	manager: SDKManager;
	versions: SDKVersion[];
	activeVersion?: string;
	installed: boolean;
	path?: string;
	lastChecked: Date;
}

export interface SDK {
	id: string;
	name: string;
	type: SDKType;
	manager: SDKManager;
	description: string;
	website?: string;
	icon?: string;
	installation: SDKInstallation;
}

export interface SDKManagerInfo {
	type: SDKManager | string;
	name: string;
	display_name?: string;
	sdk_type?: string;
	category?: string;
	installed: boolean | string;
	version?: string;
	path?: string;
	description: string;
	website?: string;
}

export interface SDKInstallRequest {
	type: SDKType | string;
	version: string;
	manager: SDKManager | string;
	projectPath?: string;
	library?: string;
}

export interface SDKSwitchRequest {
	type: SDKType | string;
	version: string;
	projectPath?: string;
	global?: boolean;
	manager?: string;
}

export interface SDKDetectionResult {
	managers: SDKManagerInfo[];
	sdks: SDK[];
	errors: string[];
}

export interface SDKCommand {
	command: string;
	args: string[];
	workingDirectory?: string;
	environment?: Record<string, string>;
}

export interface SDKCommandResult {
	success: boolean;
	output: string;
	error?: string;
	exitCode: number;
	duration: number;
}

export interface SDKVersionSource {
	version: string;
	source: string;
	library?: string;
	description?: string;
	stable: boolean;
	prerelease: boolean;
	downloadUrl?: string;
	checksum?: string;
}

/**
 * Version source type
 */
export type VersionSource = 'static' | 'sdk_manager' | 'system' | 'custom';

/**
 * Environment scope
 */
export type EnvironmentScope = 'global' | 'project' | 'session';

/**
 * SDK Installation from detection
 */
export interface SDKInstallationInfo {
	version: string;
	path: string;
	source: VersionSource;
	manager?: string;
	installed: boolean;
	active: boolean;
	binaryPath?: string;
	detectedAt?: string;
}

/**
 * Detection result for a specific SDK
 */
export interface SDKDetectionInfo {
	sdkType: string;
	installations: SDKInstallationInfo[];
	activeVersion?: string;
	errors: string[];
}

/**
 * Custom path configuration
 */
export interface CustomPath {
	id?: string;
	sdkType: string;
	path: string;
	version?: string;
	enabled: boolean;
	createdAt?: string;
	updatedAt?: string;
}

/**
 * Environment variable configuration
 */
export interface EnvironmentVariable {
	name: string;
	value: string;
	scope: EnvironmentScope;
	isExported: boolean;
	sdkType?: string;
	createdAt?: string;
	updatedAt?: string;
}

/**
 * Environment configuration
 */
export interface EnvironmentConfig {
	sdkType: string;
	pathEntries: PathEntry[];
	environmentVariables: EnvironmentVariable[];
	aliases: VersionAlias[];
	pathManagedBy: 'app' | 'system' | 'none';
	lastUpdated: string;
}

/**
 * PATH entry
 */
export interface PathEntry {
	path: string;
	sdkType: string;
	version: string;
	scope: EnvironmentScope;
	isActive: boolean;
	priority: number;
}

/**
 * Version alias
 */
export interface VersionAlias {
	id?: string;
	sdkType: string;
	version: string;
	alias: string;
	createdAt?: string;
	updatedAt?: string;
}

/**
 * Environment status
 */
export interface EnvironmentStatus {
	sdkType: string;
	currentVersion?: string;
	pathManagedBy: 'app' | 'system' | 'none';
	binariesInPath: string[];
	environmentVariables: EnvironmentVariable[];
	lastUpdated: string;
}
