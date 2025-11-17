use sea_orm_migration::prelude::*;

/// Migration: Create credentials table
///
/// This migration creates the credentials table with the following structure:
/// - id: Primary key (String/UUID)
/// - name: Credential display name
/// - credential_type: Type of credential (api_key, password, token, etc.)
/// - status: Credential status (active, inactive, expired, etc.)
/// - description: Credential description (optional)
/// - tags: JSON array of tags
/// - encrypted_value: Encrypted credential value
/// - encrypted_fields: JSON object of encrypted additional fields
/// - metadata: JSON object of metadata
/// - created_at: Timestamp when record was created
/// - updated_at: Timestamp when record was last updated
/// - last_used: Timestamp when credential was last used (optional)
/// - expires_at: Timestamp when credential expires (optional)
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Credentials::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Credentials::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Credentials::Name).string().not_null())
                    .col(ColumnDef::new(Credentials::CredentialType).string().not_null())
                    .col(ColumnDef::new(Credentials::Status).string().not_null().default("active"))
                    .col(ColumnDef::new(Credentials::Description).string().null())
                    .col(ColumnDef::new(Credentials::Tags).text().not_null().default("[]"))
                    .col(ColumnDef::new(Credentials::EncryptedValue).text().not_null())
                    .col(ColumnDef::new(Credentials::EncryptedFields).text().not_null().default("{}"))
                    .col(ColumnDef::new(Credentials::Metadata).text().not_null().default("{}"))
                    .col(ColumnDef::new(Credentials::CreatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Credentials::UpdatedAt).timestamp_with_time_zone().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Credentials::LastUsed).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Credentials::ExpiresAt).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Credentials::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Credentials {
    Table,
    Id,
    Name,
    CredentialType,
    Status,
    Description,
    Tags,
    EncryptedValue,
    EncryptedFields,
    Metadata,
    CreatedAt,
    UpdatedAt,
    LastUsed,
    ExpiresAt,
}

