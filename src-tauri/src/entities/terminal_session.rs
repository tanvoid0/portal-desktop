use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "terminal_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tab_id: String,

    pub working_directory: String,

    /// Stored as JSON string for easy interop with frontend.
    #[sea_orm(column_type = "Text")]
    pub environment_json: String,

    /// Stored as JSON string of `string[]` for scrollback persistence.
    #[sea_orm(column_type = "Text")]
    pub scrollback_buffer_json: String,

    pub cursor_x: i32,
    pub cursor_y: i32,

    pub terminal_cols: i32,
    pub terminal_rows: i32,

    pub last_activity: String,

    pub process_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// JSON decoding helpers live in `commands.rs` where the DB rows are mapped.

