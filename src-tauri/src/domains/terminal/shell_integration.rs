use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
        self.buffer[start..]
            .find("\x1b\\")
            .map(|pos| start + pos + 2)
    }

    fn process_marker(&mut self, marker: &str) -> Vec<ShellIntegrationEvent> {
        let mut events = Vec::new();

        if marker.contains("133;A") {
            // Command start marker
            self.command_counter += 1;

            let working_directory = Self::extract_marker_arg(marker, "133;A;").unwrap_or_default();

            let block = CommandBlock {
                id: format!("cmd_{}", self.command_counter),
                command: String::new(), // Will be filled when we detect the actual command
                start_time: Utc::now(),
                end_time: None,
                duration: None,
                exit_code: None,
                working_directory,
                output: String::new(),
                status: CommandStatus::Running,
            };
            self.current_block = Some(block.clone());
            events.push(ShellIntegrationEvent::CommandStarted(block));
        } else if marker.contains("133;B") {
            // Orphan B (e.g. the shell's first prompt after startup, before any
            // command): no block to complete, but it proves the shell hooks are
            // live — surface it so the frontend can stop creating manual blocks.
            if self.current_block.is_none() {
                events.push(ShellIntegrationEvent::PromptDetected);
                return events;
            }
            // Command end marker
            if let Some(mut block) = self.current_block.take() {
                block.end_time = Some(Utc::now());
                block.duration = block
                    .end_time
                    .and_then(|end| Some((end - block.start_time).num_milliseconds() as u64));

                let exit_code = Self::extract_marker_arg(marker, "133;B;")
                    .and_then(|s| s.trim().parse::<i32>().ok());
                block.exit_code = exit_code;

                block.status = CommandStatus::Completed;

                self.blocks.push(block.clone());
                events.push(ShellIntegrationEvent::CommandCompleted(block));
            }
        } else if marker.contains("133;C") {
            // Command text marker: `133;C;<command>` (emitted by our shell
            // hooks right after A). Bare `133;C` is a plain prompt marker.
            if let Some(command) = Self::extract_marker_arg(marker, "133;C;") {
                if let Some(ref mut block) = self.current_block {
                    block.command = command.clone();
                }
                events.push(ShellIntegrationEvent::CommandDetected(command));
            } else {
                events.push(ShellIntegrationEvent::PromptDetected);
            }
        }

        events
    }

    /// Extracts the argument payload for an OSC marker like:
    /// `ESC ]133;A;<payload> ESC \\`
    fn extract_marker_arg(marker: &str, prefix: &str) -> Option<String> {
        let start = marker.find(prefix)?;
        let rest = &marker[start + prefix.len()..];
        // Marker terminates with ESC \\ (ESC + backslash). Stop at ESC if present.
        let before_esc = rest.split('\u{001b}').next().unwrap_or(rest);
        let cleaned = before_esc.trim_end_matches('\\').to_string();
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    fn process_content(&mut self, content: &str) -> Vec<ShellIntegrationEvent> {
        // Command text comes from the explicit `133;C` marker; content between
        // markers is just output for the current block.
        if let Some(ref mut block) = self.current_block {
            block.output.push_str(content);
        }
        Vec::new()
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

/// v2 wrapper so the frontend can scope command blocks per PTY process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellIntegrationEventV2 {
    pub process_id: String,
    pub event: ShellIntegrationEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_command_cycle_captures_command_output_and_exit() {
        let mut p = ShellIntegrationParser::new();
        let events = p.process_output(
            "\x1b]133;A;/home/tan\x1b\\\x1b]133;C;ls -la\x1b\\file1\nfile2\n\x1b]133;B;0\x1b\\",
        );

        let started = events.iter().any(|e| matches!(e, ShellIntegrationEvent::CommandStarted(b) if b.working_directory == "/home/tan"));
        let detected = events.iter().any(|e| matches!(e, ShellIntegrationEvent::CommandDetected(c) if c == "ls -la"));
        let completed = events.iter().find_map(|e| match e {
            ShellIntegrationEvent::CommandCompleted(b) => Some(b.clone()),
            _ => None,
        });

        assert!(started, "expected CommandStarted with cwd");
        assert!(detected, "expected CommandDetected with command text");
        let block = completed.expect("expected CommandCompleted");
        assert_eq!(block.command, "ls -la");
        assert_eq!(block.exit_code, Some(0));
        assert!(block.output.contains("file1"));
    }

    #[test]
    fn orphan_end_marker_is_ignored() {
        let mut p = ShellIntegrationParser::new();
        // First prompt after shell start emits B with no preceding A.
        let events = p.process_output("\x1b]133;B;0\x1b\\");
        assert!(!events
            .iter()
            .any(|e| matches!(e, ShellIntegrationEvent::CommandCompleted(_))));
    }
}
