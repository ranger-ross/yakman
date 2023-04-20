use std::{fs::File, io::Write};

use chrono::Utc;
use rocket::serde::json::serde_json;
use std::{cmp::Ordering, collections::HashMap};
use uuid::Uuid;
use yak_man_core::model::{Config, ConfigInstance, ConfigInstanceRevision, Label, LabelType};

use crate::adapters::{
    errors::{ConfigNotFoundError, CreateConfigError, CreateLabelError},
    FileBasedStorageAdapter,
};

#[async_trait] // TODO: refactor out to other file
pub trait StorageService: Sync + Send {
    async fn get_configs(&self) -> Result<Vec<Config>, ()>;

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

    async fn save_config_instance(
        &self,
        config_name: &str,
        instance: &str,
        labels: Vec<Label>,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct FileBasedStorageService {
    pub adapter: Box<dyn FileBasedStorageAdapter>,
}

#[async_trait]
impl StorageService for FileBasedStorageService {
    async fn get_configs(&self) -> Result<Vec<Config>, ()> {
        return Ok(self.adapter.get_configs().await.unwrap());
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
        if let Some(mut instances) = self.adapter.get_config_instance_metadata(config_name).await {
            let instance = Uuid::new_v4().to_string();
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            // Create new file with data
            self.adapter
                .save_config_instance_data_file(config_name, &data_key, data)
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
                .save_revision_data(config_name, &revision)
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
        return Ok(self.adapter.get_config_instance_metadata(config_name).await);
    }

    async fn get_config_data(
        &self,
        config_name: &str,
        instance: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(instances) = self.adapter.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());

            println!("Search for instance ID {}", instance);
            let selected_instance = instances.iter().find(|i| i.instance == instance);

            if let Some(instance) = selected_instance {
                return Ok(self
                    .adapter
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await);
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
        if let Some(instances) = self.adapter.get_config_instance_metadata(config_name).await {
            println!("Found {} instances", instances.len());
            let label_types = self.get_labels().await.unwrap();
            let selected_instance = select_instance(instances, labels, label_types);

            if let Some(instance) = selected_instance {
                return Ok(self
                    .adapter
                    .get_data_by_revision(config_name, &instance.current_revision)
                    .await);
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
        if let Some(mut instances) = self.adapter.get_config_instance_metadata(config_name).await {
            let revision_key = Uuid::new_v4().to_string();
            let data_key = Uuid::new_v4().to_string();

            // Create new file with data
            self.adapter.save_config_instance_data_file(config_name, &data_key, data).await.unwrap();

            // Create revision
            let revision = ConfigInstanceRevision {
                revision: String::from(&revision_key),
                data_key: String::from(&data_key),
                labels: labels,
                timestamp_ms: Utc::now().timestamp_millis(),
                approved: false,
            };
            self.adapter.save_revision_data(config_name, &revision).await?;

            // Update instance data
            if let Some(instance) = instances.iter_mut().find(|inst| inst.instance == instance) {
                instance.pending_revision = Some(String::from(&revision.revision));
                self.adapter.save_instance_metadata(config_name, instances).await?;
                println!("Updated instance metadata for config: {config_name}");
                return Ok(());
            } // TODO: Throw a new custom for failed to update config metadata
        }

        return Err(Box::new(ConfigNotFoundError {
            description: format!("Config not found: {config_name}"),
        }));
    }
}

/// Common logic to select a config instance from a selected labels
/// labels = selected labels, label_types = all label types avaiable, instances = all instances to select from
pub fn select_instance(
    instances: Vec<ConfigInstance>,
    labels: Vec<Label>,
    label_types: Vec<LabelType>,
) -> Option<ConfigInstance> {
    let label_type_map: HashMap<String, LabelType> = label_types
        .into_iter()
        .map(|label| (label.name.to_owned(), label))
        .collect();
    let selected_label_type_map: HashMap<String, &Label> = labels
        .iter()
        .map(|label| (label.label_type.to_owned(), label))
        .collect();
    let label_count = labels.len();

    let mut matched_instance: Option<ConfigInstance> = None;
    let mut matched_instance_labels: Vec<&Label> = vec![];

    todo!("Need to fix this logic after moving labels to revisions");

    // for instance in instances {
    //     if instance.labels == labels {
    //         // All labels are a perfect match, just return early
    //         return Some(instance);
    //     }

    //     // Find all matching labels for this instance
    //     let mut matched_labels: Vec<&Label> = vec![];
    //     for label in &instance.labels {
    //         let label_type = label_type_map.get(&label.label_type).unwrap(); // todo: handle
    //         let selected_label = selected_label_type_map.get(&label_type.name);
    //         match selected_label {
    //             Some(selected_label) => {
    //                 if selected_label.value == label.value {
    //                     matched_labels.push(selected_label.to_owned());
    //                 }
    //             }
    //             _ => {
    //                 continue;
    //             }
    //         }
    //     }

    //     // If the current instance is missing a label, it is not eligible, so continue to the next instance
    //     if label_count > matched_labels.len() {
    //         continue;
    //     }

    //     if matched_labels.len() > matched_instance_labels.len() {
    //         matched_instance = Some(instance);
    //         matched_instance_labels = matched_labels;
    //     } else {
    //         // IF THE MATCHING LABELS ARE THE SAME, CHECK IF THE LABELS ARE HIGHER PRIORITY
    //         matched_labels.sort_by(|a, b| order_by_priority(a, b, &label_type_map));
    //         matched_instance_labels.sort_by(|a, b| order_by_priority(a, b, &label_type_map));

    //         for i in 0..matched_labels.len() {
    //             let lbl = label_type_map
    //                 .get(&matched_labels.get(i).unwrap().label_type)
    //                 .unwrap(); // todo: handle
    //             let matched_lbl = label_type_map
    //                 .get(&matched_instance_labels.get(i).unwrap().label_type)
    //                 .unwrap(); // todo: handle

    //             if lbl.priority > matched_lbl.priority {
    //                 println!("Found better match");
    //                 matched_instance = Some(instance);
    //                 matched_instance_labels = matched_labels;
    //                 break;
    //             }
    //         }
    //     }
    // }

    // return matched_instance;
}

fn order_by_priority(
    a: &Label,
    b: &Label,
    label_type_map: &HashMap<String, LabelType>,
) -> Ordering {
    if let Some(a_type) = label_type_map.get(&a.label_type) {
        if let Some(b_type) = label_type_map.get(&b.label_type) {
            return a_type.priority.cmp(&b_type.priority);
        }
        return Ordering::Greater;
    }
    return Ordering::Less;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_instance_exact_match() {
        let instances = vec![
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance1".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "dev".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
            },
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance2".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
            },
        ];
        let labels = vec![
            Label {
                label_type: "environment".to_owned(),
                value: "dev".to_owned(),
            },
            Label {
                label_type: "service".to_owned(),
                value: "api".to_owned(),
            },
        ];
        let label_types = vec![
            LabelType {
                name: "environment".to_owned(),
                description: "".to_owned(),
                priority: 1,
                options: vec!["dev".to_owned(), "prod".to_owned()],
            },
            LabelType {
                name: "service".to_owned(),
                description: "".to_owned(),
                priority: 2,
                options: vec!["api".to_owned(), "web".to_owned()],
            },
        ];

        let result = select_instance(instances, labels, label_types);

        assert!(result.is_some());
        let result = result.unwrap();

        assert_eq!("config1", result.config_name);
        assert_eq!("instance1", result.instance);
        assert_eq!(2, result.labels.len());
        assert_eq!("environment", result.labels[0].label_type);
        assert_eq!("dev", result.labels[0].value);
        assert_eq!("service", result.labels[1].label_type);
        assert_eq!("api", result.labels[1].value);
    }

    #[test]
    fn test_select_instance_partial_match() {
        let instances = vec![
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance1".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "dev".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
            },
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance2".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
            },
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance3".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "web".to_owned(),
                    },
                ],
            },
        ];
        let labels = vec![Label {
            label_type: "service".to_owned(),
            value: "api".to_owned(),
        }];
        let label_types = vec![
            LabelType {
                name: "environment".to_owned(),
                priority: 1,
                description: "".to_owned(),
                options: vec!["dev".to_owned(), "prod".to_owned()],
            },
            LabelType {
                name: "service".to_owned(),
                priority: 2,
                description: "".to_owned(),
                options: vec!["api".to_owned(), "web".to_owned()],
            },
        ];

        let result = select_instance(instances, labels, label_types);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!("config1", result.config_name);
        assert_eq!("instance1", result.instance);
        assert_eq!(2, result.labels.len());
        assert_eq!("environment", result.labels[0].label_type);
        assert_eq!("dev", result.labels[0].value);
        assert_eq!("service", result.labels[1].label_type);
        assert_eq!("api", result.labels[1].value);
    }

    #[test]
    fn test_no_instance_match() {
        let instances = vec![
            ConfigInstance {
                config_name: "instance1_config".to_owned(),
                instance: "instance1".to_owned(),
                labels: vec![
                    Label {
                        label_type: "env".to_owned(),
                        value: "dev".to_owned(),
                    },
                    Label {
                        label_type: "app".to_owned(),
                        value: "frontend".to_owned(),
                    },
                ],
            },
            ConfigInstance {
                config_name: "instance2_config".to_owned(),
                instance: "instance2".to_owned(),
                labels: vec![
                    Label {
                        label_type: "env".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "app".to_owned(),
                        value: "backend".to_owned(),
                    },
                ],
            },
        ];

        let labels = vec![
            Label {
                label_type: "env".to_owned(),
                value: "staging".to_owned(),
            },
            Label {
                label_type: "app".to_owned(),
                value: "frontend".to_owned(),
            },
        ];

        let label_types = vec![
            LabelType {
                name: "env".to_owned(),
                priority: 1,
                description: "".to_owned(),
                options: vec![],
            },
            LabelType {
                name: "app".to_owned(),
                priority: 2,
                description: "".to_owned(),
                options: vec![],
            },
        ];

        let selected_instance = select_instance(instances, labels, label_types);
        assert!(selected_instance.is_none());
    }
}
