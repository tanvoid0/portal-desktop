/**
 * SDK Configuration Definitions
 * 
 * All SDK configurations are defined here in the backend, organized by category.
 * Frontend receives processed, formatted data from backend services.
 * 
 * Categories:
 * - Language: Programming languages (Python, Node.js, Java, etc.)
 * - Database: Database systems (PostgreSQL, MySQL, etc.)
 * - AI: AI/ML tools (Ollama, etc.)
 * - Server: Web servers (Nginx, Apache, etc.)
 * - Container: Container platforms (Docker, Kubernetes, etc.)
 * - Package: Package managers (npm, pip, etc.)
 */

use super::types::*;

/// Get configuration for a specific SDK (by ID)
pub fn get_sdk_config(sdk_id: &str) -> Option<SDKConfig> {
    // Try language SDKs first
    match sdk_id.to_lowercase().as_str() {
        "python" => Some(python_config()),
        "nodejs" | "node" => Some(nodejs_config()),
        "java" => Some(java_config()),
        "rust" => Some(rust_config()),
        "go" | "golang" => Some(go_config()),
        // Add other categories as needed
        "postgresql" => Some(postgresql_config()),
        "mysql" => Some(mysql_config()),
        "mongodb" => Some(mongodb_config()),
        "ollama" => Some(ollama_config()),
        "nginx" => Some(nginx_config()),
        "docker" => Some(docker_config()),
        _ => None,
    }
}

/// Get all SDK configurations
pub fn get_all_sdk_configs() -> Vec<SDKConfig> {
    vec![
        // Language SDKs
        python_config(),
        nodejs_config(),
        java_config(),
        rust_config(),
        go_config(),
        // Database SDKs
        postgresql_config(),
        mysql_config(),
        mongodb_config(),
        // AI SDKs
        ollama_config(),
        // Server SDKs
        nginx_config(),
        // Container SDKs
        docker_config(),
    ]
}

/// Get SDKs by category
pub fn get_sdks_by_category(category: SDKCategory) -> Vec<SDKConfig> {
    get_all_sdk_configs()
        .into_iter()
        .filter(|config| config.category == category)
        .collect()
}

/// Check if an SDK configuration exists
pub fn has_sdk_config(sdk_id: &str) -> bool {
    get_sdk_config(sdk_id).is_some()
}

// ========== Language SDK Configs ==========

