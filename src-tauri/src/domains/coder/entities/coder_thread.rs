use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "coder_threads")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: String,
    pub workspace_root: String,
    pub model: Option<String>,
    /// Agent-platform `coder_chat_threads.id` when using `/api/v1/coder/chat/stream`.
    pub platform_thread_id: Option<i64>,
    /// Backend LLM provider id (e.g. `ollama`) for agent-platform routing.
    pub llm_provider: Option<String>,
    /// The full transcript (`Vec<ChatMessage>`) serialized as JSON.
    pub messages_json: String,
    pub thread_kind: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
