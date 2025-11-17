use tauri::State;
use std::sync::Mutex;
use crate::domains::kubernetes::manager::KubernetesManager;
use crate::domains::kubernetes::types::{*, JobInfo, CronJobInfo, ConfigMapInfo, SecretInfo, IngressInfo, ResourceMetrics, StatefulSetInfo, DaemonSetInfo, EventInfo};

#[tauri::command]
pub async fn k8s_initialize_manager(
    _manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<(), String> {
    KubernetesManager::initialize().await
}

#[tauri::command]
pub async fn k8s_load_clusters(
    _manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<Vec<KubernetesCluster>, String> {
    let mgr = KubernetesManager::new();
    mgr.load_clusters().await
}

#[tauri::command]
pub async fn k8s_connect_cluster(
    manager: State<'_, Mutex<KubernetesManager>>,
    cluster_name: String,
) -> Result<(), String> {
    // Do the async connection work first
    let mut temp_mgr = KubernetesManager::new();
    temp_mgr.connect_cluster(&cluster_name).await?;
    
    // Then update the shared state
    let cluster = temp_mgr.current_cluster.clone();
    let mut mgr = manager.lock().unwrap();
    mgr.current_cluster = cluster;
    
    Ok(())
}

#[tauri::command]
pub async fn k8s_list_pods(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<PodInfo>, String> {
    // Create a temporary manager instance - it uses static client internally
    let mgr = KubernetesManager::new();
    mgr.list_pods(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_get_pod_logs(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    pod_name: String,
    container: Option<String>,
    follow: bool,
    tail_lines: Option<i32>,
) -> Result<String, String> {
    let mgr = KubernetesManager::new();
    mgr.get_pod_logs(
        &namespace,
        &pod_name,
        container.as_deref(),
        follow,
        tail_lines,
    ).await
}

#[tauri::command]
pub async fn k8s_list_services(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<ServiceInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_services(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_deployments(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<DeploymentInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_deployments(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_statefulsets(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<StatefulSetInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_statefulsets(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_daemonsets(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<DaemonSetInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_daemonsets(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_jobs(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<JobInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_jobs(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_events(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<EventInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_events(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_cronjobs(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<CronJobInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_cronjobs(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_ingresses(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<IngressInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_ingresses(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_configmaps(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<ConfigMapInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_configmaps(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_secrets(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<Vec<SecretInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_secrets(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_get_resource_yaml(
    _manager: State<'_, Mutex<KubernetesManager>>,
    kind: String,
    namespace: String,
    name: String,
) -> Result<String, String> {
    let mgr = KubernetesManager::new();
    mgr.get_resource_yaml(&kind, &namespace, &name).await
}

#[tauri::command]
pub async fn k8s_apply_resource_yaml(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    yaml_content: String,
) -> Result<String, String> {
    let mgr = KubernetesManager::new();
    mgr.apply_resource_yaml(&namespace, &yaml_content).await
}

#[tauri::command]
pub async fn k8s_delete_configmap(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    name: String,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    mgr.delete_configmap(&namespace, &name).await
}

#[tauri::command]
pub async fn k8s_delete_secret(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    name: String,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    mgr.delete_secret(&namespace, &name).await
}

#[tauri::command]
pub async fn k8s_rollback_deployment(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    name: String,
) -> Result<String, String> {
    let mgr = KubernetesManager::new();
    mgr.rollback_deployment(&namespace, &name).await
}

#[tauri::command]
pub async fn k8s_get_pod_metrics(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    pod_name: String,
) -> Result<ResourceMetrics, String> {
    let mgr = KubernetesManager::new();
    mgr.get_pod_metrics(&namespace, &pod_name).await
}

#[tauri::command]
pub async fn k8s_get_all_pods_metrics(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: Option<String>,
) -> Result<std::collections::HashMap<String, ResourceMetrics>, String> {
    let mgr = KubernetesManager::new();
    mgr.get_all_pods_metrics(namespace.as_deref()).await
}

#[tauri::command]
pub async fn k8s_list_namespaces(
    _manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<Vec<NamespaceInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_namespaces().await
}

#[tauri::command]
pub async fn k8s_get_current_cluster(
    manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<Option<KubernetesCluster>, String> {
    let mgr = manager.lock().unwrap();
    Ok(mgr.get_current_cluster().cloned())
}

#[tauri::command]
pub async fn k8s_is_connected(
    manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<bool, String> {
    let mgr = manager.lock().unwrap();
    Ok(mgr.is_connected())
}

#[tauri::command]
pub async fn k8s_get_pod_yaml(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    pod_name: String,
) -> Result<String, String> {
    let mgr = KubernetesManager::new();
    mgr.get_pod_yaml(&namespace, &pod_name).await
}

#[tauri::command]
pub async fn k8s_delete_pod(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    pod_name: String,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    mgr.delete_pod(&namespace, &pod_name).await
}

#[tauri::command]
pub async fn k8s_scale_deployment(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    deployment_name: String,
    replicas: u32,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    mgr.scale_deployment(&namespace, &deployment_name, replicas).await
}

#[tauri::command]
pub async fn k8s_start_watching_pods(
    _manager: State<'_, Mutex<KubernetesManager>>,
    window: tauri::Window,
    namespace: Option<String>,
) -> Result<(), String> {
    // Create a temporary manager instance - it uses static client internally
    let mgr = KubernetesManager::new();
    let namespace = namespace.as_deref().unwrap_or("default");
    mgr.watch_pods(namespace, window).await
}

#[tauri::command]
pub async fn k8s_start_watching_services(
    _manager: State<'_, Mutex<KubernetesManager>>,
    window: tauri::Window,
    namespace: Option<String>,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    let namespace = namespace.as_deref().unwrap_or("default");
    mgr.watch_services(namespace, window).await
}

#[tauri::command]
pub async fn k8s_start_watching_deployments(
    _manager: State<'_, Mutex<KubernetesManager>>,
    window: tauri::Window,
    namespace: Option<String>,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    let namespace = namespace.as_deref().unwrap_or("default");
    mgr.watch_deployments(namespace, window).await
}

#[tauri::command]
pub async fn k8s_exec_pod(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    pod_name: String,
    container: Option<String>,
    command: Vec<String>,
) -> Result<String, String> {
    let mgr = KubernetesManager::new();
    mgr.exec_pod(&namespace, &pod_name, container.as_deref(), command).await
}

#[tauri::command]
pub async fn k8s_start_port_forward(
    _manager: State<'_, Mutex<KubernetesManager>>,
    namespace: String,
    pod_name: String,
    local_port: u16,
    remote_port: u16,
) -> Result<crate::domains::kubernetes::types::PortForwardInfo, String> {
    let mgr = KubernetesManager::new();
    mgr.start_port_forward(&namespace, &pod_name, local_port, remote_port).await
}

#[tauri::command]
pub async fn k8s_list_port_forwards(
    _manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<Vec<crate::domains::kubernetes::types::PortForwardInfo>, String> {
    let mgr = KubernetesManager::new();
    mgr.list_port_forwards().await
}

#[tauri::command]
pub async fn k8s_stop_port_forward(
    _manager: State<'_, Mutex<KubernetesManager>>,
    id: String,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    mgr.stop_port_forward(&id).await
}

#[tauri::command]
pub async fn k8s_stop_all_watches(
    _manager: State<'_, Mutex<KubernetesManager>>,
) -> Result<(), String> {
    let mgr = KubernetesManager::new();
    mgr.stop_all_watches();
    Ok(())
}