fn python_config() -> SDKConfig {
    SDKConfig {
        id: "python".to_string(),
        name: "Python".to_string(),
        display_name: "Python".to_string(),
        description: "Python programming language and runtime".to_string(),
        icon: "devicon-python-plain".to_string(),
        category: SDKCategory::Language,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "package-manager".to_string(), label: "Package Manager".to_string(), enabled: true },
            SDKTabConfig { id: "projects".to_string(), label: "Projects".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::Static,
            VersionSource::SdkManager,
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::SdkManager),
        sdk_managers: vec![
            SDKManagerConfig {
                id: "pyenv".to_string(),
                name: "pyenv".to_string(),
                display_name: "Pyenv".to_string(),
                binary: "pyenv".to_string(),
                version_command: Some("pyenv --version".to_string()),
                supports_installation: true,
                supports_version_switching: true,
                install_command: Some("curl https://pyenv.run | bash".to_string()),
                website: Some("https://github.com/pyenv/pyenv".to_string()),
            },
            SDKManagerConfig {
                id: "conda".to_string(),
                name: "conda".to_string(),
                display_name: "Conda".to_string(),
                binary: "conda".to_string(),
                version_command: Some("conda --version".to_string()),
                supports_installation: true,
                supports_version_switching: true,
                install_command: Some("curl -O https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh && bash Miniconda3-latest-Linux-x86_64.sh".to_string()),
                website: Some("https://docs.conda.io/".to_string()),
            },
        ],
        package_managers: vec![
            PackageManagerConfig {
                id: "pip".to_string(),
                name: "pip".to_string(),
                display_name: "pip".to_string(),
                binary: "pip".to_string(),
                version_command: Some("pip --version".to_string()),
                install_command: Some("python -m ensurepip --upgrade".to_string()),
                website: Some("https://pip.pypa.io/".to_string()),
            },
            PackageManagerConfig {
                id: "pip3".to_string(),
                name: "pip3".to_string(),
                display_name: "pip3".to_string(),
                binary: "pip3".to_string(),
                version_command: Some("pip3 --version".to_string()),
                install_command: None,
                website: Some("https://pip.pypa.io/".to_string()),
            },
        ],
        detection: DetectionMethod {
            binary_names: vec!["python".to_string(), "python3".to_string(), "python2".to_string()],
            version_command: Some("python --version".to_string()),
            path_patterns: vec![
                "/usr/bin/python*".to_string(),
                "/usr/local/bin/python*".to_string(),
                "~/.pyenv/versions/*".to_string(),
                "~/.local/bin/python*".to_string(),
            ],
            version_file_patterns: vec![
                ".python-version".to_string(),
                "pyproject.toml".to_string(),
                "requirements.txt".to_string(),
            ],
        },
        category_features: Some(serde_json::json!({
            "virtualEnvironments": true,
            "packageManagement": true,
            "projectSupport": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "PYTHONPATH": "${PYTHONPATH}",
                "VIRTUAL_ENV": "${VIRTUAL_ENV}",
                "PYENV_ROOT": "${HOME}/.pyenv",
                "PYENV_VERSION": "${PYENV_VERSION}"
            },
            "defaultScope": "project"
        })),
        service_config: None,
    }
}

fn nodejs_config() -> SDKConfig {
    SDKConfig {
        id: "nodejs".to_string(),
        name: "Node.js".to_string(),
        display_name: "Node.js".to_string(),
        description: "JavaScript runtime built on Chrome's V8 JavaScript engine".to_string(),
        icon: "devicon-nodejs-plain".to_string(),
        category: SDKCategory::Language,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "package-manager".to_string(), label: "Package Manager".to_string(), enabled: true },
            SDKTabConfig { id: "projects".to_string(), label: "Projects".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::Static,
            VersionSource::SdkManager,
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::SdkManager),
        sdk_managers: vec![
            SDKManagerConfig {
                id: "nvm".to_string(),
                name: "nvm".to_string(),
                display_name: "NVM (Node Version Manager)".to_string(),
                binary: "nvm".to_string(),
                version_command: Some("nvm --version".to_string()),
                supports_installation: true,
                supports_version_switching: true,
                install_command: Some("curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash".to_string()),
                website: Some("https://github.com/nvm-sh/nvm".to_string()),
            },
            SDKManagerConfig {
                id: "fnm".to_string(),
                name: "fnm".to_string(),
                display_name: "Fast Node Manager".to_string(),
                binary: "fnm".to_string(),
                version_command: Some("fnm --version".to_string()),
                supports_installation: true,
                supports_version_switching: true,
                install_command: Some("curl -fsSL https://fnm.vercel.app/install | bash".to_string()),
                website: Some("https://github.com/Schniz/fnm".to_string()),
            },
        ],
        package_managers: vec![
            PackageManagerConfig {
                id: "npm".to_string(),
                name: "npm".to_string(),
                display_name: "npm".to_string(),
                binary: "npm".to_string(),
                version_command: Some("npm --version".to_string()),
                install_command: None,
                website: Some("https://www.npmjs.com/".to_string()),
            },
            PackageManagerConfig {
                id: "yarn".to_string(),
                name: "yarn".to_string(),
                display_name: "Yarn".to_string(),
                binary: "yarn".to_string(),
                version_command: Some("yarn --version".to_string()),
                install_command: Some("npm install -g yarn".to_string()),
                website: Some("https://yarnpkg.com/".to_string()),
            },
            PackageManagerConfig {
                id: "pnpm".to_string(),
                name: "pnpm".to_string(),
                display_name: "pnpm".to_string(),
                binary: "pnpm".to_string(),
                version_command: Some("pnpm --version".to_string()),
                install_command: Some("npm install -g pnpm".to_string()),
                website: Some("https://pnpm.io/".to_string()),
            },
        ],
        detection: DetectionMethod {
            binary_names: vec!["node".to_string(), "nodejs".to_string()],
            version_command: Some("node --version".to_string()),
            path_patterns: vec![
                "/usr/bin/node".to_string(),
                "/usr/local/bin/node".to_string(),
                "~/.nvm/versions/node/*".to_string(),
                "~/.fnm/node-versions/*".to_string(),
            ],
            version_file_patterns: vec![
                ".nvmrc".to_string(),
                "package.json".to_string(),
                ".node-version".to_string(),
            ],
        },
        category_features: Some(serde_json::json!({
            "packageManagement": true,
            "projectSupport": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "NODE_PATH": "${NODE_PATH}",
                "NVM_DIR": "${HOME}/.nvm",
                "NVM_VERSION": "${NVM_VERSION}",
                "NODE_ENV": "development"
            },
            "defaultScope": "project"
        })),
        service_config: None,
    }
}

