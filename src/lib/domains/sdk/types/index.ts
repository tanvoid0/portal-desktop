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
