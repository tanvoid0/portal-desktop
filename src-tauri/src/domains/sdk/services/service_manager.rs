/**
 * Service Manager
 * 
 * Manages service lifecycle for databases and web servers
 */

use super::{ServiceConfig, ServiceInstance, ServiceStatus, ServiceLog};
use crate::domains::sdk::SDKError;
use std::collections::HashMap;
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub is_healthy: bool,
    pub last_check: String,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f32>,
    pub port_open: bool,
    pub error_message: Option<String>,
}

pub struct ServiceManager {
    services: HashMap<String, ServiceInstance>,
    port_manager: super::PortManager,
    process_tracker: super::ProcessTracker,
    log_senders: HashMap<String, mpsc::UnboundedSender<ServiceLog>>,
    health_status: HashMap<String, ServiceHealth>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            port_manager: super::PortManager::new(),
            process_tracker: super::ProcessTracker::new(),
            log_senders: HashMap::new(),
            health_status: HashMap::new(),
        }
    }

    /// Start a service with the given configuration
    pub async fn start_service(
        &mut self,
        service_type: &str,
        version: &str,
        config: ServiceConfig,
    ) -> Result<ServiceInstance, SDKError> {
        let service_id = Uuid::new_v4().to_string();
        let service_name = format!("{}-{}", service_type, version);
        
        // Allocate port if needed
        let port = if let Some(requested_port) = config.port {
            self.port_manager.allocate_port(requested_port).await?
        } else {
            self.port_manager.allocate_any_port().await?
        };

        // Create service instance
        let mut service = ServiceInstance {
            id: service_id.clone(),
            name: service_name,
            version: version.to_string(),
            status: ServiceStatus::Starting,
            pid: None,
            port: Some(port),
            config,
            start_time: None,
            logs: Vec::new(),
        };

        // Create log channel for this service
        let (log_sender, mut log_receiver) = mpsc::unbounded_channel::<ServiceLog>();
        self.log_senders.insert(service_id.clone(), log_sender);

        // Start the service process
        let pid = self.start_service_process(service_type, version, &service).await?;
        service.pid = Some(pid);
        service.status = ServiceStatus::Running;
        service.start_time = Some(chrono::Utc::now().to_rfc3339());

        // Track the process
        self.process_tracker.track_process(pid, &service_id).await?;

        // Start log streaming task
        let _service_id_clone = service_id.clone();
        let mut service_clone = service.clone();
        tokio::spawn(async move {
            while let Some(log) = log_receiver.recv().await {
                service_clone.logs.push(log.message.clone());
                // Keep only last 1000 logs to prevent memory issues
                if service_clone.logs.len() > 1000 {
                    service_clone.logs.remove(0);
                }
            }
        });

        // Start health monitoring
        self.start_health_monitoring(&service_id, pid, port).await;

        // Store the service
        self.services.insert(service_id.clone(), service.clone());

        Ok(service)
    }

    /// Stop a service by PID
    pub async fn stop_service(&mut self, pid: u32) -> Result<(), SDKError> {
        // Find the service by PID
        let service_id = self.process_tracker.get_service_id(pid).await?;
        
        if let Some(service) = self.services.get(&service_id) {
            let port = service.port;
            
            // Stop the process
            self.stop_service_process(pid).await?;
            
            // Release the port
            if let Some(port) = port {
                self.port_manager.release_port(port).await?;
            }
            
            // Update service status
            if let Some(service) = self.services.get_mut(&service_id) {
                service.status = ServiceStatus::Stopped;
                service.pid = None;
            }
            
            // Stop tracking
            self.process_tracker.untrack_process(pid).await?;
        }

        Ok(())
    }

    /// Get service status by PID
    pub async fn get_service_status(&self, pid: u32) -> Result<ServiceStatus, SDKError> {
        if let Some(service) = self.services.values().find(|s| s.pid == Some(pid)) {
            Ok(service.status.clone())
        } else {
            Err(SDKError::ManagerNotFound(format!("Service with PID {} not found", pid)))
        }
    }

    /// Get all running services
    pub async fn get_all_services(&self) -> Vec<ServiceInstance> {
        self.services.values().cloned().collect()
    }

    /// Get service logs
    pub async fn get_service_logs(&self, pid: u32) -> Result<Vec<ServiceLog>, SDKError> {
        if let Some(service) = self.services.values().find(|s| s.pid == Some(pid)) {
            Ok(service.logs.iter().map(|log| ServiceLog {
                timestamp: chrono::Utc::now().to_rfc3339(),
                level: "INFO".to_string(),
                message: log.clone(),
            }).collect())
        } else {
            Err(SDKError::ManagerNotFound(format!("Service with PID {} not found", pid)))
        }
    }

    /// Start the actual service process
    async fn start_service_process(
        &self,
        service_type: &str,
        version: &str,
        service: &ServiceInstance,
    ) -> Result<u32, SDKError> {
        let mut cmd = match service_type {
            "postgresql" => self.create_postgresql_command(version, service).await?,
            "mysql" => self.create_mysql_command(version, service).await?,
            "redis" => self.create_redis_command(version, service).await?,
            "nginx" => self.create_nginx_command(version, service).await?,
            "apache" => self.create_apache_command(version, service).await?,
            _ => return Err(SDKError::ManagerNotFound(format!("Unsupported service type: {}", service_type))),
        };

        let child = cmd.spawn()
            .map_err(|e| SDKError::ManagerNotFound(format!("Failed to start service: {}", e)))?;

        Ok(child.id().unwrap_or(0) as u32)
    }

    /// Stop the service process
    async fn stop_service_process(&self, pid: u32) -> Result<(), SDKError> {
        if cfg!(target_os = "windows") {
            Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output()
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to stop service: {}", e)))?;
        } else {
            Command::new("kill")
                .args(&["-TERM", &pid.to_string()])
                .output()
                .map_err(|e| SDKError::ManagerNotFound(format!("Failed to stop service: {}", e)))?;
        }
        Ok(())
    }

    /// Create PostgreSQL command
    async fn create_postgresql_command(
        &self,
        version: &str,
        service: &ServiceInstance,
    ) -> Result<AsyncCommand, SDKError> {
        let data_dir = service.config.data_dir.clone()
            .unwrap_or_else(|| format!("~/.portal/data/postgresql-{}", version));
        
        let mut cmd = AsyncCommand::new("postgres");
        cmd.args(&[
            "-D", &data_dir,
            "-p", &service.port.unwrap_or(5432).to_string(),
        ]);
        
        if let Some(host) = &service.config.host {
            cmd.args(&["-h", host]);
        }

        Ok(cmd)
    }

    /// Create MySQL command
    async fn create_mysql_command(
        &self,
        version: &str,
        service: &ServiceInstance,
    ) -> Result<AsyncCommand, SDKError> {
        let data_dir = service.config.data_dir.clone()
            .unwrap_or_else(|| format!("~/.portal/data/mysql-{}", version));
        
        let mut cmd = AsyncCommand::new("mysqld");
        cmd.args(&[
            "--datadir", &data_dir,
            "--port", &service.port.unwrap_or(3306).to_string(),
        ]);

        Ok(cmd)
    }

    /// Create Redis command
    async fn create_redis_command(
        &self,
        _version: &str,
        service: &ServiceInstance,
    ) -> Result<AsyncCommand, SDKError> {
        let mut cmd = AsyncCommand::new("redis-server");
        cmd.args(&[
            "--port", &service.port.unwrap_or(6379).to_string(),
        ]);

        if let Some(config_file) = &service.config.config_file {
            cmd.args(&["--config", config_file]);
        }

        Ok(cmd)
    }

    /// Create Nginx command
    async fn create_nginx_command(
        &self,
        _version: &str,
        service: &ServiceInstance,
    ) -> Result<AsyncCommand, SDKError> {
        let mut cmd = AsyncCommand::new("nginx");
        cmd.args(&["-g", "daemon off;"]);
        
        if let Some(config_file) = &service.config.config_file {
            cmd.args(&["-c", config_file]);
        }

        Ok(cmd)
    }

    /// Create Apache command
    async fn create_apache_command(
        &self,
        _version: &str,
        service: &ServiceInstance,
    ) -> Result<AsyncCommand, SDKError> {
        let mut cmd = AsyncCommand::new("httpd");
        cmd.args(&["-D", "FOREGROUND"]);
        
        if let Some(config_file) = &service.config.config_file {
            cmd.args(&["-f", config_file]);
        }

        Ok(cmd)
    }

    /// Start health monitoring for a service
    async fn start_health_monitoring(&mut self, service_id: &str, pid: u32, port: u16) {
        let service_id = service_id.to_string();
        let mut health = ServiceHealth {
            is_healthy: true,
            last_check: chrono::Utc::now().to_rfc3339(),
            memory_usage: None,
            cpu_usage: None,
            port_open: false,
            error_message: None,
        };
        
        self.health_status.insert(service_id.clone(), health.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // Check if process is still running
                let is_running = Self::is_process_running(pid).await;
                if !is_running {
                    break;
                }

                // Check port availability
                let port_open = Self::is_port_open(port).await;
                
                // Get resource usage
                let (memory_usage, cpu_usage) = Self::get_process_resources(pid).await;
                
                // Update health status
                health.is_healthy = is_running && port_open;
                health.last_check = chrono::Utc::now().to_rfc3339();
                health.memory_usage = memory_usage;
                health.cpu_usage = cpu_usage;
                health.port_open = port_open;
                health.error_message = if !is_running {
                    Some("Process not running".to_string())
                } else if !port_open {
                    Some("Port not accessible".to_string())
                } else {
                    None
                };
            }
        });
    }

    /// Check if a process is still running
    async fn is_process_running(pid: u32) -> bool {
        if cfg!(target_os = "windows") {
            let output = Command::new("tasklist")
                .args(&["/FI", &format!("PID eq {}", pid)])
                .output()
                .ok();
            output.map_or(false, |o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        } else {
            let output = Command::new("ps")
                .args(&["-p", &pid.to_string()])
                .output()
                .ok();
            output.map_or(false, |o| !o.stdout.is_empty())
        }
    }

    /// Check if a port is open
    async fn is_port_open(port: u16) -> bool {
        use std::net::{TcpListener, SocketAddr};
        use std::str::FromStr;
        
        let addr = format!("127.0.0.1:{}", port);
        if let Ok(socket_addr) = SocketAddr::from_str(&addr) {
            TcpListener::bind(socket_addr).is_ok()
        } else {
            false
        }
    }

    /// Get process resource usage
    async fn get_process_resources(pid: u32) -> (Option<u64>, Option<f32>) {
        if cfg!(target_os = "windows") {
            // Windows implementation would use WMI or similar
            (None, None)
        } else {
            // Unix implementation using ps
            let output = Command::new("ps")
                .args(&["-p", &pid.to_string(), "-o", "rss,pcpu", "--no-headers"])
                .output()
                .ok();
            
            if let Some(output) = output {
                let line = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let memory = parts[0].parse::<u64>().ok().map(|kb| kb * 1024); // Convert KB to bytes
                    let cpu = parts[1].parse::<f32>().ok();
                    (memory, cpu)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        }
    }

    /// Get service health status
    pub async fn get_service_health(&self, service_id: &str) -> Result<ServiceHealth, SDKError> {
        self.health_status.get(service_id)
            .cloned()
            .ok_or_else(|| SDKError::ManagerNotFound(format!("Service {} not found", service_id)))
    }

    /// Stream service logs
    pub async fn stream_service_logs(&self, service_id: &str) -> Result<mpsc::UnboundedReceiver<ServiceLog>, SDKError> {
        if let Some(_sender) = self.log_senders.get(service_id) {
            let (_tx, rx) = mpsc::unbounded_channel();
            // In a real implementation, you'd connect the sender to the receiver
            Ok(rx)
        } else {
            Err(SDKError::ManagerNotFound(format!("Service {} not found", service_id)))
        }
    }

    /// Restart a service
    pub async fn restart_service(&mut self, service_id: &str) -> Result<ServiceInstance, SDKError> {
        // Clone the service data to avoid borrow checker issues
        let service_data = if let Some(service) = self.services.get(service_id) {
            (service.name.clone(), service.version.clone(), service.config.clone(), service.pid)
        } else {
            return Err(SDKError::ManagerNotFound(format!("Service {} not found", service_id)));
        };

        if let Some(pid) = service_data.3 {
            // Stop the service
            self.stop_service(pid).await?;
            
            // Wait a bit for graceful shutdown
            sleep(Duration::from_secs(2)).await;
            
            // Start the service again
            self.start_service(&service_data.0.split('-').next().unwrap_or("unknown"), &service_data.1, service_data.2).await
        } else {
            Err(SDKError::ManagerNotFound(format!("Service {} has no PID", service_id)))
        }
    }

    /// Update service configuration
    pub async fn update_service_config(&mut self, service_id: &str, config: ServiceConfig) -> Result<(), SDKError> {
        if let Some(service) = self.services.get_mut(service_id) {
            service.config = config;
            Ok(())
        } else {
            Err(SDKError::ManagerNotFound(format!("Service {} not found", service_id)))
        }
    }

    /// Get service logs with limit
    pub async fn get_service_logs_limited(&self, service_id: &str, lines: usize) -> Result<Vec<ServiceLog>, SDKError> {
        if let Some(service) = self.services.get(service_id) {
            let logs: Vec<ServiceLog> = service.logs
                .iter()
                .rev()
                .take(lines)
                .map(|msg| ServiceLog {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    level: "INFO".to_string(),
                    message: msg.clone(),
                })
                .collect();
            Ok(logs)
        } else {
            Err(SDKError::ManagerNotFound(format!("Service {} not found", service_id)))
        }
    }
}
