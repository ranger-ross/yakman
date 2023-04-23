use std::fmt;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error accessing storage: {message}")]
pub struct GenericStorageError {
    pub message: String,
    pub raw_message: String,
}

impl GenericStorageError {
    fn new(message: String, raw_message: String) -> GenericStorageError {
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
