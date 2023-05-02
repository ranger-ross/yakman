use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::Header;
use jwt::RegisteredClaims;
use jwt::SignWithKey;
use jwt::Token;
use jwt::VerifyWithKey;
use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;
use std::{
    collections::BTreeMap,
    env::{self, VarError},
};
use thiserror::Error;
use yak_man_core::model::YakManRole;

pub struct JwtService {
    secret: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct YakManJwtClaims {
    pub iss: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub yakman_role: String,
}

impl JwtService {
    pub fn from_env() -> Result<JwtService, JwtServiceCreateError> {
        let secret = env::var("YAKMAN_JWT_SECRET")
            .map_err(|e| JwtServiceCreateError::FailedToLoadEnvVar(Box::new(e)))?;

        Ok(JwtService {
            secret: String::from(secret),
        })
    }

    pub fn create_acess_token(
        &self,
        user: &str,
        role: &YakManRole,
    ) -> Result<String, JwtCreateError> {
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.secret.as_bytes())
            .map_err(|e| JwtCreateError::InvalidSecret(Box::new(e)))?;

        let token_time_to_live_seconds = 60 * 60; // TODO: Make overridable
        let now = Utc::now().timestamp_millis() / 1000;

        let header: Header = Default::default();
        let claims = YakManJwtClaims {
            iat: now,
            sub: user.into(),
            exp: now + (token_time_to_live_seconds),
            iss: "YakMan Backend".into(),
            yakman_role: role.to_string(),
        };
        let unsigned_token = Token::new(header, claims);

        let token_str = unsigned_token
            .sign_with_key(&key)
            .map_err(|e| JwtCreateError::SigingError(Box::new(e)))?;

        return Ok(token_str.as_str().to_string());
    }

    pub fn validate_token(&self, token: &str) -> Result<YakManJwtClaims, JwtValidationError> {
        debug!("Validating token");
        let key: Hmac<Sha256> = Hmac::new_from_slice(self.secret.as_bytes())
            .map_err(|e| JwtValidationError::InvalidSecret(Box::new(e)))?;

        let claims: YakManJwtClaims = token
            .verify_with_key(&key)
            .map_err(|e| JwtValidationError::InvalidToken(Box::new(e)))?;

        // TODO: throw if expired

        return Ok(claims);
    }
}

#[derive(Error, Debug)]
pub enum JwtServiceCreateError {
    #[error("Failed to load YAKMAN_JWT_SECRET env var")]
    FailedToLoadEnvVar(Box<VarError>),
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
}
