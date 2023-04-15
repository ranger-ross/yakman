use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateConfigError {
    #[error("Duplicate config: `{name}`")]
    DuplicateConfigError { name: String },
    #[error("Error storing config: {message}")]
    StorageError { message: String },
}

impl CreateConfigError {
    pub fn duplicate_config_error(name: &str) -> CreateConfigError {
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
