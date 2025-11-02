use serde::{Deserialize, Serialize};
use k8s_openapi::api::core::v1::{Pod, Service, ConfigMap, Secret};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::batch::v1::CronJob;
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesCluster {
    pub name: String,
    pub context: String,
    pub namespace: String,
    pub status: ClusterStatus,
    pub server: Option<String>,
    pub version: Option<String>,
    pub last_connected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterStatus {
    Connected,
    Disconnected,
    Error(String),
    Connecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesResource {
    pub name: String,
    pub namespace: String,
    pub kind: String,
    pub api_version: String,
    pub status: ResourceStatus,
    pub age: Option<String>,
    pub labels: std::collections::HashMap<String, String>,
    pub annotations: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceStatus {
    Running,
    Pending,
    Failed,
    Succeeded,
    Unknown,
    Terminating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ready: String,
    pub restarts: i32,
    pub age: String,
    pub ip: Option<String>,
    pub node: Option<String>,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub cluster_ip: Option<String>,
    pub external_ip: Option<String>,
    pub ports: Vec<PortInfo>,
    pub selector: std::collections::HashMap<String, String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub name: Option<String>,
    pub port: i32,
    pub target_port: Option<String>,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub desired: i32,
    pub current: i32,
    pub up_to_date: i32,
    pub available: i32,
    pub age: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub status: String,
    pub age: String,
    pub labels: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub container: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardRequest {
    pub namespace: String,
    pub pod_name: String,
    pub local_port: u16,
    pub remote_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecRequest {
    pub namespace: String,
    pub pod_name: String,
    pub container: Option<String>,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEvent {
    pub event_type: String,
    pub resource: KubernetesResource,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub revision: i32,
    pub status: String,
    pub chart: String,
    pub app_version: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub cpu_limit: Option<f64>,
    pub memory_limit: Option<f64>,
    pub timestamp: String,
}

// Request/Response types for Tauri commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPodsRequest {
    pub namespace: Option<String>,
    pub label_selector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPodLogsRequest {
    pub namespace: String,
    pub pod_name: String,
    pub container: Option<String>,
    pub follow: bool,
    pub tail_lines: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceRequest {
    pub namespace: String,
    pub resource_yaml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceRequest {
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub resource_yaml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResourceRequest {
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleResourceRequest {
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub replicas: i32,
}
