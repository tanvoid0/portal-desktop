/**
 * Port Manager
 * 
 * Manages port allocation and conflict detection
 */

use crate::domains::sdk::SDKError;
use std::collections::HashSet;
use std::net::TcpListener;

pub struct PortManager {
    allocated_ports: HashSet<u16>,
    default_ports: std::collections::HashMap<String, u16>,
}

impl PortManager {
    pub fn new() -> Self {
        let mut default_ports = std::collections::HashMap::new();
        default_ports.insert("postgresql".to_string(), 5432);
        default_ports.insert("mysql".to_string(), 3306);
        default_ports.insert("redis".to_string(), 6379);
        default_ports.insert("nginx".to_string(), 80);
        default_ports.insert("apache".to_string(), 80);
        default_ports.insert("mongodb".to_string(), 27017);
        default_ports.insert("elasticsearch".to_string(), 9200);

        Self {
            allocated_ports: HashSet::new(),
            default_ports,
        }
    }

    /// Allocate a specific port
    pub async fn allocate_port(&mut self, port: u16) -> Result<u16, SDKError> {
        if self.is_port_available(port).await {
            self.allocated_ports.insert(port);
            Ok(port)
        } else {
            Err(SDKError::ManagerNotFound(format!("Port {} is not available", port)))
        }
    }

    /// Allocate any available port
    pub async fn allocate_any_port(&mut self) -> Result<u16, SDKError> {
        // Try common port ranges
        for port in 8000..9000 {
            if self.is_port_available(port).await && !self.allocated_ports.contains(&port) {
                self.allocated_ports.insert(port);
                return Ok(port);
            }
        }
        
        Err(SDKError::ManagerNotFound("No available ports found".to_string()))
    }

    /// Release a port
    pub async fn release_port(&mut self, port: u16) -> Result<(), SDKError> {
        self.allocated_ports.remove(&port);
        Ok(())
    }

    /// Check if a port is available
    pub async fn is_port_available(&self, port: u16) -> bool {
        // Check if we've already allocated it
        if self.allocated_ports.contains(&port) {
            return false;
        }

        // Check if the port is actually available on the system
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get default port for a service type
    pub fn get_default_port(&self, service_type: &str) -> Option<u16> {
        self.default_ports.get(service_type).copied()
    }

    /// Get all allocated ports
    pub fn get_allocated_ports(&self) -> HashSet<u16> {
        self.allocated_ports.clone()
    }

    /// Find an available port in a range
    pub async fn find_available_port_in_range(&self, start: u16, end: u16) -> Option<u16> {
        for port in start..=end {
            if self.is_port_available(port).await {
                return Some(port);
            }
        }
        None
    }

    /// Check port conflicts with system services
    pub async fn check_system_conflicts(&self, port: u16) -> bool {
        // Try to bind to the port to see if it's available
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => false, // Port is available, no conflict
            Err(_) => true,  // Port is in use, conflict exists
        }
    }
}
