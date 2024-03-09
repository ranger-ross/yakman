use std::sync::Arc;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::CreateProjectError,
    model::YakManProject,
};

use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait YakManProjectService: Sync + Send {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError>;

    async fn create_project(&self, project_name: &str) -> Result<String, CreateProjectError>;
}

pub struct ProjectService {
    pub adapter: Arc<dyn KVStorageAdapter>,
}

#[async_trait]
impl YakManProjectService for ProjectService {
    async fn get_projects(&self) -> Result<Vec<YakManProject>, GenericStorageError> {
        return Ok(self.adapter.get_projects().await?);
    }

    async fn create_project(&self, project_name: &str) -> Result<String, CreateProjectError> {
        let mut projects = self.adapter.get_projects().await?;

        // Prevent duplicates
        for prj in &projects {
            if &prj.name == &project_name {
                return Err(CreateProjectError::DuplicateNameError {
                    name: String::from(project_name),
                });
            }
        }

        let project_uuid = Uuid::new_v4();

        projects.push(YakManProject {
            name: String::from(project_name),
            uuid: project_uuid.to_string(),
        });

        self.adapter.save_projects(projects).await?;

        return Ok(project_uuid.to_string());
    }
}

impl ProjectService {
    pub fn new(adapter: Arc<dyn KVStorageAdapter>) -> Self {
        Self { adapter: adapter }
    }
}