fn java_config() -> SDKConfig {
    SDKConfig {
        id: "java".to_string(),
        name: "Java".to_string(),
        display_name: "Java".to_string(),
        description: "Java Development Kit (JDK)".to_string(),
        icon: "devicon-java-plain".to_string(),
        category: SDKCategory::Language,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "maven".to_string(), label: "Maven".to_string(), enabled: true },
            SDKTabConfig { id: "projects".to_string(), label: "Java Projects".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::Static,
            VersionSource::SdkManager,
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::SdkManager),
        sdk_managers: vec![
            SDKManagerConfig {
                id: "sdkman".to_string(),
                name: "sdkman".to_string(),
                display_name: "SDKMAN!".to_string(),
                binary: "sdk".to_string(),
                version_command: Some("sdk version".to_string()),
                supports_installation: true,
                supports_version_switching: true,
                install_command: Some("curl -s \"https://get.sdkman.io\" | bash".to_string()),
                website: Some("https://sdkman.io/".to_string()),
            },
        ],
        package_managers: vec![
            PackageManagerConfig {
                id: "maven".to_string(),
                name: "maven".to_string(),
                display_name: "Maven".to_string(),
                binary: "mvn".to_string(),
                version_command: Some("mvn --version".to_string()),
                install_command: Some("sdk install maven".to_string()),
                website: Some("https://maven.apache.org/".to_string()),
            },
            PackageManagerConfig {
                id: "gradle".to_string(),
                name: "gradle".to_string(),
                display_name: "Gradle".to_string(),
                binary: "gradle".to_string(),
                version_command: Some("gradle --version".to_string()),
                install_command: Some("sdk install gradle".to_string()),
                website: Some("https://gradle.org/".to_string()),
            },
        ],
        detection: DetectionMethod {
            binary_names: vec!["java".to_string(), "javac".to_string()],
            version_command: Some("java -version".to_string()),
            path_patterns: vec![
                "/usr/bin/java".to_string(),
                "/usr/local/bin/java".to_string(),
                "/Library/Java/JavaVirtualMachines/*/Contents/Home/bin/java".to_string(),
                "~/.sdkman/candidates/java/*/bin/java".to_string(),
            ],
            version_file_patterns: vec![
                ".java-version".to_string(),
                "pom.xml".to_string(),
                "build.gradle".to_string(),
            ],
        },
        category_features: Some(serde_json::json!({
            "packageManagement": true,
            "projectSupport": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "JAVA_HOME": "${JAVA_HOME}",
                "JDK_HOME": "${JDK_HOME}",
                "SDKMAN_DIR": "${HOME}/.sdkman",
                "SDKMAN_CANDIDATES_DIR": "${SDKMAN_DIR}/candidates"
            },
            "defaultScope": "global"
        })),
        service_config: None,
    }
}

