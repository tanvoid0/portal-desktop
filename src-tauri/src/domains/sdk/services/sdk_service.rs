/**
 * SDK Service - Business logic for SDK management
 */

use std::process::{Command, Stdio};
use std::collections::HashMap;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait};
use crate::domains::sdk::entities::ActiveModel as SDKInstallationActive;
use super::super::SDKError;
use super::super::factory::SDKManagerFactory;
// Using println! for logging as per codebase convention

#[derive(Debug)]
pub struct SDKService {
    db: DatabaseConnection,
    factory: SDKManagerFactory,
}

impl SDKService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { 
            db,
            factory: SDKManagerFactory::new(),
        }
    }

    /// Detect installed SDK managers using the unified factory system
    pub async fn detect_managers(&self) -> Result<Vec<HashMap<String, String>>, SDKError> {
        self.factory.detect_installed_managers().await
    }

    /// Get all available SDKs and languages (both installed and not installed)
    pub async fn get_all_available_sdks(&self) -> Result<Vec<HashMap<String, String>>, SDKError> {
        let mut all_sdks = Vec::new();
        
        // Define all available programming languages and tools
        let available_languages = vec![
            // Language & Runtime
            ("java", "Java", "language", "Java Development Kit"),
            ("python", "Python", "language", "Python Programming Language"),
            ("node", "Node.js", "language", "JavaScript Runtime"),
            ("rust", "Rust", "language", "Systems Programming Language"),
            ("go", "Go", "language", "Go Programming Language"),
            ("php", "PHP", "language", "PHP Programming Language"),
            ("ruby", "Ruby", "language", "Ruby Programming Language"),
            ("csharp", "C#", "language", "C# Programming Language"),
            ("cpp", "C++", "language", "C++ Programming Language"),
            ("c", "C", "language", "C Programming Language"),
            ("swift", "Swift", "language", "Swift Programming Language"),
            ("kotlin", "Kotlin", "language", "Kotlin Programming Language"),
            ("scala", "Scala", "language", "Scala Programming Language"),
            ("clojure", "Clojure", "language", "Clojure Programming Language"),
            ("haskell", "Haskell", "language", "Haskell Programming Language"),
            ("elixir", "Elixir", "language", "Elixir Programming Language"),
            ("erlang", "Erlang", "language", "Erlang Programming Language"),
            ("dart", "Dart", "language", "Dart Programming Language"),
            ("typescript", "TypeScript", "language", "TypeScript Programming Language"),
            ("deno", "Deno", "language", "Deno JavaScript Runtime"),
            ("bun", "Bun", "language", "Bun JavaScript Runtime"),
            
            // Database
            ("postgresql", "PostgreSQL", "database", "PostgreSQL Database"),
            ("mysql", "MySQL", "database", "MySQL Database"),
            ("mongodb", "MongoDB", "database", "MongoDB Database"),
            ("redis", "Redis", "database", "Redis Database"),
            ("sqlite", "SQLite", "database", "SQLite Database"),
            ("mariadb", "MariaDB", "database", "MariaDB Database"),
            
            // Web Server
            ("nginx", "Nginx", "web", "Nginx Web Server"),
            ("apache", "Apache", "web", "Apache HTTP Server"),
            ("caddy", "Caddy", "web", "Caddy Web Server"),
            
            // Container
            ("docker", "Docker", "container", "Docker Container Platform"),
            ("podman", "Podman", "container", "Podman Container Platform"),
            ("kubernetes", "Kubernetes", "container", "Kubernetes Container Orchestration"),
            
            // Package Managers
            ("npm", "npm", "package", "Node Package Manager"),
            ("yarn", "Yarn", "package", "Yarn Package Manager"),
            ("pip", "pip", "package", "Python Package Manager"),
            ("cargo", "Cargo", "package", "Rust Package Manager"),
            ("maven", "Maven", "package", "Java Build Tool"),
            ("gradle", "Gradle", "package", "Java Build Tool"),
        ];
        
        // Add all predefined languages and tools
        for (id, name, category, description) in available_languages {
            let mut sdk_info = HashMap::new();
            sdk_info.insert("name".to_string(), id.to_string());
            sdk_info.insert("display_name".to_string(), name.to_string());
            sdk_info.insert("sdk_type".to_string(), id.to_string());
            sdk_info.insert("category".to_string(), category.to_string());
            sdk_info.insert("type".to_string(), id.to_string());
            sdk_info.insert("description".to_string(), description.to_string());
            sdk_info.insert("installed".to_string(), "false".to_string());
            sdk_info.insert("version".to_string(), "Not installed".to_string());
            
            all_sdks.push(sdk_info);
        }
        
        // Add SDK managers as a separate category
        let managers = self.factory.get_all_managers();
        for (name, manager) in managers {
            let mut sdk_info = HashMap::new();
            sdk_info.insert("name".to_string(), manager.name().to_string());
            sdk_info.insert("display_name".to_string(), manager.display_name().to_string());
            sdk_info.insert("sdk_type".to_string(), manager.sdk_type().to_string());
            sdk_info.insert("category".to_string(), "manager".to_string());
            sdk_info.insert("type".to_string(), name.clone());
            sdk_info.insert("description".to_string(), format!("{} - {}", manager.display_name(), manager.sdk_type()));
            
            // Check if this manager is installed
            match manager.is_installed().await {
                Ok(true) => {
                    sdk_info.insert("installed".to_string(), "true".to_string());
                    if let Ok(version) = manager.get_manager_version().await {
                        sdk_info.insert("version".to_string(), version);
                    }
                },
                Ok(false) => {
                    sdk_info.insert("installed".to_string(), "false".to_string());
                },
                Err(_) => {
                    sdk_info.insert("installed".to_string(), "false".to_string());
                }
            }
            
            all_sdks.push(sdk_info);
        }
        
        Ok(all_sdks)
    }

    /// List available versions for a specific SDK
    pub async fn list_versions(&self, sdk_type: &str) -> Result<Vec<String>, SDKError> {
        // Find the manager for this SDK type
        let managers = self.factory.get_managers_by_sdk_type(sdk_type);
        if let Some(manager) = managers.first() {
            manager.list_versions().await
        } else {
            Err(SDKError::ManagerNotFound(sdk_type.to_string()))
        }
    }

    /// Get active version for a specific SDK
    pub async fn get_active_version(&self, sdk_type: &str) -> Result<Option<String>, SDKError> {
        let managers = self.factory.get_managers_by_sdk_type(sdk_type);
        if let Some(manager) = managers.first() {
            manager.get_current_version().await
        } else {
            Err(SDKError::ManagerNotFound(sdk_type.to_string()))
        }
    }

    /// Switch to a specific version (system-wide or project-specific)
    pub async fn switch_version(&self, sdk_type: &str, version: &str, project_path: Option<&str>) -> Result<(), SDKError> {
        let managers = self.factory.get_managers_by_sdk_type(sdk_type);
        if let Some(manager) = managers.first() {
            if let Some(path) = project_path {
                println!("[SDKService] Performing project-specific switch for {} to version {}", sdk_type, version);
                manager.switch_version_for_project(version, path).await?;
            } else {
                println!("[SDKService] Performing system-wide switch for {} to version {}", sdk_type, version);
                manager.switch_version(version).await?;
            }
            
            // Save to database with project context
            self.save_sdk_installation(sdk_type, version, project_path).await?;
            Ok(())
        } else {
            Err(SDKError::ManagerNotFound(sdk_type.to_string()))
        }
    }

    /// Check if a switch should be system-wide or project-specific
    #[allow(dead_code)]
    pub async fn determine_switch_scope(&self, sdk_type: &str, version: &str, project_path: Option<&str>) -> Result<bool, SDKError> {
        // If project_path is provided, it's project-specific
        if project_path.is_some() {
            return Ok(true);
        }

        // Check if there are any project-specific config files that would override
        if let Some(path) = project_path {
            let project_path = std::path::Path::new(path);
            let config_files = vec![
                ".nvmrc", ".python-version", ".ruby-version", 
                ".php-version", "rust-toolchain", "go.mod"
            ];

            for config_file in config_files {
                let config_path = project_path.join(config_file);
                if config_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&config_path) {
                        let project_version = content.trim();
                        if project_version != version {
                            println!("[SDKService] Project has different {} version ({} vs {}), using project-specific switch", 
                                  sdk_type, project_version, version);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Install a new SDK version
    pub async fn install_version(&self, sdk_type: &str, version: &str) -> Result<(), SDKError> {
        match sdk_type {
            "node" => self.install_node_version(version).await?,
            "rust" => self.install_rust_version(version).await?,
            "python" => self.install_python_version(version).await?,
            "java" => self.install_java_version(version).await?,
            "go" => self.install_go_version(version).await?,
            _ => return Err(SDKError::ManagerNotFound(sdk_type.to_string())),
        }

        Ok(())
    }

    // Node.js specific methods
    async fn list_nvm_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_shell_command("nvm list --no-colors").await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("v") {
                    Some(line.trim().replace("v", "").replace("*", "").trim().to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(versions)
    }

    async fn get_active_node_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("node", &["--version"]).await?;
        Ok(Some(output.trim().replace("v", "")))
    }

    async fn switch_node_version(&self, version: &str, project_path: Option<&str>) -> Result<(), SDKError> {
        let command = if let Some(path) = project_path {
            format!("nvm use {} --directory {}", version, path)
        } else {
            format!("nvm use {}", version)
        };
        self.execute_shell_command(&command).await?;
        Ok(())
    }

    async fn install_node_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("nvm install {}", version)).await?;
        Ok(())
    }

    // Rust specific methods
    async fn list_rustup_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_command("rustup", &["toolchain", "list"]).await?;
        let versions: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .collect();
        Ok(versions)
    }

    async fn get_active_rust_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("rustc", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    async fn switch_rust_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["default", version]).await?;
        Ok(())
    }

    async fn install_rust_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_command("rustup", &["toolchain", "install", version]).await?;
        Ok(())
    }

    // Python specific methods
    async fn list_pyenv_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_shell_command("pyenv versions --bare").await?;
        let versions: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .collect();
        Ok(versions)
    }

    async fn get_active_python_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("python", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    async fn switch_python_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("pyenv global {}", version)).await?;
        Ok(())
    }

    async fn install_python_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("pyenv install {}", version)).await?;
        Ok(())
    }

    // Java specific methods
    async fn list_sdkman_versions(&self, sdk: &str) -> Result<Vec<String>, SDKError> {
        let output = self.execute_shell_command(&format!("sdk list {}", sdk)).await?;
        let versions: Vec<String> = output
            .lines()
            .filter_map(|line| {
                if line.contains("|") && !line.contains("Available") {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() > 1 {
                        Some(parts[1].trim().to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        Ok(versions)
    }

    async fn get_active_java_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("java", &["-version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    async fn switch_java_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("sdk use java {}", version)).await?;
        Ok(())
    }

    async fn install_java_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("sdk install java {}", version)).await?;
        Ok(())
    }

    // Go specific methods
    async fn list_go_versions(&self) -> Result<Vec<String>, SDKError> {
        // Go doesn't have a built-in version manager, so we'll check what's available
        let output = self.execute_command("go", &["version"]).await?;
        Ok(vec![output.trim().to_string()])
    }

    async fn get_active_go_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("go", &["version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    async fn switch_go_version(&self, _version: &str) -> Result<(), SDKError> {
        // Go version switching would require manual installation
        Err(SDKError::ManagerNotFound("go".to_string()))
    }

    async fn install_go_version(&self, _version: &str) -> Result<(), SDKError> {
        // Go version installation would require manual setup
        Err(SDKError::ManagerNotFound("go".to_string()))
    }

    // Ruby specific methods
    async fn list_rbenv_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_shell_command("rbenv versions --bare").await?;
        let versions: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .collect();
        Ok(versions)
    }

    #[allow(dead_code)]
    async fn get_active_ruby_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("ruby", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    async fn switch_ruby_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("rbenv global {}", version)).await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn install_ruby_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("rbenv install {}", version)).await?;
        Ok(())
    }

    // PHP specific methods
    async fn list_phpenv_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_shell_command("phpenv versions --bare").await?;
        let versions: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .collect();
        Ok(versions)
    }

    #[allow(dead_code)]
    async fn get_active_php_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("php", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    async fn switch_php_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("phpenv global {}", version)).await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn install_php_version(&self, version: &str) -> Result<(), SDKError> {
        self.execute_shell_command(&format!("phpenv install {}", version)).await?;
        Ok(())
    }

    // Bun specific methods
    async fn list_bun_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_command("bun", &["--version"]).await?;
        Ok(vec![output.trim().to_string()])
    }

    #[allow(dead_code)]
    async fn get_active_bun_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("bun", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    #[allow(dead_code)]
    async fn switch_bun_version(&self, _version: &str) -> Result<(), SDKError> {
        // Bun version switching would require manual installation
        Err(SDKError::ManagerNotFound("bun".to_string()))
    }

    #[allow(dead_code)]
    async fn install_bun_version(&self, _version: &str) -> Result<(), SDKError> {
        // Bun version installation would require manual setup
        Err(SDKError::ManagerNotFound("bun".to_string()))
    }

    // Deno specific methods
    async fn list_deno_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_command("deno", &["--version"]).await?;
        Ok(vec![output.trim().to_string()])
    }

    #[allow(dead_code)]
    async fn get_active_deno_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("deno", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    #[allow(dead_code)]
    async fn switch_deno_version(&self, _version: &str) -> Result<(), SDKError> {
        // Deno version switching would require manual installation
        Err(SDKError::ManagerNotFound("deno".to_string()))
    }

    #[allow(dead_code)]
    async fn install_deno_version(&self, _version: &str) -> Result<(), SDKError> {
        // Deno version installation would require manual setup
        Err(SDKError::ManagerNotFound("deno".to_string()))
    }

    // Gradle specific methods
    async fn list_gradle_versions(&self) -> Result<Vec<String>, SDKError> {
        let output = self.execute_command("gradle", &["--version"]).await?;
        Ok(vec![output.trim().to_string()])
    }

    #[allow(dead_code)]
    async fn get_active_gradle_version(&self) -> Result<Option<String>, SDKError> {
        let output = self.execute_command("gradle", &["--version"]).await?;
        Ok(Some(output.trim().to_string()))
    }

    #[allow(dead_code)]
    async fn switch_gradle_version(&self, _version: &str) -> Result<(), SDKError> {
        // Gradle version switching would require manual installation
        Err(SDKError::ManagerNotFound("gradle".to_string()))
    }

    #[allow(dead_code)]
    async fn install_gradle_version(&self, _version: &str) -> Result<(), SDKError> {
        // Gradle version installation would require manual setup
        Err(SDKError::ManagerNotFound("gradle".to_string()))
    }

    /// Execute a shell command
    async fn execute_command(&self, command: &str, args: &[&str]) -> Result<String, SDKError> {
        let output = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| SDKError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(SDKError::CommandFailed(error.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn execute_shell_command(&self, command: &str) -> Result<String, SDKError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| SDKError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(SDKError::CommandFailed(error.to_string()));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Save SDK installation to database
    async fn save_sdk_installation(&self, sdk_type: &str, version: &str, project_path: Option<&str>) -> Result<(), SDKError> {
        let installation = SDKInstallationActive {
            id: Set(format!("{}-{}", sdk_type, version)),
            sdk_type: Set(sdk_type.to_string()),
            manager_type: Set(self.get_manager_type(sdk_type)),
            version: Set(version.to_string()),
            path: Set(None),
            active: Set(true),
            installed_at: Set(chrono::Utc::now().naive_utc()),
            last_used: Set(Some(chrono::Utc::now().naive_utc())),
            project_path: Set(project_path.map(|s| s.to_string())),
        };

        installation.insert(&self.db).await?;
        Ok(())
    }

    fn get_manager_type(&self, sdk_type: &str) -> String {
        match sdk_type {
            "node" => "nvm".to_string(),
            "rust" => "rustup".to_string(),
            "python" => "pyenv".to_string(),
            "java" => "sdkman".to_string(),
            "go" => "manual".to_string(),
            "ruby" => "rbenv".to_string(),
            "php" => "phpenv".to_string(),
            "bun" => "manual".to_string(),
            "deno" => "manual".to_string(),
            "gradle" => "manual".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// FlyEnv-style project-level environment isolation
    /// Automatically switch SDK versions when entering project directories
    /// This creates temporary project-specific environment overrides
    pub async fn setup_project_environment(&self, project_path: &str) -> Result<(), SDKError> {
        use std::path::Path;
        use std::fs;

        let project_path = Path::new(project_path);
        
        // Check for project-specific configuration files
        let config_files = vec![
            ".nvmrc",           // Node.js
            ".python-version",  // Python
            ".ruby-version",    // Ruby
            ".php-version",     // PHP
            "rust-toolchain",   // Rust
            "go.mod",           // Go
        ];

        for config_file in config_files {
            let config_path = project_path.join(config_file);
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    let version = content.trim();
                    let sdk_type = self.get_sdk_type_from_config(&config_file);
                    
                    if let Some(sdk_type) = sdk_type {
                        println!("[SDKService] Auto-switching {} to version {} for project {} (temporary)", 
                                    sdk_type, version, project_path.display());
                        
                        // Create temporary project-specific environment
                        self.create_temporary_project_environment(project_path, &sdk_type, version).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Create temporary project-specific environment override
    /// This doesn't change the system-wide environment, only affects terminals in this project
    async fn create_temporary_project_environment(&self, project_path: &std::path::Path, sdk_type: &str, version: &str) -> Result<(), SDKError> {
        use std::fs;

        // Create project-specific environment script
        let env_script_path = project_path.join(".portal_env");
        let mut env_script = String::new();

        // Add SDK-specific environment setup
        match sdk_type {
            "node" => {
                env_script.push_str(&format!(
                    "export NVM_DIR=\"$HOME/.nvm\"\n\
                     [ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"\n\
                     nvm use {}\n",
                    version
                ));
            },
            "python" => {
                env_script.push_str(&format!(
                    "export PYENV_VERSION={}\n\
                     export PATH=\"$(pyenv root)/versions/{}/bin:$PATH\"\n",
                    version, version
                ));
            },
            "rust" => {
                env_script.push_str(&format!(
                    "export RUSTUP_TOOLCHAIN={}\n\
                     export PATH=\"$HOME/.rustup/toolchains/{}/bin:$PATH\"\n",
                    version, version
                ));
            },
            "ruby" => {
                env_script.push_str(&format!(
                    "export RBENV_VERSION={}\n\
                     export PATH=\"$(rbenv root)/versions/{}/bin:$PATH\"\n",
                    version, version
                ));
            },
            "php" => {
                env_script.push_str(&format!(
                    "export PHPENV_VERSION={}\n\
                     export PATH=\"$(phpenv root)/versions/{}/bin:$PATH\"\n",
                    version, version
                ));
            },
            _ => {
                println!("[SDKService] No specific environment setup for {}", sdk_type);
                return Ok(());
            }
        }

        // Write the environment script
        fs::write(&env_script_path, env_script)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to create project environment script: {}", e)))?;

        println!("[SDKService] Created temporary environment script for project {}: {}", 
              project_path.display(), env_script_path.display());

        Ok(())
    }

    /// Get SDK type from configuration file name
    fn get_sdk_type_from_config(&self, config_file: &str) -> Option<String> {
        match config_file {
            ".nvmrc" => Some("node".to_string()),
            ".python-version" => Some("python".to_string()),
            ".ruby-version" => Some("ruby".to_string()),
            ".php-version" => Some("php".to_string()),
            "rust-toolchain" => Some("rust".to_string()),
            "go.mod" => Some("go".to_string()),
            _ => None,
        }
    }

    /// Create project-specific environment configuration
    pub async fn create_project_config(&self, project_path: &str, sdk_type: &str, version: &str) -> Result<(), SDKError> {
        use std::path::Path;
        use std::fs;

        let project_path = Path::new(project_path);
        let config_file = self.get_config_file_for_sdk(sdk_type);
        let config_path = project_path.join(config_file);

        fs::write(&config_path, version)
            .map_err(|e| SDKError::CommandFailed(format!("Failed to create config file: {}", e)))?;

        println!("[SDKService] Created {} config file for project {} with version {}", 
                    config_file, project_path.display(), version);

        Ok(())
    }

    /// Get configuration file name for SDK type
    fn get_config_file_for_sdk(&self, sdk_type: &str) -> &'static str {
        match sdk_type {
            "node" => ".nvmrc",
            "python" => ".python-version",
            "ruby" => ".ruby-version",
            "php" => ".php-version",
            "rust" => "rust-toolchain",
            "go" => "go.mod",
            _ => ".sdk-version",
        }
    }

    /// Get SDK installations from database
    #[allow(dead_code)]
    pub async fn get_installations(&self) -> Result<Vec<HashMap<String, String>>, SDKError> {
        use crate::domains::sdk::entities::Entity as SDKInstallationEntity;
        
        let installations = SDKInstallationEntity::find().all(&self.db).await
            .map_err(|e| SDKError::CommandFailed(e.to_string()))?;

        let result: Vec<HashMap<String, String>> = installations
            .into_iter()
            .map(|installation| {
                HashMap::from([
                    ("id".to_string(), installation.id),
                    ("sdk_type".to_string(), installation.sdk_type),
                    ("manager_type".to_string(), installation.manager_type),
                    ("version".to_string(), installation.version),
                    ("path".to_string(), installation.path.unwrap_or_default()),
                    ("active".to_string(), installation.active.to_string()),
                    ("installed_at".to_string(), installation.installed_at.to_string()),
                    ("last_used".to_string(), installation.last_used.map(|d| d.to_string()).unwrap_or_default()),
                    ("project_path".to_string(), installation.project_path.unwrap_or_default()),
                ])
            })
            .collect();

        Ok(result)
    }
}
