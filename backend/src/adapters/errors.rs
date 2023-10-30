use std::string::FromUtf8Error;

use aws_sdk_s3::error::SdkError;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error accessing storage: {message}")]
pub struct GenericStorageError {
    pub message: String,
    pub raw_message: String,
}

impl GenericStorageError {
    pub fn new(message: String, raw_message: String) -> GenericStorageError {
        GenericStorageError {
            message: message,
            raw_message: raw_message,
        }
    }
}

impl From<std::io::Error> for GenericStorageError {
    fn from(e: std::io::Error) -> Self {
        GenericStorageError::new(String::from("IO Error"), e.to_string())
    }
}

impl From<serde_json::Error> for GenericStorageError {
    fn from(e: serde_json::Error) -> Self {
        GenericStorageError::new(String::from("JSON Error"), e.to_string())
    }
}

impl<E, R> From<SdkError<E, R>> for GenericStorageError {
    fn from(e: SdkError<E, R>) -> Self {
        GenericStorageError::new(String::from("AWS S3 Error"), e.to_string())
    }
}

impl From<google_cloud_storage::http::Error> for GenericStorageError {
    fn from(e: google_cloud_storage::http::Error) -> Self {
        GenericStorageError::new(String::from("Google Cloud Storage Error"), e.to_string())
    }
}

impl From<FromUtf8Error> for GenericStorageError {
    fn from(e: FromUtf8Error) -> Self {
        GenericStorageError::new(String::from("Error converting utf-8 bytes to String"), e.to_string())
    }
}
