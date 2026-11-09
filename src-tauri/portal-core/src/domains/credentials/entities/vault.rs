/**
 * Vault Entity
 *
 * Single-row table (`id = 1`) holding the credential master key (MK),
 * wrapped two ways: under a key derived from the master password (PBKDF2)
 * and under a key derived from the recovery key (HKDF). MK itself never
 * changes — see `services::vault_service`.
 */
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "vault")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub pw_salt: String,
    pub pw_iterations: i32,
    pub pw_wrapped_mk: String,
    pub recovery_salt: String,
    pub recovery_wrapped_mk: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
