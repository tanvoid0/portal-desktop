// GCP-specific type definitions matching backend Rust types
// These are the raw types from Tauri commands, before mapping to abstract types

export interface KubernetesCluster {
  name: string;
  context: string;
  namespace: string;
  status: 'Connected' | 'Disconnected' | 'Connecting' | { Error: string };
  server?: string;
  version?: string;
  last_connected?: string;
}

export interface PodInfo {
  name: string;
  namespace: string;
  status: string;
  ready: string;
  restarts: number;
  age: string;
  ip?: string;
  node?: string;
  containers: ContainerInfo[];
}

export interface ContainerInfo {
  name: string;
  image: string;
  ready: boolean;
  restart_count: number;
  state: string;
}

export interface ServiceInfo {
  name: string;
  namespace: string;
  cluster_ip?: string;
  external_ip?: string;
  ports: PortInfo[];
  selector: Record<string, string>;
  age: string;
}

export interface PortInfo {
  name?: string;
  port: number;
  target_port?: string;
  protocol: string;
}

export interface DeploymentInfo {
  name: string;
  namespace: string;
  desired: number;
  current: number;
  up_to_date: number;
  available: number;
  age: string;
  labels: Record<string, string>;
}

export interface NamespaceInfo {
  name: string;
  status: string;
  age: string;
  labels: Record<string, string>;
}

export interface PortForwardInfo {
  id: string;
  namespace: string;
  pod_name: string;
  local_port: number;
  remote_port: number;
  status: string;
  created_at: string;
  url: string;
}

