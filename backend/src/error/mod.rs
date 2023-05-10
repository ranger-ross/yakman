use actix_web::{error, http::header::ContentType, HttpResponse};
use derive_more::{Display, Error};
use serde::Serialize;

use crate::adapters::errors::GenericStorageError;

#[derive(Debug, Display, Error, Serialize)]
pub struct YakManError {
    error: String,
}

impl YakManError {
    pub fn new(error: &str) -> YakManError {
        YakManError {
            error: String::from(error),
        }
    }
}

impl error::ResponseError for YakManError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::to_string(self).unwrap_or(String::from("{}"))) // TODO: add internal server error message
    }
}

impl From<GenericStorageError> for YakManError {
    fn from(err: GenericStorageError) -> Self {
        YakManError {
            error: err.to_string(),
        }
    }
}
