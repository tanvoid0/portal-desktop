use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub provider: String,
    pub log_type: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub request_data: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub response_data: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub error_message: Option<String>,
    pub timestamp: String,
    pub conversation_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::domains::ai::entities::ai_conversation::Entity",
        from = "Column::ConversationId",
        to = "crate::domains::ai::entities::ai_conversation::Column::Id"
    )]
    Conversation,
}

impl Related<crate::domains::ai::entities::ai_conversation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

