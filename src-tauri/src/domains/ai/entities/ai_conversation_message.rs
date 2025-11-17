use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_conversation_messages")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub timestamp: String,
    pub sequence: i32,
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

