use std::fmt;
use thiserror::Error;

use crate::adapters::errors::GenericStorageError;

#[derive(Error, Debug)]
pub enum CreateConfigError {
    #[error("Duplicate config: `{name}`")]
    DuplicateConfigError { name: String },
    #[error("Error storing config: {message}")]
    StorageError { message: String },
}

impl CreateConfigError {
    pub fn duplicate_config(name: &str) -> CreateConfigError {
        CreateConfigError::DuplicateConfigError {
            name: String::from(name),
        }
    }
    pub fn storage_error(message: &str) -> CreateConfigError {
        CreateConfigError::StorageError {
            message: String::from(message),
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateLabelError {
    #[error("Duplicate label: `{name}`")]
    DuplicateLabelError { name: String },
    #[error("Labels must have at least one option")]
    EmptyOptionsError,
    #[error("Label prioity is invalid: {prioity}")]
    InvalidPriorityError { prioity: i32 },
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl CreateLabelError {
    pub fn duplicate_label(name: &str) -> CreateLabelError {
        CreateLabelError::DuplicateLabelError {
            name: String::from(name),
        }
    }
    pub fn invalid_priority_error(prioity: i32) -> CreateLabelError {
        CreateLabelError::InvalidPriorityError { prioity: prioity }
    }

    pub fn storage_label(message: &str) -> CreateLabelError {
        CreateLabelError::StorageError {
            message: String::from(message),
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateConfigInstanceError {
    #[error("No config found")]
    NoConfigFound,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for CreateConfigInstanceError {
    fn from(e: GenericStorageError) -> Self {
        CreateConfigInstanceError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum ApproveRevisionError {
    #[error("Invalid config")]
    InvalidConfig,
    #[error("Invalid instance")]
    InvalidInstance,
    #[error("Invalid revision")]
    InvalidRevision,
    #[error("Error storing approval: {message}")]
    StorageError { message: String },
}

impl ApproveRevisionError {
    pub fn storage_error(message: &str) -> ApproveRevisionError {
        ApproveRevisionError::StorageError {
            message: String::from(message),
        }
    }
}

#[derive(Debug)]
pub struct ConfigNotFoundError {
    pub description: String,
}

impl fmt::Display for ConfigNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for ConfigNotFoundError {
    fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug)]
pub struct LabelAlreadyExistsError {
    pub description: String,
}

impl fmt::Display for LabelAlreadyExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for LabelAlreadyExistsError {
    fn description(&self) -> &str {
        &self.description
    }
}
