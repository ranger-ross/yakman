pub mod oauth_service;
pub mod token;

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
    FailedToParseUsername,
}

#[derive(Error, Debug)]
pub enum RefreshTokenError {
    #[error("Failed to refresh token from OAuth provider")]
    FailedToRefreshToken(Box<dyn std::error::Error>),
    #[error("Failed to parse username from openid connect response")]
    FailedToParseUsername,
}
