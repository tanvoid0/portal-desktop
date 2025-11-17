use kube::{Client, Config, Api};
use kube::api::{ListParams, LogParams, Patch, PatchParams, PostParams};
use kube::runtime::watcher::{watcher, Config as WatcherConfig, Event};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Window, Emitter};
use crate::domains::kubernetes::types::*;
use k8s_openapi::api::core::v1::{Pod, Service, Namespace, ConfigMap, Secret};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use k8s_openapi::api::networking::v1::Ingress;
use std::sync::OnceLock;
use tokio::process::Child;
use tokio::task::JoinHandle;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;

// Static Kubernetes client using OnceLock (thread-safe initialization)
static K8S_CLIENT: OnceLock<Client> = OnceLock::new();

// Global storage for active port forward processes
type PortForwardMap = Arc<Mutex<HashMap<String, (Child, PortForwardInfo)>>>;
static PORT_FORWARDS: OnceLock<PortForwardMap> = OnceLock::new();

// Global storage for active watch tasks
type WatchTaskMap = Arc<Mutex<HashMap<String, JoinHandle<()>>>>;
static WATCH_TASKS: OnceLock<WatchTaskMap> = OnceLock::new();

pub struct KubernetesManager {
    pub(crate) current_cluster: Option<KubernetesCluster>,
    kubeconfig_path: Option<String>,
}

impl KubernetesManager {
    pub fn new() -> Self {
        Self {
            current_cluster: None,
            kubeconfig_path: None,
        }
    }

    // Get the static Kubernetes client
    fn get_client() -> Result<Client, String> {
        K8S_CLIENT.get()
            .cloned()
            .ok_or_else(|| "Kubernetes client not initialized".to_string())
    }

