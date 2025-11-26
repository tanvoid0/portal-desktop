use std::path::Path;
use std::fs;
use std::process::Command;
use chrono::Utc;
use regex::Regex;

use crate::database::{DatabaseManager, ProjectModel};
use crate::domains::projects::entities::ProjectAnalysis;
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use std::sync::Arc;

pub struct ProjectService {
    repository: ProjectRepository,
}

impl ProjectService {
    pub fn new(db_manager: &Arc<DatabaseManager>) -> Self {
        Self {
            repository: ProjectRepository::new(db_manager.clone()),
        }
    }

    pub async fn get_all_projects(&self) -> Result<Vec<ProjectModel>, String> {
        self.repository.get_all().await
    }

    pub async fn get_project(&self, id: i32) -> Result<Option<ProjectModel>, String> {
        self.repository.get_by_id(id).await
    }

    pub async fn create_project(
        &self,
        name: String,
        description: Option<String>,
        path: String,
        framework_ids: Vec<i32>,
        package_manager_ids: Vec<i32>,
        language_ids: Vec<i32>,
        build_command: Option<String>,
        start_command: Option<String>,
        test_command: Option<String>,
        output_directory: Option<String>,
        dev_port: Option<i32>,
        prod_port: Option<i32>,
    ) -> Result<ProjectModel, String> {
        // Validate project path
        self.validate_project_path(&path).await?;

        self.repository.create(
            name,
            description,
            path,
            framework_ids,
            package_manager_ids,
            language_ids,
            build_command,
            start_command,
            test_command,
            output_directory,
            dev_port,
            prod_port,
        ).await
    }

    pub async fn update_project(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        path: Option<String>,
        status: Option<String>,
        framework_ids: Option<Vec<i32>>,
        package_manager_ids: Option<Vec<i32>>,
        language_ids: Option<Vec<i32>>,
        build_command: Option<String>,
        start_command: Option<String>,
        test_command: Option<String>,
        output_directory: Option<String>,
        dev_port: Option<i32>,
        prod_port: Option<i32>,
    ) -> Result<Option<ProjectModel>, String> {
        self.repository.update(
            id,
            name,
            description,
            path,
            status,
            framework_ids,
            package_manager_ids,
            language_ids,
            build_command,
            start_command,
            test_command,
            output_directory,
            dev_port,
            prod_port,
            None, // starred
            None, // open_count
            None, // last_opened
            None, // size
            None, // file_count
            None, // git_repository
            None, // git_branch
            None, // git_commit
            None, // has_uncommitted_changes
            None, // last_commit
        ).await
    }

    pub async fn delete_project(&self, id: i32) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    pub async fn toggle_project_star(&self, id: i32) -> Result<Option<ProjectModel>, String> {
        // Get current project to check if it exists and get current starred status
        let current_project = self.repository.get_by_id(id).await?;
        if let Some(project) = current_project {
            let new_starred = !project.starred;
            self.repository.update(
                id,
                None, // name
                None, // description
                None, // path
                None, // status
                None, // framework_ids
                None, // package_manager_ids
                None, // language_ids
                None, // build_command
                None, // start_command
                None, // test_command
                None, // output_directory
                None, // dev_port
                None, // prod_port
                Some(new_starred), // starred
                None, // open_count
                None, // last_opened
                None, // size
                None, // file_count
                None, // git_repository
                None, // git_branch
                None, // git_commit
                None, // has_uncommitted_changes
                None, // last_commit
            ).await
        } else {
            Ok(None)
        }
    }

    pub async fn open_project(&self, id: i32) -> Result<Option<ProjectModel>, String> {
        // Get current project to check if it exists and get current open_count
        let current_project = self.repository.get_by_id(id).await?;
        if let Some(project) = current_project {
            let new_open_count = project.open_count + 1;
            self.repository.update(
                id,
                None, // name
                None, // description
                None, // path
                None, // status
                None, // framework_ids
                None, // package_manager_ids
                None, // language_ids
                None, // build_command
                None, // start_command
                None, // test_command
                None, // output_directory
                None, // dev_port
                None, // prod_port
                None, // starred
                Some(new_open_count), // open_count
                Some(Utc::now()), // last_opened
                None, // size
                None, // file_count
                None, // git_repository
                None, // git_branch
                None, // git_commit
                None, // has_uncommitted_changes
                None, // last_commit
            ).await
        } else {
            Ok(None)
        }
    }

