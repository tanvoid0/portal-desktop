use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::{Utc, DateTime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    // General settings
    pub theme: String,
    pub language: String,
    pub timezone: String,
    pub date_format: String,
    pub time_format: String,
    
    // Window settings
    pub window_state: WindowState,
    pub startup_behavior: StartupBehavior,
    
    // Notifications
    pub notifications: NotificationSettings,
    
    // Privacy
    pub privacy: PrivacySettings,
    
    // Updates
    pub updates: UpdateSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub maximized: bool,
    pub remember_position: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartupBehavior {
    pub open_last_session: bool,
    pub restore_windows: bool,
    pub show_welcome_screen: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub desktop_notifications: bool,
    pub sound_enabled: bool,
    pub show_in_taskbar: bool,
    pub types: NotificationTypeSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationTypeSettings {
    pub success: bool,
    pub info: bool,
    pub warning: bool,
    pub error: bool,
    pub updates: bool,
    pub security: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivacySettings {
    pub analytics: bool,
    pub crash_reports: bool,
    pub telemetry: bool,
    pub usage_data: bool,
    pub marketing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSettings {
    pub auto_check: bool,
    pub auto_download: bool,
    pub auto_install: bool,
    pub check_interval: u32, // hours
    pub channel: String,
    pub notify_on_update: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorSettings {
    // Code editor
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub tab_size: u32,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
    pub show_minimap: bool,
    pub show_whitespace: bool,
    
    // Syntax highlighting
    pub syntax_highlighting: bool,
    pub bracket_matching: bool,
    pub auto_indent: bool,
    
    // Code completion
    pub auto_complete: bool,
    pub suggestions: bool,
    pub parameter_hints: bool,
    
    // Themes
    pub editor_theme: String,
    pub terminal_theme: String,
    
    // Keybindings
    pub keybindings: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalSettings {
    // Terminal appearance
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub cursor_style: String,
    pub cursor_blink: bool,
    
    // Terminal behavior
    pub scrollback: u32,
    pub bell_style: String,
    pub right_click_selects_word: bool,
    pub selection_mode: String,
    
    // Shell integration
    pub shell_integration: bool,
    pub command_history: bool,
    pub command_suggestions: bool,
    
    // Terminal themes
    pub theme: TerminalTheme,
    
    // Advanced
    pub encoding: String,
    pub locale: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalTheme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub selection: String,
    pub colors: TerminalColors,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TerminalColors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_magenta: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeSettings {
    // Color scheme
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub surface_color: String,
    pub text_color: String,
    
    // UI elements
    pub border_radius: f32,
    pub shadow_intensity: f32,
    pub animation_speed: String,
    
    // Custom themes
    pub custom_themes: Vec<CustomTheme>,
    pub active_theme: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomTheme {
    pub id: String,
    pub name: String,
    pub description: String,
    pub colors: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub id: String,
    pub app: AppSettings,
    pub editor: EditorSettings,
    pub terminal: TerminalSettings,
    pub theme: ThemeSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SettingsService {
    settings_path: PathBuf,
}

impl SettingsService {
    pub fn new() -> Self {
        let mut settings_path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        settings_path.push("portal-desktop");
        settings_path.push("settings.json");
        
        Self { settings_path }
    }

    /// Load settings from file
    pub fn load_settings(&self) -> Result<Settings, String> {
        if !self.settings_path.exists() {
            return Ok(self.get_default_settings());
        }

        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: Settings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))?;

        Ok(settings)
    }

    /// Save settings to file
    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&self.settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    /// Get default settings
    pub fn get_default_settings(&self) -> Settings {
        let now = Utc::now();
        
        Settings {
            id: "default".to_string(),
            app: AppSettings {
                theme: "system".to_string(),
                language: "en".to_string(),
                timezone: "UTC".to_string(),
                date_format: "%Y-%m-%d".to_string(),
                time_format: "24h".to_string(),
                window_state: WindowState {
                    width: 1200,
                    height: 800,
                    x: None,
                    y: None,
                    maximized: false,
                    remember_position: true,
                },
                startup_behavior: StartupBehavior {
                    open_last_session: true,
                    restore_windows: true,
                    show_welcome_screen: false,
                    minimize_to_tray: false,
                    start_minimized: false,
                },
                notifications: NotificationSettings {
                    enabled: true,
                    desktop_notifications: true,
                    sound_enabled: true,
                    show_in_taskbar: true,
                    types: NotificationTypeSettings {
                        success: true,
                        info: true,
                        warning: true,
                        error: true,
                        updates: true,
                        security: true,
                    },
                },
                privacy: PrivacySettings {
                    analytics: false,
                    crash_reports: true,
                    telemetry: false,
                    usage_data: false,
                    marketing: false,
                },
                updates: UpdateSettings {
                    auto_check: true,
                    auto_download: false,
                    auto_install: false,
                    check_interval: 24,
                    channel: "stable".to_string(),
                    notify_on_update: true,
                },
            },
            editor: EditorSettings {
                font_family: "Monaco, Consolas, 'Courier New', monospace".to_string(),
                font_size: 14,
                line_height: 1.5,
                tab_size: 4,
                insert_spaces: true,
                word_wrap: true,
                show_line_numbers: true,
                show_minimap: true,
                show_whitespace: false,
                syntax_highlighting: true,
                bracket_matching: true,
                auto_indent: true,
                auto_complete: true,
                suggestions: true,
                parameter_hints: true,
                editor_theme: "vs-dark".to_string(),
                terminal_theme: "dark".to_string(),
                keybindings: HashMap::new(),
            },
            terminal: TerminalSettings {
                font_family: "Monaco, Consolas, 'Courier New', monospace".to_string(),
                font_size: 14,
                line_height: 1.2,
                cursor_style: "block".to_string(),
                cursor_blink: true,
                scrollback: 1000,
                bell_style: "none".to_string(),
                right_click_selects_word: true,
                selection_mode: "normal".to_string(),
                shell_integration: true,
                command_history: true,
                command_suggestions: true,
                theme: TerminalTheme {
                    name: "default".to_string(),
                    background: "#1e1e1e".to_string(),
                    foreground: "#d4d4d4".to_string(),
                    cursor: "#ffffff".to_string(),
                    selection: "#264f78".to_string(),
                    colors: TerminalColors {
                        black: "#000000".to_string(),
                        red: "#cd3131".to_string(),
                        green: "#0dbc79".to_string(),
                        yellow: "#e5e510".to_string(),
                        blue: "#2472c8".to_string(),
                        magenta: "#bc3fbc".to_string(),
                        cyan: "#11a8cd".to_string(),
                        white: "#e5e5e5".to_string(),
                        bright_black: "#666666".to_string(),
                        bright_red: "#f14c4c".to_string(),
                        bright_green: "#23d18b".to_string(),
                        bright_yellow: "#f5f543".to_string(),
                        bright_blue: "#3b8eea".to_string(),
                        bright_magenta: "#d670d6".to_string(),
                        bright_cyan: "#29b8db".to_string(),
                        bright_white: "#e5e5e5".to_string(),
                    },
                },
                encoding: "utf-8".to_string(),
                locale: "en_US.UTF-8".to_string(),
            },
            theme: ThemeSettings {
                primary_color: "#3b82f6".to_string(),
                secondary_color: "#64748b".to_string(),
                accent_color: "#f59e0b".to_string(),
                background_color: "#ffffff".to_string(),
                surface_color: "#f8fafc".to_string(),
                text_color: "#1e293b".to_string(),
                border_radius: 6.0,
                shadow_intensity: 0.1,
                animation_speed: "normal".to_string(),
                custom_themes: Vec::new(),
                active_theme: "default".to_string(),
            },
            created_at: now,
            updated_at: now,
        }
    }

    /// Update settings
    pub fn update_settings(&self, mut settings: Settings, updates: SettingsUpdate) -> Result<Settings, String> {
        if let Some(app) = updates.app {
            settings.app = app;
        }
        if let Some(editor) = updates.editor {
            settings.editor = editor;
        }
        if let Some(terminal) = updates.terminal {
            settings.terminal = terminal;
        }
        if let Some(theme) = updates.theme {
            settings.theme = theme;
        }
        
        settings.updated_at = Utc::now();
        
        self.save_settings(&settings)?;
        Ok(settings)
    }

    /// Reset settings to defaults
    pub fn reset_settings(&self) -> Result<Settings, String> {
        let default_settings = self.get_default_settings();
        self.save_settings(&default_settings)?;
        Ok(default_settings)
    }

    /// Export settings
    pub fn export_settings(&self, settings: &Settings) -> Result<String, String> {
        serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to export settings: {}", e))
    }

    /// Import settings
    pub fn import_settings(&self, settings_json: &str) -> Result<Settings, String> {
        let settings: Settings = serde_json::from_str(settings_json)
            .map_err(|e| format!("Failed to import settings: {}", e))?;
        
        self.save_settings(&settings)?;
        Ok(settings)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub app: Option<AppSettings>,
    pub editor: Option<EditorSettings>,
    pub terminal: Option<TerminalSettings>,
    pub theme: Option<ThemeSettings>,
}
