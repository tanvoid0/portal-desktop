// Core abstract types and interfaces for cloud providers
// These define the contract that all cloud providers must implement

export enum CloudProviderType {
  GCP = 'gcp',
  AWS = 'aws',
  AZURE = 'azure',
  DIGITAL_OCEAN = 'digital-ocean'
}

export enum ResourceType {
  POD = 'pod',
  SERVICE = 'service',
  DEPLOYMENT = 'deployment',
  STATEFULSET = 'statefulset',
  DAEMONSET = 'daemonset',
  JOB = 'job',
  CRONJOB = 'cronjob',
  CONFIGMAP = 'configmap',
  SECRET = 'secret',
  INGRESS = 'ingress',
  NAMESPACE = 'namespace'
}

export enum ResourceStatus {
  RUNNING = 'running',
  PENDING = 'pending',
  FAILED = 'failed',
  SUCCEEDED = 'succeeded',
  UNKNOWN = 'unknown',
  TERMINATING = 'terminating'
}

export enum ClusterStatus {
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  ERROR = 'error',
  READY = 'ready'
}

export interface ICluster {
  id: string;
  name: string;
  provider: CloudProviderType;
  status: ClusterStatus;
  region?: string;
  context?: string;
  namespace?: string;
  server?: string;
  version?: string;
  metadata: Record<string, any>;
}

export interface ICloudResource {
  id: string;
  name: string;
  namespace: string;
  type: ResourceType;
  status: ResourceStatus;
  metadata: Record<string, any>;
  provider: CloudProviderType;
  
  // Common operations (optional - not all resources support all operations)
  getLogs?(): Promise<string>;
  exec?(command: string[]): Promise<string>;
  delete?(): Promise<void>;
  scale?(replicas: number): Promise<void>;
}

export interface ResourceEvent {
  type: 'added' | 'modified' | 'deleted';
  resource: ICloudResource;
}

export interface ProviderFeature {
  name: string;
  description: string;
  enabled: boolean;
}

// Base interface that all cloud providers must implement
export interface ICloudProvider {
  readonly name: string;
  readonly type: CloudProviderType;
  
  // Connection management
  initialize(): Promise<void>;
  connect(clusterId: string): Promise<void>;
  disconnect(): Promise<void>;
  isConnected(): Promise<boolean>;
  
  // Cluster operations
  listClusters(): Promise<ICluster[]>;
  getCluster(id: string): Promise<ICluster | null>;
  getCurrentCluster(): Promise<ICluster | null>;
  
  // Resource operations (provider-agnostic interface)
  listResources(type: ResourceType, namespace?: string): Promise<ICloudResource[]>;
  getResource(type: ResourceType, id: string, namespace: string): Promise<ICloudResource | null>;
  watchResources(type: ResourceType, namespace?: string): AsyncIterable<ResourceEvent>;
  
  // Namespace operations
  listNamespaces(): Promise<string[]>;
  
  // Provider-specific operations (optional)
  getProviderSpecificFeatures?(): ProviderFeature[];
}

