use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalProcess {
    pub id: String,
    pub tab_id: String,
    pub command: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub status: String,
    pub pid: Option<u32>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub process_id: String,
    pub content: String,
    pub output_type: String, // "stdout", "stderr", "exit"
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommand {
    pub id: String,
    pub process_id: String,
    pub command: String,
    pub timestamp: String,
    pub status: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProcessRequest {
    pub tab_id: String,
    pub shell: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub working_directory: Option<String>,
    pub environment: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    pub theme: String,
    pub font_size: u32,
    pub font_family: String,
    pub cursor_style: String,
    pub scrollback_lines: u32,
    pub bell_sound: bool,
    pub auto_close: bool,
    pub confirm_close: bool,
    pub default_shell: String,
    pub working_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTab {
    pub id: String,
    pub name: String,
    pub project_id: Option<String>,
    pub working_directory: String,
    pub status: String,
    pub is_active: bool,
    pub process_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalContext {
    pub tab_id: String,
    pub working_directory: String,
    pub environment: HashMap<String, String>,
    pub shell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInterceptor {
    pub pattern: String,
    pub handler_type: String, // "block", "modify", "monitor"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputParser {
    pub pattern: String,
    pub parser_type: String, // "highlight", "extract", "transform"
}
