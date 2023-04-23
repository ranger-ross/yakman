use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

use crate::{
    adapters::{
        errors::{ApproveRevisionError, ConfigNotFoundError, CreateConfigError, CreateLabelError},
        FileBasedStorageAdapter, GenericStorageError,
    },
    services::service_utils::select_instance,
};

#[async_trait] // TODO: refactor out to other file
pub trait StorageService: Sync + Send {
    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError>;

    // Labels CRUD
    async fn get_labels(&self) -> Result<Vec<LabelType>, ()>;

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError>;

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn get_config_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, Box<dyn std::error::Error>>;

    async fn get_config_data(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>>;

    async fn get_config_data_by_labels(
        &self,
        config_name: &str,
        labels: Vec<Label>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>>;

    async fn get_data_by_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>>;

    async fn save_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, Box<dyn std::error::Error>>;

    async fn update_instance_current_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn approve_pending_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), ApproveRevisionError>;

    async fn initialize_storage(&self) -> Result<(), GenericStorageError>;
}

pub struct FileBasedStorageService {
    pub adapter: Box<dyn FileBasedStorageAdapter>,
}

#[async_trait]
impl StorageService for FileBasedStorageService {
    async fn get_configs(&self) -> Result<Vec<Config>, GenericStorageError> {
        return Ok(self.adapter.get_configs().await?);
    }

    async fn get_labels(&self) -> Result<Vec<LabelType>, ()> {
        return Ok(self.adapter.get_labels().await.unwrap());
    }

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError> {
        let mut labels = self.adapter.get_labels().await.unwrap();

        let mut max_prioity: Option<i32> = None;

        // Prevent duplicates
        for lbl in &labels {
            if &lbl.name == &label.name {
                return Err(CreateLabelError::duplicate_label(&label.name));
            }
            if max_prioity.is_none() || lbl.priority > max_prioity.unwrap() {
                max_prioity = Some(lbl.priority);
            }
        }

        if let Some(max_prioity) = max_prioity {
            if max_prioity < label.priority - 1 {
                return Err(CreateLabelError::invalid_priority_error(label.priority));
            }
        }

        for lbl in labels.iter_mut() {
            if lbl.priority >= label.priority {
                lbl.priority += 1;
            }
        }

        labels.push(label);

        self.adapter
            .save_labels(labels)
            .await
            .map_err(|e| CreateLabelError::storage_label(&e.to_string()))?;

        return Ok(());
    }

    async fn create_config_instance(
        &self,
        config_name: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut instances) = self.adapter.get_instance_metadata(config_name).await.unwrap() {
            let instance = Uuid::new_v4().to_string();
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            // Create new file with data
            self.adapter
                .save_instance_data(config_name, &data_key, data)
                .await?;

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: Utc::now().timestamp_millis(),
                approved: false,
            };
            self.adapter
                .save_revision(config_name, &revision)
                .await
                .unwrap();

            // Add new instance to instances and update the instance datafile
            instances.push(ConfigInstance {
                config_name: config_name.to_string(),
                instance: instance,
                labels: revision.labels,
                current_revision: String::from(&revision.revision),
                pending_revision: None,
                revisions: vec![revision.revision],
            });
            self.adapter
                .save_instance_metadata(config_name, instances)
                .await?;
            println!("Update instance metadata for config: {}", config_name);

            return Ok(());
        }

