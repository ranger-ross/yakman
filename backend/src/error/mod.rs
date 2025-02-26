use actix_web::{error, http::header::ContentType, HttpResponse};
use chrono::Utc;
use derive_more::Display;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{
    adapters::errors::GenericStorageError,
    services::password::{PasswordHashError, PasswordStrengthError},
};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Display, derive_more::Error, Serialize)]
#[display("{timestamp} {message}")]
pub struct YakManApiError {
    #[serde(skip_serializing)]
    status: StatusCode,

    timestamp: i64,
    message: String,
}

impl YakManApiError {
    pub fn set_message(mut self, message: &str) -> YakManApiError {
        self.message = message.to_string();
        return self;
    }

    pub fn bad_request(reason: &str) -> YakManApiError {
        YakManApiError {
            status: StatusCode::BAD_REQUEST,
            timestamp: Utc::now().timestamp_millis(),
            message: String::from(reason),
        }
    }
    pub fn unauthorized() -> YakManApiError {
        YakManApiError {
            status: StatusCode::UNAUTHORIZED,
            timestamp: Utc::now().timestamp_millis(),
            message: String::from("unauthorized"),
        }
    }
    pub fn forbidden() -> YakManApiError {
        YakManApiError {
            status: StatusCode::FORBIDDEN,
            timestamp: Utc::now().timestamp_millis(),
            message: String::from("forbidden"),
        }
    }
    pub fn not_found<'a>(message: impl Into<Option<&'a str>>) -> YakManApiError {
        YakManApiError {
            status: StatusCode::FORBIDDEN,
            timestamp: Utc::now().timestamp_millis(),
            message: String::from(message.into().unwrap_or("not found")),
        }
    }
    pub fn server_error(message: &str) -> YakManApiError {
        YakManApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            timestamp: Utc::now().timestamp_millis(),
            message: String::from(message),
        }
    }
}

impl error::ResponseError for YakManApiError {
    fn error_response(&self) -> HttpResponse {
        let status =
            actix_web::http::StatusCode::from_u16(self.status.as_u16()).unwrap_or_else(|invalid| {
                log::error!("Invalid status code returned: {invalid}");
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            });

        HttpResponse::build(status)
            .insert_header(ContentType::json())
            .body(serde_json::to_string(self).unwrap_or(generic_yakman_server_error_response()))
    }
}

impl From<GenericStorageError> for YakManApiError {
    fn from(err: GenericStorageError) -> Self {
        YakManApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            timestamp: Utc::now().timestamp_millis(),
            message: err.to_string(),
        }
    }
}

/// This function panics if YakManApiError cannot be serialized.
/// In theory this should never happen because the error is a hardcoded string
fn generic_yakman_server_error_response() -> String {
    return serde_json::to_string(&YakManApiError::server_error(
        "an internal server error occurred",
    ))
    .unwrap();
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
pub enum UpdateProjectError {
    #[error("Project not found")]
    ProjectNotFound,
    #[error("Duplicate project name: `{name}`")]
    DuplicateNameError { name: String },
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for UpdateProjectError {
    fn from(e: GenericStorageError) -> Self {
        UpdateProjectError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum DeleteProjectError {
    #[error("Project not found")]
    ProjectNotFound,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for DeleteProjectError {
    fn from(e: GenericStorageError) -> Self {
        DeleteProjectError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum CreateLabelError {
    #[error("Duplicate label: `{name}`")]
    DuplicateLabelError { name: String },
    #[error("Labels must have at least one option")]
    EmptyOptionsError,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl CreateLabelError {
    pub fn duplicate_label(name: &str) -> CreateLabelError {
        CreateLabelError::DuplicateLabelError {
            name: String::from(name),
        }
    }
}

impl From<GenericStorageError> for CreateLabelError {
    fn from(e: GenericStorageError) -> Self {
        CreateLabelError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum UpdateLabelError {
    #[error("Duplicate label: `{name}`")]
    DuplicateLabelError { name: String },
    #[error("Labels must have at least one option")]
    EmptyOptionsError,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl UpdateLabelError {
    pub fn duplicate_label(name: &str) -> UpdateLabelError {
        UpdateLabelError::DuplicateLabelError {
            name: String::from(name),
        }
    }
}

impl From<GenericStorageError> for UpdateLabelError {
    fn from(e: GenericStorageError) -> Self {
        UpdateLabelError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum DeleteLabelError {
    #[error("Label not found")]
    LabelNotFound,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for DeleteLabelError {
    fn from(e: GenericStorageError) -> Self {
        DeleteLabelError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum CreateConfigInstanceError {
    #[error("No config found")]
    NoConfigFound,
    #[error("Invalid label")]
    InvalidLabel,
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
    #[error("Invalid config")]
    InvalidConfig,
    #[error("Invalid instance")]
    InvalidInstance,
    #[error("Invalid label")]
    InvalidLabel,
    #[error("Error storing label: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for SaveConfigInstanceError {
    fn from(e: GenericStorageError) -> Self {
        SaveConfigInstanceError::StorageError { message: e.message }
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

#[derive(Error, Debug)]
pub enum RollbackRevisionError {
    #[error("Invalid config")]
    InvalidConfig,
    #[error("Invalid instance")]
    InvalidInstance,
    #[error("Invalid revision")]
    InvalidRevision,
    #[error("Error storing approval: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for RollbackRevisionError {
    fn from(e: GenericStorageError) -> Self {
        RollbackRevisionError::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum DeleteConfigInstanceError {
    #[error("Invalid config")]
    InvalidConfig,
    #[error("Invalid instance")]
    InvalidInstance,
    #[error("Error storing approval: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for DeleteConfigInstanceError {
    fn from(e: GenericStorageError) -> Self {
        DeleteConfigInstanceError::StorageError { message: e.message }
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

#[derive(Error, Debug)]
pub enum ResetPasswordError {
    #[error("Reset link not valid")]
    ResetLinkNotFound,
    #[error("Invalid user")]
    InvalidUser,
    #[error("Invalid email")]
    InvalidEmail,
    #[error("Password reset link expired")]
    ResetLinkExpired,
    #[error("Invalid password: {error}")]
    PasswordValidationError { error: PasswordStrengthError },
    #[error("Password could not be hashed")]
    PasswordHashError { error: PasswordHashError },
    #[error("Storage Error: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for ResetPasswordError {
    fn from(e: GenericStorageError) -> Self {
        Self::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum CreatePasswordResetLinkError {
    #[error("Invalid user")]
    InvalidUser,
    #[error("Storage Error: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for CreatePasswordResetLinkError {
    fn from(e: GenericStorageError) -> Self {
        Self::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum CreateTeamError {
    #[error("Duplicate team")]
    DuplicateTeam,
    #[error("Storage Error: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for CreateTeamError {
    fn from(e: GenericStorageError) -> Self {
        Self::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum UpdateTeamError {
    #[error("Team not found")]
    TeamNotFound,
    #[error("Duplicate team")]
    DuplicateTeam,
    #[error("Storage Error: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for UpdateTeamError {
    fn from(e: GenericStorageError) -> Self {
        Self::StorageError { message: e.message }
    }
}

#[derive(Error, Debug)]
pub enum DeleteTeamError {
    #[error("Team not found")]
    TeamNotFound,
    #[error("Storage Error: {message}")]
    StorageError { message: String },
}

impl From<GenericStorageError> for DeleteTeamError {
    fn from(e: GenericStorageError) -> Self {
        Self::StorageError { message: e.message }
    }
}