fn rust_config() -> SDKConfig {
    SDKConfig {
        id: "rust".to_string(),
        name: "Rust".to_string(),
        display_name: "Rust".to_string(),
        description: "Rust programming language and toolchain".to_string(),
        icon: "devicon-rust-plain".to_string(),
        category: SDKCategory::Language,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: false },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "package-manager".to_string(), label: "Package Manager".to_string(), enabled: true },
            SDKTabConfig { id: "projects".to_string(), label: "Projects".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::Static,
            VersionSource::SdkManager,
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::SdkManager),
        sdk_managers: vec![
            SDKManagerConfig {
                id: "rustup".to_string(),
                name: "rustup".to_string(),
                display_name: "Rustup".to_string(),
                binary: "rustup".to_string(),
                version_command: Some("rustup --version".to_string()),
                supports_installation: true,
                supports_version_switching: true,
                install_command: Some("curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string()),
                website: Some("https://rustup.rs/".to_string()),
            },
        ],
        package_managers: vec![
            PackageManagerConfig {
                id: "cargo".to_string(),
                name: "cargo".to_string(),
                display_name: "Cargo".to_string(),
                binary: "cargo".to_string(),
                version_command: Some("cargo --version".to_string()),
                install_command: None,
                website: Some("https://doc.rust-lang.org/cargo/".to_string()),
            },
        ],
        detection: DetectionMethod {
            binary_names: vec!["rustc".to_string(), "cargo".to_string(), "rustup".to_string()],
            version_command: Some("rustc --version".to_string()),
            path_patterns: vec![
                "/usr/bin/rustc".to_string(),
                "/usr/local/bin/rustc".to_string(),
                "~/.cargo/bin/rustc".to_string(),
                "~/.rustup/toolchains/*/bin/rustc".to_string(),
            ],
            version_file_patterns: vec![
                "rust-toolchain.toml".to_string(),
                "rust-toolchain".to_string(),
                "Cargo.toml".to_string(),
            ],
        },
        category_features: Some(serde_json::json!({
            "packageManagement": true,
            "projectSupport": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "RUSTUP_HOME": "${HOME}/.rustup",
                "CARGO_HOME": "${HOME}/.cargo",
                "RUSTUP_TOOLCHAIN": "${RUSTUP_TOOLCHAIN}",
                "PATH": "${CARGO_HOME}/bin:${PATH}"
            },
            "defaultScope": "global"
        })),
        service_config: None,
    }
}

fn go_config() -> SDKConfig {
    SDKConfig {
        id: "go".to_string(),
        name: "Go".to_string(),
        display_name: "Go".to_string(),
        description: "Go programming language".to_string(),
        icon: "devicon-go-plain".to_string(),
        category: SDKCategory::Language,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: false },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "package-manager".to_string(), label: "Package Manager".to_string(), enabled: true },
            SDKTabConfig { id: "projects".to_string(), label: "Projects".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::Static,
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![
            PackageManagerConfig {
                id: "go".to_string(),
                name: "go".to_string(),
                display_name: "go".to_string(),
                binary: "go".to_string(),
                version_command: Some("go version".to_string()),
                install_command: None,
                website: Some("https://go.dev/".to_string()),
            },
        ],
        detection: DetectionMethod {
            binary_names: vec!["go".to_string()],
            version_command: Some("go version".to_string()),
            path_patterns: vec![
                "/usr/bin/go".to_string(),
                "/usr/local/bin/go".to_string(),
                "/usr/local/go/bin/go".to_string(),
                "~/go/bin/go".to_string(),
            ],
            version_file_patterns: vec![
                "go.mod".to_string(),
                "go.work".to_string(),
            ],
        },
        category_features: Some(serde_json::json!({
            "packageManagement": true,
            "projectSupport": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "GOROOT": "${GOROOT}",
                "GOPATH": "${HOME}/go",
                "GOBIN": "${GOPATH}/bin",
                "PATH": "${GOROOT}/bin:${GOPATH}/bin:${PATH}"
            },
            "defaultScope": "global"
        })),
        service_config: None,
    }
}

