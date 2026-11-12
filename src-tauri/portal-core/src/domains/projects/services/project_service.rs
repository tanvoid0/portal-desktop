use crate::process_ext::NoWindowExt;
use chrono::Utc;
use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::database::DatabaseManager;
use crate::domains::projects::entities::ProjectAnalysis;
use crate::domains::projects::entities::ProjectResponse;
use crate::domains::projects::modules::stack::{self, Stack};
use crate::domains::projects::move_lock;
use crate::domains::projects::repositories::project_repository::ProjectRepository;
use crate::error::{AppError, AppResult};
use std::sync::Arc;

/// [`stack::detect`] off the async runtime: it reads the disk.
pub async fn detect_stack(path: &str) -> Stack {
    let root = std::path::PathBuf::from(path);
    tokio::task::spawn_blocking(move || stack::detect(&root))
        .await
        .unwrap_or_default()
}

/// A command for an editor executable. On Windows an extensionless path (VS
/// Code's `bin\code` is a shell script beside `code.cmd`) or a bare name cannot
/// be spawned directly, so it resolves to the real `.exe`/`.cmd` beside it or on
/// PATH. Never through `cmd /C`: the project path is an argument, and a folder
/// named `a&calc` must not reach a shell.
fn editor_command(executable: &str) -> Command {
    if cfg!(windows) && Path::new(executable).extension().is_none() {
        let exe = Path::new(executable);
        let dirs: Vec<std::path::PathBuf> = if exe.components().count() > 1 {
            vec![exe.parent().map(Path::to_path_buf).unwrap_or_default()]
        } else {
            std::env::var_os("PATH")
                .map(|p| std::env::split_paths(&p).collect())
                .unwrap_or_default()
        };
        let name = exe.file_name().and_then(|n| n.to_str()).unwrap_or(executable);
        for dir in dirs {
            for ext in ["exe", "cmd", "bat"] {
                let candidate = dir.join(format!("{name}.{ext}"));
                if candidate.is_file() {
                    return Command::new(candidate);
                }
            }
        }
    }
    Command::new(executable)
}

pub struct ProjectService {
    db: Arc<DatabaseManager>,
    repository: ProjectRepository,
}

impl ProjectService {
    pub fn new(db_manager: &Arc<DatabaseManager>) -> Self {
        Self {
            db: db_manager.clone(),
            repository: ProjectRepository::new(db_manager.clone()),
        }
    }

    pub async fn get_all_projects(&self) -> AppResult<Vec<ProjectResponse>> {
        self.repository.get_all().await.map_err(AppError::from)
    }

    pub async fn get_project(&self, id: i32) -> AppResult<Option<ProjectResponse>> {
        self.repository.get_by_id(id).await.map_err(AppError::from)
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
    ) -> Result<ProjectResponse, String> {
        // Validate project path
        self.validate_project_path(&path).await?;

        self.repository
            .create(
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
            )
            .await
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
    ) -> Result<Option<ProjectResponse>, String> {
        if let Some(current) = self.repository.get_by_id(id).await? {
            move_lock::ensure_unlocked(Path::new(&current.path))?;
        }
        if let Some(p) = &path {
            move_lock::ensure_unlocked(Path::new(p))?;
        }
        self.repository
            .update(
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
            )
            .await
    }

    pub async fn delete_project(&self, id: i32) -> Result<bool, String> {
        if let Some(current) = self.repository.get_by_id(id).await? {
            move_lock::ensure_unlocked(Path::new(&current.path))?;
        }
        self.repository.delete(id).await
    }

    /// Manually group a project under another, or ungroup it (`parent_id = None`).
    pub async fn set_project_parent(
        &self,
        id: i32,
        parent_id: Option<i32>,
    ) -> Result<Option<ProjectResponse>, String> {
        if parent_id == Some(id) {
            return Err("A project cannot be its own parent".to_string());
        }
        self.repository.set_parent(id, parent_id).await
    }

    /// Sets or clears a project's SDLC stage (`idea | in_progress | stable |
    /// released`, or `None` to clear). Rejects anything else rather than
    /// writing garbage the frontend won't recognize.
    pub async fn set_project_stage(
        &self,
        id: i32,
        stage: Option<String>,
    ) -> Result<Option<ProjectResponse>, String> {
        self.repository.set_stage(id, validate_stage(stage)?).await
    }

    /// Flags a project archived or active via its own `archived` column —
    /// independent of `status`, which the disk-scan import writes for an
    /// unrelated reason (see the `archived` migration's doc comment).
    pub async fn set_project_archived(
        &self,
        id: i32,
        archived: bool,
    ) -> Result<Option<ProjectResponse>, String> {
        self.repository.set_archived(id, archived).await
    }

