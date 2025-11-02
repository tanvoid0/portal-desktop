use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::path::PathBuf;
use std::fs;
use reqwest;
use serde_json::Value;
use which::which;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaVersion {
    pub version: String,
    pub installed: bool,
    pub active: bool,
    pub size: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: String,
    pub modified_at: String,
    pub family: String,
    pub format: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaServiceStatus {
    pub running: bool,
    pub version: Option<String>,
    pub port: Option<u16>,
    pub pid: Option<u32>,
}

pub struct OllamaManager;

impl OllamaManager {
    /// Get the PID file path (similar to FlyEnv's approach)
    fn get_pid_file_path() -> Result<PathBuf, String> {
        // Try to get app data directory or use a default location
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| "Could not determine home directory".to_string())?;
        
        let pid_dir = if cfg!(target_os = "windows") {
            PathBuf::from(home).join("AppData").join("Local").join("portal-desktop").join("ollama")
        } else {
            PathBuf::from(home).join(".portal-desktop").join("ollama")
        };
        
        // Create directory if it doesn't exist
        if !pid_dir.exists() {
            fs::create_dir_all(&pid_dir)
                .map_err(|e| format!("Failed to create PID directory: {}", e))?;
        }
        
        Ok(pid_dir.join("ollama.pid"))
    }
    
    /// Read PID from file
    fn read_pid_file() -> Result<Option<u32>, String> {
        let pid_path = Self::get_pid_file_path()?;
        
        if !pid_path.exists() {
            return Ok(None);
        }
        
        let pid_str = fs::read_to_string(&pid_path)
            .map_err(|e| format!("Failed to read PID file: {}", e))?;
        
        let pid = pid_str.trim().parse::<u32>()
            .map_err(|e| format!("Invalid PID in file: {}", e))?;
        
        Ok(Some(pid))
    }
    
    /// Write PID to file
    fn write_pid_file(pid: u32) -> Result<(), String> {
        let pid_path = Self::get_pid_file_path()?;
        fs::write(&pid_path, pid.to_string())
            .map_err(|e| format!("Failed to write PID file: {}", e))?;
        Ok(())
    }
    
    /// Delete PID file
    fn delete_pid_file() -> Result<(), String> {
        let pid_path = Self::get_pid_file_path()?;
        if pid_path.exists() {
            fs::remove_file(&pid_path)
                .map_err(|e| format!("Failed to delete PID file: {}", e))?;
        }
        Ok(())
    }
    
    /// Check if a process with given PID is still running
    fn is_pid_running(pid: u32) -> bool {
        if cfg!(target_os = "windows") {
            // On Windows, try to signal the process with 0 signal (doesn't kill, just checks existence)
            Command::new("tasklist")
                .arg("/FI")
                .arg(format!("PID eq {}", pid))
                .arg("/NH")
                .output()
                .map(|output| {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    stdout.contains(&pid.to_string())
                })
                .unwrap_or(false)
        } else {
            // On Unix, send signal 0 to check if process exists
            // If the process exists, kill with signal 0 returns success
            // If it doesn't exist, it returns an error
            Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }
    }
    /// Check if Ollama is installed on the system
    pub async fn is_installed() -> bool {
        which("ollama").is_ok()
    }

    /// Get the installed Ollama version
    pub async fn get_installed_version() -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        let output = Command::new("ollama")
            .arg("--version")
            .output()
            .map_err(|e| format!("Failed to get Ollama version: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get Ollama version".to_string());
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Fetch available Ollama versions from GitHub releases
    pub async fn fetch_available_versions() -> Result<Vec<OllamaVersion>, String> {
        let url = "https://api.github.com/repos/ollama/ollama/releases";
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("User-Agent", "portal-desktop/1.0")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch releases: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            if status == 403 {
                return Err("GitHub API rate limit exceeded. Please try again later.".to_string());
            } else if status == 404 {
                return Err("Ollama repository not found".to_string());
            } else {
                return Err(format!("GitHub API returned status: {}", status));
            }
        }

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if response_text.is_empty() {
            return Err("Empty response from GitHub API".to_string());
        }

        let releases: Vec<Value> = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse releases JSON: {}. Response: {}", e, &response_text[..response_text.len().min(200)]))?;

        let mut versions = Vec::new();
        let installed_version = Self::get_installed_version().await.ok();

        for release in releases.iter().take(10) { // Limit to latest 10 releases
            if let Some(tag_name) = release.get("tag_name").and_then(|v| v.as_str()) {
                let version = tag_name.trim_start_matches('v');
                let installed = installed_version.as_ref().map_or(false, |v| v.contains(version));
                
                let size = release.get("assets")
                    .and_then(|assets| assets.as_array())
                    .and_then(|assets| assets.first())
                    .and_then(|asset| asset.get("size"))
                    .and_then(|size| size.as_u64())
                    .map(|size| format!("{:.1} MB", size as f64 / 1024.0 / 1024.0));

                let release_date = release.get("published_at")
                    .and_then(|date| date.as_str())
                    .map(|date| date.split('T').next().unwrap_or(date).to_string());

                versions.push(OllamaVersion {
                    version: version.to_string(),
                    installed,
                    active: installed,
                    size,
                    release_date,
                });
            }
        }

        Ok(versions)
    }

    /// Get installed Ollama models
    pub async fn get_installed_models() -> Result<Vec<OllamaModel>, String> {
        if !Self::is_installed().await {
            // Return empty list instead of error if Ollama is not installed
            return Ok(vec![]);
        }

        // Check if service is running first
        if !Self::is_service_running().await {
            return Err("Ollama service is not running. Please start the service first.".to_string());
        }

        // Try HTTP API first (faster and more reliable)
        let api_result = Self::get_models_via_api().await;
        if let Ok(api_models) = api_result {
            return Ok(api_models);
        }

        // Fallback to CLI if API fails (like FlyEnv does)
        let timeout_duration = std::time::Duration::from_secs(10);
        let output = tokio::time::timeout(timeout_duration, async {
            Command::new("ollama")
                .arg("list")
                .output()
        })
        .await
        .map_err(|_| "Command timed out after 10 seconds")?;

        let output = match output {
            Ok(output) => output,
            Err(_) => {
                // If CLI also fails, return empty list (like FlyEnv)
                return Ok(vec![]);
            }
        };

        if !output.status.success() {
            // If CLI fails, return empty list (like FlyEnv)
            return Ok(vec![]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut models = Vec::new();

        // Parse CLI output like FlyEnv does
        let lines: Vec<&str> = stdout.lines().filter(|s| !s.trim().is_empty()).collect();
        if lines.len() <= 1 {
            return Ok(models); // No models or just header
        }

        // Skip header line and parse each model
        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let size = parts[2].to_string(); // FlyEnv uses index 2 for size
                
                models.push(OllamaModel {
                    name,
                    size,
                    modified_at: "Unknown".to_string(),
                    family: "Unknown".to_string(),
                    format: "gguf".to_string(),
                    families: None,
                    parameter_size: None,
                    quantization_level: None,
                });
            }
        }

        Ok(models)
    }

    /// Get models via HTTP API (faster and more reliable)
    async fn get_models_via_api() -> Result<Vec<OllamaModel>, String> {
        let client = reqwest::Client::new();
        let timeout = std::time::Duration::from_secs(5);
        
        let response = tokio::time::timeout(timeout, client.get("http://localhost:11434/api/tags").send())
            .await
            .map_err(|_| "API request timed out")?
            .map_err(|e| format!("Failed to connect to Ollama API: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama API returned status: {}", response.status()));
        }

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read API response: {}", e))?;

        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse API response: {}", e))?;

        let mut models = Vec::new();
        
        if let Some(models_array) = json_response.get("models").and_then(|v| v.as_array()) {
            for model_data in models_array {
                if let Some(name) = model_data.get("name").and_then(|v| v.as_str()) {
                    let size = model_data.get("size")
                        .and_then(|v| v.as_u64())
                        .map(|s| format!("{} bytes", s))
                        .unwrap_or_else(|| "Unknown".to_string());
                    
                    let modified_at = model_data.get("modified_at")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    let family = model_data.get("family")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    
                    let format = model_data.get("format")
                        .and_then(|v| v.as_str())
                        .unwrap_or("gguf")
                        .to_string();
                    
                    let families = model_data.get("families")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect());
                    
                    let parameter_size = model_data.get("parameter_size")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    let quantization_level = model_data.get("quantization_level")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    models.push(OllamaModel {
                        name: name.to_string(),
                        size,
                        modified_at,
                        family,
                        format,
                        families,
                        parameter_size,
                        quantization_level,
                    });
                }
            }
        }

        Ok(models)
    }

    /// Install an Ollama model (simple version without progress events)
    pub async fn install_model(model_name: &str) -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        // Check if service is running first
        if !Self::is_service_running().await {
            return Err("Ollama service is not running. Please start the service first.".to_string());
        }

        // Use spawn to stream output in real-time
        let mut child = TokioCommand::new("ollama")
            .arg("pull")
            .arg(model_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ollama pull command: {}", e))?;

        let mut stdout = child.stdout.take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        
        let mut stderr = child.stderr.take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        // Read output line by line to parse progress
        let stdout_reader = BufReader::new(&mut stdout);
        let mut stdout_lines = stdout_reader.lines();
        
        let stderr_reader = BufReader::new(&mut stderr);
        let mut stderr_lines = stderr_reader.lines();

        let mut output_lines = Vec::new();

        // Read output with timeout
        let timeout_duration = std::time::Duration::from_secs(300); // 5 minutes
        let start_time = std::time::Instant::now();

        loop {
            if start_time.elapsed() > timeout_duration {
                let _ = child.kill().await;
                return Err("Command timed out after 5 minutes".to_string());
            }

            // Try to read from stdout (non-blocking check)
            tokio::select! {
                line = stdout_lines.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            let line_str = line.clone();
                            output_lines.push(line_str.clone());
                            // Parse progress information from Ollama's output
                            // Ollama shows progress like: "downloading 8934d96d3f08 in 16 239 MB part(s)"
                            // or progress percentages
                            if line_str.contains("downloading") || line_str.contains("%") || line_str.contains("MB") {
                                println!("[Ollama Progress] {}", line_str);
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            eprintln!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                line = stderr_lines.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            output_lines.push(format!("[stderr] {}", line));
                            // Check for errors in stderr
                            if line.contains("error") || line.contains("Error") || line.contains("failed") {
                                let _ = child.kill().await;
                                return Err(format!("Failed to install model: {}", line));
                            }
                        }
                        Ok(None) => {
                            // EOF on stderr, continue with stdout
                        }
                        Err(e) => {
                            eprintln!("Error reading stderr: {}", e);
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    // Check if process finished
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            if status.success() {
                                break;
                            } else {
                                let error_output = output_lines.join("\n");
                                return Err(format!("Failed to install model. Exit code: {:?}\nOutput: {}", status.code(), error_output));
                            }
                        }
                        Ok(None) => {
                            // Process still running, continue reading
                        }
                        Err(e) => {
                            return Err(format!("Failed to check process status: {}", e));
                        }
                    }
                }
            }
        }

        // Wait for process to finish
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    Ok(format!("Model {} installed successfully", model_name))
                } else {
                    let error_output = output_lines.join("\n");
                    Err(format!("Failed to install model. Exit code: {:?}\nOutput: {}", status.code(), error_output))
                }
            }
            Err(e) => Err(format!("Failed to wait for process: {}", e))
        }
    }

    /// Install an Ollama model with progress tracking via Tauri events
    pub async fn install_model_with_progress(model_name: &str, app: AppHandle) -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        if !Self::is_service_running().await {
            return Err("Ollama service is not running. Please start the service first.".to_string());
        }

        // Emit start event to all windows
        let _ = app.emit("ollama-model-progress", serde_json::json!({
            "model": model_name,
            "status": "started",
            "message": "Starting download...",
            "progress": 0
        }));

        // Use spawn to stream output in real-time
        let mut child = TokioCommand::new("ollama")
            .arg("pull")
            .arg(model_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ollama pull command: {}", e))?;

        let mut stdout = child.stdout.take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        
        let mut stderr = child.stderr.take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        let stdout_reader = BufReader::new(&mut stdout);
        let mut stdout_lines = stdout_reader.lines();
        
        let stderr_reader = BufReader::new(&mut stderr);
        let mut stderr_lines = stderr_reader.lines();

        let mut output_lines = Vec::new();
        let timeout_duration = std::time::Duration::from_secs(300);
        let start_time = std::time::Instant::now();

        loop {
            if start_time.elapsed() > timeout_duration {
                let _ = child.kill().await;
                let _ = app.emit("ollama-model-progress", serde_json::json!({
                    "model": model_name,
                    "status": "error",
                    "message": "Command timed out after 5 minutes",
                    "progress": 0
                }));
                return Err("Command timed out after 5 minutes".to_string());
            }

            tokio::select! {
                line = stdout_lines.next_line() => {
                    match line {
                        Ok(Some(line_str)) => {
                            output_lines.push(line_str.clone());
                            
                            // Parse and emit progress
                            let progress = Self::parse_progress_from_line(&line_str);
                            let message = line_str.clone();
                            
                            let _ = app.emit("ollama-model-progress", serde_json::json!({
                                "model": model_name,
                                "status": "downloading",
                                "message": message.clone(),
                                "progress": progress
                            }));
                            
                            println!("[Ollama Progress] {}% - {}", progress, line_str);
                        }
                        Ok(None) => break,
                        Err(e) => {
                            eprintln!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                line = stderr_lines.next_line() => {
                    match line {
                        Ok(Some(line_str)) => {
                            output_lines.push(format!("[stderr] {}", line_str));
                            if line_str.contains("error") || line_str.contains("Error") || line_str.contains("failed") {
                                let _ = child.kill().await;
                                let _ = app.emit("ollama-model-progress", serde_json::json!({
                                    "model": model_name,
                                    "status": "error",
                                    "message": line_str.clone(),
                                    "progress": 0
                                }));
                                return Err(format!("Failed to install model: {}", line_str));
                            }
                        }
                        Ok(None) => {}
                        Err(_) => {}
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            if status.success() {
                                break;
                            } else {
                                let error_output = output_lines.join("\n");
                                let _ = app.emit("ollama-model-progress", serde_json::json!({
                                    "model": model_name,
                                    "status": "error",
                                    "message": error_output.clone(),
                                    "progress": 0
                                }));
                                return Err(format!("Failed to install model. Exit code: {:?}", status.code()));
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            return Err(format!("Failed to check process status: {}", e));
                        }
                    }
                }
            }
        }

        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    let _ = app.emit("ollama-model-progress", serde_json::json!({
                        "model": model_name,
                        "status": "completed",
                        "message": "Installation complete!",
                        "progress": 100
                    }));
                    Ok(format!("Model {} installed successfully", model_name))
                } else {
                    let error_output = output_lines.join("\n");
                    let _ = app.emit("ollama-model-progress", serde_json::json!({
                        "model": model_name,
                        "status": "error",
                        "message": error_output.clone(),
                        "progress": 0
                    }));
                    Err(format!("Failed to install model. Exit code: {:?}", status.code()))
                }
            }
            Err(e) => Err(format!("Failed to wait for process: {}", e))
        }
    }

    /// Parse progress percentage from Ollama's output line
    fn parse_progress_from_line(line: &str) -> u8 {
        // Look for percentage patterns like "50%" or "progress: 75%"
        if let Some(percent_pos) = line.find('%') {
            // Look backwards for numbers
            let mut num_str = String::new();
            let chars: Vec<char> = line.chars().collect();
            let mut i = percent_pos.saturating_sub(1);
            
            while i < chars.len() && i > 0 && chars[i].is_ascii_digit() {
                num_str.insert(0, chars[i]);
                if i == 0 {
                    break;
                }
                i -= 1;
            }
            
            if let Ok(percent) = num_str.parse::<u8>() {
                return percent;
            }
        }
        
        // If no percentage found, estimate based on keywords
        if line.contains("pulling manifest") {
            10
        } else if line.contains("downloading") {
            // Try to extract MB downloaded vs total
            // Pattern: "downloading 8934d96d3f08 in 16 239 MB part(s)"
            if let Some(_mb_pos) = line.find("MB") {
                // Rough estimate: if we see MB, we're somewhere in the middle
                30
            } else {
                20
            }
        } else if line.contains("verifying") {
            90
        } else if line.contains("success") || line.contains("complete") {
            100
        } else {
            0
        }
    }

    /// Remove an Ollama model
    pub async fn remove_model(model_name: &str) -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        // Check if service is running first
        if !Self::is_service_running().await {
            return Err("Ollama service is not running. Please start the service first.".to_string());
        }

        // Execute ollama rm with timeout
        let timeout_duration = std::time::Duration::from_secs(30);
        let output = tokio::time::timeout(timeout_duration, async {
            Command::new("ollama")
                .arg("rm")
                .arg(model_name)
                .output()
        })
        .await
        .map_err(|_| "Command timed out after 30 seconds")?
        .map_err(|e| format!("Failed to remove model: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to remove model: {}", error));
        }

        Ok(format!("Model {} removed successfully", model_name))
    }

    /// Check if Ollama service is running
    pub async fn is_service_running() -> bool {
        if !Self::is_installed().await {
            return false;
        }

        // Try to connect to Ollama API - use the correct endpoint
        let client = reqwest::Client::new();
        let timeout = std::time::Duration::from_secs(2);
        
        match tokio::time::timeout(timeout, client.get("http://localhost:11434/api/tags").send()).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        }
    }

    /// Get Ollama service status
    pub async fn get_service_status() -> Result<OllamaServiceStatus, String> {
        println!("[OllamaManager] Checking service status...");
        let running = Self::is_service_running().await;
        println!("[OllamaManager] Service running: {}", running);
        
        // Try to get PID from file
        let pid = Self::read_pid_file().ok().flatten();
        
        // Verify PID is still running if we have one
        let verified_pid = if let Some(p) = pid {
            if Self::is_pid_running(p) {
                Some(p)
            } else {
                // PID file exists but process is dead, clean it up
                let _ = Self::delete_pid_file();
                None
            }
        } else {
            None
        };
        
        let version = if running {
            Self::get_installed_version().await.ok()
        } else {
            None
        };

        let status = OllamaServiceStatus {
            running,
            version,
            port: if running { Some(11434) } else { None },
            pid: verified_pid,
        };
        
        println!("[OllamaManager] Service status: running={}, port={:?}, pid={:?}", 
                 status.running, status.port, status.pid);
        
        Ok(status)
    }

    /// Start Ollama service
    pub async fn start_service() -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        if Self::is_service_running().await {
            return Ok("Ollama service is already running".to_string());
        }

        // Check if there's a stale PID file
        if let Ok(Some(old_pid)) = Self::read_pid_file() {
            // Check if the old process is still running
            if !Self::is_pid_running(old_pid) {
                // PID file exists but process is dead, clean it up
                let _ = Self::delete_pid_file();
            } else {
                // Process is still running, don't start a new one
                return Ok(format!("Ollama service is already running (PID: {})", old_pid));
            }
        }

        // Start Ollama service in background
        let child = Command::new("ollama")
            .arg("serve")
            .spawn()
            .map_err(|e| format!("Failed to start Ollama service: {}", e))?;

        let pid = child.id();
        
        // Store PID in file for later use when stopping
        if let Err(e) = Self::write_pid_file(pid) {
            eprintln!("Warning: Failed to write PID file: {}", e);
            // Continue anyway - service started successfully
        }

        Ok(format!("Ollama service started with PID: {}", pid))
    }

    /// Stop Ollama service
    pub async fn stop_service() -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed. Please install Ollama first from https://ollama.com/download".to_string());
        }

        // First, try to stop using PID from file (most precise method)
        if let Ok(Some(pid)) = Self::read_pid_file() {
            // Check if the PID is still running
            if Self::is_pid_running(pid) {
                let output = if cfg!(target_os = "windows") {
                    // Windows: Use taskkill with PID
                    Command::new("taskkill")
                        .arg("/F")
                        .arg("/PID")
                        .arg(pid.to_string())
                        .output()
                        .map_err(|e| format!("Failed to stop Ollama service (PID {}): {}", pid, e))?
                } else {
                    // Unix/Linux: Use kill with PID
                    Command::new("kill")
                        .arg("-TERM") // Try graceful shutdown first
                        .arg(pid.to_string())
                        .output()
                        .map_err(|e| format!("Failed to stop Ollama service (PID {}): {}", pid, e))?
                };

                if output.status.success() {
                    // Wait a bit for graceful shutdown
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    
                    // If still running, force kill
                    if Self::is_pid_running(pid) {
                        let force_output = if cfg!(target_os = "windows") {
                            Command::new("taskkill")
                                .arg("/F")
                                .arg("/PID")
                                .arg(pid.to_string())
                                .output()
                                .ok()
                        } else {
                            Command::new("kill")
                                .arg("-KILL")
                                .arg(pid.to_string())
                                .output()
                                .ok()
                        };
                        
                        if let Some(out) = force_output {
                            if !out.status.success() {
                                eprintln!("Warning: Force kill also failed for PID {}", pid);
                            }
                        }
                    }
                    
                    // Clean up PID file
                    let _ = Self::delete_pid_file();
                    
                    // Verify service is stopped
                    if !Self::is_service_running().await {
                        return Ok(format!("Ollama service stopped successfully (PID: {})", pid));
                    }
                } else {
                    // PID-based kill failed, but might be stopped already
                    let _ = Self::delete_pid_file();
                }
            } else {
                // PID file exists but process is dead, clean it up
                let _ = Self::delete_pid_file();
            }
        }

        // Fallback: Try to stop by process name if PID method failed or no PID file
        if Self::is_service_running().await {
            let output = if cfg!(target_os = "windows") {
                // Windows: Use taskkill to stop ollama.exe processes
                // Note: This will kill ALL ollama.exe processes
                Command::new("taskkill")
                    .arg("/F")
                    .arg("/IM")
                    .arg("ollama.exe")
                    .output()
                    .map_err(|e| format!("Failed to stop Ollama service: {}", e))?
            } else {
                // Unix/Linux: Try to find and kill ollama serve processes specifically
                // Use pgrep to find PIDs first, then kill them
                let pgrep_output = Command::new("pgrep")
                    .arg("-f")
                    .arg("ollama serve")
                    .output()
                    .ok();
                
                if let Some(pgrep) = pgrep_output {
                    if pgrep.status.success() {
                        let stdout_str = String::from_utf8_lossy(&pgrep.stdout);
                        let pids: Vec<&str> = stdout_str.trim().split('\n').collect();
                        
                        // Kill each PID
                        for pid_str in pids {
                            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                let _ = Command::new("kill")
                                    .arg("-TERM")
                                    .arg(pid.to_string())
                                    .output();
                            }
                        }
                        
                        // Wait for graceful shutdown
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        
                        // Force kill if still running
                        let pgrep_output2 = Command::new("pgrep")
                            .arg("-f")
                            .arg("ollama serve")
                            .output()
                            .ok();
                        
                        if let Some(pgrep2) = pgrep_output2 {
                            if pgrep2.status.success() {
                                let stdout_str2 = String::from_utf8_lossy(&pgrep2.stdout);
                                let pids2: Vec<&str> = stdout_str2.trim().split('\n').collect();
                                
                                for pid_str in pids2 {
                                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                        let _ = Command::new("kill")
                                            .arg("-KILL")
                                            .arg(pid.to_string())
                                            .output();
                                    }
                                }
                            }
                        }
                        
                        // Return success if service is no longer running
                        if !Self::is_service_running().await {
                            return Ok("Ollama service stopped successfully".to_string());
                        }
                    }
                }
                
                // Last resort: use pkill (less precise but should work)
                Command::new("pkill")
                    .arg("-f")
                    .arg("ollama serve")
                    .output()
                    .map_err(|e| format!("Failed to stop Ollama service: {}", e))?
            };

            if output.status.success() {
                // Clean up PID file if it exists
                let _ = Self::delete_pid_file();
                
                // Verify service is stopped
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if !Self::is_service_running().await {
                    return Ok("Ollama service stopped successfully".to_string());
                }
            }
        }

        // Final check - wait a bit longer and check multiple times to ensure service is really stopped
        // Sometimes the API takes a moment to reflect that the process has been killed
        let mut service_stopped = false;
        for _ in 0..3 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if !Self::is_service_running().await {
                service_stopped = true;
                break;
            }
        }
        
        // Clean up PID file regardless (we already tried to stop the process)
        let _ = Self::delete_pid_file();
        
        // If service is stopped (verified), return success
        if service_stopped || !Self::is_service_running().await {
            Ok("Ollama service stopped successfully".to_string())
        } else {
            // Service might still appear running due to API caching/race conditions
            // If we had a PID file, we tried to stop our own process, so be lenient
            // Check one more time after a longer delay
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if !Self::is_service_running().await {
                Ok("Ollama service stopped successfully".to_string())
            } else {
                // Last resort: if we got here after all attempts, the service might be running
                // under a different process/user, but we did our best
                // Return error only if we're absolutely sure it's still running
                Err("Failed to stop Ollama service. It may be running under a different user or process.".to_string())
            }
        }
    }

    /// Get available models from Ollama library
    pub async fn get_available_models() -> Result<std::collections::HashMap<String, Vec<serde_json::Value>>, String> {
        println!("[OllamaManager] Getting available models...");
        
        // Try multiple sources for available models
        
        // 1. Try FlyEnv's custom API (like they do)
        println!("[OllamaManager] Trying FlyEnv API...");
        if let Ok(models) = Self::fetch_from_flyenv_api().await {
            println!("[OllamaManager] FlyEnv API returned {} families", models.len());
            if !models.is_empty() {
                return Ok(models);
            }
        } else {
            println!("[OllamaManager] FlyEnv API failed");
        }
        
        // 2. Try Ollama's official registry (if available)
        println!("[OllamaManager] Trying Ollama registry...");
        if let Ok(models) = Self::fetch_from_ollama_registry().await {
            println!("[OllamaManager] Ollama registry returned {} models", models.len());
            if !models.is_empty() {
                // Convert flat array to hierarchical structure
                let mut hierarchical = std::collections::HashMap::new();
                for model_name in models {
                    let family = if let Some(colon_pos) = model_name.find(':') {
                        &model_name[..colon_pos]
                    } else {
                        &model_name
                    };
                    
                    let model_obj = serde_json::json!({
                        "name": model_name,
                        "size": Self::estimate_model_size(&model_name)
                    });
                    
                    hierarchical.entry(family.to_string())
                        .or_insert_with(Vec::new)
                        .push(model_obj);
                }
                println!("[OllamaManager] Converted to {} families", hierarchical.len());
                return Ok(hierarchical);
            }
        } else {
            println!("[OllamaManager] Ollama registry failed");
        }
        
        // 3. Fall back to curated list of popular models
        println!("[OllamaManager] Falling back to curated list...");
        let curated_models = Self::get_curated_models_list();
        println!("[OllamaManager] Curated list has {} models", curated_models.len());
        let mut hierarchical = std::collections::HashMap::new();
        for model_name in curated_models {
            let family = if let Some(colon_pos) = model_name.find(':') {
                &model_name[..colon_pos]
            } else {
                &model_name
            };
            
            let model_obj = serde_json::json!({
                "name": model_name,
                "size": Self::estimate_model_size(&model_name)
            });
            
            hierarchical.entry(family.to_string())
                .or_insert_with(Vec::new)
                .push(model_obj);
        }
        println!("[OllamaManager] Final result: {} families", hierarchical.len());
        Ok(hierarchical)
    }
    
    /// Fetch models from FlyEnv's custom API (similar to their approach)
    async fn fetch_from_flyenv_api() -> Result<std::collections::HashMap<String, Vec<serde_json::Value>>, String> {
        let client = reqwest::Client::new();
        let timeout = std::time::Duration::from_secs(15);
        
        // Get system architecture
        let arch = if cfg!(target_arch = "x86_64") { "x86" } else { "arm" };
        let os = if cfg!(target_os = "windows") { "win" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
        
        let response = tokio::time::timeout(timeout, client
            .post("https://api.one-env.com/api/version/fetch")
            .json(&serde_json::json!({
                "app": "ollama_models",
                "os": os,
                "arch": arch
            }))
            .send())
            .await
            .map_err(|_| "FlyEnv API request timed out")?
            .map_err(|e| format!("Failed to connect to FlyEnv API: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("FlyEnv API returned status: {}", response.status()));
        }

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read FlyEnv API response: {}", e))?;

        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse FlyEnv API response: {}", e))?;

        let mut models = std::collections::HashMap::new();
        
        // FlyEnv API returns: { "data": [...] } - we need to extract the data array
        let models_array = if let Some(data) = json_response.get("data") {
            data.as_array()
        } else {
            json_response.as_array()
        };
        
        // Parse the response structure - FlyEnv returns a flat array of model names
        // We need to group them by family to create a hierarchical structure
        if let Some(models_array) = models_array {
            for model_name in models_array {
                if let Some(name) = model_name.as_str() {
                    // Extract family name (everything before the first colon or the whole name if no colon)
                    let family = if let Some(colon_pos) = name.find(':') {
                        &name[..colon_pos]
                    } else {
                        name
                    };
                    
                    // Create model object with name and size (we'll estimate size based on model type)
                    let model_obj = serde_json::json!({
                        "name": name,
                        "size": Self::estimate_model_size(name)
                    });
                    
                    // Add to family group
                    models.entry(family.to_string())
                        .or_insert_with(Vec::new)
                        .push(model_obj);
                }
            }
        }
        
        Ok(models)
    }
    
    /// Estimate model size based on model name
    fn estimate_model_size(model_name: &str) -> u64 {
        // Estimate sizes based on common model patterns
        if model_name.contains("70b") || model_name.contains("70B") {
            40 * 1024 * 1024 * 1024 // ~40GB
        } else if model_name.contains("34b") || model_name.contains("34B") {
            20 * 1024 * 1024 * 1024 // ~20GB
        } else if model_name.contains("32b") || model_name.contains("32B") {
            20 * 1024 * 1024 * 1024 // ~20GB
        } else if model_name.contains("27b") || model_name.contains("27B") {
            15 * 1024 * 1024 * 1024 // ~15GB
        } else if model_name.contains("22b") || model_name.contains("22B") {
            12 * 1024 * 1024 * 1024 // ~12GB
        } else if model_name.contains("14b") || model_name.contains("14B") {
            8 * 1024 * 1024 * 1024 // ~8GB
        } else if model_name.contains("13b") || model_name.contains("13B") {
            7 * 1024 * 1024 * 1024 // ~7GB
        } else if model_name.contains("9b") || model_name.contains("9B") {
            5 * 1024 * 1024 * 1024 // ~5GB
        } else if model_name.contains("8b") || model_name.contains("8B") {
            4 * 1024 * 1024 * 1024 // ~4GB
        } else if model_name.contains("7b") || model_name.contains("7B") {
            3 * 1024 * 1024 * 1024 // ~3GB
        } else if model_name.contains("3b") || model_name.contains("3B") {
            2 * 1024 * 1024 * 1024 // ~2GB
        } else if model_name.contains("2b") || model_name.contains("2B") {
            1 * 1024 * 1024 * 1024 // ~1GB
        } else if model_name.contains("1b") || model_name.contains("1B") {
            512 * 1024 * 1024 // ~512MB
        } else if model_name.contains("mini") {
            1 * 1024 * 1024 * 1024 // ~1GB
        } else if model_name.contains("tiny") {
            512 * 1024 * 1024 // ~512MB
        } else {
            2 * 1024 * 1024 * 1024 // Default ~2GB
        }
    }
    
    /// Fetch models from Ollama's official registry (if available)
    async fn fetch_from_ollama_registry() -> Result<Vec<String>, String> {
        let client = reqwest::Client::new();
        let timeout = std::time::Duration::from_secs(10);
        
        // Try Ollama's registry API (if it exists)
        let response = tokio::time::timeout(timeout, client
            .get("https://ollama.com/api/library")
            .header("User-Agent", "portal-desktop/1.0")
            .header("Accept", "application/json")
            .send())
            .await
            .map_err(|_| "Ollama registry request timed out")?
            .map_err(|e| format!("Failed to connect to Ollama registry: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama registry returned status: {}", response.status()));
        }

        let response_text = response.text().await
            .map_err(|e| format!("Failed to read Ollama registry response: {}", e))?;

        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse Ollama registry response: {}", e))?;

        let mut models = Vec::new();
        
        // Parse the response structure
        if let Some(library) = json_response.get("library").and_then(|v| v.as_array()) {
            for item in library {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    models.push(name.to_string());
                }
            }
        }
        
        Ok(models)
    }
    
    /// Get curated list of popular Ollama models
    fn get_curated_models_list() -> Vec<String> {
        vec![
            // Meta models
            "llama2".to_string(),
            "llama2:7b".to_string(),
            "llama2:13b".to_string(),
            "llama2:70b".to_string(),
            "llama3".to_string(),
            "llama3:8b".to_string(),
            "llama3:70b".to_string(),
            "llama3.1".to_string(),
            "llama3.1:8b".to_string(),
            "llama3.1:70b".to_string(),
            "llama3.2".to_string(),
            "llama3.2:3b".to_string(),
            "llama3.2:1b".to_string(),
            
            // Code models
            "codellama".to_string(),
            "codellama:7b".to_string(),
            "codellama:13b".to_string(),
            "codellama:34b".to_string(),
            "codellama:python".to_string(),
            "codegemma".to_string(),
            "codegemma:7b".to_string(),
            "codegemma:2b".to_string(),
            
            // Mistral models
            "mistral".to_string(),
            "mistral:7b".to_string(),
            "mixtral".to_string(),
            "mixtral:8x7b".to_string(),
            "mixtral:8x22b".to_string(),
            
            // Google models
            "gemma".to_string(),
            "gemma:2b".to_string(),
            "gemma:7b".to_string(),
            "gemma2".to_string(),
            "gemma2:9b".to_string(),
            "gemma2:27b".to_string(),
            
            // Microsoft models
            "phi".to_string(),
            "phi3".to_string(),
            "phi3:mini".to_string(),
            "phi3:medium".to_string(),
            
            // Other popular models
            "neural-chat".to_string(),
            "starling-lm".to_string(),
            "orca-mini".to_string(),
            "vicuna".to_string(),
            "wizard-vicuna".to_string(),
            "dolphin".to_string(),
            "openchat".to_string(),
            "zephyr".to_string(),
            "qwen".to_string(),
            "qwen2.5".to_string(),
            "qwen2.5:7b".to_string(),
            "qwen2.5:14b".to_string(),
            "qwen2.5:32b".to_string(),
            "qwen2.5:72b".to_string(),
            "tinyllama".to_string(),
            "tinyllama:1.1b".to_string(),
            
            // Specialized models
            "nomic-embed-text".to_string(),
            "nomic-embed-text:latest".to_string(),
            "all-minilm".to_string(),
            "all-minilm:latest".to_string(),
        ]
    }

    /// Check for Ollama updates
    pub async fn check_for_updates() -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        let current_version = Self::get_installed_version().await?;
        let available_versions = Self::fetch_available_versions().await?;

        if let Some(latest) = available_versions.first() {
            if latest.version != current_version {
                Ok(format!("Update available: {} -> {}", current_version, latest.version))
            } else {
                Ok("Ollama is up to date".to_string())
            }
        } else {
            Ok("Unable to check for updates".to_string())
        }
    }

    /// Update Ollama to latest version
    pub async fn update_ollama() -> Result<String, String> {
        if !Self::is_installed().await {
            return Err("Ollama is not installed".to_string());
        }

        // This would typically involve downloading and installing the latest version
        // For now, return a message indicating manual update is needed
        Ok("Please update Ollama manually by downloading the latest version from https://ollama.ai".to_string())
    }
}
