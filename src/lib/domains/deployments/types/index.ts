/**
 * Deployments Domain Types
 */

export enum DeploymentStatus {
	CREATING = 'creating',
	RUNNING = 'running',
	STOPPED = 'stopped',
	FAILED = 'failed',
	RESTARTING = 'restarting',
	REMOVING = 'removing'
}

export enum ContainerStatus {
	CREATED = 'created',
	RUNNING = 'running',
	PAUSED = 'paused',
	RESTARTING = 'restarting',
	REMOVING = 'removing',
	DEAD = 'dead',
	EXITED = 'exited'
}

export enum ProjectType {
	NODE = 'node',
	RUST = 'rust',
	PYTHON = 'python',
	JAVA = 'java',
	GO = 'go',
	PHP = 'php',
	RUBY = 'ruby',
	STATIC = 'static'
}

export interface DockerContainer {
	id: string;
	name: string;
	image: string;
	status: ContainerStatus;
	ports: PortMapping[];
	volumes: VolumeMapping[];
	environment: Record<string, string>;
	createdAt: Date;
	startedAt?: Date;
	stoppedAt?: Date;
	restartCount: number;
	resourceUsage: ResourceUsage;
}

export interface PortMapping {
	hostPort: number;
	containerPort: number;
	protocol: 'tcp' | 'udp';
}

export interface VolumeMapping {
	hostPath: string;
	containerPath: string;
	mode: 'ro' | 'rw';
}

export interface ResourceUsage {
	cpuPercent: number;
	memoryBytes: number;
	memoryPercent: number;
	networkRxBytes: number;
	networkTxBytes: number;
	blockReadBytes: number;
	blockWriteBytes: number;
}

export interface EnvironmentConfig {
	variables: Record<string, string>;
	secrets: string[]; // References to credential IDs
	volumes: VolumeMapping[];
	ports: PortMapping[];
	networks: string[];
}

export interface BuildScript {
	id: string;
	name: string;
	projectType: ProjectType;
	version: string;
	commands: string[];
	environment: Record<string, string>;
	dockerfile: string;
	context: string;
	output: string;
	createdAt: Date;
	updatedAt: Date;
}

export interface Deployment {
	id: string;
	name: string;
	description?: string;
	projectPath: string;
	projectType: ProjectType;
	status: DeploymentStatus;
	container?: DockerContainer;
	environment: EnvironmentConfig;
	buildScript?: BuildScript;
	createdAt: Date;
	updatedAt: Date;
	startedAt?: Date;
	stoppedAt?: Date;
	lastHealthCheck?: Date;
	healthStatus: HealthStatus;
	metadata: DeploymentMetadata;
}

export interface HealthStatus {
	status: 'healthy' | 'unhealthy' | 'unknown';
	lastCheck: Date;
	responseTime?: number;
	error?: string;
	checks: HealthCheck[];
}

export interface HealthCheck {
	name: string;
	type: 'http' | 'tcp' | 'command';
	endpoint?: string;
	command?: string;
	interval: number; // seconds
	timeout: number; // seconds
	retries: number;
	status: 'passing' | 'failing' | 'unknown';
	lastCheck: Date;
	responseTime?: number;
	error?: string;
}

export interface DeploymentMetadata {
	projectId?: string;
	repository?: string;
	branch?: string;
	commit?: string;
	tags: string[];
	labels: Record<string, string>;
	notes?: string;
	createdBy: string;
	team?: string;
	environment: 'development' | 'staging' | 'production';
}

export interface DeploymentCreateRequest {
	name: string;
	description?: string;
	projectPath: string;
	projectType: ProjectType;
	environment: EnvironmentConfig;
	buildScript?: string; // Build script ID
	metadata?: Partial<DeploymentMetadata>;
}

export interface DeploymentUpdateRequest {
	name?: string;
	description?: string;
	environment?: Partial<EnvironmentConfig>;
	metadata?: Partial<DeploymentMetadata>;
}

export interface DeploymentLogs {
	deploymentId: string;
	containerId: string;
	logs: LogEntry[];
	totalLines: number;
	hasMore: boolean;
	lastTimestamp: Date;
}

export interface LogEntry {
	timestamp: Date;
	level: 'info' | 'warn' | 'error' | 'debug';
	message: string;
	source: 'stdout' | 'stderr' | 'system';
	containerId?: string;
}

export interface DeploymentStats {
	deploymentId: string;
	uptime: number; // seconds
	restartCount: number;
	totalRequests: number;
	averageResponseTime: number;
	errorRate: number;
	resourceUsage: ResourceUsage;
	healthChecks: HealthCheck[];
}

export interface DockerImage {
	id: string;
	name: string;
	tag: string;
	size: number;
	createdAt: Date;
	layers: ImageLayer[];
	architecture: string;
	os: string;
}

export interface ImageLayer {
	id: string;
	size: number;
	command: string;
	createdAt: Date;
}

export interface DockerNetwork {
	id: string;
	name: string;
	driver: string;
	scope: string;
	ipam: NetworkIPAM;
	containers: string[];
	createdAt: Date;
}

export interface NetworkIPAM {
	driver: string;
	config: NetworkConfig[];
}

export interface NetworkConfig {
	subnet: string;
	gateway?: string;
	ipRange?: string;
}

export interface DeploymentTemplate {
	id: string;
	name: string;
	description: string;
	projectType: ProjectType;
	environment: EnvironmentConfig;
	buildScript: string;
	dockerfile: string;
	healthChecks: HealthCheck[];
	metadata: Record<string, string>;
	createdAt: Date;
	updatedAt: Date;
}

export interface DeploymentAction {
	type: 'start' | 'stop' | 'restart' | 'remove' | 'rebuild';
	deploymentId: string;
	reason?: string;
	force?: boolean;
}

export interface DeploymentActionResult {
	success: boolean;
	message: string;
	error?: string;
	actionId: string;
	startedAt: Date;
	completedAt?: Date;
}
