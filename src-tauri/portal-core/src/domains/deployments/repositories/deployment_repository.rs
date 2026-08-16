use crate::domains::deployments::services::docker_service::{
    Deployment, DeploymentStatus, DeploymentType,
};
use crate::entities::deployment::{
    ActiveModel as DeploymentActiveModel, Column, Entity as DeploymentEntity, Model,
};
use sea_orm::sea_query::OnConflict;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, Set};
use std::sync::Arc;

pub struct DeploymentRepository {
    conn: DatabaseConnection,
}

impl DeploymentRepository {
    pub fn new(db_manager: Arc<crate::database::DatabaseManager>) -> Self {
        Self {
            conn: db_manager.get_connection_clone(),
        }
    }

    pub async fn save(&self, deployment: &Deployment) -> Result<(), String> {
        let data_json = serde_json::to_string(deployment)
            .map_err(|e| format!("Failed to serialize deployment: {}", e))?;

        let model = DeploymentActiveModel {
            id: Set(deployment.id.clone()),
            project_id: Set(deployment.project_id.clone()),
            name: Set(deployment.name.clone()),
            deployment_type: Set(deployment_type_to_str(&deployment.deployment_type)),
            status: Set(deployment_status_to_str(&deployment.status)),
            data_json: Set(data_json),
            created_at: Set(deployment.created_at.clone()),
            updated_at: Set(deployment.updated_at.clone()),
        };

        DeploymentEntity::insert(model)
            .on_conflict(
                OnConflict::column(Column::Id)
                    .update_columns([
                        Column::ProjectId,
                        Column::Name,
                        Column::DeploymentType,
                        Column::Status,
                        Column::DataJson,
                        Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.conn)
            .await
            .map_err(|e| format!("Failed to save deployment: {}", e))?;

        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<Deployment>, String> {
        let rows = DeploymentEntity::find()
            .order_by_desc(Column::CreatedAt)
            .all(&self.conn)
            .await
            .map_err(|e| format!("Failed to load deployments: {}", e))?;

        rows.into_iter().map(model_to_deployment).collect()
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Deployment>, String> {
        let row = DeploymentEntity::find_by_id(id)
            .one(&self.conn)
            .await
            .map_err(|e| format!("Failed to load deployment: {}", e))?;

        match row {
            Some(model) => Ok(Some(model_to_deployment(model)?)),
            None => Ok(None),
        }
    }

    pub async fn delete(&self, id: &str) -> Result<(), String> {
        DeploymentEntity::delete_by_id(id)
            .exec(&self.conn)
            .await
            .map_err(|e| format!("Failed to delete deployment: {}", e))?;
        Ok(())
    }
}

fn model_to_deployment(model: Model) -> Result<Deployment, String> {
    serde_json::from_str(&model.data_json).map_err(|e| format!("Failed to parse deployment: {}", e))
}

fn deployment_type_to_str(deployment_type: &DeploymentType) -> String {
    serde_json::to_string(deployment_type)
        .unwrap_or_default()
        .trim_matches('"')
        .to_string()
}

fn deployment_status_to_str(status: &DeploymentStatus) -> String {
    format!("{status:?}")
}
