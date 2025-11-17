use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use crate::entities::user_preference::{Entity, Model, ActiveModel};
use crate::entities::user_preference;

pub struct UserPreferenceRepository;

impl UserPreferenceRepository {
    /// Get all user preferences
    pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find().all(db).await
    }

    /// Get preference by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, sea_orm::DbErr> {
        Entity::find_by_id(id).one(db).await
    }

    /// Get preferences by type
    pub async fn get_by_type(
        db: &DatabaseConnection,
        preference_type: &str,
    ) -> Result<Vec<Model>, sea_orm::DbErr> {
        Entity::find()
            .filter(user_preference::Column::PreferenceType.eq(preference_type))
            .all(db)
            .await
    }

    /// Get preference by type and context
    pub async fn get_by_type_and_context(
        db: &DatabaseConnection,
        preference_type: &str,
        context: Option<&str>,
    ) -> Result<Option<Model>, sea_orm::DbErr> {
        let mut query = Entity::find()
            .filter(user_preference::Column::PreferenceType.eq(preference_type));

        if let Some(ctx) = context {
            query = query.filter(user_preference::Column::Context.eq(ctx));
        } else {
            query = query.filter(user_preference::Column::Context.is_null());
        }

        query.one(db).await
    }

    /// Create a new user preference
    pub async fn create(
        db: &DatabaseConnection,
        preference_type: String,
        context: Option<String>,
        preference_value: String,
        confidence: f64,
        learned_from: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        let active_model = ActiveModel {
            preference_type: Set(preference_type),
            context: Set(context),
            preference_value: Set(preference_value),
            confidence: Set(confidence),
            learned_from: Set(learned_from),
            created_at: Set(Some(chrono::Utc::now().into())),
            updated_at: Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        };

        active_model.insert(db).await
    }

    /// Update a user preference
    pub async fn update(
        db: &DatabaseConnection,
        id: i32,
        preference_value: Option<String>,
        confidence: Option<f64>,
        learned_from: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        let mut active_model: ActiveModel = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::Custom(format!("Preference with id {} not found", id)))?
            .into();

        if let Some(value) = preference_value {
            active_model.preference_value = Set(value);
        }
        if let Some(conf) = confidence {
            active_model.confidence = Set(conf);
        }
        if let Some(from) = learned_from {
            active_model.learned_from = Set(Some(from));
        }
        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(db).await
    }

    /// Delete a preference
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(id).exec(db).await?;
        Ok(())
    }

    /// Find or create preference by type and context
    pub async fn find_or_create(
        db: &DatabaseConnection,
        preference_type: String,
        context: Option<String>,
        preference_value: String,
        confidence: f64,
        learned_from: Option<String>,
    ) -> Result<Model, sea_orm::DbErr> {
        // Try to find existing preference
        let existing = Self::get_by_type_and_context(db, &preference_type, context.as_deref()).await?;

        if let Some(preference) = existing {
            // Update existing preference
            let current_confidence = preference.confidence;
            let new_confidence = (current_confidence + confidence) / 2.0; // Average confidence
            
            let mut active_model: ActiveModel = preference.into();
            active_model.preference_value = Set(preference_value);
            active_model.confidence = Set(new_confidence.min(1.0));
            active_model.updated_at = Set(Some(chrono::Utc::now().into()));

            active_model.update(db).await
        } else {
            // Create new preference
            Self::create(db, preference_type, context, preference_value, confidence, learned_from).await
        }
    }
}

