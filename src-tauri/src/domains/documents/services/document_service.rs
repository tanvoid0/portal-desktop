use crate::domains::documents::repositories::document_repository::{DocumentRepository, CreateDocumentRequest, UpdateDocumentRequest};
use crate::entities::document::Model as DocumentModel;
use sea_orm::DatabaseConnection;

pub struct DocumentService {
    repository: DocumentRepository,
}

impl DocumentService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            repository: DocumentRepository::new(db),
        }
    }

    pub async fn create_document(&self, request: CreateDocumentRequest) -> Result<DocumentModel, sea_orm::DbErr> {
        self.repository.create(request).await
    }

    pub async fn update_document(&self, id: i32, request: UpdateDocumentRequest) -> Result<DocumentModel, sea_orm::DbErr> {
        self.repository.update(id, request).await
    }

    pub async fn update_draft(&self, id: i32, content_draft: String) -> Result<DocumentModel, sea_orm::DbErr> {
        self.repository.update_draft(id, content_draft).await
    }

    pub async fn save_document(&self, id: i32, title: Option<String>, content: Option<String>, tags: Option<Vec<String>>, is_archived: Option<bool>) -> Result<DocumentModel, sea_orm::DbErr> {
        self.repository.save_document(id, title, content, tags, is_archived).await
    }

    pub async fn delete_document(&self, id: i32) -> Result<(), sea_orm::DbErr> {
        self.repository.delete(id).await
    }

    pub async fn get_document(&self, id: i32) -> Result<Option<DocumentModel>, sea_orm::DbErr> {
        self.repository.find_by_id(id).await
    }

    pub async fn get_documents(&self) -> Result<Vec<DocumentModel>, sea_orm::DbErr> {
        self.repository.find_all().await
    }

    pub async fn search_documents(&self, query: &str) -> Result<Vec<DocumentModel>, sea_orm::DbErr> {
        self.repository.search(query).await
    }

    pub async fn get_documents_by_tag(&self, tag: &str) -> Result<Vec<DocumentModel>, sea_orm::DbErr> {
        self.repository.find_by_tag(tag).await
    }
}

