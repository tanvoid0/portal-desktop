use serde::{Deserialize, Serialize};
use reqwest;
use serde_json::Value;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct SDKVersion {
    pub version: String,
    pub installed: bool,
    pub active: bool,
    pub size: Option<String>,
    pub release_date: Option<String>,
}

pub async fn fetch_nodejs_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://nodejs.org/dist/index.json";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    for item in json.iter().take(20) { // Limit to latest 20 versions
        if let (Some(version), Some(date)) = (item.get("version"), item.get("date")) {
            let version_str = version.as_str().unwrap_or("").trim_start_matches('v');
            let date_str = date.as_str().unwrap_or("");
            
            // Check if version is installed locally
            let installed = check_nodejs_installed(version_str).await.unwrap_or(false);
            
            versions.push(SDKVersion {
                version: version_str.to_string(),
                installed,
                active: false, // Will be determined separately
                size: None,
                release_date: Some(date_str.to_string()),
            });
        }
    }
    
    Ok(versions)
}

pub async fn fetch_python_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://api.github.com/repos/python/cpython/releases";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    for item in json.iter().take(15) { // Limit to latest 15 versions
        if let Some(tag_name) = item.get("tag_name") {
            let version_str = tag_name.as_str().unwrap_or("").trim_start_matches('v');
            if version_str.starts_with("3.") { // Only Python 3.x
                let installed = check_python_installed(version_str).await.unwrap_or(false);
                
                versions.push(SDKVersion {
                    version: version_str.to_string(),
                    installed,
                    active: false,
                    size: None,
                    release_date: None,
                });
            }
        }
    }
    
    Ok(versions)
}

pub async fn fetch_java_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://api.adoptium.net/v3/info/release_versions?page_size=20&version=%5B8%2C21%5D&sort_order=DESC&sort_method=DEFAULT";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    if let Some(releases) = json.get("releases").and_then(|r| r.as_array()) {
        for release in releases.iter().take(15) {
            if let Some(version) = release.get("version_data").and_then(|v| v.get("openjdk_version")) {
                let version_str = version.as_str().unwrap_or("");
                let installed = check_java_installed(version_str).await.unwrap_or(false);
                
                versions.push(SDKVersion {
                    version: version_str.to_string(),
                    installed,
                    active: false,
                    size: None,
                    release_date: None,
                });
            }
        }
    }
    
    Ok(versions)
}

pub async fn fetch_rust_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://api.github.com/repos/rust-lang/rust/releases";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    for item in json.iter().take(10) { // Limit to latest 10 versions
        if let Some(tag_name) = item.get("tag_name") {
            let version_str = tag_name.as_str().unwrap_or("").trim_start_matches("rust-");
            let installed = check_rust_installed(version_str).await.unwrap_or(false);
            
            versions.push(SDKVersion {
                version: version_str.to_string(),
                installed,
                active: false,
                size: None,
                release_date: None,
            });
        }
    }
    
    Ok(versions)
}

pub async fn fetch_go_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://go.dev/dl/?mode=json";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    for item in json.iter().take(10) { // Limit to latest 10 versions
        if let Some(version) = item.get("version") {
            let version_str = version.as_str().unwrap_or("").trim_start_matches("go");
            let installed = check_go_installed(version_str).await.unwrap_or(false);
            
            versions.push(SDKVersion {
                version: version_str.to_string(),
                installed,
                active: false,
                size: None,
                release_date: None,
            });
        }
    }
    
    Ok(versions)
}

pub async fn fetch_php_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://api.github.com/repos/php/php-src/releases";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    for item in json.iter().take(10) { // Limit to latest 10 versions
        if let Some(tag_name) = item.get("tag_name") {
            let version_str = tag_name.as_str().unwrap_or("").trim_start_matches("php-");
            if version_str.starts_with("8.") || version_str.starts_with("7.") {
                let installed = check_php_installed(version_str).await.unwrap_or(false);
                
                versions.push(SDKVersion {
                    version: version_str.to_string(),
                    installed,
                    active: false,
                    size: None,
                    release_date: None,
                });
            }
        }
    }
    
    Ok(versions)
}

pub async fn fetch_ruby_versions() -> Result<Vec<SDKVersion>, String> {
    let url = "https://api.github.com/repos/ruby/ruby/releases";
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    let json: Vec<Value> = response.json().await.map_err(|e| e.to_string())?;
    
    let mut versions = Vec::new();
    for item in json.iter().take(10) { // Limit to latest 10 versions
        if let Some(tag_name) = item.get("tag_name") {
            let version_str = tag_name.as_str().unwrap_or("").trim_start_matches('v');
            let installed = check_ruby_installed(version_str).await.unwrap_or(false);
            
            versions.push(SDKVersion {
                version: version_str.to_string(),
                installed,
                active: false,
                size: None,
                release_date: None,
            });
        }
    }
    
    Ok(versions)
}

// Helper functions to check if versions are installed locally
async fn check_nodejs_installed(version: &str) -> Result<bool, String> {
    // Check if nvm is available and has the version
    if let Ok(output) = Command::new("nvm")
        .args(&["list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    // Check if fnm is available and has the version
    if let Ok(output) = Command::new("fnm")
        .args(&["list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

async fn check_python_installed(version: &str) -> Result<bool, String> {
    // Check if pyenv is available and has the version
    if let Ok(output) = Command::new("pyenv")
        .args(&["versions"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

async fn check_java_installed(version: &str) -> Result<bool, String> {
    // Check if sdkman is available and has the version
    if let Ok(output) = Command::new("sdk")
        .args(&["list", "java"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

async fn check_rust_installed(version: &str) -> Result<bool, String> {
    // Check if rustup is available and has the version
    if let Ok(output) = Command::new("rustup")
        .args(&["toolchain", "list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

async fn check_go_installed(version: &str) -> Result<bool, String> {
    // Check if g is available and has the version
    if let Ok(output) = Command::new("g")
        .args(&["list"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

async fn check_php_installed(version: &str) -> Result<bool, String> {
    // Check if phpenv is available and has the version
    if let Ok(output) = Command::new("phpenv")
        .args(&["versions"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}

async fn check_ruby_installed(version: &str) -> Result<bool, String> {
    // Check if rbenv is available and has the version
    if let Ok(output) = Command::new("rbenv")
        .args(&["versions"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(version) {
            return Ok(true);
        }
    }
    
    Ok(false)
}
