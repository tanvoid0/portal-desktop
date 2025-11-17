use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QueryOrder};
use crate::entities::learned_pattern::{Entity, Model, ActiveModel};
use crate::entities::learned_pattern;

pub struct LearnedPatternRepository;

impl LearnedPatternRepository {
    /// Get all learned patterns
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find()
            .order_by_desc(learned_pattern::Column::Frequency)
            .all(db)
            .await
    }

    /// Get pattern by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get patterns by type
    pub async fn get_by_type(
        db: &DatabaseConnection,
        pattern_type: &str,
    ) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find()
            .filter(learned_pattern::Column::PatternType.eq(pattern_type))
            .order_by_desc(learned_pattern::Column::Frequency)
            .all(db)
            .await
    }

    /// Get patterns by type and context
    pub async fn get_by_type_and_context(
        db: &DatabaseConnection,
        pattern_type: &str,
        context: Option<&str>,
    ) -> Result<Vec<Model>, sea_orm::DbErr> {
        let mut query = Entity::find()
            .filter(learned_pattern::Column::PatternType.eq(pattern_type));

        if let Some(ctx) = context {
            query = query.filter(learned_pattern::Column::Context.eq(ctx));
        }

        query
            .order_by_desc(learned_pattern::Column::Frequency)
            .all(db)
            .await
    }

    /// Create a new learned pattern
    pub async fn create(
        db: &DatabaseConnection,
        pattern_type: String,
        pattern_data: String,
        context: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        let active_model = ActiveModel {
            pattern_type: Set(pattern_type),
            pattern_data: Set(pattern_data),
            context: Set(context),
            frequency: Set(1),
            last_used: Set(Some(chrono::Utc::now().into())),
            success_rate: Set(1.0),
            created_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        active_model.insert(db).await
    }

    /// Update pattern frequency and last used
    pub async fn increment_frequency(
        db: &DatabaseConnection,
        id: i32,
        success: bool,
    ) -> Result<Model, sea_orm::DbErr> {
        let model = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(format!("Pattern with id {} not found", id)))?;

        let current_frequency = model.frequency;
        let current_success_rate = model.success_rate;
        
        let mut active_model: ActiveModel = model.into();

        // Update frequency
        active_model.frequency = Set(current_frequency + 1);
        active_model.last_used = Set(Some(chrono::Utc::now().into()));

        // Update success rate (weighted average)
        let total_successes = (current_success_rate * current_frequency as f64).round() as i32 + if success { 1 } else { 0 };
        let new_success_rate = total_successes as f64 / (current_frequency + 1) as f64;
        active_model.success_rate = Set(new_success_rate);

        active_model.update(db).await
    }

    /// Delete a pattern
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    /// Find or create pattern by matching pattern_data
    pub async fn find_or_create(
        db: &DatabaseConnection,
        pattern_type: String,
        pattern_data: String,
        context: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        // Try to find existing pattern
        let mut query = Entity::find()
            .filter(learned_pattern::Column::PatternType.eq(&pattern_type))
            .filter(learned_pattern::Column::PatternData.eq(&pattern_data));
        
        if let Some(ctx) = context.as_ref() {
            query = query.filter(learned_pattern::Column::Context.eq(ctx));
        }
        
        let existing = query.one(db).await?;

        if let Some(pattern) = existing {
            Ok(pattern)
        } else {
            Self::create(db, pattern_type, pattern_data, context).await
        }
    }

    /// Mark pattern as important (never auto-delete)
    pub async fn mark_important(
        db: &DatabaseConnection,
        id: i32,
        is_important: bool,
    ) -> Result<Model, sea_orm::DbErr> {
        let mut active_model: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(format!("Pattern with id {} not found", id)))?
            .into();

        active_model.is_important = Set(is_important);
        active_model.update(db).await
    }
}