// ========== Database SDK Configs ==========

fn postgresql_config() -> SDKConfig {
    SDKConfig {
        id: "postgresql".to_string(),
        name: "PostgreSQL".to_string(),
        display_name: "PostgreSQL".to_string(),
        description: "PostgreSQL database server".to_string(),
        icon: "devicon-postgresql-plain".to_string(),
        category: SDKCategory::Database,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "configuration".to_string(), label: "Configuration".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![],
        detection: DetectionMethod {
            binary_names: vec!["psql".to_string(), "postgres".to_string()],
            version_command: Some("psql --version".to_string()),
            path_patterns: vec!["/usr/bin/psql".to_string(), "/usr/local/bin/psql".to_string()],
            version_file_patterns: vec![],
        },
        category_features: Some(serde_json::json!({
            "databaseManagement": true,
            "serviceManagement": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "PGHOST": "localhost",
                "PGPORT": "5432",
                "PGUSER": "${USER}",
                "PGDATABASE": "postgres",
                "PGPASSWORD": ""
            },
            "defaultScope": "global"
        })),
        service_config: Some(serde_json::json!({
            "port": 5432,
            "dataDir": "/var/lib/postgresql",
            "configFile": "/etc/postgresql/postgresql.conf"
        })),
    }
}

fn mysql_config() -> SDKConfig {
    SDKConfig {
        id: "mysql".to_string(),
        name: "MySQL".to_string(),
        display_name: "MySQL".to_string(),
        description: "MySQL database server".to_string(),
        icon: "devicon-mysql-plain".to_string(),
        category: SDKCategory::Database,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "configuration".to_string(), label: "Configuration".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![],
        detection: DetectionMethod {
            binary_names: vec!["mysql".to_string(), "mysqld".to_string()],
            version_command: Some("mysql --version".to_string()),
            path_patterns: vec!["/usr/bin/mysql".to_string(), "/usr/local/bin/mysql".to_string()],
            version_file_patterns: vec![],
        },
        category_features: Some(serde_json::json!({
            "databaseManagement": true,
            "serviceManagement": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "MYSQL_HOST": "localhost",
                "MYSQL_PORT": "3306",
                "MYSQL_USER": "root",
                "MYSQL_PASSWORD": ""
            },
            "defaultScope": "global"
        })),
        service_config: Some(serde_json::json!({
            "port": 3306,
            "dataDir": "/var/lib/mysql",
            "configFile": "/etc/mysql/my.cnf"
        })),
    }
}

fn mongodb_config() -> SDKConfig {
    SDKConfig {
        id: "mongodb".to_string(),
        name: "MongoDB".to_string(),
        display_name: "MongoDB".to_string(),
        description: "MongoDB NoSQL database server".to_string(),
        icon: "devicon-mongodb-plain".to_string(),
        category: SDKCategory::Database,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "configuration".to_string(), label: "Configuration".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![],
        detection: DetectionMethod {
            binary_names: vec!["mongod".to_string(), "mongo".to_string(), "mongosh".to_string()],
            version_command: Some("mongod --version".to_string()),
            path_patterns: vec![
                "/usr/bin/mongod".to_string(),
                "/usr/local/bin/mongod".to_string(),
                "/opt/mongodb/bin/mongod".to_string(),
            ],
            version_file_patterns: vec![],
        },
        category_features: Some(serde_json::json!({
            "databaseManagement": true,
            "serviceManagement": true,
            "nosql": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "MONGO_HOST": "localhost",
                "MONGO_PORT": "27017",
                "MONGO_DB": "admin",
                "MONGO_USER": "",
                "MONGO_PASSWORD": ""
            },
            "defaultScope": "global"
        })),
        service_config: Some(serde_json::json!({
            "port": 27017,
            "dataDir": "/var/lib/mongodb",
            "configFile": "/etc/mongod.conf"
        })),
    }
}

