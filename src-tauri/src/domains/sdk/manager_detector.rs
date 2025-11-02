use serde::{Deserialize, Serialize};
use std::process::Command;
use which::which;

#[derive(Debug, Serialize, Deserialize)]
pub struct SDKInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub installed: bool,
}

pub async fn detect_sdk_managers() -> Result<Vec<SDKInfo>, String> {
    println!("[SDK] Detecting SDK managers");
    
    let mut managers = Vec::new();
    
    // Check for Node.js
    if let Ok(output) = Command::new("node")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = which("node")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "node".to_string());
            
            managers.push(SDKInfo {
                name: "Node.js".to_string(),
                version: version.trim_start_matches('v').to_string(),
                path,
                installed: true,
            });
        }
    }
    
    // Check for Python
    if let Ok(output) = Command::new("python3")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = which("python3")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "python3".to_string());
            
            managers.push(SDKInfo {
                name: "Python".to_string(),
                version: version.trim_start_matches("Python ").to_string(),
                path,
                installed: true,
            });
        }
    }
    
    // Check for Rust
    if let Ok(output) = Command::new("rustc")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = which("rustc")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "rustc".to_string());
            
            managers.push(SDKInfo {
                name: "Rust".to_string(),
                version: version.split_whitespace().nth(1).unwrap_or("unknown").to_string(),
                path,
                installed: true,
            });
        }
    }
    
    // Check for Java
    if let Ok(output) = Command::new("java")
        .arg("-version")
        .output()
    {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stderr);
            if let Some(version_line) = version_output.lines().next() {
                if let Some(version) = version_line.split('"').nth(1) {
                    let path = which("java")
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| "java".to_string());
                    
                    managers.push(SDKInfo {
                        name: "Java".to_string(),
                        version: version.to_string(),
                        path,
                        installed: true,
                    });
                }
            }
        }
    }
    
    // Check for Go
    if let Ok(output) = Command::new("go")
        .arg("version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = which("go")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "go".to_string());
            
            managers.push(SDKInfo {
                name: "Go".to_string(),
                version: version.split_whitespace().nth(2).unwrap_or("unknown").to_string(),
                path,
                installed: true,
            });
        }
    }
    
    // Check for PHP
    if let Ok(output) = Command::new("php")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = which("php")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "php".to_string());
            
            managers.push(SDKInfo {
                name: "PHP".to_string(),
                version: version.split_whitespace().nth(1).unwrap_or("unknown").to_string(),
                path,
                installed: true,
            });
        }
    }
    
    // Check for Ruby
    if let Ok(output) = Command::new("ruby")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = which("ruby")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "ruby".to_string());
            
            managers.push(SDKInfo {
                name: "Ruby".to_string(),
                version: version.split_whitespace().nth(1).unwrap_or("unknown").to_string(),
                path,
                installed: true,
            });
        }
    }
    
    Ok(managers)
}