    pub async fn refresh_project_metadata(&self, id: i32) -> Result<Option<ProjectModel>, String> {
        // Get current project
        let current_project = self.repository.get_by_id(id).await?;
        if let Some(project) = current_project {
            // Scan project directory
            let (size, file_count) = self.scan_project_directory(&project.path).await?;
            let git_info = self.get_git_info(&project.path).await.ok();
            
            // Update metadata using repository
            self.repository.update(
                id,
                None, // name
                None, // description
                None, // path
                None, // status
                None, // framework_ids
                None, // package_manager_ids
                None, // language_ids
                None, // build_command
                None, // start_command
                None, // test_command
                None, // output_directory
                None, // dev_port
                None, // prod_port
                None, // starred
                None, // open_count
                None, // last_opened
                Some(size), // size
                Some(file_count), // file_count
                git_info.as_ref().and_then(|g| g.repository.clone()), // git_repository
                git_info.as_ref().and_then(|g| g.branch.clone()), // git_branch
                git_info.as_ref().and_then(|g| g.commit.clone()), // git_commit
                git_info.as_ref().map(|g| g.has_uncommitted_changes), // has_uncommitted_changes
                git_info.as_ref().and_then(|g| g.last_commit), // last_commit
            ).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_projects_with_filters(
        &self,
        status_filter: Option<String>,
        sort_by: String,
        search_query: Option<String>,
    ) -> Result<Vec<ProjectModel>, String> {
        // Use repository methods instead of direct database access
        let mut projects = self.repository.get_all().await?;

        // Apply filters

        if let Some(status) = status_filter {
            projects.retain(|p| p.status == status);
        }

        if let Some(query_str) = search_query {
            let query_lower = query_str.to_lowercase();
            projects.retain(|p| {
                p.name.to_lowercase().contains(&query_lower) ||
                p.path.to_lowercase().contains(&query_lower) ||
                p.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
            });
        }

        // Apply sorting
        projects.sort_by(|a, b| {
            match sort_by.as_str() {
                "name" => a.name.cmp(&b.name),
                "name_desc" => b.name.cmp(&a.name),
                "created_at" => b.created_at.unwrap_or_default().cmp(&a.created_at.unwrap_or_default()),
                "updated_at" => b.updated_at.unwrap_or_default().cmp(&a.updated_at.unwrap_or_default()),
                "last_opened" => b.last_opened.unwrap_or_default().cmp(&a.last_opened.unwrap_or_default()),
                "size" => b.size.cmp(&a.size),
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(projects)
    }


    pub async fn get_frameworks(&self) -> Result<Vec<String>, String> {
        // TODO: Load frameworks from junction tables using find_with_related
        // For now, return empty since we're using many-to-many relationships
        Ok(vec![])
    }

    pub async fn validate_project_path(&self, path: &str) -> Result<(), String> {
        let path_obj = Path::new(path);
        if !path_obj.exists() {
            return Err("Project path does not exist".to_string());
        }
        if !path_obj.is_dir() {
            return Err("Project path is not a directory".to_string());
        }
        Ok(())
    }

    pub async fn generate_project_name(&self, path: &str) -> Result<String, String> {
        let path_obj = Path::new(path);
        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project")
            .to_string();
        
        Ok(name)
    }

    pub async fn detect_frameworks(&self, path: &str) -> Result<Vec<String>, String> {
        let path_obj = Path::new(path);
        let mut frameworks = Vec::new();
        
        // Check for common framework indicators (check ALL, not just first)
        let framework_indicators = [
            ("tauri.conf.json", "Tauri"),
            ("angular.json", "Angular"),
            ("vue.config.js", "Vue.js"),
            ("next.config.js", "Next.js"),
            ("nuxt.config.js", "Nuxt.js"),
            ("svelte.config.js", "Svelte"),
            ("svelte.config.ts", "Svelte"),
            ("vite.config.js", "Vite"),
            ("vite.config.ts", "Vite"),
            ("webpack.config.js", "Webpack"),
            ("remix.config.js", "Remix"),
            ("gatsby-config.js", "Gatsby"),
            ("package.json", "Node.js"), // Keep this for general Node.js detection
            ("requirements.txt", "Python"),
            ("pom.xml", "Java/Maven"),
            ("build.gradle", "Java/Gradle"),
            ("Cargo.toml", "Rust"),
            ("go.mod", "Go"),
            ("composer.json", "PHP"),
            ("Gemfile", "Ruby"),
            ("Dockerfile", "Docker"),
            ("docker-compose.yml", "Docker Compose"),
        ];

        for (file, framework) in framework_indicators.iter() {
            if path_obj.join(file).exists() {
                frameworks.push((*framework).to_string());
            }
        }

        // Check for framework-specific directories (only if no frameworks found yet)
        if frameworks.is_empty() {
            let framework_dirs = [
                ("node_modules", "Node.js"),
                ("vendor", "PHP"),
                ("target", "Java/Maven"),
                ("build", "Java/Gradle"),
            ];

            for (dir, framework) in framework_dirs.iter() {
                if path_obj.join(dir).exists() {
                    frameworks.push((*framework).to_string());
                }
            }
        }

        Ok(frameworks)
    }

    pub async fn detect_languages(&self, path: &str) -> Result<Vec<String>, String> {
        let path_obj = Path::new(path);
        let mut languages = Vec::new();
        
        // Language detection based on file extensions and config files
        let language_indicators = [
            ("Cargo.toml", "Rust"),
            ("go.mod", "Go"),
            ("package.json", "JavaScript"), // JavaScript if package.json exists
            ("tsconfig.json", "TypeScript"),
            ("requirements.txt", "Python"),
            ("pom.xml", "Java"),
            ("build.gradle", "Java"),
            ("composer.json", "PHP"),
            ("Gemfile", "Ruby"),
            ("mix.exs", "Elixir"),
        ];

        for (file, language) in language_indicators.iter() {
            if path_obj.join(file).exists() {
                languages.push((*language).to_string());
            }
        }

        // Also check for source files in common directories (more efficient than scanning all files)
        let source_dirs = ["src", "src-tauri/src", "lib", "app"];
        for dir in &source_dirs {
            let dir_path = path_obj.join(dir);
            if dir_path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten().take(20) { // Limit to first 20 files for performance
                        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                            match ext {
                                "rs" if !languages.contains(&"Rust".to_string()) => languages.push("Rust".to_string()),
                                "go" if !languages.contains(&"Go".to_string()) => languages.push("Go".to_string()),
                                "js" if !languages.contains(&"JavaScript".to_string()) => languages.push("JavaScript".to_string()),
                                "ts" | "tsx" if !languages.contains(&"TypeScript".to_string()) => languages.push("TypeScript".to_string()),
                                "py" if !languages.contains(&"Python".to_string()) => languages.push("Python".to_string()),
                                "java" if !languages.contains(&"Java".to_string()) => languages.push("Java".to_string()),
                                "php" if !languages.contains(&"PHP".to_string()) => languages.push("PHP".to_string()),
                                "rb" if !languages.contains(&"Ruby".to_string()) => languages.push("Ruby".to_string()),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        Ok(languages)
    }

    pub async fn analyze_project_directory(&self, path: &str) -> Result<ProjectAnalysis, String> {

        let path_obj = Path::new(path);
        
        // Extract project name from path
        let name = path_obj.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project")
            .to_string();
        
        // Format name with proper capitalization and spaces
        let formatted_name = self.format_project_name(&name);

        // Detect multiple frameworks
        let frameworks = self.detect_frameworks(path).await.unwrap_or_default();

        // Detect multiple languages
        let languages = self.detect_languages(path).await.unwrap_or_default();

        // Detect multiple package managers
        let package_managers = self.detect_package_managers(path).await;

        // For backward compatibility with command detection, use first framework/package manager
        let primary_framework = frameworks.first().cloned();
        let primary_package_manager = package_managers.first().cloned();

        // Detect build/start/test commands
        let (build_command, start_command, test_command) = self.detect_commands(path, &primary_framework, &primary_package_manager).await;

        // Detect output directory
        let output_directory = self.detect_output_directory(path, &primary_framework).await;

        // Detect ports
        let (dev_port, prod_port) = self.detect_ports(path, &primary_framework).await;

        Ok(ProjectAnalysis {
            name: formatted_name,
            frameworks,
            languages,
            package_managers,
            build_command,
            start_command,
            test_command,
            output_directory,
            dev_port,
            prod_port,
        })
    }

    fn format_project_name(&self, name: &str) -> String {
        // Convert kebab-case, snake_case, or camelCase to Title Case
        let formatted = name
            .replace('-', " ")
            .replace('_', " ")
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 || (i > 0 && name.chars().nth(i - 1).map_or(false, |prev| prev == ' ' || prev == '-' || prev == '_')) {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect::<String>()
                }
            })
            .collect::<String>();
        
        // Clean up multiple spaces
        Regex::new(r"\s+").unwrap().replace_all(&formatted, " ").to_string()
    }


    async fn detect_package_managers(&self, path: &str) -> Vec<String> {
        use std::path::Path;
        
        let path_obj = Path::new(path);
        let mut package_managers = Vec::new();
        
        // Check for all package manager indicators (not just first)
        if path_obj.join("package-lock.json").exists() {
            package_managers.push("npm".to_string());
        }
        
        if path_obj.join("yarn.lock").exists() {
            package_managers.push("yarn".to_string());
        }
        
        if path_obj.join("pnpm-lock.yaml").exists() {
            package_managers.push("pnpm".to_string());
        }
        
        if path_obj.join("requirements.txt").exists() {
            package_managers.push("pip".to_string());
        }
        
        if path_obj.join("Cargo.toml").exists() {
            package_managers.push("cargo".to_string());
        }
        
        if path_obj.join("go.mod").exists() {
            package_managers.push("go".to_string());
        }
        
        if path_obj.join("composer.json").exists() {
            package_managers.push("composer".to_string());
        }
        
        if path_obj.join("Gemfile").exists() {
            package_managers.push("bundle".to_string());
        }
        
        package_managers
    }

    async fn detect_commands(&self, path: &str, _framework: &Option<String>, package_manager: &Option<String>) -> (Option<String>, Option<String>, Option<String>) {
        use std::path::Path;
        use std::fs;
        
        let path_obj = Path::new(path);
        let mut build_command = None;
        let mut start_command = None;
        let mut test_command = None;
        
        // Check package.json for scripts
        if let Ok(contents) = fs::read_to_string(path_obj.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                    build_command = scripts.get("build").and_then(|v| v.as_str()).map(|s| s.to_string());
                    start_command = scripts.get("start").or_else(|| scripts.get("dev")).and_then(|v| v.as_str()).map(|s| s.to_string());
                    test_command = scripts.get("test").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
            }
        }
        
        // Set defaults based on package manager if no scripts found
        if build_command.is_none() {
            if let Some(pm) = package_manager {
                match pm.as_str() {
                    "npm" | "yarn" | "pnpm" => build_command = Some("npm run build".to_string()),
                    "cargo" => build_command = Some("cargo build".to_string()),
                    "go" => build_command = Some("go build".to_string()),
                    _ => {}
                }
            }
        }
        
        if start_command.is_none() {
            if let Some(pm) = package_manager {
                match pm.as_str() {
                    "npm" | "yarn" | "pnpm" => start_command = Some("npm start".to_string()),
                    "cargo" => start_command = Some("cargo run".to_string()),
                    "go" => start_command = Some("go run .".to_string()),
                    _ => {}
                }
            }
        }
        
        if test_command.is_none() {
            if let Some(pm) = package_manager {
                match pm.as_str() {
                    "npm" | "yarn" | "pnpm" => test_command = Some("npm test".to_string()),
                    "cargo" => test_command = Some("cargo test".to_string()),
                    "go" => test_command = Some("go test".to_string()),
                    _ => {}
                }
            }
        }
        
        (build_command, start_command, test_command)
    }

    async fn detect_output_directory(&self, path: &str, framework: &Option<String>) -> Option<String> {
        use std::path::Path;
        
        let path_obj = Path::new(path);
        
        // Check for common output directories
        let common_dirs = ["dist", "build", "out", "public", "www"];
        
        for dir in &common_dirs {
            if path_obj.join(dir).exists() {
                return Some(dir.to_string());
            }
        }
        
        // Framework-specific defaults
        if let Some(fw) = framework {
            match fw.as_str() {
                "Vite" | "Svelte" | "Vue.js" => return Some("dist".to_string()),
                "Next.js" => return Some("out".to_string()),
                "Angular" => return Some("dist".to_string()),
                _ => {}
            }
        }
        
        None
    }

    async fn detect_ports(&self, path: &str, framework: &Option<String>) -> (Option<i32>, Option<i32>) {
        use std::path::Path;
        use std::fs;
        use regex::Regex;
        
        let path_obj = Path::new(path);
        let mut dev_port = None;
        let prod_port = None;
        
        // Check for port configurations in common files
        let config_files = ["package.json", "vite.config.js", "next.config.js", "nuxt.config.js", "svelte.config.js"];
        
        for file in &config_files {
            if let Ok(contents) = fs::read_to_string(path_obj.join(file)) {
                // Look for port patterns
                let port_regex = Regex::new(r"port[:\s]*(\d+)").unwrap();
                if let Some(captures) = port_regex.captures(&contents) {
                    if let Ok(port) = captures[1].parse::<i32>() {
                        dev_port = Some(port);
                        break;
                    }
                }
            }
        }
        
        // Framework-specific defaults
        if dev_port.is_none() {
            if let Some(fw) = framework {
                match fw.as_str() {
                    "Vite" | "Svelte" => dev_port = Some(5173),
                    "Next.js" => dev_port = Some(3000),
                    "Vue.js" | "Nuxt.js" => dev_port = Some(3000),
                    "Angular" => dev_port = Some(4200),
                    "React" => dev_port = Some(3000),
                    _ => {}
                }
            }
        }
        
        // Production port is usually 80 or 443, but we'll leave it as None for now
        (dev_port, prod_port)
    }

    pub async fn get_project_stats(&self) -> Result<ProjectStats, String> {
        let projects = self.get_all_projects().await?;
        
        let active_projects = projects.iter().filter(|p| p.status == "active").count() as u32;
        let archived_projects = projects.iter().filter(|p| p.status == "archived").count() as u32;
        
        let total_size: i64 = projects.iter().map(|p| p.size).sum();
        
        // Count frameworks
        // TODO: Load frameworks from junction tables using find_with_related
        // For now, return empty since we're using many-to-many relationships
        let framework_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        
        let most_used_framework = framework_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(framework, _)| framework.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Get recent projects (last 5 opened)
        let mut recent_projects: Vec<ProjectModel> = projects.iter()
            .filter(|p| p.status == "active")
            .cloned()
            .collect();
        
        recent_projects.sort_by(|a, b| {
            let a_time = a.last_opened.map_or(0, |t| t.timestamp());
            let b_time = b.last_opened.map_or(0, |t| t.timestamp());
            b_time.cmp(&a_time)
        });
        
        recent_projects.truncate(5);

        Ok(ProjectStats {
            total_projects: projects.len() as u32,
            active_projects,
            archived_projects,
            total_size,
            most_used_framework,
            recent_projects,
        })
    }

    async fn scan_project_directory(&self, project_path: &str) -> Result<(i64, i32), String> {
        let mut size = 0i64;
        let mut file_count = 0i32;

        fn scan_dir(dir: &Path, size: &mut i64, file_count: &mut i32) -> Result<(), std::io::Error> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if path.is_dir() {
                        // Skip common directories that shouldn't be counted
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if matches!(name, "node_modules" | ".git" | "target" | "dist" | "build") {
                                continue;
                            }
                        }
                        scan_dir(&path, size, file_count)?;
                    } else {
                        *file_count += 1;
                        if let Ok(metadata) = entry.metadata() {
                            *size += metadata.len() as i64;
                        }
                    }
                }
            }
            Ok(())
        }

        let path_obj = Path::new(project_path);
        scan_dir(path_obj, &mut size, &mut file_count)
            .map_err(|e| format!("Failed to scan directory: {}", e))?;

        Ok((size, file_count))
    }

    async fn get_git_info(&self, project_path: &str) -> Result<GitInfo, String> {
        let git_dir = Path::new(project_path).join(".git");
        if !git_dir.exists() {
            return Err("Not a git repository".to_string());
        }

        // Get current branch
        let branch_output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(project_path)
            .output();

        let branch = if let Ok(output) = branch_output {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Get last commit
        let commit_output = Command::new("git")
            .args(&["log", "-1", "--format=%H"])
            .current_dir(project_path)
            .output();

        let commit = if let Ok(output) = commit_output {
            if output.status.success() {
                String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Check for uncommitted changes
        let status_output = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(project_path)
            .output();

        let has_uncommitted_changes = if let Ok(output) = status_output {
            output.status.success() && !output.stdout.is_empty()
        } else {
            false
        };

        Ok(GitInfo {
            repository: None, // Could be extracted from git remote
            branch,
            commit,
            has_uncommitted_changes,
            last_commit: None, // Could be extracted from git log
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectStats {
    pub total_projects: u32,
    pub active_projects: u32,
    pub archived_projects: u32,
    pub total_size: i64,
    pub most_used_framework: String,
    pub recent_projects: Vec<ProjectModel>,
}

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit: Option<chrono::DateTime<Utc>>,
}
