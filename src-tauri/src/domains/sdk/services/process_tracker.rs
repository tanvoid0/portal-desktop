/**
 * Process Tracker
 * 
 * Tracks running SDK services and their PIDs
 */

use crate::domains::sdk::SDKError;
use std::collections::HashMap;
use sysinfo::{System, Pid};

pub struct ProcessTracker {
    tracked_processes: HashMap<u32, String>, // PID -> Service ID
}

impl ProcessTracker {
    pub fn new() -> Self {
        Self {
            tracked_processes: HashMap::new(),
        }
    }

    /// Track a process with its service ID
    pub async fn track_process(&mut self, pid: u32, service_id: &str) -> Result<(), SDKError> {
        self.tracked_processes.insert(pid, service_id.to_string());
        Ok(())
    }

    /// Stop tracking a process
    pub async fn untrack_process(&mut self, pid: u32) -> Result<(), SDKError> {
        self.tracked_processes.remove(&pid);
        Ok(())
    }

    /// Get service ID for a PID
    pub async fn get_service_id(&self, pid: u32) -> Result<String, SDKError> {
        self.tracked_processes.get(&pid)
            .ok_or_else(|| SDKError::ManagerNotFound(format!("Process {} not tracked", pid)))
            .map(|id| id.clone())
    }

    /// Check if a process is still running
    pub async fn is_process_running(&self, pid: u32) -> bool {
        let mut sys = System::new_all();
        sys.refresh_processes();
        
        if let Some(_process) = sys.process(Pid::from(pid as usize)) {
            true // Process exists, assume it's running
        } else {
            false
        }
    }

    /// Get all tracked processes
    pub async fn get_tracked_processes(&self) -> HashMap<u32, String> {
        self.tracked_processes.clone()
    }

    /// Clean up dead processes
    pub async fn cleanup_dead_processes(&mut self) -> Vec<u32> {
        let mut dead_pids = Vec::new();
        let mut sys = System::new_all();
        sys.refresh_processes();

        for (pid, _service_id) in &self.tracked_processes {
            if !self.is_process_running(*pid).await {
                dead_pids.push(*pid);
            }
        }

        for pid in &dead_pids {
            self.tracked_processes.remove(pid);
        }

        dead_pids
    }
}
