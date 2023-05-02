pub mod oauth_service;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoginError {
    #[error("Failed to exchange token")]
    FailedToExchangeCode,
    #[error("User not registered")]
    UserNotRegistered,
    #[error("Failed to check registered users")]
    FailedToCheckRegisteredUsers,
    #[error("Failed to get user data from OAuth provider")]
    FailedToFetchUserData(Box<dyn std::error::Error>),
}
