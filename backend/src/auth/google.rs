use std::collections::HashMap;

use super::{OAuthEmailResolver, OAuthEmailResolverError};
use async_trait::async_trait;
use reqwest::Client;

pub struct GoogleEmailResolver;

impl GoogleEmailResolver {
    pub fn new() -> GoogleEmailResolver {
        GoogleEmailResolver {}
    }
}

#[async_trait]
impl OAuthEmailResolver for GoogleEmailResolver {
    async fn resolve_email(&self, access_token: &str) -> Result<String, OAuthEmailResolverError> {
        let client = Client::builder()
            .user_agent("YakMan Backend")
            .build()
            .map_err(|e| OAuthEmailResolverError::FailedToBuildRequest(Box::new(e)))?;

        let response = client
            .get("https://www.googleapis.com/oauth2/v3/tokeninfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OAuthEmailResolverError::FailedToFetchUserData(Box::new(e)))?;

        let response_data = response
            .json::<HashMap<String, String>>()
            .await
            .map_err(|e| OAuthEmailResolverError::FailedToExtractEmail(Box::new(e)))?;

        if let Some(email) = response_data.get("email") {
            Ok(email.to_string())
        } else {
            Err(OAuthEmailResolverError::EmailNotFound)
        }
    }
}

// async fn get_google_email(access_token: &str) -> Result<String, Box<dyn std::error::Error>> {
//     let url = format!("https://www.googleapis.com/oauth2/v3/tokeninfo?access_token={access_token}");

//     let resp = reqwest::get(url)
//         .await?
//         .json::<HashMap<String, String>>()
//         .await?;

//     let username = resp.get("email").unwrap().to_owned();
//     Ok(username)
// }
