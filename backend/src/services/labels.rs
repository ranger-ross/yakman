use std::sync::Arc;

use crate::{
    adapters::{errors::GenericStorageError, KVStorageAdapter},
    error::CreateLabelError,
    model::{LabelType, YakManLabel},
};
use async_trait::async_trait;

#[async_trait]
pub trait YakManLabelService: Sync + Send {
    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError>;

    async fn create_label(&self, label: LabelType) -> Result<(), CreateLabelError>;

    /// Returns true if all labels exist and have valid values
    async fn validate_labels(&self, labels: &Vec<YakManLabel>)
        -> Result<bool, GenericStorageError>;
}

pub struct LabelService {
    pub adapter: Arc<dyn KVStorageAdapter>,
}

#[async_trait]
impl YakManLabelService for LabelService {
    async fn get_labels(&self) -> Result<Vec<LabelType>, GenericStorageError> {
        return Ok(self.adapter.get_labels().await?);
    }

    async fn create_label(&self, mut label: LabelType) -> Result<(), CreateLabelError> {
        let santized_options = label
            .options
            .into_iter()
            .filter_map(|opt| if !opt.is_empty() { Some(opt) } else { None })
            .collect::<Vec<String>>();

        if santized_options.len() == 0 {
            return Err(CreateLabelError::EmptyOptionsError);
        }

        label.options = santized_options;

        let mut labels = self.adapter.get_labels().await?;

        // Prevent duplicates
        for lbl in &labels {
            if &lbl.name == &label.name {
                return Err(CreateLabelError::duplicate_label(&label.name));
            }
        }

        labels.push(label);

        self.adapter.save_labels(labels).await?;

        return Ok(());
    }

    async fn validate_labels(
        &self,
        labels: &Vec<YakManLabel>,
    ) -> Result<bool, GenericStorageError> {
        let all_labels = self.get_labels().await?;
        for label in labels {
            if let Some(label_type) = all_labels.iter().find(|l| l.name == label.label_type) {
                if !label_type.options.iter().any(|opt| opt == &label.value) {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
        return Ok(true);
    }
}

impl LabelService {
    pub fn new(adapter: Arc<dyn KVStorageAdapter>) -> Self {
        Self { adapter: adapter }
    }
}
