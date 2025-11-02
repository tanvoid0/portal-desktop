/**
 * Credentials Domain Entities
 */

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub mod vault;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "credentials")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
    pub credential_type: String,
    pub status: String,
    pub description: Option<String>,
    pub tags: String, // JSON array
    pub encrypted_value: String,
    pub encrypted_fields: String, // JSON object
    pub metadata: String, // JSON object
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub last_used: Option<DateTime>,
    pub expires_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
