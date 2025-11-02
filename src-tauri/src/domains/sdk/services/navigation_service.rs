/**
 * SDK Navigation Service
 * 
 * Provides dynamic SDK navigation items with installation status and version information
 * for the SDK management interface. Focuses specifically on SDK-related tools and managers.
 */

use serde::{Deserialize, Serialize};
use crate::domains::sdk::factory::SDKManagerFactory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub url: String,
    pub badge: Option<usize>,
    pub installed: bool,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub manager_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSection {
    pub title: String,
    pub items: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResponse {
    pub sections: Vec<NavigationSection>,
    pub total_installed: usize,
    pub total_available: usize,
}

pub struct NavigationService {
    factory: SDKManagerFactory,
}

impl NavigationService {
    pub fn new() -> Self {
        Self {
            factory: SDKManagerFactory::new(),
        }
    }

    /// Get SDK-specific navigation sections with dynamic status
    pub async fn get_sdk_navigation_items(&self) -> Result<NavigationResponse, Box<dyn std::error::Error>> {
        let mut sections = Vec::new();

        // SDK Navigation section
        sections.push(NavigationSection {
            title: "SDK Navigation".to_string(),
            items: vec![
                NavigationItem {
                    id: "sdk-overview".to_string(),
                    title: "SDK Overview".to_string(),
                    description: "SDK management dashboard".to_string(),
                    icon: "code".to_string(),
                    category: "sdk-navigation".to_string(),
                    url: "/sdk".to_string(),
                    badge: None,
                    installed: true, // Always available
                    version: None,
                    latest_version: None,
                    manager_type: None,
                },
                NavigationItem {
                    id: "sdk-managers".to_string(),
                    title: "SDK Managers".to_string(),
                    description: "Install and manage SDK version managers".to_string(),
                    icon: "settings".to_string(),
                    category: "sdk-navigation".to_string(),
                    url: "/sdk/managers".to_string(),
                    badge: None,
                    installed: true, // Always available
                    version: None,
                    latest_version: None,
                    manager_type: None,
                },
                NavigationItem {
                    id: "sdk-installations".to_string(),
                    title: "Installed SDKs".to_string(),
                    description: "View and manage installed SDK versions".to_string(),
                    icon: "download".to_string(),
                    category: "sdk-navigation".to_string(),
                    url: "/sdk/versions".to_string(),
                    badge: None,
                    installed: true, // Always available
                    version: None,
                    latest_version: None,
                    manager_type: None,
                },
                NavigationItem {
                    id: "sdk-services".to_string(),
                    title: "SDK Services".to_string(),
                    description: "Manage running SDK services".to_string(),
                    icon: "play".to_string(),
                    category: "sdk-navigation".to_string(),
                    url: "/sdk/services".to_string(),
                    badge: None,
                    installed: true, // Always available
                    version: None,
                    latest_version: None,
                    manager_type: None,
                },
            ],
        });

        // Language section - dynamically check installation status
        let language_items = self.get_language_items().await?;
        sections.push(NavigationSection {
            title: "Language".to_string(),
            items: language_items,
        });

        // Database section
        let database_items = self.get_database_items().await?;
        sections.push(NavigationSection {
            title: "Database".to_string(),
            items: database_items,
        });

        // Web Server section
        let web_server_items = self.get_web_server_items().await?;
        sections.push(NavigationSection {
            title: "Web Server".to_string(),
            items: web_server_items,
        });

        // Container section
        let container_items = self.get_container_items().await?;
        sections.push(NavigationSection {
            title: "Container".to_string(),
            items: container_items,
        });

            // AI section
            let ai_items = self.get_ai_items().await?;
            sections.push(NavigationSection {
                title: "AI".to_string(),
                items: ai_items,
            });

            // SDK Manager Tools section
            let sdk_tools_items = self.get_sdk_tools_items().await?;
            sections.push(NavigationSection {
                title: "SDK Tools".to_string(),
                items: sdk_tools_items,
            });

        // Calculate totals
        let total_installed = sections.iter()
            .flat_map(|s| &s.items)
            .filter(|item| item.installed)
            .count();
        
        let total_available = sections.iter()
            .flat_map(|s| &s.items)
            .count();

        Ok(NavigationResponse {
            sections,
            total_installed,
            total_available,
        })
    }

    async fn get_language_items(&self) -> Result<Vec<NavigationItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();

        // Node.js
        let (node_installed, node_version) = self.check_sdk_status("nodejs").await?;
        items.push(NavigationItem {
            id: "nodejs".to_string(),
            title: "Node.js".to_string(),
            description: "JavaScript runtime".to_string(),
            icon: "nodejs".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/nodejs".to_string(),
            badge: None,
            installed: node_installed,
            version: node_version,
            latest_version: Some("20.10.0".to_string()),
            manager_type: Some("nvm".to_string()),
        });

        // Python
        let (python_installed, python_version) = self.check_sdk_status("python").await?;
        items.push(NavigationItem {
            id: "python".to_string(),
            title: "Python".to_string(),
            description: "Python programming".to_string(),
            icon: "python".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/python".to_string(),
            badge: None,
            installed: python_installed,
            version: python_version,
            latest_version: Some("3.12.0".to_string()),
            manager_type: Some("pyenv".to_string()),
        });

        // Java
        let (java_installed, java_version) = self.check_sdk_status("java").await?;
        items.push(NavigationItem {
            id: "java".to_string(),
            title: "Java".to_string(),
            description: "Java development".to_string(),
            icon: "java".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/java".to_string(),
            badge: None,
            installed: java_installed,
            version: java_version,
            latest_version: Some("21.0.0".to_string()),
            manager_type: Some("sdkman".to_string()),
        });

        // Rust
        let (rust_installed, rust_version) = self.check_sdk_status("rust").await?;
        items.push(NavigationItem {
            id: "rust".to_string(),
            title: "Rust".to_string(),
            description: "Rust programming".to_string(),
            icon: "rust".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/rust".to_string(),
            badge: None,
            installed: rust_installed,
            version: rust_version,
            latest_version: Some("1.75.0".to_string()),
            manager_type: Some("rustup".to_string()),
        });

        // Go
        let (go_installed, go_version) = self.check_sdk_status("go").await?;
        items.push(NavigationItem {
            id: "go".to_string(),
            title: "Go".to_string(),
            description: "Go programming".to_string(),
            icon: "go".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/go".to_string(),
            badge: None,
            installed: go_installed,
            version: go_version,
            latest_version: Some("1.21.0".to_string()),
            manager_type: Some("g".to_string()),
        });

        // PHP
        let (php_installed, php_version) = self.check_sdk_status("php").await?;
        items.push(NavigationItem {
            id: "php".to_string(),
            title: "PHP".to_string(),
            description: "PHP development".to_string(),
            icon: "php".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/php".to_string(),
            badge: None,
            installed: php_installed,
            version: php_version,
            latest_version: Some("8.3.0".to_string()),
            manager_type: Some("phpenv".to_string()),
        });

        // Ruby
        let (ruby_installed, ruby_version) = self.check_sdk_status("ruby").await?;
        items.push(NavigationItem {
            id: "ruby".to_string(),
            title: "Ruby".to_string(),
            description: "Ruby programming".to_string(),
            icon: "ruby".to_string(),
            category: "language".to_string(),
            url: "/sdk/manager/ruby".to_string(),
            badge: None,
            installed: ruby_installed,
            version: ruby_version,
            latest_version: Some("3.3.0".to_string()),
            manager_type: Some("rbenv".to_string()),
        });

        Ok(items)
    }

    async fn get_database_items(&self) -> Result<Vec<NavigationItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();

        // PostgreSQL
        let (postgres_installed, postgres_version) = self.check_sdk_status("postgresql").await?;
        items.push(NavigationItem {
            id: "postgresql".to_string(),
            title: "PostgreSQL".to_string(),
            description: "PostgreSQL database".to_string(),
            icon: "database".to_string(),
            category: "database".to_string(),
            url: "/sdk/database/postgresql".to_string(),
            badge: None,
            installed: postgres_installed,
            version: postgres_version,
            latest_version: Some("16.0".to_string()),
            manager_type: None,
        });

        // MySQL
        let (mysql_installed, mysql_version) = self.check_sdk_status("mysql").await?;
        items.push(NavigationItem {
            id: "mysql".to_string(),
            title: "MySQL".to_string(),
            description: "MySQL database".to_string(),
            icon: "database".to_string(),
            category: "database".to_string(),
            url: "/sdk/database/mysql".to_string(),
            badge: None,
            installed: mysql_installed,
            version: mysql_version,
            latest_version: Some("8.0.35".to_string()),
            manager_type: None,
        });

        // MongoDB
        let (mongodb_installed, mongodb_version) = self.check_sdk_status("mongodb").await?;
        items.push(NavigationItem {
            id: "mongodb".to_string(),
            title: "MongoDB".to_string(),
            description: "MongoDB database".to_string(),
            icon: "database".to_string(),
            category: "database".to_string(),
            url: "/sdk/database/mongodb".to_string(),
            badge: None,
            installed: mongodb_installed,
            version: mongodb_version,
            latest_version: Some("7.0.0".to_string()),
            manager_type: None,
        });

        // Redis
        let (redis_installed, redis_version) = self.check_sdk_status("redis").await?;
        items.push(NavigationItem {
            id: "redis".to_string(),
            title: "Redis".to_string(),
            description: "Redis cache".to_string(),
            icon: "database".to_string(),
            category: "database".to_string(),
            url: "/sdk/database/redis".to_string(),
            badge: None,
            installed: redis_installed,
            version: redis_version,
            latest_version: Some("7.2.0".to_string()),
            manager_type: None,
        });

        Ok(items)
    }

    async fn get_web_server_items(&self) -> Result<Vec<NavigationItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();

        // Nginx
        let (nginx_installed, nginx_version) = self.check_sdk_status("nginx").await?;
        items.push(NavigationItem {
            id: "nginx".to_string(),
            title: "Nginx".to_string(),
            description: "Nginx web server".to_string(),
            icon: "globe".to_string(),
            category: "web-server".to_string(),
            url: "/sdk/server/nginx".to_string(),
            badge: None,
            installed: nginx_installed,
            version: nginx_version,
            latest_version: Some("1.25.0".to_string()),
            manager_type: None,
        });

        // Apache
        let (apache_installed, apache_version) = self.check_sdk_status("apache").await?;
        items.push(NavigationItem {
            id: "apache".to_string(),
            title: "Apache".to_string(),
            description: "Apache web server".to_string(),
            icon: "globe".to_string(),
            category: "web-server".to_string(),
            url: "/sdk/server/apache".to_string(),
            badge: None,
            installed: apache_installed,
            version: apache_version,
            latest_version: Some("2.4.58".to_string()),
            manager_type: None,
        });

        // Caddy
        let (caddy_installed, caddy_version) = self.check_sdk_status("caddy").await?;
        items.push(NavigationItem {
            id: "caddy".to_string(),
            title: "Caddy".to_string(),
            description: "Caddy web server".to_string(),
            icon: "globe".to_string(),
            category: "web-server".to_string(),
            url: "/sdk/server/caddy".to_string(),
            badge: None,
            installed: caddy_installed,
            version: caddy_version,
            latest_version: Some("2.7.0".to_string()),
            manager_type: None,
        });

        Ok(items)
    }

    async fn get_container_items(&self) -> Result<Vec<NavigationItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();

        // Docker
        let (docker_installed, docker_version) = self.check_sdk_status("docker").await?;
        items.push(NavigationItem {
            id: "docker".to_string(),
            title: "Docker".to_string(),
            description: "Docker containers".to_string(),
            icon: "container".to_string(),
            category: "container".to_string(),
            url: "/sdk/container/docker".to_string(),
            badge: None,
            installed: docker_installed,
            version: docker_version,
            latest_version: Some("24.0.0".to_string()),
            manager_type: None,
        });

        // Podman
        let (podman_installed, podman_version) = self.check_sdk_status("podman").await?;
        items.push(NavigationItem {
            id: "podman".to_string(),
            title: "Podman".to_string(),
            description: "Podman containers".to_string(),
            icon: "container".to_string(),
            category: "container".to_string(),
            url: "/sdk/container/podman".to_string(),
            badge: None,
            installed: podman_installed,
            version: podman_version,
            latest_version: Some("4.8.0".to_string()),
            manager_type: None,
        });

        Ok(items)
    }

    async fn get_sdk_tools_items(&self) -> Result<Vec<NavigationItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();

        // SDK Terminal
        items.push(NavigationItem {
            id: "sdk-terminal".to_string(),
            title: "SDK Terminal".to_string(),
            description: "Terminal with SDK environment".to_string(),
            icon: "terminal".to_string(),
            category: "sdk-tools".to_string(),
            url: "/terminal".to_string(),
            badge: None,
            installed: true, // Always available
            version: None,
            latest_version: None,
            manager_type: None,
        });

        // SDK Projects
        items.push(NavigationItem {
            id: "sdk-projects".to_string(),
            title: "SDK Projects".to_string(),
            description: "Projects with SDK configurations".to_string(),
            icon: "folder".to_string(),
            category: "sdk-tools".to_string(),
            url: "/projects".to_string(),
            badge: None,
            installed: true, // Always available
            version: None,
            latest_version: None,
            manager_type: None,
        });

        // SDK Environment
        items.push(NavigationItem {
            id: "sdk-environment".to_string(),
            title: "SDK Environment".to_string(),
            description: "Manage SDK environment variables".to_string(),
            icon: "settings".to_string(),
            category: "sdk-tools".to_string(),
            url: "/sdk/tools/environment".to_string(),
            badge: None,
            installed: true, // Always available
            version: None,
            latest_version: None,
            manager_type: None,
        });

        // SDK Logs
        items.push(NavigationItem {
            id: "sdk-logs".to_string(),
            title: "SDK Logs".to_string(),
            description: "View SDK installation and service logs".to_string(),
            icon: "file-text".to_string(),
            category: "sdk-tools".to_string(),
            url: "/sdk/tools/logs".to_string(),
            badge: None,
            installed: true, // Always available
            version: None,
            latest_version: None,
            manager_type: None,
        });

        Ok(items)
    }

    /// Check if an SDK is installed and get its version
    async fn check_sdk_status(&self, sdk_name: &str) -> Result<(bool, Option<String>), Box<dyn std::error::Error>> {
        // This would integrate with the actual SDK managers to check installation status
        // For now, we'll simulate some responses based on common installations
        
        match sdk_name {
            "nodejs" => {
                // Check if Node.js is installed via nvm or system
                let version = self.check_command_version("node", "--version").await?;
                Ok((version.is_some(), version))
            },
            "python" => {
                let version = self.check_command_version("python3", "--version").await?;
                Ok((version.is_some(), version))
            },
            "java" => {
                let version = self.check_command_version("java", "-version").await?;
                Ok((version.is_some(), version))
            },
            "rust" => {
                let version = self.check_command_version("rustc", "--version").await?;
                Ok((version.is_some(), version))
            },
            "go" => {
                let version = self.check_command_version("go", "version").await?;
                Ok((version.is_some(), version))
            },
            "php" => {
                let version = self.check_command_version("php", "--version").await?;
                Ok((version.is_some(), version))
            },
            "ruby" => {
                let version = self.check_command_version("ruby", "--version").await?;
                Ok((version.is_some(), version))
            },
            "docker" => {
                let version = self.check_command_version("docker", "--version").await?;
                Ok((version.is_some(), version))
            },
            "postgresql" => {
                let version = self.check_command_version("psql", "--version").await?;
                Ok((version.is_some(), version))
            },
            "mysql" => {
                let version = self.check_command_version("mysql", "--version").await?;
                Ok((version.is_some(), version))
            },
            "mongodb" => {
                let version = self.check_command_version("mongod", "--version").await?;
                Ok((version.is_some(), version))
            },
            "redis" => {
                let version = self.check_command_version("redis-server", "--version").await?;
                Ok((version.is_some(), version))
            },
            "nginx" => {
                let version = self.check_command_version("nginx", "-v").await?;
                Ok((version.is_some(), version))
            },
            "apache" => {
                let version = self.check_command_version("apache2", "-v").await?;
                Ok((version.is_some(), version))
            },
            "caddy" => {
                let version = self.check_command_version("caddy", "version").await?;
                Ok((version.is_some(), version))
            },
            "podman" => {
                let version = self.check_command_version("podman", "--version").await?;
                Ok((version.is_some(), version))
            },
            _ => Ok((false, None)),
        }
    }

    /// Check if a command exists and get its version
    async fn check_command_version(&self, command: &str, version_flag: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let output = Command::new(command)
            .arg(version_flag)
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    let version = String::from_utf8_lossy(&result.stdout).to_string();
                    Ok(Some(version.trim().to_string()))
                } else {
                    Ok(None)
                }
            },
            Err(_) => Ok(None),
        }
    }

    /// Get AI navigation items
    async fn get_ai_items(&self) -> Result<Vec<NavigationItem>, Box<dyn std::error::Error>> {
        let mut items = Vec::new();

        // Check if Ollama is installed
        let ollama_installed = self.check_ai_service_installed("ollama").await?;
        let ollama_version = if ollama_installed {
            self.get_ai_service_version("ollama").await.ok().flatten()
        } else {
            None
        };

        // Ollama
        items.push(NavigationItem {
            id: "ollama".to_string(),
            title: "Ollama".to_string(),
            description: "Local AI model runner".to_string(),
            icon: "robot".to_string(),
            category: "ai".to_string(),
            url: "/sdk/ai/ollama".to_string(),
            badge: None,
            installed: ollama_installed,
            version: ollama_version,
            latest_version: Some("0.12.6".to_string()), // Mock latest version
            manager_type: Some("ollama".to_string()),
        });

        Ok(items)
    }

    /// Check if an AI service is installed
    async fn check_ai_service_installed(&self, service: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match service {
            "ollama" => {
                let version = self.check_command_version("ollama", "--version").await?;
                Ok(version.is_some())
            },
            _ => Ok(false),
        }
    }

    /// Get AI service version
    async fn get_ai_service_version(&self, service: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match service {
            "ollama" => {
                self.check_command_version("ollama", "--version").await
            },
            _ => Ok(None),
        }
    }

    /// Get detailed information about a specific SDK
    pub async fn get_sdk_details(&self, sdk_type: &str) -> Result<Option<SdkDetails>, Box<dyn std::error::Error>> {
        let sdk_details = match sdk_type {
            "nodejs" => SdkDetails {
                id: "nodejs".to_string(),
                name: "Node.js".to_string(),
                description: "JavaScript runtime built on Chrome's V8 JavaScript engine".to_string(),
                icon: "nodejs".to_string(),
                category: "language".to_string(),
                website: "https://nodejs.org/".to_string(),
                install_command: "curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash".to_string(),
                features: vec![
                    "JavaScript runtime".to_string(),
                    "NPM package manager".to_string(),
                    "Event-driven architecture".to_string(),
                    "Non-blocking I/O".to_string(),
                ],
                default_port: Some(3000),
                manager_type: Some("nvm".to_string()),
            },
            "python" => SdkDetails {
                id: "python".to_string(),
                name: "Python".to_string(),
                description: "High-level programming language with dynamic semantics".to_string(),
                icon: "python".to_string(),
                category: "language".to_string(),
                website: "https://python.org/".to_string(),
                install_command: "curl https://pyenv.run | bash".to_string(),
                features: vec![
                    "Interpreted language".to_string(),
                    "Pip package manager".to_string(),
                    "Virtual environments".to_string(),
                    "Extensive libraries".to_string(),
                ],
                default_port: Some(8000),
                manager_type: Some("pyenv".to_string()),
            },
            "java" => SdkDetails {
                id: "java".to_string(),
                name: "Java".to_string(),
                description: "Object-oriented programming language and computing platform".to_string(),
                icon: "java".to_string(),
                category: "language".to_string(),
                website: "https://java.com/".to_string(),
                install_command: "curl -s \"https://get.sdkman.io\" | bash".to_string(),
                features: vec![
                    "Write once, run anywhere".to_string(),
                    "Maven/Gradle build tools".to_string(),
                    "Spring framework".to_string(),
                    "Enterprise applications".to_string(),
                ],
                default_port: Some(8080),
                manager_type: Some("sdkman".to_string()),
            },
            "rust" => SdkDetails {
                id: "rust".to_string(),
                name: "Rust".to_string(),
                description: "Systems programming language focused on safety and performance".to_string(),
                icon: "rust".to_string(),
                category: "language".to_string(),
                website: "https://rust-lang.org/".to_string(),
                install_command: "curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string(),
                features: vec![
                    "Memory safety".to_string(),
                    "Zero-cost abstractions".to_string(),
                    "Cargo package manager".to_string(),
                    "Systems programming".to_string(),
                ],
                default_port: Some(7878),
                manager_type: Some("rustup".to_string()),
            },
            "go" => SdkDetails {
                id: "go".to_string(),
                name: "Go".to_string(),
                description: "Open source programming language for building simple, reliable software".to_string(),
                icon: "go".to_string(),
                category: "language".to_string(),
                website: "https://golang.org/".to_string(),
                install_command: "curl -sSL https://raw.githubusercontent.com/voidint/g/master/install.sh | bash".to_string(),
                features: vec![
                    "Static typing".to_string(),
                    "Goroutines for concurrency".to_string(),
                    "Go modules".to_string(),
                    "Fast compilation".to_string(),
                ],
                default_port: Some(8080),
                manager_type: Some("g".to_string()),
            },
            "php" => SdkDetails {
                id: "php".to_string(),
                name: "PHP".to_string(),
                description: "Server-side scripting language designed for web development".to_string(),
                icon: "php".to_string(),
                category: "language".to_string(),
                website: "https://php.net/".to_string(),
                install_command: "curl -fsSL https://github.com/phpenv/phpenv-installer/raw/HEAD/bin/phpenv-installer | bash".to_string(),
                features: vec![
                    "Web development".to_string(),
                    "Composer package manager".to_string(),
                    "Laravel framework".to_string(),
                    "WordPress support".to_string(),
                ],
                default_port: Some(8000),
                manager_type: Some("phpenv".to_string()),
            },
            "ruby" => SdkDetails {
                id: "ruby".to_string(),
                name: "Ruby".to_string(),
                description: "Dynamic, open source programming language with focus on simplicity".to_string(),
                icon: "ruby".to_string(),
                category: "language".to_string(),
                website: "https://ruby-lang.org/".to_string(),
                install_command: "curl -fsSL https://github.com/rbenv/rbenv-installer/raw/HEAD/bin/rbenv-installer | bash".to_string(),
                features: vec![
                    "RubyGems package manager".to_string(),
                    "Rails framework".to_string(),
                    "Metaprogramming".to_string(),
                    "Object-oriented".to_string(),
                ],
                default_port: Some(3000),
                manager_type: Some("rbenv".to_string()),
            },
            "postgresql" => SdkDetails {
                id: "postgresql".to_string(),
                name: "PostgreSQL".to_string(),
                description: "Advanced open source relational database".to_string(),
                icon: "database".to_string(),
                category: "database".to_string(),
                website: "https://postgresql.org/".to_string(),
                install_command: "brew install postgresql".to_string(),
                features: vec![
                    "ACID compliance".to_string(),
                    "Extensible".to_string(),
                    "Advanced indexing".to_string(),
                    "JSON support".to_string(),
                ],
                default_port: Some(5432),
                manager_type: None,
            },
            "mysql" => SdkDetails {
                id: "mysql".to_string(),
                name: "MySQL".to_string(),
                description: "Open source relational database management system".to_string(),
                icon: "database".to_string(),
                category: "database".to_string(),
                website: "https://mysql.com/".to_string(),
                install_command: "brew install mysql".to_string(),
                features: vec![
                    "High performance".to_string(),
                    "Scalable".to_string(),
                    "Wide compatibility".to_string(),
                    "Replication support".to_string(),
                ],
                default_port: Some(3306),
                manager_type: None,
            },
            "mongodb" => SdkDetails {
                id: "mongodb".to_string(),
                name: "MongoDB".to_string(),
                description: "Document-oriented NoSQL database".to_string(),
                icon: "database".to_string(),
                category: "database".to_string(),
                website: "https://mongodb.com/".to_string(),
                install_command: "brew install mongodb-community".to_string(),
                features: vec![
                    "Document-based".to_string(),
                    "Flexible schema".to_string(),
                    "Horizontal scaling".to_string(),
                    "Rich queries".to_string(),
                ],
                default_port: Some(27017),
                manager_type: None,
            },
            "redis" => SdkDetails {
                id: "redis".to_string(),
                name: "Redis".to_string(),
                description: "In-memory data structure store used as database, cache, and message broker".to_string(),
                icon: "database".to_string(),
                category: "database".to_string(),
                website: "https://redis.io/".to_string(),
                install_command: "brew install redis".to_string(),
                features: vec![
                    "In-memory storage".to_string(),
                    "High performance".to_string(),
                    "Data structures".to_string(),
                    "Pub/Sub messaging".to_string(),
                ],
                default_port: Some(6379),
                manager_type: None,
            },
            "nginx" => SdkDetails {
                id: "nginx".to_string(),
                name: "Nginx".to_string(),
                description: "High performance web server and reverse proxy".to_string(),
                icon: "globe".to_string(),
                category: "web-server".to_string(),
                website: "https://nginx.org/".to_string(),
                install_command: "brew install nginx".to_string(),
                features: vec![
                    "High performance".to_string(),
                    "Reverse proxy".to_string(),
                    "Load balancing".to_string(),
                    "SSL termination".to_string(),
                ],
                default_port: Some(80),
                manager_type: None,
            },
            "apache" => SdkDetails {
                id: "apache".to_string(),
                name: "Apache".to_string(),
                description: "Open source web server software".to_string(),
                icon: "globe".to_string(),
                category: "web-server".to_string(),
                website: "https://httpd.apache.org/".to_string(),
                install_command: "brew install httpd".to_string(),
                features: vec![
                    "Modular architecture".to_string(),
                    "Virtual hosting".to_string(),
                    "SSL support".to_string(),
                    "URL rewriting".to_string(),
                ],
                default_port: Some(80),
                manager_type: None,
            },
            "caddy" => SdkDetails {
                id: "caddy".to_string(),
                name: "Caddy".to_string(),
                description: "Modern web server with automatic HTTPS".to_string(),
                icon: "globe".to_string(),
                category: "web-server".to_string(),
                website: "https://caddyserver.com/".to_string(),
                install_command: "brew install caddy".to_string(),
                features: vec![
                    "Automatic HTTPS".to_string(),
                    "HTTP/2 support".to_string(),
                    "Easy configuration".to_string(),
                    "Plugin system".to_string(),
                ],
                default_port: Some(80),
                manager_type: None,
            },
            "docker" => SdkDetails {
                id: "docker".to_string(),
                name: "Docker".to_string(),
                description: "Containerization platform for developing, shipping, and running applications".to_string(),
                icon: "container".to_string(),
                category: "container".to_string(),
                website: "https://docker.com/".to_string(),
                install_command: "brew install --cask docker".to_string(),
                features: vec![
                    "Containerization".to_string(),
                    "Image management".to_string(),
                    "Orchestration".to_string(),
                    "Multi-platform support".to_string(),
                ],
                default_port: Some(2375),
                manager_type: None,
            },
            "podman" => SdkDetails {
                id: "podman".to_string(),
                name: "Podman".to_string(),
                description: "Daemonless container engine for developing, managing, and running containers".to_string(),
                icon: "container".to_string(),
                category: "container".to_string(),
                website: "https://podman.io/".to_string(),
                install_command: "brew install podman".to_string(),
                features: vec![
                    "Daemonless".to_string(),
                    "Rootless containers".to_string(),
                    "Pod support".to_string(),
                    "Docker compatible".to_string(),
                ],
                default_port: None,
                manager_type: None,
            },
            _ => return Ok(None),
        };

        Ok(Some(sdk_details))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkDetails {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub website: String,
    pub install_command: String,
    pub features: Vec<String>,
    pub default_port: Option<u16>,
    pub manager_type: Option<String>,
}
