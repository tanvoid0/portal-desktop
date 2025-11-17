use sea_orm::{DatabaseConnection, EntityTrait, Set, NotSet, ActiveModelTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use crate::entities::document::{Entity as DocumentEntity, Model as DocumentModel, ActiveModel, Column};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub is_archived: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_archived: Option<bool>,
    pub tags: Option<Vec<String>>,
}

pub struct DocumentRepository {
    db: DatabaseConnection,
}

impl DocumentRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, request: CreateDocumentRequest) -> Result<DocumentModel, sea_orm::DbErr> {
        let tags_json = request.tags
            .map(|tags| serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or_else(|| "[]".to_string());

        let now = chrono::Utc::now();
        let active_model = ActiveModel {
            id: NotSet,
            title: Set(request.title),
            content: Set(request.content),
            is_archived: Set(request.is_archived.unwrap_or(false)),
            content_draft: Set(None),
            is_draft: Set(false),
            tags: Set(Some(tags_json)),
            created_at: Set(Some(now.into())),
            updated_at: Set(Some(now.into())),
            last_edited_at: Set(Some(now.into())),
        };

        active_model.insert(&self.db).await
    }

    pub async fn update(&self, id: i32, request: UpdateDocumentRequest) -> Result<DocumentModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = DocumentEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Document not found".to_string()))?
            .into();

        if let Some(title) = request.title {
            active_model.title = Set(title);
        }
        if let Some(content) = request.content {
            active_model.content = Set(content);
            active_model.last_edited_at = Set(Some(chrono::Utc::now().into()));
        }
        if let Some(is_archived) = request.is_archived {
            active_model.is_archived = Set(is_archived);
        }
        if let Some(tags) = request.tags {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            active_model.tags = Set(Some(tags_json));
        }

        active_model.updated_at = Set(Some(chrono::Utc::now().into()));
        active_model.is_draft = Set(false);

        active_model.update(&self.db).await
    }

    pub async fn update_draft(&self, id: i32, content_draft: String) -> Result<DocumentModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = DocumentEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Document not found".to_string()))?
            .into();

        active_model.content_draft = Set(Some(content_draft));
        active_model.is_draft = Set(true);
        active_model.last_edited_at = Set(Some(chrono::Utc::now().into()));
        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(&self.db).await
    }

    pub async fn save_document(&self, id: i32, title: Option<String>, content: Option<String>, tags: Option<Vec<String>>, is_archived: Option<bool>) -> Result<DocumentModel, sea_orm::DbErr> {
        let mut active_model: ActiveModel = DocumentEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Document not found".to_string()))?
            .into();

        if let Some(title) = title {
            active_model.title = Set(title);
        }
        if let Some(content) = content {
            active_model.content = Set(content.clone());
            active_model.content_draft = Set(None);
            active_model.last_edited_at = Set(Some(chrono::Utc::now().into()));
        } else if let Some(draft) = active_model.content_draft.clone().into_value() {
            // Commit draft to content if no content provided
            if let sea_orm::Value::String(Some(draft_str)) = draft {
                active_model.content = Set((*draft_str).clone());
                active_model.content_draft = Set(None);
                active_model.last_edited_at = Set(Some(chrono::Utc::now().into()));
            }
        }
        if let Some(tags) = tags {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            active_model.tags = Set(Some(tags_json));
        }
        if let Some(is_archived) = is_archived {
            active_model.is_archived = Set(is_archived);
        }

        active_model.is_draft = Set(false);
        active_model.updated_at = Set(Some(chrono::Utc::now().into()));

        active_model.update(&self.db).await
    }

    pub async fn delete(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        DocumentEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<DocumentModel>, sea_orm::DbErr> {
        DocumentEntity::find_by_id(id).one(&self.db).await
    }

    pub async fn find_all(&self) -> Result<Vec<DocumentModel>, sea_orm::DbErr> {
        DocumentEntity::find()
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn search(&self, query: &str) -> Result<Vec<DocumentModel>, sea_orm::DbErr> {
        DocumentEntity::find()
            .filter(
                Column::Title.contains(query)
                    .or(Column::Content.contains(query))
            )
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }

    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<DocumentModel>, sea_orm::DbErr> {
        // Simple tag search - in a production system, you'd want proper JSON query support
        DocumentEntity::find()
            .filter(Column::Tags.contains(tag))
            .order_by_desc(Column::CreatedAt)
            .all(&self.db)
            .await
    }
}

