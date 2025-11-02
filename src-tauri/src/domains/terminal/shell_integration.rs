use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a command block with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBlock {
    pub id: String,
    pub command: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<u64>, // milliseconds
    pub exit_code: Option<i32>,
    pub working_directory: String,
    pub output: String,
    pub status: CommandStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Parser for detecting OSC markers in terminal output
pub struct ShellIntegrationParser {
    current_block: Option<CommandBlock>,
    blocks: Vec<CommandBlock>,
    buffer: String,
    command_counter: u64,
}

impl ShellIntegrationParser {
    pub fn new() -> Self {
        Self {
            current_block: None,
            blocks: Vec::new(),
            buffer: String::new(),
            command_counter: 0,
        }
    }

    /// Process incoming terminal output and detect OSC markers
    pub fn process_output(&mut self, content: &str) -> Vec<ShellIntegrationEvent> {
        let mut events = Vec::new();
        self.buffer.push_str(content);

        // Look for OSC markers in the buffer
        while let Some(marker_pos) = self.find_next_marker() {
            // Process content before the marker
            let before_marker = self.buffer[..marker_pos].to_string();
            if !before_marker.is_empty() {
                events.extend(self.process_content(&before_marker));
            }

            // Process the marker
            if let Some(marker_end) = self.find_marker_end(marker_pos) {
                let marker = self.buffer[marker_pos..marker_end].to_string();
                events.extend(self.process_marker(&marker));
                
                // Remove processed content
                self.buffer = self.buffer[marker_end..].to_string();
            } else {
                // Incomplete marker, wait for more data
                break;
            }
        }

        events
    }

    /// Process any remaining content in the buffer
    pub fn flush(&mut self) -> Vec<ShellIntegrationEvent> {
        let mut events = Vec::new();
        if !self.buffer.is_empty() {
            let buffer_content = self.buffer.clone();
            events.extend(self.process_content(&buffer_content));
            self.buffer.clear();
        }
        events
    }

    fn find_next_marker(&self) -> Option<usize> {
        // Look for OSC 133 markers (command start/end)
        self.buffer.find("\x1b]133;")
    }

    fn find_marker_end(&self, start: usize) -> Option<usize> {
        // OSC sequences end with \x1b\\
        self.buffer[start..].find("\x1b\\").map(|pos| start + pos + 2)
    }

    fn process_marker(&mut self, marker: &str) -> Vec<ShellIntegrationEvent> {
        let mut events = Vec::new();

        if marker.contains("133;A") {
            // Command start marker
            self.command_counter += 1;
            let block = CommandBlock {
                id: format!("cmd_{}", self.command_counter),
                command: String::new(), // Will be filled when we detect the actual command
                start_time: Utc::now(),
                end_time: None,
                duration: None,
                exit_code: None,
                working_directory: String::new(),
                output: String::new(),
                status: CommandStatus::Running,
            };
            self.current_block = Some(block.clone());
            events.push(ShellIntegrationEvent::CommandStarted(block));
        } else if marker.contains("133;B") {
            // Command end marker
            if let Some(mut block) = self.current_block.take() {
                block.end_time = Some(Utc::now());
                block.duration = block.end_time
                    .and_then(|end| Some((end - block.start_time).num_milliseconds() as u64));
                block.status = CommandStatus::Completed;
                
                self.blocks.push(block.clone());
                events.push(ShellIntegrationEvent::CommandCompleted(block));
            }
        } else if marker.contains("133;C") {
            // Prompt marker - could be used for command detection
            events.push(ShellIntegrationEvent::PromptDetected);
        }

        events
    }

    fn process_content(&mut self, content: &str) -> Vec<ShellIntegrationEvent> {
        let mut events = Vec::new();

        // Add content to current block if we have one
        if let Some(ref mut block) = self.current_block {
            block.output.push_str(content);
        }

        // Check for command detection patterns
        if let Some(command) = self.detect_command(content) {
            if let Some(ref mut block) = self.current_block {
                block.command = command.clone();
            }
            events.push(ShellIntegrationEvent::CommandDetected(command));
        }

        events
    }

    fn detect_command(&self, content: &str) -> Option<String> {
        // Simple command detection - look for common shell prompt patterns
        // This is a basic implementation; more sophisticated detection could be added
        let lines: Vec<&str> = content.lines().collect();
        for line in lines {
            // Look for lines that might be commands (not output)
            if line.trim().starts_with('$') || line.trim().starts_with('#') {
                return Some(line.trim().to_string());
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellIntegrationEvent {
    CommandStarted(CommandBlock),
    CommandCompleted(CommandBlock),
    CommandDetected(String),
    PromptDetected,
    OutputContent(String),
}