    pub async fn toggle_project_star(&self, id: i32) -> Result<Option<ProjectResponse>, String> {
        // Get current project to check if it exists and get current starred status
        let current_project = self.repository.get_by_id(id).await?;
        if let Some(project) = current_project {
            let new_starred = !project.starred;
            self.repository
                .update(
                    id,
                    None,              // name
                    None,              // description
                    None,              // path
                    None,              // status
                    None,              // framework_ids
                    None,              // package_manager_ids
                    None,              // language_ids
                    None,              // build_command
                    None,              // start_command
                    None,              // test_command
                    None,              // output_directory
                    None,              // dev_port
                    None,              // prod_port
                    Some(new_starred), // starred
                    None,              // open_count
                    None,              // last_opened
                    None,              // size
                    None,              // file_count
                    None,              // git_repository
                    None,              // git_branch
                    None,              // git_commit
                    None,              // has_uncommitted_changes
                    None,              // last_commit
                )
                .await
        } else {
            Ok(None)
        }
    }

    pub async fn open_project(&self, id: i32) -> Result<Option<ProjectResponse>, String> {
        // Get current project to check if it exists and get current open_count
        let current_project = self.repository.get_by_id(id).await?;
        if let Some(project) = current_project {
            let new_open_count = project.open_count + 1;
            self.repository
                .update(
                    id,
                    None,                 // name
                    None,                 // description
                    None,                 // path
                    None,                 // status
                    None,                 // framework_ids
                    None,                 // package_manager_ids
                    None,                 // language_ids
                    None,                 // build_command
                    None,                 // start_command
                    None,                 // test_command
                    None,                 // output_directory
                    None,                 // dev_port
                    None,                 // prod_port
                    None,                 // starred
                    Some(new_open_count), // open_count
                    Some(Utc::now()),     // last_opened
                    None,                 // size
                    None,                 // file_count
                    None,                 // git_repository
                    None,                 // git_branch
                    None,                 // git_commit
                    None,                 // has_uncommitted_changes
                    None,                 // last_commit
                )
                .await
        } else {
            Ok(None)
        }
    }

    /// Re-reads one project from disk: size, git, and the stack (overwriting what
    /// was detected before, per [`Self::store_stack`]).
    pub async fn refresh_project_metadata(
        &self,
        id: i32,
    ) -> Result<Option<ProjectResponse>, String> {
        let Some(project) = self.repository.get_by_id(id).await? else {
            return Ok(None);
        };
        move_lock::ensure_unlocked(Path::new(&project.path))?;
        let (size, file_count) = self.scan_project_directory(&project.path).await?;
        let git_info = self.get_git_info(&project.path).await.ok();
        let stack = detect_stack(&project.path).await;
        self.store_stack(&project, &stack, true).await?;
        let primary_framework = stack.frameworks.first().cloned();
        let output_directory = self
            .detect_output_directory(&project.path, &primary_framework)
            .await;
        let (dev_port, prod_port) = self.detect_ports(&project.path, &primary_framework).await;

        self.repository
            .update(
                id,
                None, // name
                None, // description
                None, // path
                None, // status
                None, // framework_ids: written by store_stack
                None, // package_manager_ids
                None, // language_ids
                None, // build_command: filled by store_stack when blank
                None, // start_command
                None, // test_command
                output_directory,
                dev_port,
                prod_port,
                None, // starred
                None, // open_count
                None, // last_opened
                Some(size),
                Some(file_count),
                git_info.as_ref().and_then(|g| g.repository.clone()),
                git_info.as_ref().and_then(|g| g.branch.clone()),
                git_info.as_ref().and_then(|g| g.commit.clone()),
                git_info.as_ref().map(|g| g.has_uncommitted_changes),
                git_info.as_ref().and_then(|g| g.last_commit),
            )
            .await
    }