        return Err(Box::new(ConfigNotFoundError {
            description: format!("Config not found: {config_name}"),
        }));
    }

    async fn create_config(&self, config_name: &str) -> Result<(), CreateConfigError> {
        let mut configs = self.get_configs().await.unwrap(); // TODO: Handle

        if configs
            .iter()
            .find(|config| config.name == config_name)
            .is_some()
        {
            return Err(CreateConfigError::duplicate_config(config_name));
        }

        configs.push(Config {
            name: String::from(config_name),
            description: String::from(""), // TODO: support descriptions?
        });

        // Create instance metadata file
        self.adapter
            .save_instance_metadata(config_name, vec![])
            .await
            .unwrap();

        // Create config instances directory
        self.adapter
            .create_config_instance_dir(config_name)
            .await
            .map_err(|_| {
                CreateConfigError::storage_error("Failed to create instances directory")
            })?;

        // Create config revisions directory
        self.adapter
            .create_revision_instance_dir(config_name)
            .await
            .map_err(|_| {
                CreateConfigError::storage_error("Failed to create revisions directory")
            })?;

        // Add config to base config file
        self.adapter
            .save_configs(configs)
            .await
            .map_err(|_| CreateConfigError::storage_error("Failed to update configs file"))?;

        Ok(())
    }

    async fn get_config_instance_metadata(
        &self,
        config_name: &str,
    ) -> Result<Option<Vec<ConfigInstance>>, Box<dyn std::error::Error>> {
        return Ok(self.adapter.get_instance_metadata(config_name).await.unwrap());
    }

    async fn get_config_data(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(instances) = self.adapter.get_instance_metadata(config_name).await.unwrap() {
            println!("Found {} instances", instances.len());

            println!("Search for instance ID {}", instance);
            let selected_instance = instances.iter().find(|i| i.instance == instance);

            if let Some(instance) = selected_instance {
                return self
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await;
            }
            println!("No selected instance found");
            return Ok(None);
        }
        return Ok(None);
    }

    async fn get_config_data_by_labels(
        &self,
        config_name: &str,
        labels: Vec<Label>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(instances) = self.adapter.get_instance_metadata(config_name).await.unwrap() {
            println!("Found {} instances", instances.len());
            let label_types = self.get_labels().await.unwrap();
            let selected_instance = select_instance(instances, labels, label_types);

            if let Some(instance) = selected_instance {
                return self
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await;
            }
            println!("No selected instance found");
            return Ok(None);
        }
        return Ok(None);
    }

    async fn save_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut instances) = self.adapter.get_instance_metadata(config_name).await.unwrap() {
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            // Create new file with data
            self.adapter
                .save_instance_data(config_name, &data_key, data)
                .await
                .unwrap();

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: Utc::now().timestamp_millis(),
                approved: false,
            };
            self.adapter.save_revision(config_name, &revision).await?;

            // Update instance data
            if let Some(instance) = instances.iter_mut().find(|inst| inst.instance == instance) {
                instance.pending_revision = Some(String::from(&revision.revision));
                self.adapter
                    .save_instance_metadata(config_name, instances)
                    .await?;
                println!("Updated instance metadata for config: {config_name}");
                return Ok(());
            } // TODO: Throw a new custom for failed to update config metadata
        }

        return Err(Box::new(ConfigNotFoundError {
            description: format!("Config not found: {config_name}"),
        }));
    }

    async fn get_instance_revisions(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<Vec<ConfigInstanceRevision>>, Box<dyn std::error::Error>> {
        let instances = match self
            .get_config_instance_metadata(&config_name)
            .await
            .unwrap()
        {
            Some(value) => value,
            None => return Ok(None),
        };

        let instance = match instances.iter().find(|inst| inst.instance == instance) {
            Some(value) => value,
            None => return Ok(None),
        };

        println!("found {} revisions", instance.revisions.len());

        let mut revisions: Vec<ConfigInstanceRevision> = vec![];

        for rev in instance.revisions.iter() {
            if let Some(revision) = self.adapter.get_revsion(config_name, &rev).await {
                revisions.push(revision);
            }
        }

        return Ok(Some(revisions));
    }

    async fn get_data_by_revision(
        &self,
        config_name: &str,
        revision: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(revision_data) = self.adapter.get_revsion(config_name, revision).await {
            let key = &revision_data.data_key;
            return Ok(self.adapter.get_instance_data(config_name, key).await.ok());
        }
        println!("Fetching revision not found");
        return Ok(None);
    }

    async fn update_instance_current_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut instances = self
            .get_config_instance_metadata(config_name)
            .await
            .unwrap()
            .unwrap(); // TODO: propagate error

        let mut instance = instances
            .iter_mut()
            .find(|i| i.instance == instance)
            .unwrap(); // TODO: propagate error

        if !instance.revisions.contains(&String::from(revision)) {
            panic!("revision not found!"); // TODO: propagate error
        }
        instance.pending_revision = Some(String::from(revision));

        self.adapter
            .save_instance_metadata(config_name, instances)
            .await?;

        return Ok(());
    }

    async fn approve_pending_instance_revision(
        &self,
        config_name: &str,
        instance: &str,
        revision: &str,
    ) -> Result<(), ApproveRevisionError> {
        let mut metadata = match self
            .get_config_instance_metadata(config_name)
            .await
            .unwrap()
        {
            Some(metadata) => metadata,
            None => return Err(ApproveRevisionError::InvalidConfig),
        };

        let mut instance = match metadata.iter_mut().find(|i| i.instance == instance) {
            Some(instance) => instance,
            None => return Err(ApproveRevisionError::InvalidInstance),
        };

        // Verify instance is the pending revision
        if let Some(pending_revision) = &instance.pending_revision {
            if pending_revision != revision {
                return Err(ApproveRevisionError::InvalidRevision);
            }
        } else {
            return Err(ApproveRevisionError::InvalidRevision);
        }

        let mut revision_data = match self.adapter.get_revsion(config_name, revision).await {
            Some(revision_data) => revision_data,
            None => return Err(ApproveRevisionError::InvalidRevision),
        };

        // if revision_data.approved {
        //     return Err(ApproveRevisionError::AlreadyApproved);
        // }

        revision_data.approved = true;
        self.adapter
            .save_revision(config_name, &revision_data)
            .await
            .map_err(|e| ApproveRevisionError::StorageError {
                message: e.to_string(),
            })?;

        instance.current_revision = String::from(revision);
        instance.pending_revision = None;
        instance.labels = revision_data.labels;

        if !instance.revisions.contains(&String::from(revision)) {
            instance.revisions.push(String::from(revision));
        }

        self.adapter
            .save_instance_metadata(config_name, metadata)
            .await
            .map_err(|e| ApproveRevisionError::StorageError {
                message: e.to_string(),
            })?;

        return Ok(());
    }

    async fn initialize_storage(&self) -> Result<(), GenericStorageError> {
        println!("initializing local storage adapter");

        self.adapter.create_yakman_required_files().await?;

        Ok(())
    }
}
