use std::path::PathBuf;
use std::process::Command;

pub struct IdeService;

impl IdeService {
    pub fn new() -> Self {
        Self
    }

    /// Detect installed IDEs on the system
    pub fn detect_installed_ides(&self) -> Vec<String> {
        let mut detected: Vec<String> = Vec::new();
        
        // Common IDE executables to check
        let common_ides = if cfg!(windows) {
            vec![
                ("code", "Visual Studio Code"),
                ("idea64.exe", "IntelliJ IDEA"),
                ("idea.exe", "IntelliJ IDEA"),
                ("studio64.exe", "Android Studio"),
                ("studio.exe", "Android Studio"),
                ("rider64.exe", "JetBrains Rider"),
                ("rider.exe", "JetBrains Rider"),
                ("pycharm64.exe", "PyCharm"),
                ("pycharm.exe", "PyCharm"),
                ("webstorm64.exe", "WebStorm"),
                ("webstorm.exe", "WebStorm"),
                ("clion64.exe", "CLion"),
                ("clion.exe", "CLion"),
                ("goland64.exe", "GoLand"),
                ("goland.exe", "GoLand"),
                ("phpstorm64.exe", "PhpStorm"),
                ("phpstorm.exe", "PhpStorm"),
                ("rubymine64.exe", "RubyMine"),
                ("rubymine.exe", "RubyMine"),
                ("sublime_text.exe", "Sublime Text"),
                ("notepad++.exe", "Notepad++"),
                ("devenv.exe", "Visual Studio"),
            ]
        } else {
            vec![
                ("code", "Visual Studio Code"),
                ("idea", "IntelliJ IDEA"),
                ("studio", "Android Studio"),
                ("rider", "JetBrains Rider"),
                ("pycharm", "PyCharm"),
                ("webstorm", "WebStorm"),
                ("clion", "CLion"),
                ("goland", "GoLand"),
                ("phpstorm", "PhpStorm"),
                ("rubymine", "RubyMine"),
                ("sublime_text", "Sublime Text"),
                ("subl", "Sublime Text"),
                ("vim", "Vim"),
                ("nvim", "Neovim"),
                ("emacs", "Emacs"),
                ("atom", "Atom"),
            ]
        };

        // Check PATH for executables
        for (executable, _name) in &common_ides {
            if self.is_executable_in_path(executable) {
                if let Ok(full_path) = self.find_executable_path(executable) {
                    detected.push(full_path);
                }
            }
        }

        // Check common installation directories
        if cfg!(windows) {
            // Get user home directory for AppData paths
            if let Ok(home_dir) = std::env::var("USERPROFILE") {
                let home = PathBuf::from(&home_dir);
                
                // JetBrains Toolbox installation path (most common on Windows)
                // Toolbox creates symlinks in AppData\Local\JetBrains\Toolbox\bin
                let jetbrains_toolbox_bin = home.join(r"AppData\Local\JetBrains\Toolbox\bin");
                if jetbrains_toolbox_bin.exists() {
                    if let Ok(entries) = std::fs::read_dir(&jetbrains_toolbox_bin) {
                        for entry in entries.flatten() {
                            let exe_path = entry.path();
                            if let Some(ext) = exe_path.extension() {
                                if ext == "exe" {
                                    if let Ok(canonical) = std::fs::canonicalize(&exe_path) {
                                        if let Some(path_str) = canonical.to_str() {
                                            if !detected.contains(&path_str.to_string()) {
                                                detected.push(path_str.to_string());
                                            }
                                        }
                                    } else if let Some(path_str) = exe_path.to_str() {
                                        if !detected.contains(&path_str.to_string()) {
                                            detected.push(path_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // JetBrains Toolbox apps directory - check actual IDE installations
                let toolbox_apps = home.join(r"AppData\Local\JetBrains\Toolbox\apps");
                if toolbox_apps.exists() {
                    if let Ok(entries) = std::fs::read_dir(&toolbox_apps) {
                        for entry in entries.flatten() {
                            let ide_dir = entry.path();
                            if ide_dir.is_dir() {
                                // Each IDE has a "ch-0" or "ch-1" directory for versions
                                if let Ok(version_dirs) = std::fs::read_dir(&ide_dir) {
                                    for version_entry in version_dirs.flatten() {
                                        let version_dir = version_entry.path();
                                        if version_dir.is_dir() {
                                            // Look for bin directory with executables
                                            let bin_dir = version_dir.join("bin");
                                            if bin_dir.exists() {
                                                let launchers = vec!["idea64.exe", "idea.exe", "pycharm64.exe", "pycharm.exe",
                                                                    "webstorm64.exe", "webstorm.exe", "clion64.exe", "clion.exe",
                                                                    "goland64.exe", "goland.exe", "phpstorm64.exe", "phpstorm.exe",
                                                                    "rider64.exe", "rider.exe", "rubymine64.exe", "rubymine.exe",
                                                                    "studio64.exe", "studio.exe"];
                                                for launcher in launchers {
                                                    let exe_path = bin_dir.join(launcher);
                                                    if exe_path.exists() {
                                                        if let Ok(canonical) = std::fs::canonicalize(&exe_path) {
                                                            if let Some(path_str) = canonical.to_str() {
                                                                if !detected.contains(&path_str.to_string()) {
                                                                    detected.push(path_str.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Check for individual JetBrains IDE installations (non-Toolbox)
                let jetbrains_apps = home.join(r"AppData\Local\JetBrains");
                if jetbrains_apps.exists() {
                    if let Ok(entries) = std::fs::read_dir(&jetbrains_apps) {
                        for entry in entries.flatten() {
                            let app_dir = entry.path();
                            // Skip Toolbox directory, we already checked it
                            if app_dir.file_name().unwrap().to_string_lossy() == "Toolbox" {
                                continue;
                            }
                            // Look for bin directories with .exe files
                            let bin_dir = app_dir.join("bin");
                            if bin_dir.exists() {
                                if let Ok(bin_entries) = std::fs::read_dir(&bin_dir) {
                                    for bin_entry in bin_entries.flatten() {
                                        let exe_path = bin_entry.path();
                                        if let Some(ext) = exe_path.extension() {
                                            if ext == "exe" {
                                                if let Ok(canonical) = std::fs::canonicalize(&exe_path) {
                                                    if let Some(path_str) = canonical.to_str() {
                                                        if !detected.contains(&path_str.to_string()) {
                                                            detected.push(path_str.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // VS Code paths
                let vs_code_paths = vec![
                    home.join(r"AppData\Local\Programs\Microsoft VS Code\Code.exe"),
                    PathBuf::from(r"C:\Program Files\Microsoft VS Code\Code.exe"),
                    PathBuf::from(r"C:\Program Files (x86)\Microsoft VS Code\Code.exe"),
                ];
                for path in vs_code_paths {
                    if path.exists() {
                        if let Ok(canonical) = std::fs::canonicalize(&path) {
                            if let Some(path_str) = canonical.to_str() {
                                if !detected.contains(&path_str.to_string()) {
                                    detected.push(path_str.to_string());
                                }
                            }
                        }
                    }
                }
                
                // Check Program Files for JetBrains IDEs
                let program_files_paths = vec![
                    PathBuf::from(r"C:\Program Files\JetBrains"),
                    PathBuf::from(r"C:\Program Files (x86)\JetBrains"),
                ];
                for program_files in program_files_paths {
                    if program_files.exists() {
                        if let Ok(entries) = std::fs::read_dir(&program_files) {
                            for entry in entries.flatten() {
                                let ide_dir = entry.path();
                                let bin_dir = ide_dir.join("bin");
                                if bin_dir.exists() {
                                    // Look for launcher exes
                                    let launchers = vec!["idea64.exe", "idea.exe", "pycharm64.exe", "pycharm.exe",
                                                        "webstorm64.exe", "webstorm.exe", "clion64.exe", "clion.exe",
                                                        "goland64.exe", "goland.exe", "phpstorm64.exe", "phpstorm.exe",
                                                        "rider64.exe", "rider.exe", "rubymine64.exe", "rubymine.exe"];
                                    for launcher in launchers {
                                        let exe_path = bin_dir.join(launcher);
                                        if exe_path.exists() {
                                            if let Ok(canonical) = std::fs::canonicalize(&exe_path) {
                                                if let Some(path_str) = canonical.to_str() {
                                                    if !detected.contains(&path_str.to_string()) {
                                                        detected.push(path_str.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let common_paths = vec![
                PathBuf::from("/usr/bin/code"),
                PathBuf::from("/usr/local/bin/code"),
                PathBuf::from("/snap/bin/code"),
                PathBuf::from("/opt/idea/bin/idea.sh"),
                PathBuf::from("/opt/pycharm/bin/pycharm.sh"),
                PathBuf::from("/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code"),
                PathBuf::from("/Applications/IntelliJ IDEA.app/Contents/MacOS/idea"),
            ];
            for path in common_paths {
                if path.exists() {
                    if let Ok(canonical) = std::fs::canonicalize(&path) {
                        if let Some(path_str) = canonical.to_str() {
                            if !detected.contains(&path_str.to_string()) {
                                detected.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        detected
    }

    fn is_executable_in_path(&self, executable: &str) -> bool {
        if cfg!(windows) {
            // On Windows, try "where" command
            if let Ok(output) = Command::new("where").arg(executable).output() {
                return output.status.success();
            }
        } else {
            // On Unix, try "which" command
            if let Ok(output) = Command::new("which").arg(executable).output() {
                return output.status.success();
            }
        }
        false
    }

    fn find_executable_path(&self, executable: &str) -> Result<String, String> {
        if cfg!(windows) {
            let output = Command::new("where")
                .arg(executable)
                .output()
                .map_err(|e| e.to_string())?;
            
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .ok_or("No path found")?
                    .trim()
                    .to_string();
                Ok(path)
            } else {
                Err("Executable not found".to_string())
            }
        } else {
            let output = Command::new("which")
                .arg(executable)
                .output()
                .map_err(|e| e.to_string())?;
            
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(path)
            } else {
                Err("Executable not found".to_string())
            }
        }
    }
}