    pub async fn get_projects_with_filters(
        &self,
        status_filter: Option<String>,
        sort_by: String,
        search_query: Option<String>,
    ) -> Result<Vec<ProjectResponse>, String> {
        // Use repository methods instead of direct database access
        let mut projects = self.repository.get_all().await?;

        // Apply filters

        if let Some(status) = status_filter {
            projects.retain(|p| p.status == status);
        }

        if let Some(query_str) = search_query {
            let query_lower = query_str.to_lowercase();
            projects.retain(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.path.to_lowercase().contains(&query_lower)
                    || p.description
                        .as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&query_lower))
            });
        }

        // Apply sorting
        projects.sort_by(|a, b| match sort_by.as_str() {
            "name" => a.name.cmp(&b.name),
            "name_desc" => b.name.cmp(&a.name),
            "created_at" => b
                .created_at
                .unwrap_or_default()
                .cmp(&a.created_at.unwrap_or_default()),
            "updated_at" => b
                .updated_at
                .unwrap_or_default()
                .cmp(&a.updated_at.unwrap_or_default()),
            "last_opened" => b
                .last_opened
                .unwrap_or_default()
                .cmp(&a.last_opened.unwrap_or_default()),
            "size" => b.size.cmp(&a.size),
            _ => a.name.cmp(&b.name),
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
        Ok(detect_stack(path).await.frameworks)
    }

    pub async fn analyze_project_directory(&self, path: &str) -> Result<ProjectAnalysis, String> {
        let name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Project")
            .to_string();
        let stack = detect_stack(path).await;
        let primary_framework = stack.frameworks.first().cloned();
        let output_directory = self.detect_output_directory(path, &primary_framework).await;
        let (dev_port, prod_port) = self.detect_ports(path, &primary_framework).await;

        Ok(ProjectAnalysis {
            name: self.format_project_name(&name),
            frameworks: stack.frameworks,
            languages: stack.languages,
            package_managers: stack.package_managers,
            build_command: stack.build,
            start_command: stack.start,
            test_command: stack.test,
            output_directory,
            dev_port,
            prod_port,
        })
    }

    /// Saves a detected stack onto a project. With `overwrite` false a project that
    /// already has a stack is left alone, so bulk rescans keep hand-picked values.
    /// A category detection found nothing for is never cleared, and run commands
    /// are only filled where blank, so a hand-edited command survives a re-detect.
    pub async fn store_stack(
        &self,
        project: &ProjectResponse,
        stack: &Stack,
        overwrite: bool,
    ) -> Result<(), String> {
        // Fill-only callers hold a snapshot from before a slow scan; re-read so a
        // stack the user saved meanwhile is not overwritten.
        let fresh;
        let project = if overwrite {
            project
        } else {
            fresh = self.repository.get_by_id(project.id).await?;
            match &fresh {
                Some(p) if !p.has_stack() => p,
                _ => return Ok(()),
            }
        };
        let ids = |table, names| self.repository.lookup_ids(table, names);
        let found = |ids: Vec<i32>| (!ids.is_empty()).then_some(ids);
        let framework_ids = found(ids("frameworks", &stack.frameworks).await?);
        let language_ids = found(ids("languages", &stack.languages).await?);
        let package_manager_ids = found(ids("package_managers", &stack.package_managers).await?);
        let fill = |current: &Option<String>, detected: &Option<String>| {
            current.is_none().then(|| detected.clone()).flatten()
        };
        let build = fill(&project.build_command, &stack.build);
        let start = fill(&project.start_command, &stack.start);
        let test = fill(&project.test_command, &stack.test);
        if framework_ids.is_none()
            && language_ids.is_none()
            && package_manager_ids.is_none()
            && build.is_none()
            && start.is_none()
            && test.is_none()
        {
            return Ok(());
        }
        self.repository
            .update(
                project.id,
                None,
                None,
                None,
                None,
                framework_ids,
                package_manager_ids,
                language_ids,
                build,
                start,
                test,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    /// Opens the project in an editor: the one mapped to its main framework, else
    /// the default editor, else the first editor found installed. Returns the
    /// editor's name for the confirmation toast.
    pub async fn open_in_ide(&self, id: i32) -> Result<String, String> {
        use crate::domains::ide::repositories::framework_ide_mapping_repository::FrameworkIdeMappingRepository;
        use crate::domains::ide::repositories::ide_repository::IdeRepository;
        use crate::domains::ide::services::ide_service::IdeService;
        use sea_orm::EntityTrait;

        let project = self
            .repository
            .get_by_id(id)
            .await?
            .ok_or_else(|| "Project not found".to_string())?;
        move_lock::ensure_unlocked(Path::new(&project.path))?;
        if !Path::new(&project.path).is_dir() {
            return Err(format!("Folder not found: {}", project.path));
        }
        let conn = self.db.get_connection();
        let mut editor: Option<(String, String)> = None;
        // The stored stack (it may be hand-picked), main framework first.
        let mut frameworks = Vec::new();
        for id in &project.framework_ids {
            if let Some(f) = crate::entities::framework::Entity::find_by_id(*id)
                .one(conn)
                .await
                .map_err(|e| e.to_string())?
            {
                frameworks.push(f.name);
            }
        }
        for framework in frameworks {
            let Some(mapping) = FrameworkIdeMappingRepository::get_by_framework(conn, &framework)
                .await
                .map_err(|e| e.to_string())?
            else {
                continue;
            };
            if let Some(ide) = IdeRepository::get_by_id(conn, mapping.ide_id)
                .await
                .map_err(|e| e.to_string())?
            {
                editor = Some((ide.name, ide.executable));
                break;
            }
        }
        if editor.is_none() {
            editor = IdeRepository::get_default(conn)
                .await
                .map_err(|e| e.to_string())?
                .map(|ide| (ide.name, ide.executable));
        }
        let (name, executable) = match editor {
            Some(e) => e,
            None => {
                let exe = tokio::task::spawn_blocking(|| IdeService::new().detect_installed_ides())
                    .await
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .next()
                    .ok_or_else(|| "No editor found. Add one in Settings.".to_string())?;
                let name = Path::new(&exe)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("editor")
                    .to_string();
                (name, exe)
            }
        };
        editor_command(&executable)
            .arg(&project.path)
            .no_window()
            .spawn()
            .map_err(|e| format!("Could not start {name}: {e}"))?;
        self.open_project(id).await?;
        Ok(name)
    }

    fn format_project_name(&self, name: &str) -> String {
        // Convert kebab-case, snake_case, or camelCase to Title Case
        let formatted = name
            .replace('-', " ")
            .replace('_', " ")
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0
                    || (i > 0
                        && name
                            .chars()
                            .nth(i - 1)
                            .map_or(false, |prev| prev == ' ' || prev == '-' || prev == '_'))
                {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect::<String>()
                }
            })
            .collect::<String>();

        // Clean up multiple spaces
        Regex::new(r"\s+")
            .unwrap()
            .replace_all(&formatted, " ")
            .to_string()
    }


    async fn detect_output_directory(
        &self,
        path: &str,
        framework: &Option<String>,
    ) -> Option<String> {
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
                "Vite" | "Svelte" | "Vue" => return Some("dist".to_string()),
                "Next.js" => return Some("out".to_string()),
                "Angular" => return Some("dist".to_string()),
                _ => {}
            }
        }

        None
    }

    async fn detect_ports(
        &self,
        path: &str,
        framework: &Option<String>,
    ) -> (Option<i32>, Option<i32>) {
        use regex::Regex;
        use std::fs;
        use std::path::Path;

        let path_obj = Path::new(path);
        let mut dev_port = None;
        let prod_port = None;

        // Check for port configurations in common files
        let config_files = [
            "package.json",
            "vite.config.js",
            "next.config.js",
            "nuxt.config.js",
            "svelte.config.js",
        ];

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
                    "Vue" | "Nuxt" => dev_port = Some(3000),
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
        let framework_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        let most_used_framework = framework_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(framework, _)| framework.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Get recent projects (last 5 opened)
        let mut recent_projects: Vec<ProjectResponse> = projects
            .iter()
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

        fn scan_dir(
            dir: &Path,
            size: &mut i64,
            file_count: &mut i32,
        ) -> Result<(), std::io::Error> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        // Skip common directories that shouldn't be counted
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if matches!(name, "node_modules" | ".git" | "target" | "dist" | "build")
                            {
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
            .no_window()
            .args(&["branch", "--show-current"])
            .current_dir(project_path)
            .output();

        let branch = if let Ok(output) = branch_output {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Get last commit
        let commit_output = Command::new("git")
            .no_window()
            .args(&["log", "-1", "--format=%H"])
            .current_dir(project_path)
            .output();

        let commit = if let Ok(output) = commit_output {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Check for uncommitted changes
        let status_output = Command::new("git")
            .no_window()
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
    pub recent_projects: Vec<ProjectResponse>,
}

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub has_uncommitted_changes: bool,
    pub last_commit: Option<chrono::DateTime<Utc>>,
}

/// Rejects unknown stages rather than storing them: the UI filters and badges
/// on these exact values, and a typo would make a project silently invisible.
fn validate_stage(stage: Option<String>) -> Result<Option<String>, String> {
    match stage.as_deref() {
        None => Ok(None),
        Some(s) if matches!(s, "idea" | "in_progress" | "stable" | "released") => {
            Ok(Some(s.to_string()))
        }
        Some(other) => Err(format!(
            "unknown stage `{other}` (expected idea, in_progress, stable or released)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_stages_and_none() {
        for s in ["idea", "in_progress", "stable", "released"] {
            assert_eq!(validate_stage(Some(s.into())).unwrap(), Some(s.into()));
        }
        assert_eq!(validate_stage(None).unwrap(), None);
    }

    #[test]
    fn rejects_unknown_stages() {
        assert!(validate_stage(Some("demo".into())).is_err());
        assert!(validate_stage(Some("shipped".into())).is_err());
        assert!(validate_stage(Some("garbage".into())).is_err());
    }
}
