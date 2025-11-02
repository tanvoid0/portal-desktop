use std::process::Command;

pub async fn install_nodejs_version(version: &str) -> Result<String, String> {
    // Try to use nvm first
    if let Ok(output) = Command::new("nvm")
        .args(&["install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Node.js {} installed via nvm", version));
        }
    }
    
    // Try to use fnm
    if let Ok(output) = Command::new("fnm")
        .args(&["install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Node.js {} installed via fnm", version));
        }
    }
    
    // Fallback to direct download (like FlyEnv does)
    Err("No version manager found (nvm/fnm). Please install a Node.js version manager first.".to_string())
}

pub async fn install_python_version(version: &str) -> Result<String, String> {
    // Try to use pyenv
    if let Ok(output) = Command::new("pyenv")
        .args(&["install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Python {} installed via pyenv", version));
        }
    }
    
    Err("pyenv not found. Please install pyenv first.".to_string())
}

pub async fn install_java_version(version: &str) -> Result<String, String> {
    // Try to use sdkman
    if let Ok(output) = Command::new("sdk")
        .args(&["install", "java", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Java {} installed via sdkman", version));
        }
    }
    
    Err("sdkman not found. Please install sdkman first.".to_string())
}

pub async fn install_rust_version(version: &str) -> Result<String, String> {
    // Try to use rustup
    if let Ok(output) = Command::new("rustup")
        .args(&["toolchain", "install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Rust {} installed via rustup", version));
        }
    }
    
    Err("rustup not found. Please install rustup first.".to_string())
}

pub async fn install_go_version(version: &str) -> Result<String, String> {
    // Try to use g (Go version manager)
    if let Ok(output) = Command::new("g")
        .args(&["install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Go {} installed via g", version));
        }
    }
    
    Err("g (Go version manager) not found. Please install g first.".to_string())
}

pub async fn install_php_version(version: &str) -> Result<String, String> {
    // Try to use phpenv
    if let Ok(output) = Command::new("phpenv")
        .args(&["install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("PHP {} installed via phpenv", version));
        }
    }
    
    Err("phpenv not found. Please install phpenv first.".to_string())
}

pub async fn install_ruby_version(version: &str) -> Result<String, String> {
    // Try to use rbenv
    if let Ok(output) = Command::new("rbenv")
        .args(&["install", version])
        .output()
    {
        if output.status.success() {
            return Ok(format!("Ruby {} installed via rbenv", version));
        }
    }
    
    Err("rbenv not found. Please install rbenv first.".to_string())
}
