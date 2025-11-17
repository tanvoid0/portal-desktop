use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QueryOrder};
use crate::entities::learning_event::{Entity, Model, ActiveModel};
use crate::entities::learning_event;

pub struct LearningEventRepository;

impl LearningEventRepository {
    /// Get all learning events
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find()
            .order_by_desc(learning_event::Column::CreatedAt)
            .all(db)
            .await
    }

    /// Get event by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get events by type
    pub async fn get_by_type(
        db: &DatabaseConnection,
        event_type: &str,
    ) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find()
            .filter(learning_event::Column::EventType.eq(event_type))
            .order_by_desc(learning_event::Column::CreatedAt)
            .all(db)
            .await
    }

    /// Get recent events (limit)
    pub async fn get_recent(
        db: &DatabaseConnection,
        limit: u64,
    ) -> Result<Vec<Model>, sea_orm::DbErr> {
        let all = Entity::find()
            .order_by_desc(learning_event::Column::CreatedAt)
            .all(db)
            .await?;
        
        Ok(all.into_iter().take(limit as usize).collect())
    }

    /// Create a new learning event
    pub async fn create(
        db: &DatabaseConnection,
        event_type: String,
        event_data: String,
        outcome: Option<String>,
        context: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        let active_model = ActiveModel {
            event_type: Set(event_type),
            event_data: Set(event_data),
            outcome: Set(outcome),
            context: Set(context),
            created_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        active_model.insert(db).await
    }

    /// Update event outcome
    pub async fn update_outcome(
        db: &DatabaseConnection,
        id: i32,
        outcome: String,
    ) -> Result<Model, sea_orm::DbErr> {
        let mut active_model: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(format!("Event with id {} not found", id)))?
            .into();

        active_model.outcome = Set(Some(outcome));

        active_model.update(db).await
    }

    /// Delete an event
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    /// Delete old events (cleanup)
    pub async fn delete_older_than(
        db: &DatabaseConnection,
        days: i64,
    ) -> Result<u64, sea_orm::DbErr> {
        use sea_orm::prelude::*;
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let cutoff_value: sea_orm::Value = cutoff.into();
        
        let result = Entity::delete_many()
            .filter(learning_event::Column::CreatedAt.lt(cutoff_value))
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }
}

