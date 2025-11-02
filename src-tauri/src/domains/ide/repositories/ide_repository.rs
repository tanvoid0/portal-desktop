use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use crate::entities::ide::{Entity, Model, ActiveModel};
use crate::entities::ide;

pub struct IdeRepository;

impl IdeRepository {
    /// Get all IDEs
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find().all(db).await
    }

    /// Get IDE by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get default IDE
    pub async fn get_default(db: &DatabaseConnection) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find()
            .filter(ide::Column::IsDefault.eq(true))
            .one(db)
            .await
    }

    /// Create a new IDE
    pub async fn create(
        db: &DatabaseConnection,
        name: String,
        executable: String,
        is_default: bool,
    ) -> Result<Model, sea_orm::DbErr> {
        // If setting as default, unset all other defaults first
        if is_default {
            let all_ides = Entity::find().all(db).await?;
            for ide_model in all_ides {
                if ide_model.is_default {
                    let mut active_model: ActiveModel = ide_model.into();
                    active_model.is_default = Set(false);
                    active_model.updated_at = Set(Some(chrono::Utc::now().into()));
                    active_model.update(db).await?;
                }
            }
        }

        let active_model = ActiveModel {
            name: Set(name),
            executable: Set(executable),
            is_default: Set(is_default),
            created_at: Set(Some(chrono::Utc::now().into())),
            updated_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        active_model.insert(db).await
    }

    /// Update an IDE
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        name: Option<String>,
        executable: Option<String>,
        is_default: Option<bool>,
    ) -> Result<Model, sea_orm::DbErr> {
        let mut active_model: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(format!("IDE with id {} not found", id)))?
            .into();

        // If setting as default, unset all other defaults first
        if let Some(true) = is_default {
            let all_ides = Entity::find().all(db).await?;
            for ide_model in all_ides {
                if ide_model.is_default && ide_model.id != id {
                    let mut other_active_model: ActiveModel = ide_model.into();
                    other_active_model.is_default = Set(false);
                    other_active_model.updated_at = Set(Some(chrono::Utc::now().into()));
                    other_active_model.update(db).await?;
                }
            }
        }

        if let Some(name) = name {
            active_model.name = Set(name);
        }
        if let Some(executable) = executable {
            active_model.executable = Set(executable);
        }
        if let Some(is_default) = is_default {
            active_model.is_default = Set(is_default);
        }
        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(db).await
    }

    /// Delete an IDE
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}

