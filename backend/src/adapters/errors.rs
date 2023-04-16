use thiserror::Error;

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
pub enum ApproveRevisionError {
    #[error("Invalid config")]
    InvalidConfig,
    #[error("Invalid instance")]
    InvalidInstance,
    #[error("Invalid revision")]
    InvalidRevision,
    #[error("Revision already approved")]
    AlreadyApproved,
    #[error("Error storing approval: {message}")]
    StorageError { message: String },
}

impl ApproveRevisionError {
    pub fn storage_label(message: &str) -> ApproveRevisionError {
        ApproveRevisionError::StorageError {
            message: String::from(message),
        }
    }
}
