use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domains::ai::entities::ConversationModel;
use crate::domains::ai::entities::ConversationMessageModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub provider: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub sequence: i32,
}

impl From<ConversationModel> for Conversation {
    fn from(model: ConversationModel) -> Self {
        Self {
            id: model.id,
            title: model.title,
            provider: model.provider,
            created_at: model.created_at,
            updated_at: model.updated_at,
            message_count: None,
        }
    }
}

impl From<ConversationMessageModel> for ConversationMessage {
    fn from(model: ConversationMessageModel) -> Self {
        Self {
            id: model.id,
            conversation_id: model.conversation_id,
            role: model.role,
            content: model.content,
            timestamp: model.timestamp,
            sequence: model.sequence,
        }
    }
}

impl Conversation {
    pub fn new(title: String, provider: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            provider,
            created_at: now.clone(),
            updated_at: now,
            message_count: None,
        }
    }
}

impl ConversationMessage {
    pub fn new(
        conversation_id: String,
        role: String,
        content: String,
        sequence: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            role,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            sequence,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationWithMessages {
    pub conversation: Conversation,
    pub messages: Vec<ConversationMessage>,
}


