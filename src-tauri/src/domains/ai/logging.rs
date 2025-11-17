use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domains::ai::providers::ProviderType;
use crate::domains::ai::entities::AILogModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AILog {
    pub id: String,
    pub provider: String,
    pub log_type: String,
    pub request_data: Option<String>,
    pub response_data: Option<String>,
    pub error_message: Option<String>,
    pub timestamp: String,
    pub conversation_id: Option<String>,
}

impl From<AILogModel> for AILog {
    fn from(model: AILogModel) -> Self {
        Self {
            id: model.id,
            provider: model.provider,
            log_type: model.log_type,
            request_data: model.request_data,
            response_data: model.response_data,
            error_message: model.error_message,
            timestamp: model.timestamp,
            conversation_id: model.conversation_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogFilters {
    pub provider: Option<String>,
    pub log_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search_query: Option<String>,
}

impl AILog {
    pub fn new_request(
        provider: ProviderType,
        request_data: String,
        conversation_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            provider: format!("{:?}", provider),
            log_type: "request".to_string(),
            request_data: Some(request_data),
            response_data: None,
            error_message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            conversation_id,
        }
    }

    pub fn new_response(
        provider: ProviderType,
        response_data: String,
        conversation_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            provider: format!("{:?}", provider),
            log_type: "response".to_string(),
            request_data: None,
            response_data: Some(response_data),
            error_message: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            conversation_id,
        }
    }

    pub fn new_error(
        provider: ProviderType,
        error_message: String,
        conversation_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            provider: format!("{:?}", provider),
            log_type: "error".to_string(),
            request_data: None,
            response_data: None,
            error_message: Some(error_message),
            timestamp: chrono::Utc::now().to_rfc3339(),
            conversation_id,
        }
    }
}


