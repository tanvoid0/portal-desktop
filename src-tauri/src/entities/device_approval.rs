use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "device_approvals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub device_id: String, // Unique identifier for the device
    pub device_name: String, // User-friendly device name
    pub device_info: String, // JSON object with device details (user agent, IP, etc.)
    pub passcode: String, // 6-digit passcode (hashed)
    pub passcode_expires_at: DateTimeWithTimeZone, // Passcode expiration (5 minutes)
    pub approved: bool, // Whether device is approved
    pub approval_type: String, // "temporary" or "long_term"
    pub approved_at: Option<DateTimeWithTimeZone>, // When device was approved
    pub expires_at: Option<DateTimeWithTimeZone>, // Approval expiration (1 month max)
    pub access_token: Option<String>, // JWT or session token for authenticated requests
    pub token_expires_at: Option<DateTimeWithTimeZone>, // Token expiration
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub last_used_at: Option<DateTimeWithTimeZone>, // Last time device accessed the API
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

