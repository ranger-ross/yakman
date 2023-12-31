use crate::model::YakManUser;
use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::Header;
use jwt::SignWithKey;
use jwt::Token;
use jwt::VerifyWithKey;
use log::debug;
#[cfg(test)]
use mockall::automock;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use short_crypt::ShortCrypt;
use std::{
    env::{self, VarError},
    string::FromUtf8Error,
};
use thiserror::Error;

#[cfg_attr(test, automock)]
pub trait TokenService: Sync + Send {
    /// Creates a JWT token and returns the token as a string and the expiration timestamp in unix milliseconds
    fn create_acess_token_jwt(
        &self,
        username: &str,
        user: &YakManUser,
    ) -> Result<(String, i64), JwtCreateError>;

    fn encrypt_refresh_token(&self, refresh_token: &str) -> String;

    fn decrypt_refresh_token(
        &self,
        encoded_ciphertext: &str,
    ) -> Result<String, RefreshTokenDecryptError>;

    fn validate_access_token(&self, token: &str) -> Result<YakManJwtClaims, JwtValidationError>;
}

pub struct YakManTokenService {
    access_token_signing_key: String,
    refresh_token_shortcrypt: ShortCrypt,
    access_token_time_to_live_seconds: i64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct YakManJwtClaims {
    pub iss: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub uuid: String,
}

impl YakManTokenService {
    pub fn from_env() -> Result<YakManTokenService, JwtServiceCreateError> {
        let access_token_signing_key = env::var("YAKMAN_ACCESS_TOKEN_SIGNING_KEY")
            .map_err(|e| JwtServiceCreateError::FailedToLoadSigningKey(Box::new(e)))?;
        let refresh_token_encryption_key = env::var("YAKMAN_REFRESH_TOKEN_ENCRYPTION_KEY")
            .map_err(|e| JwtServiceCreateError::FailedToLoadEncryptionKey(Box::new(e)))?;

        let default_access_token_ttl_seconds: i64 = 60 * 60;
        let access_token_time_to_live_seconds = env::var("YAKMAN_ACCESS_TOKEN_TTL_SECONDS")
            .map(|v| v.parse::<i64>().unwrap_or(default_access_token_ttl_seconds))
            .unwrap_or(default_access_token_ttl_seconds);

        Ok(YakManTokenService {
            access_token_signing_key: String::from(access_token_signing_key),
            refresh_token_shortcrypt: ShortCrypt::new(refresh_token_encryption_key),
            access_token_time_to_live_seconds: access_token_time_to_live_seconds,
        })
    }
}

impl TokenService for YakManTokenService {
    /// Creates a JWT token and returns the token as a string and the expiration timestamp in unix milliseconds
    fn create_acess_token_jwt(
        &self,
        username: &str,
        user: &YakManUser,
    ) -> Result<(String, i64), JwtCreateError> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.access_token_signing_key.as_bytes())
            .map_err(|e| JwtCreateError::InvalidSecret(Box::new(e)))?;

        let token_time_to_live_seconds = self.access_token_time_to_live_seconds;
        let now = Utc::now().timestamp_millis() / 1000;

        let header: Header = Default::default();
        let claims = YakManJwtClaims {
            iat: now,
            sub: username.into(),
            exp: now + (token_time_to_live_seconds),
            iss: "YakMan Backend".into(),
            uuid: user.uuid.to_string(),
        };
        let unsigned_token = Token::new(header, claims);

        let token_str = unsigned_token
            .sign_with_key(&key)
            .map_err(|e| JwtCreateError::SigingError(Box::new(e)))?;

        return Ok((
            token_str.as_str().to_string(),
            (now + (token_time_to_live_seconds)) * 1000,
        ));
    }

    fn encrypt_refresh_token(&self, refresh_token: &str) -> String {
        return self
            .refresh_token_shortcrypt
            .encrypt_to_url_component(refresh_token);
    }

    fn decrypt_refresh_token(
        &self,
        encoded_ciphertext: &str,
    ) -> Result<String, RefreshTokenDecryptError> {
        let decrypted_bytes = self
            .refresh_token_shortcrypt
            .decrypt_url_component(encoded_ciphertext)?;
        return Ok(String::from_utf8(decrypted_bytes)?);
    }

    fn validate_access_token(&self, token: &str) -> Result<YakManJwtClaims, JwtValidationError> {
        debug!("Validating token");
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.access_token_signing_key.as_bytes())
            .map_err(|e| JwtValidationError::InvalidSecret(Box::new(e)))?;

        let claims: YakManJwtClaims = token
            .verify_with_key(&key)
            .map_err(|e| JwtValidationError::InvalidToken(Box::new(e)))?;

        // If expired, return an error
        if claims.exp < (Utc::now().timestamp_millis() / 1000) {
            return Err(JwtValidationError::TokenExpired);
        }

        return Ok(claims);
    }
}

#[derive(Error, Debug)]
pub enum JwtServiceCreateError {
    #[error("Failed to load YAKMAN_ACCESS_TOKEN_SIGNING_KEY env var")]
    FailedToLoadSigningKey(Box<VarError>),
    #[error("Failed to load YAKMAN_REFRESH_TOKEN_ENCRYPTION_KEY env var")]
    FailedToLoadEncryptionKey(Box<VarError>),
}

#[derive(Error, Debug)]
pub enum JwtCreateError {
    #[error("Invalid JWT signing secret")]
    InvalidSecret(Box<dyn std::error::Error>),
    #[error("Failed to create JWT")]
    SigingError(Box<dyn std::error::Error>),
}

#[derive(Error, Debug)]
pub enum JwtValidationError {
    #[error("Invalid JWT signing secret")]
    InvalidSecret(Box<dyn std::error::Error>),
    #[error("Invalid token")]
    InvalidToken(Box<dyn std::error::Error>),
    #[error("Token expired")]
    TokenExpired,
}

#[derive(Error, Debug)]
pub enum RefreshTokenDecryptError {
    #[error("Failed to decrypt token")]
    FailedToDecrypt(String),
}

impl From<FromUtf8Error> for RefreshTokenDecryptError {
    fn from(value: FromUtf8Error) -> Self {
        RefreshTokenDecryptError::FailedToDecrypt(value.to_string())
    }
}

impl From<&str> for RefreshTokenDecryptError {
    fn from(value: &str) -> Self {
        RefreshTokenDecryptError::FailedToDecrypt(value.to_string())
    }
}
