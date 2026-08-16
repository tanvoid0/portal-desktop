//! The shapes an AI task-generation request carries.
//!
//! These were declared in `tasks/commands.rs` and read by `ai_task_generator`,
//! which made a service depend on its own Tauri adapter. Phase 1 of the crate
//! split moved them down here, where both sides can see them.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub parent_task: Option<ParentTaskInfo>,
    pub existing_children: Option<Vec<ChildTaskInfo>>,
    pub existing_siblings: Option<Vec<SiblingTaskInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentTaskInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub type_: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildTaskInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiblingTaskInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}