// ========== AI SDK Configs ==========

fn ollama_config() -> SDKConfig {
    SDKConfig {
        id: "ollama".to_string(),
        name: "Ollama".to_string(),
        display_name: "Ollama".to_string(),
        description: "Ollama AI runtime".to_string(),
        icon: "ollama".to_string(),
        category: SDKCategory::AI,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "models".to_string(), label: "Models".to_string(), enabled: true },
            SDKTabConfig { id: "configuration".to_string(), label: "Configuration".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::Static,
            VersionSource::System,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![],
        detection: DetectionMethod {
            binary_names: vec!["ollama".to_string()],
            version_command: Some("ollama --version".to_string()),
            path_patterns: vec!["~/.local/bin/ollama".to_string(), "/usr/local/bin/ollama".to_string()],
            version_file_patterns: vec![],
        },
        category_features: Some(serde_json::json!({
            "modelManagement": true,
            "serviceManagement": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "OLLAMA_HOST": "localhost",
                "OLLAMA_PORT": "11434"
            },
            "defaultScope": "global"
        })),
        service_config: Some(serde_json::json!({
            "port": 11434,
            "host": "localhost"
        })),
    }
}

// ========== Server SDK Configs ==========

fn nginx_config() -> SDKConfig {
    SDKConfig {
        id: "nginx".to_string(),
        name: "Nginx".to_string(),
        display_name: "Nginx".to_string(),
        description: "Nginx web server".to_string(),
        icon: "devicon-nginx-plain".to_string(),
        category: SDKCategory::Server,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "configuration".to_string(), label: "Configuration".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::System,
            VersionSource::Custom,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![],
        detection: DetectionMethod {
            binary_names: vec!["nginx".to_string()],
            version_command: Some("nginx -v".to_string()),
            path_patterns: vec!["/usr/sbin/nginx".to_string(), "/usr/local/bin/nginx".to_string()],
            version_file_patterns: vec![],
        },
        category_features: Some(serde_json::json!({
            "serviceManagement": true,
            "configurationManagement": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "NGINX_CONF": "/etc/nginx/nginx.conf"
            },
            "defaultScope": "global"
        })),
        service_config: Some(serde_json::json!({
            "port": 80,
            "configFile": "/etc/nginx/nginx.conf"
        })),
    }
}

// ========== Container SDK Configs ==========

fn docker_config() -> SDKConfig {
    SDKConfig {
        id: "docker".to_string(),
        name: "Docker".to_string(),
        display_name: "Docker".to_string(),
        description: "Docker container platform".to_string(),
        icon: "devicon-docker-plain".to_string(),
        category: SDKCategory::Container,
        tabs: vec![
            SDKTabConfig { id: "service".to_string(), label: "Service".to_string(), enabled: true },
            SDKTabConfig { id: "version".to_string(), label: "Version".to_string(), enabled: true },
            SDKTabConfig { id: "containers".to_string(), label: "Containers".to_string(), enabled: true },
        ],
        supported_sources: vec![
            VersionSource::System,
        ],
        default_source: Some(VersionSource::System),
        sdk_managers: vec![],
        package_managers: vec![],
        detection: DetectionMethod {
            binary_names: vec!["docker".to_string()],
            version_command: Some("docker --version".to_string()),
            path_patterns: vec!["/usr/bin/docker".to_string(), "/usr/local/bin/docker".to_string()],
            version_file_patterns: vec![],
        },
        category_features: Some(serde_json::json!({
            "containerManagement": true,
            "serviceManagement": true
        })),
        environment_variables: Some(serde_json::json!({
            "templates": {
                "DOCKER_HOST": "unix:///var/run/docker.sock"
            },
            "defaultScope": "global"
        })),
        service_config: Some(serde_json::json!({
            "socketPath": "/var/run/docker.sock"
        })),
    }
}
