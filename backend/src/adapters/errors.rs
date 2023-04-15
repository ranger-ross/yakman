use std::fmt;

#[derive(Debug)]
pub enum CreateConfigError {
    DuplicateConfigError { name: String },
    StorageError { message: String },
}

impl std::error::Error for CreateConfigError {}

impl fmt::Display for CreateConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CreateConfigError::DuplicateConfigError { name } => {
                write!(f, "Duplicate config: `{name}`")
            }
            CreateConfigError::StorageError { message } => {
                write!(f, "Error storing config: {message}")
            }
        }
    }
}

impl CreateConfigError {
    pub fn duplicate_config_error(name: &str) -> CreateConfigError {
        return CreateConfigError::DuplicateConfigError {
            name: String::from(name),
        };
    }
    pub fn storage_error(message: &str) -> CreateConfigError {
        return CreateConfigError::StorageError {
            message: String::from(message),
        };
    }
}