    pub async fn initialize() -> Result<(), String> {
        // If already initialized, return success
        if K8S_CLIENT.get().is_some() {
            return Ok(());
        }

        match Client::try_default().await {
            Ok(client) => {
                K8S_CLIENT.set(client)
                    .map_err(|_| "K8S client already initialized".to_string())?;
                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize Kubernetes client: {}", e))
        }
    }

    pub async fn load_clusters(&self) -> Result<Vec<KubernetesCluster>, String> {
        // Try to load kubeconfig using Config::infer which automatically finds kubeconfig
        match Config::infer().await {
            Ok(config) => {
                let mut clusters = Vec::new();
                
                // Try to get current context
                if let Ok(client) = Client::try_from(config.clone()) {
                    // Test connection by getting API server version
                    if let Ok(version) = client.apiserver_version().await {
                        let cluster = KubernetesCluster {
                            name: "default".to_string(),
                            context: "default".to_string(),
                            namespace: config.default_namespace.clone(),
                            status: ClusterStatus::Connected,
                            server: Some(config.cluster_url.to_string()),
                            version: Some(version.git_version),
                            last_connected: Some(chrono::Utc::now().to_rfc3339()),
                        };
                        clusters.push(cluster);
                    }
                }

                Ok(clusters)
            }
            Err(_) => {
                // No kubeconfig found or invalid, return empty list
                Ok(vec![])
            }
        }
    }

    pub async fn connect_cluster(&mut self, cluster_name: &str) -> Result<(), String> {
        // Ensure client is initialized
        Self::initialize().await?;
        
        let client = Self::get_client()?;
        
        // Test connection
        match client.apiserver_version().await {
            Ok(version) => {
                let config = Config::infer().await
                    .map_err(|e| format!("Failed to load kubeconfig: {}", e))?;
                
                self.current_cluster = Some(KubernetesCluster {
                    name: cluster_name.to_string(),
                    context: "default".to_string(),
                    namespace: config.default_namespace.clone(),
                    status: ClusterStatus::Connected,
                    server: Some(config.cluster_url.to_string()),
                    version: Some(version.git_version),
                    last_connected: Some(chrono::Utc::now().to_rfc3339()),
                });
                Ok(())
            }
            Err(e) => Err(format!("Failed to connect to cluster: {}", e))
        }
    }

    pub async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<PodInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<Pod> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(pods) => {
                let mut pod_infos = Vec::new();
                for pod in pods.items {
                    let pod_info = self.pod_to_info(&pod);
                    pod_infos.push(pod_info);
                }
                Ok(pod_infos)
            }
            Err(e) => Err(format!("Failed to list pods: {}", e))
        }
    }

    pub async fn get_pod_logs(
        &self,
        namespace: &str,
        pod_name: &str,
        container: Option<&str>,
        follow: bool,
        tail_lines: Option<i32>,
    ) -> Result<String, String> {
        let client = Self::get_client()?;

        let api: Api<Pod> = Api::namespaced(client, namespace);
        
        let mut log_params = LogParams::default();
        if let Some(container_name) = container {
            log_params.container = Some(container_name.to_string());
        }
        if let Some(tail) = tail_lines {
            log_params.tail_lines = Some(tail as i64);
        }
        log_params.follow = follow;

        match api.logs(pod_name, &log_params).await {
            Ok(logs) => Ok(logs),
            Err(e) => Err(format!("Failed to get pod logs: {}", e))
        }
    }

    pub async fn watch_pods(&self, namespace: &str, window: Window) -> Result<(), String> {
        // Stop existing watch if any
        self.stop_watch("pods", namespace);
        
        let client = Self::get_client()?;
        let api: Api<Pod> = Api::namespaced(client, namespace);
        
        let stream = watcher(api, WatcherConfig::default());
        let mut stream = Box::pin(stream);
        
        // Clone window for use in spawned task
        let window_clone = window.clone();
        let namespace_str = namespace.to_string();
        
        // Spawn a task to watch for events with error recovery
        let handle = tokio::spawn(async move {
            loop {
                match stream.next().await {
                    Some(Ok(Event::Applied(pod))) => {
                        let pod_info = Self::pod_to_info_static(&pod);
                        if let Err(e) = window_clone.emit("k8s:pod-updated", &pod_info) {
                            eprintln!("Failed to emit pod update: {}", e);
                        }
                    }
                    Some(Ok(Event::Deleted(pod))) => {
                        let pod_info = Self::pod_to_info_static(&pod);
                        if let Err(e) = window_clone.emit("k8s:pod-deleted", &pod_info) {
                            eprintln!("Failed to emit pod deletion: {}", e);
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore other event types
                    }
                    Some(Err(e)) => {
                        let error_str = format!("{}", e);
                        let error_msg = format!("Watch error for pods in {}: {}", namespace_str, error_str);
                        eprintln!("{}", error_msg);
                        if let Err(emit_err) = window_clone.emit("k8s:watch-error", &error_msg) {
                            eprintln!("Failed to emit watch error: {}", emit_err);
                        }
                        
                        // Check if it's a connection error - these are fatal and we should stop
                        if error_str.contains("tcp connect error") 
                            || error_str.contains("Cannot assign requested address")
                            || error_str.contains("error trying to connect")
                            || error_str.contains("connection refused")
                            || error_str.contains("connection reset") {
                            eprintln!("Fatal connection error detected, stopping watch for pods in {}", namespace_str);
                            break;
                        }
                        
                        // For other errors, wait a bit before continuing (stream might recover)
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                    None => {
                        eprintln!("Watch stream ended for pods in {}", namespace_str);
                        break;
                    }
                }
            }
        });
        
        // Store the task handle
        let watch_tasks = WATCH_TASKS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let task_key = format!("pods:{}", namespace);
        watch_tasks.lock().unwrap().insert(task_key, handle);
        
        Ok(())
    }

    pub async fn watch_services(&self, namespace: &str, window: Window) -> Result<(), String> {
        // Stop existing watch if any
        self.stop_watch("services", namespace);
        
        let client = Self::get_client()?;
        let api: Api<Service> = Api::namespaced(client, namespace);
        
        let stream = watcher(api, WatcherConfig::default());
        let mut stream = Box::pin(stream);
        
        // Clone window for use in spawned task
        let window_clone = window.clone();
        let namespace_str = namespace.to_string();
        
        // Spawn a task to watch for events with error recovery
        let handle = tokio::spawn(async move {
            loop {
                match stream.next().await {
                    Some(Ok(Event::Applied(service))) => {
                        let service_info = Self::service_to_info_static(&service);
                        if let Err(e) = window_clone.emit("k8s:service-updated", &service_info) {
                            eprintln!("Failed to emit service update: {}", e);
                        }
                    }
                    Some(Ok(Event::Deleted(service))) => {
                        let service_info = Self::service_to_info_static(&service);
                        if let Err(e) = window_clone.emit("k8s:service-deleted", &service_info) {
                            eprintln!("Failed to emit service deletion: {}", e);
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore other event types
                    }
                    Some(Err(e)) => {
                        let error_str = format!("{}", e);
                        let error_msg = format!("Watch error for services in {}: {}", namespace_str, error_str);
                        eprintln!("{}", error_msg);
                        if let Err(emit_err) = window_clone.emit("k8s:watch-error", &error_msg) {
                            eprintln!("Failed to emit watch error: {}", emit_err);
                        }
                        
                        // Check if it's a connection error - these are fatal and we should stop
                        if error_str.contains("tcp connect error") 
                            || error_str.contains("Cannot assign requested address")
                            || error_str.contains("error trying to connect")
                            || error_str.contains("connection refused")
                            || error_str.contains("connection reset") {
                            eprintln!("Fatal connection error detected, stopping watch for services in {}", namespace_str);
                            break;
                        }
                        
                        // For other errors, wait a bit before continuing (stream might recover)
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                    None => {
                        eprintln!("Watch stream ended for services in {}", namespace_str);
                        break;
                    }
                }
            }
        });
        
        // Store the task handle
        let watch_tasks = WATCH_TASKS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let task_key = format!("services:{}", namespace);
        watch_tasks.lock().unwrap().insert(task_key, handle);
        
        Ok(())
    }

    pub async fn watch_deployments(&self, namespace: &str, window: Window) -> Result<(), String> {
        // Stop existing watch if any
        self.stop_watch("deployments", namespace);
        
        let client = Self::get_client()?;
        let api: Api<Deployment> = Api::namespaced(client, namespace);
        
        let stream = watcher(api, WatcherConfig::default());
        let mut stream = Box::pin(stream);
        
        // Clone window for use in spawned task
        let window_clone = window.clone();
        let namespace_str = namespace.to_string();
        
        // Spawn a task to watch for events with error recovery
        let handle = tokio::spawn(async move {
            loop {
                match stream.next().await {
                    Some(Ok(Event::Applied(deployment))) => {
                        let deployment_info = Self::deployment_to_info_static(&deployment);
                        if let Err(e) = window_clone.emit("k8s:deployment-updated", &deployment_info) {
                            eprintln!("Failed to emit deployment update: {}", e);
                        }
                    }
                    Some(Ok(Event::Deleted(deployment))) => {
                        let deployment_info = Self::deployment_to_info_static(&deployment);
                        if let Err(e) = window_clone.emit("k8s:deployment-deleted", &deployment_info) {
                            eprintln!("Failed to emit deployment deletion: {}", e);
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore other event types
                    }
                    Some(Err(e)) => {
                        let error_str = format!("{}", e);
                        let error_msg = format!("Watch error for deployments in {}: {}", namespace_str, error_str);
                        eprintln!("{}", error_msg);
                        if let Err(emit_err) = window_clone.emit("k8s:watch-error", &error_msg) {
                            eprintln!("Failed to emit watch error: {}", emit_err);
                        }
                        
                        // Check if it's a connection error - these are fatal and we should stop
                        if error_str.contains("tcp connect error") 
                            || error_str.contains("Cannot assign requested address")
                            || error_str.contains("error trying to connect")
                            || error_str.contains("connection refused")
                            || error_str.contains("connection reset") {
                            eprintln!("Fatal connection error detected, stopping watch for deployments in {}", namespace_str);
                            break;
                        }
                        
                        // For other errors, wait a bit before continuing (stream might recover)
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                    None => {
                        eprintln!("Watch stream ended for deployments in {}", namespace_str);
                        break;
                    }
                }
            }
        });
        
        // Store the task handle
        let watch_tasks = WATCH_TASKS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let task_key = format!("deployments:{}", namespace);
        watch_tasks.lock().unwrap().insert(task_key, handle);
        
        Ok(())
    }

    pub async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<ServiceInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<Service> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(services) => {
                let mut service_infos = Vec::new();
                for service in services.items {
                    let service_info = self.service_to_info(&service);
                    service_infos.push(service_info);
                }
                Ok(service_infos)
            }
            Err(e) => Err(format!("Failed to list services: {}", e))
        }
    }

    pub async fn list_deployments(&self, namespace: Option<&str>) -> Result<Vec<DeploymentInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<Deployment> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(deployments) => {
                let mut deployment_infos = Vec::new();
                for deployment in deployments.items {
                    let deployment_info = self.deployment_to_info(&deployment);
                    deployment_infos.push(deployment_info);
                }
                Ok(deployment_infos)
            }
            Err(e) => Err(format!("Failed to list deployments: {}", e))
        }
    }

    pub async fn list_statefulsets(&self, namespace: Option<&str>) -> Result<Vec<StatefulSetInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<StatefulSet> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(statefulsets) => {
                let mut statefulset_infos = Vec::new();
                for statefulset in statefulsets.items {
                    let statefulset_info = self.statefulset_to_info(&statefulset);
                    statefulset_infos.push(statefulset_info);
                }
                Ok(statefulset_infos)
            }
            Err(e) => Err(format!("Failed to list statefulsets: {}", e))
        }
    }

    pub async fn list_daemonsets(&self, namespace: Option<&str>) -> Result<Vec<DaemonSetInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<DaemonSet> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(daemonsets) => {
                let mut daemonset_infos = Vec::new();
                for daemonset in daemonsets.items {
                    let daemonset_info = self.daemonset_to_info(&daemonset);
                    daemonset_infos.push(daemonset_info);
                }
                Ok(daemonset_infos)
            }
            Err(e) => Err(format!("Failed to list daemonsets: {}", e))
        }
    }

    pub async fn list_jobs(&self, namespace: Option<&str>) -> Result<Vec<JobInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<Job> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(jobs) => {
                let mut job_infos = Vec::new();
                for job in jobs.items {
                    let job_info = self.job_to_info(&job);
                    job_infos.push(job_info);
                }
                Ok(job_infos)
            }
            Err(e) => Err(format!("Failed to list jobs: {}", e)),
        }
    }

    pub async fn list_events(&self, namespace: Option<&str>) -> Result<Vec<EventInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        type K8sEvent = k8s_openapi::api::core::v1::Event;
        let api: Api<K8sEvent> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(events) => {
                let mut event_infos = Vec::new();
                for event in events.items {
                    let event_info = self.event_to_info(&event);
                    event_infos.push(event_info);
                }
                Ok(event_infos)
            }
            Err(e) => Err(format!("Failed to list events: {}", e))
        }
    }

    pub async fn list_cronjobs(&self, namespace: Option<&str>) -> Result<Vec<CronJobInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<CronJob> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(cronjobs) => {
                let mut cronjob_infos = Vec::new();
                for cronjob in cronjobs.items {
                    let cronjob_info = self.cronjob_to_info(&cronjob);
                    cronjob_infos.push(cronjob_info);
                }
                Ok(cronjob_infos)
            }
            Err(e) => Err(format!("Failed to list cronjobs: {}", e)),
        }
    }

    pub async fn list_ingresses(&self, namespace: Option<&str>) -> Result<Vec<IngressInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<Ingress> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(ingresses) => {
                let mut ingress_infos = Vec::new();
                for ingress in ingresses.items {
                    let ingress_info = self.ingress_to_info(&ingress);
                    ingress_infos.push(ingress_info);
                }
                Ok(ingress_infos)
            }
            Err(e) => Err(format!("Failed to list ingresses: {}", e)),
        }
    }

    pub async fn list_configmaps(&self, namespace: Option<&str>) -> Result<Vec<ConfigMapInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<ConfigMap> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(configmaps) => {
                let mut configmap_infos = Vec::new();
                for configmap in configmaps.items {
                    let configmap_info = self.configmap_to_info(&configmap);
                    configmap_infos.push(configmap_info);
                }
                Ok(configmap_infos)
            }
            Err(e) => Err(format!("Failed to list configmaps: {}", e)),
        }
    }

    pub async fn list_secrets(&self, namespace: Option<&str>) -> Result<Vec<SecretInfo>, String> {
        let client = Self::get_client()?;

        let namespace = namespace.unwrap_or("default");
        let api: Api<Secret> = Api::namespaced(client, namespace);

        match api.list(&ListParams::default()).await {
            Ok(secrets) => {
                let mut secret_infos = Vec::new();
                for secret in secrets.items {
                    let secret_info = self.secret_to_info(&secret);
                    secret_infos.push(secret_info);
                }
                Ok(secret_infos)
            }
            Err(e) => Err(format!("Failed to list secrets: {}", e)),
        }
    }

    pub async fn list_namespaces(&self) -> Result<Vec<NamespaceInfo>, String> {
        let client = Self::get_client()?;

        let api: Api<Namespace> = Api::all(client);

        match api.list(&ListParams::default()).await {
            Ok(namespaces) => {
                let mut namespace_infos = Vec::new();
                for namespace in namespaces.items {
                    let namespace_info = self.namespace_to_info(&namespace);
                    namespace_infos.push(namespace_info);
                }
                Ok(namespace_infos)
            }
            Err(e) => Err(format!("Failed to list namespaces: {}", e))
        }
    }

    pub fn get_current_cluster(&self) -> Option<&KubernetesCluster> {
        self.current_cluster.as_ref()
    }

    pub fn is_connected(&self) -> bool {
        K8S_CLIENT.get().is_some()
    }

    // Helper methods to convert K8s resources to our types
    fn pod_to_info(&self, pod: &Pod) -> PodInfo {
        Self::pod_to_info_static(pod)
    }

    fn pod_to_info_static(pod: &Pod) -> PodInfo {
        let metadata = &pod.metadata;
        let status = pod.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let pod_status = status.and_then(|s| s.phase.clone()).unwrap_or_else(|| "Unknown".to_string());
        let pod_ip = status.and_then(|s| s.pod_ip.clone());
        let node = status.and_then(|s| s.host_ip.clone());
        
        let containers: Vec<ContainerInfo> = status
            .and_then(|s| s.container_statuses.as_ref())
            .map(|statuses| {
                statuses.iter().map(|cs| {
                    ContainerInfo {
                        name: cs.name.clone(),
                        image: cs.image.clone(),
                        ready: cs.ready,
                        restart_count: cs.restart_count,
                        state: format!("{:?}", cs.state),
                    }
                }).collect()
            })
            .unwrap_or_default();

        PodInfo {
            name,
            namespace,
            status: pod_status,
            ready: format!("{}/{}", 
                containers.iter().filter(|c| c.ready).count(),
                containers.len()
            ),
            restarts: containers.iter().map(|c| c.restart_count).sum(),
            age: "Unknown".to_string(), // Would need to calculate from creation timestamp
            ip: pod_ip,
            node,
            containers,
        }
    }

    fn service_to_info(&self, service: &Service) -> ServiceInfo {
        Self::service_to_info_static(service)
    }

    fn service_to_info_static(service: &Service) -> ServiceInfo {
        let metadata = &service.metadata;
        let spec = service.spec.as_ref();
        let status = service.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());

        let cluster_ip = spec.and_then(|s| s.cluster_ip.clone());
        let external_ip = status
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .and_then(|ingresses| ingresses.first())
            .and_then(|ingress| ingress.ip.clone());

        let ports = spec
            .and_then(|s| s.ports.as_ref())
            .map(|ports| {
                ports.iter().map(|p| {
                    PortInfo {
                        name: p.name.clone(),
                        port: p.port,
                        target_port: p.target_port.as_ref().map(|tp| format!("{:?}", tp)),
                        protocol: p.protocol.as_ref().unwrap_or(&"TCP".to_string()).clone(),
                    }
                }).collect()
            })
            .unwrap_or_default();

        let selector: HashMap<String, String> = spec
            .and_then(|s| s.selector.as_ref())
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        ServiceInfo {
            name,
            namespace,
            cluster_ip,
            external_ip,
            ports,
            selector,
            age: "Unknown".to_string(),
        }
    }

    fn deployment_to_info(&self, deployment: &Deployment) -> DeploymentInfo {
        Self::deployment_to_info_static(deployment)
    }

    fn deployment_to_info_static(deployment: &Deployment) -> DeploymentInfo {
        let metadata = &deployment.metadata;
        let status = deployment.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        let desired = status
            .and_then(|s| s.replicas)
            .unwrap_or(0);
        
        let current = status
            .and_then(|s| s.replicas)
            .unwrap_or(0);
        
        let up_to_date = status
            .and_then(|s| s.updated_replicas)
            .unwrap_or(0);
        
        let available = status
            .and_then(|s| s.available_replicas)
            .unwrap_or(0);

        DeploymentInfo {
            name,
            namespace,
            desired,
            current,
            up_to_date,
            available,
            age: "Unknown".to_string(),
            labels,
        }
    }

    fn statefulset_to_info(&self, statefulset: &StatefulSet) -> StatefulSetInfo {
        Self::statefulset_to_info_static(statefulset)
    }

    fn statefulset_to_info_static(statefulset: &StatefulSet) -> StatefulSetInfo {
        let metadata = &statefulset.metadata;
        let status = statefulset.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        let desired = status
            .map(|s| s.replicas)
            .unwrap_or(0);
        
        let current = status
            .map(|s| s.replicas)
            .unwrap_or(0);
        
        let ready = status
            .and_then(|s| s.ready_replicas)
            .unwrap_or(0);

        StatefulSetInfo {
            name,
            namespace,
            desired,
            current,
            ready,
            age: "Unknown".to_string(),
            labels,
        }
    }

    fn daemonset_to_info(&self, daemonset: &DaemonSet) -> DaemonSetInfo {
        Self::daemonset_to_info_static(daemonset)
    }

    fn daemonset_to_info_static(daemonset: &DaemonSet) -> DaemonSetInfo {
        let metadata = &daemonset.metadata;
        let status = daemonset.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        let desired = status
            .map(|s| s.desired_number_scheduled)
            .unwrap_or(0);
        
        let current = status
            .map(|s| s.current_number_scheduled)
            .unwrap_or(0);
        
        let ready = status
            .map(|s| s.number_ready)
            .unwrap_or(0);
        
        let up_to_date = status
            .and_then(|s| s.updated_number_scheduled)
            .unwrap_or(0);
        
        let available = status
            .and_then(|s| s.number_available)
            .unwrap_or(0);

        DaemonSetInfo {
            name,
            namespace,
            desired,
            current,
            ready,
            up_to_date,
            available,
            age: "Unknown".to_string(),
            labels,
        }
    }

    fn job_to_info(&self, job: &Job) -> JobInfo {
        let metadata = &job.metadata;
        let status = job.status.as_ref();
        let spec = job.spec.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let completions = spec.and_then(|s| s.completions).unwrap_or(1);
        let parallelism = spec.and_then(|s| s.parallelism);
        let backoff_limit = spec.and_then(|s| s.backoff_limit);
        
        let succeeded = status.and_then(|s| s.succeeded).unwrap_or(0);
        let failed = status.and_then(|s| s.failed).unwrap_or(0);
        let active = status.and_then(|s| s.active).unwrap_or(0);
        
        // Determine job status
        let job_status = if succeeded >= completions {
            "succeeded".to_string()
        } else if failed > 0 && backoff_limit.map(|l| failed > l).unwrap_or(false) {
            "failed".to_string()
        } else if active > 0 {
            "running".to_string()
        } else {
            "pending".to_string()
        };
        
        // Extract image from spec
        let image = spec
            .and_then(|s| s.template.spec.as_ref())
            .and_then(|s| s.containers.first())
            .and_then(|c| c.image.clone());

        JobInfo {
            name,
            namespace,
            status: job_status,
            completions,
            succeeded,
            failed,
            active,
            parallelism,
            backoff_limit,
            age: "Unknown".to_string(),
            image,
        }
    }

    fn cronjob_to_info(&self, cronjob: &CronJob) -> CronJobInfo {
        let metadata = &cronjob.metadata;
        let spec = cronjob.spec.as_ref();
        let status = cronjob.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let schedule = spec.map(|s| s.schedule.clone()).unwrap_or_default();
        let suspend = spec.and_then(|s| s.suspend).unwrap_or(false);
        
        let active = status.and_then(|s| s.active.as_ref()).map(|v| v.len() as i32).unwrap_or(0);
        let last_schedule_time = status.and_then(|s| s.last_schedule_time.as_ref())
            .map(|t| t.0.to_rfc3339());
        let last_successful_time = status.and_then(|s| s.last_successful_time.as_ref())
            .map(|t| t.0.to_rfc3339());
        
        // Extract image from spec
        let image = spec
            .and_then(|s| s.job_template.spec.as_ref())
            .and_then(|s| s.template.spec.as_ref())
            .and_then(|s| s.containers.first())
            .and_then(|c| c.image.clone());

        CronJobInfo {
            name,
            namespace,
            schedule,
            suspend,
            active,
            last_schedule_time,
            last_successful_time,
            age: "Unknown".to_string(),
            image,
        }
    }

    fn ingress_to_info(&self, ingress: &Ingress) -> IngressInfo {
        let metadata = &ingress.metadata;
        let spec = ingress.spec.as_ref();
        let status = ingress.status.as_ref();
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let class = spec.and_then(|s| s.ingress_class_name.clone());
        
        let addresses: Vec<String> = status
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .map(|ingresses| {
                ingresses.iter()
                    .filter_map(|ing| ing.ip.as_ref().or(ing.hostname.as_ref()))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        
        let ports: Vec<String> = spec
            .and_then(|s| s.rules.as_ref())
            .map(|rules| {
                rules.iter()
                    .filter_map(|rule| rule.host.clone())
                    .collect()
            })
            .unwrap_or_default();
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        IngressInfo {
            name,
            namespace,
            class,
            addresses,
            ports,
            age: "Unknown".to_string(),
            labels,
        }
    }

    fn configmap_to_info(&self, configmap: &ConfigMap) -> ConfigMapInfo {
        let metadata = &configmap.metadata;
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let data: std::collections::HashMap<String, String> = configmap.data.as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        ConfigMapInfo {
            name,
            namespace,
            data,
            age: "Unknown".to_string(),
            labels,
        }
    }

    fn secret_to_info(&self, secret: &Secret) -> SecretInfo {
        let metadata = &secret.metadata;
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        // Secrets store data as base64-encoded bytes in the API (ByteString type)
        let data: std::collections::HashMap<String, String> = secret.data.as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| {
                    // v.0 is the Vec<u8> that's already base64-encoded in Kubernetes
                    // We need to decode it first, then re-encode for transmission
                    let base64_str = general_purpose::STANDARD.encode(&v.0);
                    (k.clone(), base64_str)
                }).collect()
            })
            .unwrap_or_default();
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        let secret_type = secret.type_.as_ref().cloned();

        SecretInfo {
            name,
            namespace,
            data,
            age: "Unknown".to_string(),
            labels,
            secret_type,
        }
    }

    fn event_to_info(&self, event: &k8s_openapi::api::core::v1::Event) -> EventInfo {
        let metadata = &event.metadata;
        let involved_object = &event.involved_object;
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let namespace = metadata.namespace.as_ref().cloned().unwrap_or("default".to_string());
        
        let kind = involved_object.kind.clone().unwrap_or_default();
        let obj_name = involved_object.name.clone().unwrap_or_default();
        let obj_namespace = involved_object.namespace.clone();
        let obj_uid = involved_object.uid.clone();
        
        let reason = event.reason.as_ref().cloned().unwrap_or_default();
        let message = event.message.as_ref().cloned().unwrap_or_default();
        let count = event.count.unwrap_or(1);
        
        let first_timestamp = event.first_timestamp.as_ref()
            .map(|t| t.0.format("%Y-%m-%d %H:%M:%S").to_string());
        let last_timestamp = event.last_timestamp.as_ref()
            .map(|t| t.0.format("%Y-%m-%d %H:%M:%S").to_string());
        
        let source = event.source.as_ref().map(|s| {
            EventSource {
                component: s.component.clone(),
                host: s.host.clone(),
            }
        });
        
        let type_ = event.type_.clone();

        EventInfo {
            name,
            namespace,
            kind: kind.clone(),
            reason,
            message,
            count,
            first_timestamp,
            last_timestamp,
            involved_object: InvolvedObject {
                kind,
                name: obj_name,
                namespace: obj_namespace,
                uid: obj_uid,
            },
            source,
            type_,
        }
    }

    fn namespace_to_info(&self, namespace: &Namespace) -> NamespaceInfo {
        let metadata = &namespace.metadata;
        
        let name = metadata.name.as_ref().cloned().unwrap_or_default();
        let status = namespace.status.as_ref()
            .and_then(|s| s.phase.as_ref())
            .cloned()
            .unwrap_or_else(|| "Active".to_string());
        
        let labels: std::collections::HashMap<String, String> = metadata
            .labels
            .as_ref()
            .map(|btree| {
                btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            })
            .unwrap_or_default();

        NamespaceInfo {
            name,
            status,
            age: "Unknown".to_string(),
            labels,
        }
    }

    pub async fn get_resource_yaml(&self, kind: &str, namespace: &str, name: &str) -> Result<String, String> {
        let client = Self::get_client()?;
        
        match kind.to_lowercase().as_str() {
            "pod" => {
                let api: Api<Pod> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "configmap" => {
                let api: Api<ConfigMap> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "secret" => {
                let api: Api<Secret> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "service" => {
                let api: Api<Service> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "deployment" => {
                let api: Api<Deployment> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "statefulset" => {
                let api: Api<StatefulSet> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "daemonset" => {
                let api: Api<DaemonSet> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "job" => {
                let api: Api<Job> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "cronjob" => {
                let api: Api<CronJob> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            "ingress" => {
                let api: Api<Ingress> = Api::namespaced(client, namespace);
                match api.get(name).await {
                    Ok(resource) => serde_json::to_string_pretty(&resource)
                        .map_err(|e| format!("Failed to serialize resource: {}", e)),
                    Err(e) => Err(format!("Failed to get resource: {}", e))
                }
            }
            _ => Err(format!("Unsupported resource kind: {}", kind))
        }
    }

    pub async fn get_pod_yaml(&self, namespace: &str, pod_name: &str) -> Result<String, String> {
        self.get_resource_yaml("pod", namespace, pod_name).await
    }

    pub async fn get_configmap_yaml(&self, namespace: &str, name: &str) -> Result<String, String> {
        self.get_resource_yaml("configmap", namespace, name).await
    }

    pub async fn get_secret_yaml(&self, namespace: &str, name: &str) -> Result<String, String> {
        self.get_resource_yaml("secret", namespace, name).await
    }

    pub async fn apply_resource_yaml(&self, namespace: &str, yaml_content: &str) -> Result<String, String> {
        let client = Self::get_client()?;
        
        // Parse YAML to JSON
        let mut json_value: Value = serde_yaml::from_str(yaml_content)
            .map_err(|e| format!("Invalid YAML: {}", e))?;
        
        // Extract kind and name before modifying json_value
        let kind = json_value.get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'kind' field".to_string())?
            .to_string();
        
        let name = json_value.get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str())
            .ok_or_else(|| "Missing 'metadata.name' field".to_string())?
            .to_string();
        
        // Validate namespace matches (if specified in YAML)
        if let Some(yaml_namespace) = json_value.get("metadata")
            .and_then(|m| m.get("namespace"))
            .and_then(|n| n.as_str()) {
            if yaml_namespace != namespace {
                return Err(format!("Namespace mismatch: YAML specifies '{}' but request is for '{}'", yaml_namespace, namespace));
            }
        }
        
        // Set namespace in metadata
        if let Some(metadata) = json_value.get_mut("metadata") {
            if let Some(metadata_obj) = metadata.as_object_mut() {
                metadata_obj.insert("namespace".to_string(), Value::String(namespace.to_string()));
            }
        }
        
        match kind.to_lowercase().as_str() {
            "configmap" => {
                let configmap: ConfigMap = serde_json::from_value(json_value)
                    .map_err(|e| format!("Failed to parse ConfigMap: {}", e))?;
                let api: Api<ConfigMap> = Api::namespaced(client, namespace);
                
                // Try to get existing resource
                match api.get(&name).await {
                    Ok(_) => {
                        // Update existing resource
                        let params = PatchParams::default();
                        let patch = Patch::Apply(&configmap);
                        match api.patch(&name, &params, &patch).await {
                            Ok(_) => Ok(format!("ConfigMap '{}' updated successfully", name)),
                            Err(e) => Err(format!("Failed to update ConfigMap: {}", e))
                        }
                    }
                    Err(_) => {
                        // Create new resource
                        let params = PostParams::default();
                        match api.create(&params, &configmap).await {
                            Ok(_) => Ok(format!("ConfigMap '{}' created successfully", name)),
                            Err(e) => Err(format!("Failed to create ConfigMap: {}", e))
                        }
                    }
                }
            }
            "secret" => {
                let secret: Secret = serde_json::from_value(json_value)
                    .map_err(|e| format!("Failed to parse Secret: {}", e))?;
                let api: Api<Secret> = Api::namespaced(client, namespace);
                
                // Try to get existing resource
                match api.get(&name).await {
                    Ok(_) => {
                        // Update existing resource
                        let params = PatchParams::default();
                        let patch = Patch::Apply(&secret);
                        match api.patch(&name, &params, &patch).await {
                            Ok(_) => Ok(format!("Secret '{}' updated successfully", name)),
                            Err(e) => Err(format!("Failed to update Secret: {}", e))
                        }
                    }
                    Err(_) => {
                        // Create new resource
                        let params = PostParams::default();
                        match api.create(&params, &secret).await {
                            Ok(_) => Ok(format!("Secret '{}' created successfully", name)),
                            Err(e) => Err(format!("Failed to create Secret: {}", e))
                        }
                    }
                }
            }
            "statefulset" => {
                let statefulset: StatefulSet = serde_json::from_value(json_value)
                    .map_err(|e| format!("Failed to parse StatefulSet: {}", e))?;
                let api: Api<StatefulSet> = Api::namespaced(client, namespace);
                
                // Try to get existing resource
                match api.get(&name).await {
                    Ok(_) => {
                        // Update existing resource
                        let params = PatchParams::default();
                        let patch = Patch::Apply(&statefulset);
                        match api.patch(&name, &params, &patch).await {
                            Ok(_) => Ok(format!("StatefulSet '{}' updated successfully", name)),
                            Err(e) => Err(format!("Failed to update StatefulSet: {}", e))
                        }
                    }
                    Err(_) => {
                        // Create new resource
                        let params = PostParams::default();
                        match api.create(&params, &statefulset).await {
                            Ok(_) => Ok(format!("StatefulSet '{}' created successfully", name)),
                            Err(e) => Err(format!("Failed to create StatefulSet: {}", e))
                        }
                    }
                }
            }
            "daemonset" => {
                let daemonset: DaemonSet = serde_json::from_value(json_value)
                    .map_err(|e| format!("Failed to parse DaemonSet: {}", e))?;
                let api: Api<DaemonSet> = Api::namespaced(client, namespace);
                
                // Try to get existing resource
                match api.get(&name).await {
                    Ok(_) => {
                        // Update existing resource
                        let params = PatchParams::default();
                        let patch = Patch::Apply(&daemonset);
                        match api.patch(&name, &params, &patch).await {
                            Ok(_) => Ok(format!("DaemonSet '{}' updated successfully", name)),
                            Err(e) => Err(format!("Failed to update DaemonSet: {}", e))
                        }
                    }
                    Err(_) => {
                        // Create new resource
                        let params = PostParams::default();
                        match api.create(&params, &daemonset).await {
                            Ok(_) => Ok(format!("DaemonSet '{}' created successfully", name)),
                            Err(e) => Err(format!("Failed to create DaemonSet: {}", e))
                        }
                    }
                }
            }
            _ => Err(format!("Unsupported resource kind for apply: {}", kind))
        }
    }

    pub async fn delete_configmap(&self, namespace: &str, name: &str) -> Result<(), String> {
        let client = Self::get_client()?;
        let api: Api<ConfigMap> = Api::namespaced(client, namespace);
        match api.delete(name, &kube::api::DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete ConfigMap: {}", e))
        }
    }

    pub async fn delete_secret(&self, namespace: &str, name: &str) -> Result<(), String> {
        let client = Self::get_client()?;
        let api: Api<Secret> = Api::namespaced(client, namespace);
        match api.delete(name, &kube::api::DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete Secret: {}", e))
        }
    }

    pub async fn delete_pod(&self, namespace: &str, pod_name: &str) -> Result<(), String> {
        let client = Self::get_client()?;
        let api: Api<Pod> = Api::namespaced(client, namespace);

        match api.delete(pod_name, &kube::api::DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete pod: {}", e))
        }
    }

    pub async fn scale_deployment(&self, namespace: &str, deployment_name: &str, replicas: u32) -> Result<(), String> {
        let client = Self::get_client()?;
        let api: Api<Deployment> = Api::namespaced(client, namespace);

        match api.get(deployment_name).await {
            Ok(mut deployment) => {
                if let Some(spec) = deployment.spec.as_mut() {
                    if let Some(replicas_field) = spec.replicas.as_mut() {
                        *replicas_field = replicas as i32;
                    } else {
                        spec.replicas = Some(replicas as i32);
                    }
                } else {
                    return Err("Deployment spec not found".to_string());
                }

                match api.replace(deployment_name, &kube::api::PostParams::default(), &deployment).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("Failed to scale deployment: {}", e))
                }
            }
            Err(e) => Err(format!("Failed to get deployment: {}", e))
        }
    }

    pub async fn rollback_deployment(&self, namespace: &str, deployment_name: &str) -> Result<String, String> {
        // Use kubectl rollout undo for rollback
        use tokio::process::Command;

        let mut cmd = Command::new("kubectl");
        cmd.args(&[
            "rollout",
            "undo",
            "deployment",
            deployment_name,
            "-n",
            namespace
        ]);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    Ok(format!("Deployment '{}' rolled back successfully", deployment_name))
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Rollback failed: {}", error_msg))
                }
            }
            Err(e) => Err(format!("Failed to execute rollback: {}", e))
        }
    }

    pub async fn get_pod_metrics(&self, namespace: &str, pod_name: &str) -> Result<ResourceMetrics, String> {
        // Use kubectl top to get pod metrics
        use tokio::process::Command;
        use chrono::Utc;

        let mut cmd = Command::new("kubectl");
        cmd.args(&[
            "top",
            "pod",
            pod_name,
            "-n",
            namespace,
            "--no-headers"
        ]);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
                    
                    if parts.len() >= 3 {
                        // Format: NAME CPU(cores) MEMORY(bytes)
                        // Parse CPU (can be in m, cores, etc.)
                        let cpu_str = parts[1];
                        let cpu_usage = Self::parse_cpu(cpu_str);
                        
                        // Parse Memory (can be in Ki, Mi, Gi, etc.)
                        let memory_str = parts[2];
                        let memory_usage = Self::parse_memory(memory_str);
                        
                        Ok(ResourceMetrics {
                            cpu_usage: Some(cpu_usage),
                            memory_usage: Some(memory_usage),
                            cpu_limit: None, // kubectl top doesn't provide limits
                            memory_limit: None,
                            timestamp: Utc::now().to_rfc3339(),
                        })
                    } else {
                        Err("Failed to parse metrics output".to_string())
                    }
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed to get metrics: {}", error_msg))
                }
            }
            Err(e) => Err(format!("Failed to execute metrics command: {}", e))
        }
    }

    pub async fn get_all_pods_metrics(&self, namespace: Option<&str>) -> Result<HashMap<String, ResourceMetrics>, String> {
        // Use kubectl top pods to get all pod metrics
        use tokio::process::Command;
        use chrono::Utc;

        let mut cmd = Command::new("kubectl");
        cmd.arg("top");
        cmd.arg("pods");
        if let Some(ns) = namespace {
            cmd.args(&["-n", ns]);
        } else {
            cmd.arg("--all-namespaces");
        }
        cmd.arg("--no-headers");

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let mut metrics_map = HashMap::new();
                    
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.trim().split_whitespace().collect();
                        if parts.len() >= 3 {
                            let pod_name = if namespace.is_some() {
                                parts[0].to_string()
                            } else {
                                // Format: NAMESPACE NAME CPU MEMORY
                                if parts.len() >= 4 {
                                    format!("{}/{}", parts[0], parts[1])
                                } else {
                                    continue;
                                }
                            };
                            
                            let cpu_str = if namespace.is_some() { parts[1] } else { parts[2] };
                            let memory_str = if namespace.is_some() { parts[2] } else { parts[3] };
                            
                            let cpu_usage = Self::parse_cpu(cpu_str);
                            let memory_usage = Self::parse_memory(memory_str);
                            
                            metrics_map.insert(pod_name, ResourceMetrics {
                                cpu_usage: Some(cpu_usage),
                                memory_usage: Some(memory_usage),
                                cpu_limit: None,
                                memory_limit: None,
                                timestamp: Utc::now().to_rfc3339(),
                            });
                        }
                    }
                    
                    Ok(metrics_map)
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed to get metrics: {}", error_msg))
                }
            }
            Err(e) => Err(format!("Failed to execute metrics command: {}", e))
        }
    }

    // Helper functions for parsing CPU and memory
    fn parse_cpu(cpu_str: &str) -> f64 {
    let cpu_str = cpu_str.trim();
    if cpu_str.ends_with('m') {
        // Millicores: "100m" = 0.1 cores
        cpu_str.trim_end_matches('m').parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        // Cores: "1" = 1 core
        cpu_str.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_memory(memory_str: &str) -> f64 {
    let memory_str = memory_str.trim();
    let (value_str, unit) = if memory_str.ends_with("Ki") {
        (memory_str.trim_end_matches("Ki"), 1024.0)
    } else if memory_str.ends_with("Mi") {
        (memory_str.trim_end_matches("Mi"), 1024.0 * 1024.0)
    } else if memory_str.ends_with("Gi") {
        (memory_str.trim_end_matches("Gi"), 1024.0 * 1024.0 * 1024.0)
    } else if memory_str.ends_with("Ti") {
        (memory_str.trim_end_matches("Ti"), 1024.0 * 1024.0 * 1024.0 * 1024.0)
    } else if memory_str.ends_with('K') {
        (memory_str.trim_end_matches('K'), 1000.0)
    } else if memory_str.ends_with('M') {
        (memory_str.trim_end_matches('M'), 1000.0 * 1000.0)
    } else if memory_str.ends_with('G') {
        (memory_str.trim_end_matches('G'), 1000.0 * 1000.0 * 1000.0)
    } else if memory_str.ends_with('T') {
        (memory_str.trim_end_matches('T'), 1000.0 * 1000.0 * 1000.0 * 1000.0)
    } else {
        // Assume bytes
        (memory_str, 1.0)
    };
    
    value_str.parse::<f64>().unwrap_or(0.0) * unit
    }

    pub async fn exec_pod(
        &self,
        namespace: &str,
        pod_name: &str,
        container: Option<&str>,
        command: Vec<String>,
    ) -> Result<String, String> {
        // Use kubectl exec for simplicity
        use tokio::process::Command;

        let mut cmd = Command::new("kubectl");
        cmd.args(&["exec", "-n", namespace, pod_name]);
        
        if let Some(container_name) = container {
            cmd.args(&["-c", container_name]);
        }
        
        cmd.args(&["--"]);
        cmd.args(&command);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    Err(format!(
                        "Exec failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute command: {}", e))
        }
    }

    pub async fn start_port_forward(
        &self,
        namespace: &str,
        pod_name: &str,
        local_port: u16,
        remote_port: u16,
    ) -> Result<PortForwardInfo, String> {
        // Get or create port forward map
        let port_forwards = PORT_FORWARDS.get_or_init(|| {
            Arc::new(Mutex::new(HashMap::new()))
        }).clone();

        // Check if port is already in use
        {
            let forwards = port_forwards.lock().unwrap();
            for (_, info) in forwards.values() {
                if info.local_port == local_port {
                    return Err(format!("Local port {} is already in use", local_port));
                }
            }
        }

        // Use kubectl port-forward for simplicity
        use tokio::process::Command;
        use std::process::Stdio;

        let mut cmd = Command::new("kubectl");
        cmd.args(&[
            "port-forward",
            "-n", namespace,
            &format!("pod/{}", pod_name),
            &format!("{}:{}", local_port, remote_port),
        ]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                // Give it a moment to start
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Check if process is still running
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if status.success() {
                            Err("Port forward process exited unexpectedly".to_string())
                        } else {
                            Err(format!("Port forward failed with status: {}", status))
                        }
                    }
                    Ok(None) => {
                        // Process is running, create info and store it
                        let id = Uuid::new_v4().to_string();
                        let info = PortForwardInfo {
                            id: id.clone(),
                            namespace: namespace.to_string(),
                            pod_name: pod_name.to_string(),
                            local_port,
                            remote_port,
                            status: "Active".to_string(),
                            created_at: chrono::Utc::now().to_rfc3339(),
                            url: format!("http://localhost:{}", local_port),
                        };

                        // Store the process and info
                        {
                            let mut forwards = port_forwards.lock().unwrap();
                            forwards.insert(id.clone(), (child, info.clone()));
                        }

                        Ok(info)
                    }
                    Err(e) => Err(format!("Failed to check port forward status: {}", e))
                }
            }
            Err(e) => Err(format!("Failed to start port forward: {}", e))
        }
    }

    pub async fn list_port_forwards(&self) -> Result<Vec<PortForwardInfo>, String> {
        let port_forwards = PORT_FORWARDS.get_or_init(|| {
            Arc::new(Mutex::new(HashMap::new()))
        }).clone();

        let mut forwards = port_forwards.lock().unwrap();
        let mut active_forwards = Vec::new();
        let mut dead_ids = Vec::new();

        // Check each port forward and collect dead ones
        for (id, (child, info)) in forwards.iter_mut() {
            // Check if process is still running
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited, mark for removal
                    dead_ids.push(id.clone());
                }
                Ok(None) => {
                    // Process is still running
                    active_forwards.push(info.clone());
                }
                Err(_) => {
                    // Error checking, assume it's dead
                    dead_ids.push(id.clone());
                }
            }
        }

        // Remove dead processes
        for id in dead_ids {
            forwards.remove(&id);
        }

        Ok(active_forwards)
    }

    pub async fn stop_port_forward(&self, id: &str) -> Result<(), String> {
        let port_forwards = PORT_FORWARDS.get_or_init(|| {
            Arc::new(Mutex::new(HashMap::new()))
        }).clone();

        // Remove the child from the map first
        let child_option = {
            let mut forwards = port_forwards.lock().unwrap();
            forwards.remove(id).map(|(child, _)| child)
        };
        
        if let Some(mut child) = child_option {
            // Try to kill the process
            if let Err(e) = child.kill().await {
                return Err(format!("Failed to kill port forward process: {}", e));
            }
            Ok(())
        } else {
            Err(format!("Port forward {} not found", id))
        }
    }
    
    /// Stop a specific watch task
    fn stop_watch(&self, resource_type: &str, namespace: &str) {
        let watch_tasks = WATCH_TASKS.get();
        if let Some(tasks) = watch_tasks {
            let task_key = format!("{}:{}", resource_type, namespace);
            if let Some(handle) = tasks.lock().unwrap().remove(&task_key) {
                handle.abort();
                eprintln!("Stopped watch for {} in {}", resource_type, namespace);
            }
        }
    }
    
    /// Stop all watch tasks
    pub fn stop_all_watches(&self) {
        let watch_tasks = WATCH_TASKS.get();
        if let Some(tasks) = watch_tasks {
            let mut tasks_guard = tasks.lock().unwrap();
            for (key, handle) in tasks_guard.drain() {
                handle.abort();
                eprintln!("Stopped watch task: {}", key);
            }
        }
    }
}
