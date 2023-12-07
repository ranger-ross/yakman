use actix_web::{error, http::header::ContentType, HttpResponse};
use chrono::Utc;
use derive_more::Display;
use reqwest::StatusCode;
use serde::Serialize;

use crate::adapters::errors::GenericStorageError;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Display, derive_more::Error, Serialize)]
#[display(fmt = "{} {}", timestamp, message)]
pub struct YakManApiError {
    #[serde(skip_serializing)]
    pub status: StatusCode,

    pub timestamp: i64,
    pub message: String,
}

impl YakManApiError {
    pub fn new(message: &str) -> YakManApiError {
        YakManApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            timestamp: Utc::now().timestamp(),
            message: String::from(message),
        }
    }
    pub fn forbidden() -> YakManApiError {
        YakManApiError {
            status: StatusCode::FORBIDDEN,
            timestamp: Utc::now().timestamp(),
            message: String::from("forbidden"),
        }
    }
}

impl error::ResponseError for YakManApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status)
            .insert_header(ContentType::json())
            .body(serde_json::to_string(self).unwrap_or(generic_yakman_server_error_response()))
    }
}

impl From<GenericStorageError> for YakManApiError {
    fn from(err: GenericStorageError) -> Self {
        YakManApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            timestamp: Utc::now().timestamp(),
            message: err.to_string(),
        }
    }
}

/// This function panics if YakManApiError cannot be serialized.
/// In theory this should never happen because the error is a hardcoded string
fn generic_yakman_server_error_response() -> String {
    return serde_json::to_string(&YakManApiError::new("an internal server error occurred")).unwrap()
}

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
pub enum DeleteConfigError {
    #[error("Config does not exist")]
    ConfigDoesNotExistError,
    #[error("Error storing config: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for DeleteConfigError {
    fn from(e: GenericStorageError) -> Self {
        DeleteConfigError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum CreateProjectError {
    #[error("Duplicate project name: `{name}`")]
    DuplicateNameError { name: String },
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for CreateProjectError {
    fn from(e: GenericStorageError) -> Self {
        CreateProjectError::StorageError { message: e.message }
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
}

impl From<GenericStorageError> for CreateLabelError {
    fn from(e: GenericStorageError) -> Self {
        CreateLabelError::StorageError { message: e.message }
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
pub enum SaveConfigInstanceError {
    #[error("No config found")]
    NoConfigFound,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for SaveConfigInstanceError {
    fn from(e: GenericStorageError) -> Self {
        SaveConfigInstanceError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum UpdateConfigInstanceCurrentRevisionError {
    #[error("No config found")]
    NoConfigFound,
    #[error("Revision not found")]
    NoRevisionFound,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for UpdateConfigInstanceCurrentRevisionError {
    fn from(e: GenericStorageError) -> Self {
        UpdateConfigInstanceCurrentRevisionError::StorageError { message: e.message }
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

impl From<GenericStorageError> for ApproveRevisionError {
    fn from(e: GenericStorageError) -> Self {
        ApproveRevisionError::StorageError { message: e.message }
    }
}


#[derive(Error, Debug)]
pub enum ApplyRevisionError {
    #[error("Invalid config")]
    InvalidConfig,
    #[error("Invalid instance")]
    InvalidInstance,
    #[error("Invalid revision")]
    InvalidRevision,
    #[error("Revision not Approved")]
    NotApproved,
    #[error("Error storing approval: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for ApplyRevisionError {
    fn from(e: GenericStorageError) -> Self {
        ApplyRevisionError::StorageError { message: e.message }
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
