mod google;
mod github;
mod oauth_provider;
pub mod oauth_service;
pub mod token;

use async_trait::async_trait;
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


#[derive(Error, Debug)]
pub enum RefreshTokenError {
    #[error("Failed to refresh token from OAuth provider")]
    FailedToRefreshToken(Box<dyn std::error::Error>),
}


#[derive(Error, Debug)]
pub enum OAuthEmailResolverError {
    #[error("Failed to build request to OAuth provider")]
    FailedToBuildRequest(Box<dyn std::error::Error>),
    #[error("Failed to get data from OAuth provider")]
    FailedToFetchUserData(Box<dyn std::error::Error>),
    #[error("Failed to extract email")]
    FailedToExtractEmail(Box<dyn std::error::Error>),
    #[error("Email not found")]
    EmailNotFound,
}

/// This trait provides a method that will method that takes in an OAuth access token an returns the users email
#[async_trait]
trait OAuthEmailResolver {
    async fn resolve_email(&self, access_token: &str) -> Result<String, OAuthEmailResolverError>;
}
